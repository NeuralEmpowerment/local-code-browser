use anyhow::Result;
use rusqlite::{params, Connection};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::ConfigStore;

pub struct Db {
    pub conn: Connection,
    pub path: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProjectRecord {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub project_type: Option<String>,
    pub is_git_repo: bool,
    pub size_bytes: Option<i64>,
    pub files_count: Option<i64>,
    pub last_edited_at: Option<i64>,
    pub loc: Option<i64>,
}

#[derive(Debug, Clone, Copy)]
pub enum SortKey {
    Recent,
    Size,
    Name,
    Type,
    Loc,
}

impl Db {
    pub fn open_default() -> Result<Self> {
        let dir = ConfigStore::data_dir()?;
        fs::create_dir_all(&dir)?;
        let path = dir.join("projects.sqlite");
        Self::open(&path)
    }

    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn,
            path: path.to_path_buf(),
        };
        db.migrate()?;
        Ok(db)
    }

    /// Create or migrate schema to the latest version (idempotent)
    pub fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            PRAGMA journal_mode=WAL;
            PRAGMA foreign_keys=ON;

            CREATE TABLE IF NOT EXISTS projects (
              id INTEGER PRIMARY KEY,
              name TEXT NOT NULL,
              path TEXT NOT NULL UNIQUE,
              type TEXT,
              is_git_repo INTEGER NOT NULL DEFAULT 0,
              created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
              updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
            );

            CREATE TABLE IF NOT EXISTS metrics (
              project_id INTEGER PRIMARY KEY,
              size_bytes INTEGER,
              files_count INTEGER,
              last_edited_at INTEGER,
              loc INTEGER,
              FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_projects_path ON projects(path);
            CREATE INDEX IF NOT EXISTS idx_projects_type ON projects(type);
            CREATE INDEX IF NOT EXISTS idx_metrics_size ON metrics(size_bytes);
            CREATE INDEX IF NOT EXISTS idx_metrics_last_edit ON metrics(last_edited_at);
            CREATE INDEX IF NOT EXISTS idx_metrics_loc ON metrics(loc);

            -- git_info table for enrichment
            CREATE TABLE IF NOT EXISTS git_info (
              project_id INTEGER PRIMARY KEY,
              last_commit_at INTEGER,
              branch TEXT,
              remote_url TEXT,
              FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS idx_git_last_commit ON git_info(last_commit_at);

            -- per-language LOC breakdown (optional)
            CREATE TABLE IF NOT EXISTS loc_lang (
              project_id INTEGER NOT NULL,
              language TEXT NOT NULL,
              code INTEGER,
              PRIMARY KEY(project_id, language),
              FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
            );
        "#,
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    fn ensure_column(&self, table: &str, col: &str, ty: &str) -> Result<()> {
        let mut stmt = self.conn.prepare(&format!("PRAGMA table_info({table})"))?;
        let mut exists = false;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?; // name column
            if name == col {
                exists = true;
                break;
            }
        }
        if !exists {
            let sql = format!("ALTER TABLE {table} ADD COLUMN {col} {ty}");
            let _ = self.conn.execute(&sql, [])?;
        }
        Ok(())
    }

    pub fn upsert_project(
        &self,
        name: &str,
        path: &str,
        project_type: Option<&str>,
        is_git_repo: bool,
    ) -> Result<i64> {
        self.conn.execute(
            r#"
            INSERT INTO projects (name, path, type, is_git_repo, updated_at)
            VALUES (?1, ?2, ?3, ?4, strftime('%s','now'))
            ON CONFLICT(path) DO UPDATE SET
              name=excluded.name,
              type=excluded.type,
              is_git_repo=excluded.is_git_repo,
              updated_at=strftime('%s','now')
        "#,
            params![name, path, project_type, is_git_repo as i32],
        )?;

        let id: i64 = self.conn.query_row(
            "SELECT id FROM projects WHERE path=?1",
            params![path],
            |row| row.get(0),
        )?;
        Ok(id)
    }

    pub fn upsert_metrics(
        &self,
        project_id: i64,
        size_bytes: Option<i64>,
        files_count: Option<i64>,
        last_edited_at: Option<i64>,
        loc: Option<i64>,
    ) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO metrics (project_id, size_bytes, files_count, last_edited_at, loc)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(project_id) DO UPDATE SET
              size_bytes=excluded.size_bytes,
              files_count=excluded.files_count,
              last_edited_at=excluded.last_edited_at,
              loc=excluded.loc
        "#,
            params![project_id, size_bytes, files_count, last_edited_at, loc],
        )?;
        Ok(())
    }

    pub fn upsert_git_info(
        &self,
        project_id: i64,
        last_commit_at: Option<i64>,
        branch: Option<&str>,
        remote_url: Option<&str>,
    ) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO git_info (project_id, last_commit_at, branch, remote_url)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(project_id) DO UPDATE SET
              last_commit_at=excluded.last_commit_at,
              branch=excluded.branch,
              remote_url=excluded.remote_url
        "#,
            params![project_id, last_commit_at, branch, remote_url],
        )?;
        Ok(())
    }

    pub fn list_projects(&self, sort: SortKey, limit: usize) -> Result<Vec<ProjectRecord>> {
        let order = match sort {
            // Emulate NULLS LAST via CASE
            SortKey::Recent => {
                "CASE WHEN m.last_edited_at IS NULL THEN 1 ELSE 0 END, m.last_edited_at DESC"
            }
            SortKey::Size => "CASE WHEN m.size_bytes IS NULL THEN 1 ELSE 0 END, m.size_bytes DESC",
            SortKey::Name => "p.name ASC",
            SortKey::Type => "p.type ASC, p.name ASC",
            SortKey::Loc => "CASE WHEN m.loc IS NULL THEN 1 ELSE 0 END, m.loc DESC",
        };
        let mut stmt = self.conn.prepare(&format!(
            r#"
            SELECT p.id, p.name, p.path, p.type, p.is_git_repo,
                   m.size_bytes, m.files_count, m.last_edited_at, m.loc
            FROM projects p
            LEFT JOIN metrics m ON m.project_id = p.id
            ORDER BY {order}
            LIMIT ?1
        "#
        ))?;
        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(ProjectRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    project_type: row.get(3)?,
                    is_git_repo: {
                        let v: i64 = row.get(4)?;
                        v != 0
                    },
                    size_bytes: row.get(5)?,
                    files_count: row.get(6)?,
                    last_edited_at: row.get(7)?,
                    loc: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn count_projects(&self, search: Option<&str>) -> Result<u32> {
        let mut sql = String::from("SELECT COUNT(*) FROM projects p");
        let mut params_vec: Vec<String> = Vec::new();
        
        if let Some(q) = search {
            sql.push_str(" WHERE p.name LIKE ?1 OR p.path LIKE ?1");
            params_vec.push(format!("%{q}%"));
        }
        
        let count: i64 = if params_vec.is_empty() {
            self.conn.query_row(&sql, [], |row| row.get(0))?
        } else {
            self.conn.query_row(&sql, [&params_vec[0]], |row| row.get(0))?
        };
        
        Ok(count as u32)
    }

    pub fn query_projects(
        &self,
        search: Option<&str>,
        sort: SortKey,
        ascending: bool,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<ProjectRecord>> {
        let direction = if ascending { "ASC" } else { "DESC" };
        let order = match sort {
            SortKey::Recent => {
                format!("CASE WHEN m.last_edited_at IS NULL THEN 1 ELSE 0 END, m.last_edited_at {}", direction)
            }
            SortKey::Size => format!("CASE WHEN m.size_bytes IS NULL THEN 1 ELSE 0 END, m.size_bytes {}", direction),
            SortKey::Name => format!("p.name {}", direction),
            SortKey::Type => format!("p.type {}, p.name {}", direction, direction),
            SortKey::Loc => format!("CASE WHEN m.loc IS NULL THEN 1 ELSE 0 END, m.loc {}", direction),
        };
        let mut sql = String::from(
            "SELECT p.id, p.name, p.path, p.type, p.is_git_repo,\n                   m.size_bytes, m.files_count, m.last_edited_at, m.loc\n             FROM projects p LEFT JOIN metrics m ON m.project_id = p.id",
        );
        let mut params_vec: Vec<String> = Vec::new();
        let mut has_where = false;
        if let Some(q) = search {
            sql.push_str(" WHERE p.name LIKE ?1 OR p.path LIKE ?1");
            params_vec.push(format!("%{q}%"));
            has_where = true;
        }
        // Append ORDER/LIMIT/OFFSET; adjust indices based on whether a search param is present.
        let lim_idx = if has_where { 2 } else { 1 };
        let off_idx = lim_idx + 1;
        sql.push_str(&format!(
            " ORDER BY {order} LIMIT ?{lim_idx} OFFSET ?{off_idx}"
        ));

        let mut stmt = self.conn.prepare(&sql)?;

        // Build final params list as rusqlite params! requires concrete types.
        let limit_i = page_size as i64;
        let offset_i = (page as i64) * (page_size as i64);

        let rows = if has_where {
            let mapped =
                stmt.query_map(params![params_vec[0].as_str(), limit_i, offset_i], |row| {
                    Ok(ProjectRecord {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        path: row.get(2)?,
                        project_type: row.get(3)?,
                        is_git_repo: {
                            let v: i64 = row.get(4)?;
                            v != 0
                        },
                        size_bytes: row.get(5)?,
                        files_count: row.get(6)?,
                        last_edited_at: row.get(7)?,
                        loc: row.get(8)?,
                    })
                })?;
            mapped.collect::<Result<Vec<_>, _>>()?
        } else {
            let mapped = stmt.query_map(params![limit_i, offset_i], |row| {
                Ok(ProjectRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    project_type: row.get(3)?,
                    is_git_repo: {
                        let v: i64 = row.get(4)?;
                        v != 0
                    },
                    size_bytes: row.get(5)?,
                    files_count: row.get(6)?,
                    last_edited_at: row.get(7)?,
                    loc: row.get(8)?,
                })
            })?;
            mapped.collect::<Result<Vec<_>, _>>()?
        };
        Ok(rows)
    }

    pub fn replace_loc_breakdown(
        &self,
        project_id: i64,
        lang_code_pairs: &[(String, i64)],
    ) -> Result<()> {
        // Execute without an explicit transaction to avoid needing &mut self here.
        self.conn.execute(
            "DELETE FROM loc_lang WHERE project_id = ?1",
            params![project_id],
        )?;
        let mut stmt = self
            .conn
            .prepare("INSERT INTO loc_lang (project_id, language, code) VALUES (?1, ?2, ?3)")?;
        for (lang, code) in lang_code_pairs {
            stmt.execute(params![project_id, lang, *code])?;
        }
        Ok(())
    }
}
