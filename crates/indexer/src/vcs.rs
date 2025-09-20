#[cfg(feature = "git")]
use git2::{BranchType, Repository};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct GitInfo {
    pub last_commit_at: Option<i64>,
    pub branch: Option<String>,
    pub remote_url: Option<String>,
}

#[cfg(feature = "git")]
pub fn read_git_info(dir: &Path) -> GitInfo {
    let repo = match Repository::discover(dir) {
        Ok(r) => r,
        Err(_) => {
            return GitInfo {
                last_commit_at: None,
                branch: None,
                remote_url: None,
            }
        }
    };

    // Last commit time from HEAD
    let last_commit_at = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_commit().ok())
        .map(|c| c.time().seconds());

    // Branch name
    let branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()));

    // Remote URL from 'origin'
    let remote_url = repo
        .find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(|s| s.to_string()));

    GitInfo {
        last_commit_at,
        branch,
        remote_url,
    }
}

#[cfg(not(feature = "git"))]
pub fn read_git_info(_dir: &Path) -> GitInfo {
    GitInfo {
        last_commit_at: None,
        branch: None,
        remote_url: None,
    }
}
