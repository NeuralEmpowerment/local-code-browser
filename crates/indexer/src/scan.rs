use anyhow::Result;
use ignore::{Walk, WalkBuilder};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(feature = "analyzers")]
use crate::analyzers::{compute_loc, compute_loc_breakdown};
use crate::config::{AppConfig, ConfigStore, SizeMode};
use crate::db::Db;
use crate::detect::{detect_project_type, is_git_repo};
#[cfg(feature = "git")]
use crate::vcs::read_git_info;

#[derive(Debug, Clone, Default)]
pub struct ScanOptions {
    pub dry_run: bool,
}

pub fn scan_roots(db: &Db, cfg: &AppConfig, opts: &ScanOptions) -> Result<usize> {
    let mut found: usize = 0;
    for root in &cfg.roots {
        if !root.exists() {
            tracing::warn!(?root, "root does not exist; skipping");
            continue;
        }
        let mut wb = WalkBuilder::new(root);
        wb.git_ignore(true).hidden(true).ignore(true);
        // Per-user/app ignore files if present
        if let Ok(app_ign) = ConfigStore::app_ignore_path() {
            if app_ign.exists() {
                wb.add_ignore(app_ign);
            }
        }
        {
            let legacy = ConfigStore::user_ignore_path_legacy();
            if legacy.exists() {
                wb.add_ignore(legacy);
            }
        }
        let walk = wb.build();
        found += scan_one_root(db, cfg, opts, walk, root)?;
    }
    Ok(found)
}

fn scan_one_root(
    db: &Db,
    cfg: &AppConfig,
    opts: &ScanOptions,
    walk: Walk,
    _root: &Path,
) -> Result<usize> {
    let mut processed_roots: Vec<PathBuf> = Vec::new();
    let mut count = 0usize;

    for res in walk {
        let entry = match res {
            Ok(e) => e,
            Err(err) => {
                tracing::warn!(%err, "walk error");
                continue;
            }
        };

        let p = entry.path();
        if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            continue;
        }

        // Skip directories under previously processed project roots to avoid double work
        if processed_roots.iter().any(|r| p.starts_with(r)) {
            continue;
        }

        // Apply global ignores (simple name match)
        if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
            if cfg.global_ignores.iter().any(|ign| ign == name) {
                continue;
            }
        }

        // Detect project
        if let Some(ptype) = detect_project_type(p) {
            let name = p
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let path_str = p.to_string_lossy().to_string();
            let git = is_git_repo(p);

            #[allow(unused_mut)]
            let (size_bytes, files_count, mut last_edited_at) =
                compute_metrics(p, cfg, git).unwrap_or((None, None, None));
            #[cfg(feature = "analyzers")]
            let loc = compute_loc(p);
            #[cfg(not(feature = "analyzers"))]
            let loc: Option<i64> = None;

            // If available, use git last commit to improve recency
            #[cfg(feature = "git")]
            let git_info = {
                let info = read_git_info(p);
                if let Some(ts) = info.last_commit_at {
                    if let Some(le) = last_edited_at {
                        if ts > le {
                            last_edited_at = Some(ts);
                        }
                    } else {
                        last_edited_at = Some(ts);
                    }
                }
                Some(info)
            };
            #[cfg(not(feature = "git"))]
            let _git_info: Option<()> = None;

            if opts.dry_run {
                tracing::info!(
                    name=%name,
                    path=%path_str,
                    project_type=%ptype.as_str(),
                    git=git,
                    size=?size_bytes,
                    files=?files_count,
                    last_edited=?last_edited_at,
                    "found project"
                );
            } else {
                let id = db.upsert_project(&name, &path_str, Some(ptype.as_str()), git)?;
                db.upsert_metrics(id, size_bytes, files_count, last_edited_at, loc)?;
                #[cfg(feature = "git")]
                if let Some(info) = git_info {
                    db.upsert_git_info(
                        id,
                        info.last_commit_at,
                        info.branch.as_deref(),
                        info.remote_url.as_deref(),
                    )?;
                }
                #[cfg(feature = "analyzers")]
                if let Some((_total, breakdown)) = compute_loc_breakdown(p) {
                    db.replace_loc_breakdown(id, &breakdown)?;
                }
            }

            processed_roots.push(p.to_path_buf());
            count += 1;
        }
    }
    Ok(count)
}

fn compute_metrics(
    root: &Path,
    cfg: &AppConfig,
    _git: bool,
) -> Result<(Option<i64>, Option<i64>, Option<i64>)> {
    let mut total_size: i64 = 0;
    let mut files_count: i64 = 0;
    let mut latest_mtime: i64 = 0;

    // Honor gitignore within the project root
    let walk = WalkBuilder::new(root)
        .git_ignore(true)
        .hidden(true)
        .ignore(true)
        .build();

    for res in walk {
        let entry = match res {
            Ok(e) => e,
            Err(err) => {
                tracing::warn!(%err, "walk error (metrics)");
                continue;
            }
        };
        let p = entry.path();

        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            // Skip global ignores by name
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                if cfg.global_ignores.iter().any(|ign| ign == name) {
                    continue;
                }
            }
            continue;
        }

        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            files_count += 1;
            if let Ok(md) = fs::metadata(p) {
                total_size += md.len() as i64;
                if let Ok(mtime) = md.modified() {
                    if let Ok(secs) = mtime.duration_since(std::time::UNIX_EPOCH) {
                        latest_mtime = latest_mtime.max(secs.as_secs() as i64);
                    }
                }
            }
        }
    }

    let size_opt = match cfg.size_mode {
        SizeMode::ExactCached => Some(total_size),
        SizeMode::None => None,
    };

    let files_opt = Some(files_count);
    let last_edit_opt = if latest_mtime > 0 {
        Some(latest_mtime)
    } else {
        None
    };

    Ok((size_opt, files_opt, last_edit_opt))
}
