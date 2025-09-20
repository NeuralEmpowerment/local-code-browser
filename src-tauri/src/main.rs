#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use serde::Serialize;
use tracing_subscriber::EnvFilter;

use indexer::{scan_roots, ConfigStore, Db, ScanOptions, SortKey};

#[derive(Serialize)]
struct ProjectsPage {
    items: Vec<indexer::ProjectRecord>,
    page: u32,
    page_size: u32,
    total_count: u32,
}

#[tauri::command]
fn test_command() -> Result<String, String> {
    tracing::info!("test_command called");
    Ok("Hello from Rust!".to_string())
}

#[tauri::command]
fn open_in_editor(editor: String, path: String) -> Result<String, String> {
    tracing::info!("open_in_editor called with editor={}, path={}", editor, path);
    
    use std::process::Command;
    
    // Try common paths for editors
    let editor_paths = match editor.as_str() {
        "windsurf" => vec![
            "windsurf", 
            "/usr/local/bin/windsurf", 
            "/opt/homebrew/bin/windsurf",
            "/Applications/Windsurf.app/Contents/Resources/app/bin/windsurf",
            "/Applications/Windsurf.app/Contents/MacOS/Windsurf"
        ],
        "cursor" => vec![
            "cursor", 
            "/usr/local/bin/cursor", 
            "/opt/homebrew/bin/cursor", 
            "/Applications/Cursor.app/Contents/Resources/app/bin/cursor"
        ],
        _ => vec![editor.as_str()],
    };
    
    for editor_path in editor_paths {
        let result = Command::new(editor_path)
            .arg(&path)
            .spawn();
        
        match result {
            Ok(_) => {
                tracing::info!("Successfully launched {} with path {}", editor_path, path);
                return Ok(format!("Opened {} in {}", path, editor));
            }
            Err(e) => {
                tracing::debug!("Failed to launch {} with path {}: {}", editor_path, path, e);
                continue;
            }
        }
    }
    
    tracing::error!("Failed to launch {} with any known path", editor);
    Err(format!("Failed to open {}: command not found in common locations", editor))
}

#[tauri::command]
fn scan_start(roots: Option<Vec<String>>, dry_run: Option<bool>) -> Result<usize, String> {
    tracing::info!(?roots, "scan_start");
    let mut cfg = ConfigStore::load().map_err(|e| e.to_string())?;
    if let Some(rs) = roots {
        cfg.roots = rs
            .into_iter()
            .map(|r| shellexpand::tilde(&r).to_string().into())
            .collect();
    }
    let db = Db::open_default().map_err(|e| e.to_string())?;
    tracing::info!(db = %db.path.display(), "scan_start db path");
    let count = scan_roots(
        &db,
        &cfg,
        &ScanOptions {
            dry_run: dry_run.unwrap_or(false),
        },
    )
    .map_err(|e| e.to_string())?;
    tracing::info!(count, "scan_complete");
    Ok(count)
}

#[tauri::command]
fn projects_query(
    q: Option<String>,
    sort: Option<String>,
    sort_direction: Option<String>,
    page: u32,
    page_size: u32,
) -> Result<ProjectsPage, String> {
    tracing::info!("projects_query called with q={:?}, sort={:?}, page={}, page_size={}", q, sort, page, page_size);
    let db = Db::open_default().map_err(|e| {
        tracing::error!("Failed to open database: {}", e);
        e.to_string()
    })?;
    let sort_key = match sort.as_deref() {
        Some("size") => SortKey::Size,
        Some("name") => SortKey::Name,
        Some("type") => SortKey::Type,
        Some("loc") => SortKey::Loc,
        _ => SortKey::Recent,
    };
    let qnorm = q.as_ref().and_then(|s| if s.trim().is_empty() { None } else { Some(s.as_str()) });
    let ascending = sort_direction.as_deref() == Some("asc");
    tracing::info!(q = ?qnorm, sort = ?sort_key as i32, ascending, page, page_size, db = %db.path.display(), "projects_query");
    
    let total_count = db
        .count_projects(qnorm)
        .map_err(|e| {
            tracing::error!("Database count failed: {}", e);
            e.to_string()
        })?;
    
    let rows = db
        .query_projects(qnorm, sort_key, ascending, page, page_size)
        .map_err(|e| {
            tracing::error!("Database query failed: {}", e);
            e.to_string()
        })?;
    tracing::info!(rows = rows.len(), total_count, "projects_query_result - returning {} items of {} total", rows.len(), total_count);
    Ok(ProjectsPage {
        items: rows,
        page,
        page_size,
        total_count,
    })
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![test_command, open_in_editor, scan_start, projects_query])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
