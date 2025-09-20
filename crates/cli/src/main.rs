use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use indexer::{scan_roots, ConfigStore, Db, ScanOptions, SortKey};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(author, version, about = "Project Browser CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Print or show config
    Config {
        /// Print the effective config as JSON
        #[arg(long)]
        print: bool,
        /// Print the default DB path
        #[arg(long)]
        db_path: bool,
    },
    /// Scan roots and populate the database
    Scan {
        /// Optional roots (defaults to config roots). Repeatable.
        #[arg(long)]
        root: Vec<String>,
        /// Dry run without writing to the DB
        #[arg(long)]
        dry_run: bool,
        /// Override database path
        #[arg(long)]
        db: Option<String>,
    },
    /// List projects from the database
    List {
        /// Sort key
        #[arg(long, value_enum, default_value_t = ListSort::Recent)]
        sort: ListSort,
        /// Max rows
        #[arg(long, default_value_t = 100)]
        limit: usize,
        /// Output JSON instead of table
        #[arg(long)]
        json: bool,
        /// Override database path
        #[arg(long)]
        db: Option<String>,
        /// Show LOC column in text output
        #[arg(long)]
        show_loc: bool,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum ListSort {
    Recent,
    Size,
    Name,
    Type,
    Loc,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Config { print, db_path } => {
            let cfg = ConfigStore::load()?;
            if print {
                println!("{}", serde_json::to_string_pretty(&cfg)?);
            } else if db_path {
                let db = Db::open_default()?;
                println!("{}", db.path.display());
            } else {
                println!("Use --print or --db-path");
            }
        }
        Commands::Scan { root, dry_run, db } => {
            let mut cfg = ConfigStore::load()?;
            if !root.is_empty() {
                cfg.roots = root
                    .into_iter()
                    .map(|s| shellexpand::tilde(&s).to_string().into())
                    .collect();
            }
            let db = if let Some(path) = db {
                let p = shellexpand::tilde(&path).to_string();
                Db::open(std::path::Path::new(&p))?
            } else {
                Db::open_default()?
            };
            let count = scan_roots(&db, &cfg, &ScanOptions { dry_run })?;
            eprintln!("Scanned {count} project(s)");
        }
        Commands::List {
            sort,
            limit,
            json,
            db,
            show_loc,
        } => {
            let db = if let Some(path) = db {
                let p = shellexpand::tilde(&path).to_string();
                Db::open(std::path::Path::new(&p))?
            } else {
                Db::open_default()?
            };
            let sort_key = match sort {
                ListSort::Recent => SortKey::Recent,
                ListSort::Size => SortKey::Size,
                ListSort::Name => SortKey::Name,
                ListSort::Type => SortKey::Type,
                ListSort::Loc => SortKey::Loc,
            };
            let rows = db.list_projects(sort_key, limit)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&rows_as_json(&rows))?);
            } else if show_loc {
                for r in rows {
                    println!(
                        "{:<24}  {:<6}  {:>10}  {:>8}  {}",
                        truncate(&r.name, 24),
                        r.project_type.clone().unwrap_or_else(|| "-".into()),
                        r.size_bytes.unwrap_or_default(),
                        r.loc.unwrap_or_default(),
                        r.path
                    );
                }
            } else {
                for r in rows {
                    println!(
                        "{:<24}  {:<6}  {:>10}  {}",
                        truncate(&r.name, 24),
                        r.project_type.clone().unwrap_or_else(|| "-".into()),
                        r.size_bytes.unwrap_or_default(),
                        r.path
                    );
                }
            }
        }
    }

    Ok(())
}

fn truncate(s: &str, width: usize) -> String {
    if s.len() <= width {
        s.to_string()
    } else {
        let mut t = s.chars().take(width.saturating_sub(1)).collect::<String>();
        t.push('…');
        t
    }
}

fn rows_as_json(rows: &[indexer::ProjectRecord]) -> serde_json::Value {
    serde_json::json!(rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "name": r.name,
                "path": r.path,
                "type": r.project_type,
                "is_git_repo": r.is_git_repo,
                "size_bytes": r.size_bytes,
                "files_count": r.files_count,
                "last_edited_at": r.last_edited_at,
                "loc": r.loc,
            })
        })
        .collect::<Vec<_>>())
}
