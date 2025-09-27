use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use semver::Version;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "version-manager")]
#[command(about = "A tool to manage versions across multiple files in a Tauri project")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Bump version by type (major, minor, patch)
    Bump {
        /// Version bump type: major, minor, or patch
        #[arg(value_enum)]
        bump_type: BumpType,

        /// Commit changes after bumping
        #[arg(short, long)]
        commit: bool,

        /// Create git tag after bumping
        #[arg(short, long)]
        tag: bool,
    },
    /// Check if versions are synchronized across all files
    Check,
    /// Show current versions from all files
    Show,
}

#[derive(clap::ValueEnum, Clone)]
enum BumpType {
    Major,
    Minor,
    Patch,
}

#[derive(Deserialize)]
struct CargoToml {
    package: Option<CargoPackage>,
}

#[derive(Deserialize)]
struct CargoPackage {
    version: Option<String>,
}

#[derive(Deserialize)]
struct PackageJson {
    version: Option<String>,
}

#[derive(Deserialize)]
struct TauriConfig {
    version: Option<String>,
}

#[derive(Debug)]
struct VersionFile {
    path: String,
    version: Option<Version>,
    file_type: FileType,
}

#[derive(Debug, PartialEq)]
enum FileType {
    CargoToml,
    PackageJson,
    TauriConfig,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Bump {
            bump_type,
            commit,
            tag,
        } => {
            bump_version(bump_type, commit, tag)?;
        }
        Commands::Check => {
            check_version_sync()?;
        }
        Commands::Show => {
            show_versions()?;
        }
    }

    Ok(())
}

fn get_version_files() -> Result<Vec<VersionFile>> {
    let mut files = Vec::new();

    // Cargo.toml files
    let cargo_files = vec![
        "src-tauri/Cargo.toml",
        "crates/cli/Cargo.toml",
        "crates/indexer/Cargo.toml",
    ];

    for cargo_file in cargo_files {
        let path = Path::new(cargo_file);
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let cargo_toml: CargoToml = toml::from_str(&content)
                .with_context(|| format!("Failed to parse {cargo_file}"))?;

            let version = cargo_toml
                .package
                .and_then(|p| p.version)
                .and_then(|v| Version::parse(&v).ok());

            files.push(VersionFile {
                path: cargo_file.to_string(),
                version,
                file_type: FileType::CargoToml,
            });
        }
    }

    // package.json
    let package_json_path = "web/package.json";
    if Path::new(package_json_path).exists() {
        let content = fs::read_to_string(package_json_path)?;
        let package_json: PackageJson = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {package_json_path}"))?;

        let version = package_json.version.and_then(|v| Version::parse(&v).ok());

        files.push(VersionFile {
            path: package_json_path.to_string(),
            version,
            file_type: FileType::PackageJson,
        });
    }

    // tauri.conf.json
    let tauri_config_path = "src-tauri/tauri.conf.json";
    if Path::new(tauri_config_path).exists() {
        let content = fs::read_to_string(tauri_config_path)?;
        let tauri_config: TauriConfig = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse {tauri_config_path}"))?;

        let version = tauri_config.version.and_then(|v| Version::parse(&v).ok());

        files.push(VersionFile {
            path: tauri_config_path.to_string(),
            version,
            file_type: FileType::TauriConfig,
        });
    }

    Ok(files)
}

fn show_versions() -> Result<()> {
    let files = get_version_files()?;

    println!("{}", "Current versions:".green().bold());
    println!("{}", "==================".green().bold());

    for file in files {
        match file.version {
            Some(version) => {
                println!("{}: {}", file.path.cyan(), version.to_string().yellow());
            }
            None => {
                println!("{}: {}", file.path.cyan(), "No version found".red());
            }
        }
    }

    Ok(())
}

fn check_version_sync() -> Result<()> {
    let files = get_version_files()?;

    // Extract versions that exist
    let versions: Vec<(&Version, &String)> = files
        .iter()
        .filter_map(|f| f.version.as_ref().map(|v| (v, &f.path)))
        .collect();

    if versions.is_empty() {
        println!("{}", "No versions found in any files!".red().bold());
        return Ok(());
    }

    // Check if all versions are the same
    let first_version = versions[0].0;
    let all_same = versions.iter().all(|(v, _)| v == &first_version);

    if all_same {
        println!("{}", "✅ All versions are synchronized!".green().bold());
        println!("Version: {}", first_version.to_string().yellow());

        for (_, path) in versions {
            println!("  {}", path.cyan());
        }
    } else {
        println!(
            "{}",
            "❌ Version synchronization issues found!".red().bold()
        );
        println!("{}", "=====================================".red().bold());

        // Group by version
        let mut version_groups: HashMap<String, Vec<String>> = HashMap::new();
        for (version, path) in versions {
            version_groups
                .entry(version.to_string())
                .or_default()
                .push(path.clone());
        }

        for (version, paths) in version_groups {
            if paths.len() == 1 {
                println!("{} ({} file):", version.yellow(), paths.len());
            } else {
                println!("{} ({} files):", version.yellow(), paths.len());
            }
            for path in paths {
                println!("  {}", path.cyan());
            }
            println!();
        }
    }

    Ok(())
}

fn bump_version(bump_type: BumpType, commit: bool, tag: bool) -> Result<()> {
    let mut files = get_version_files()?;

    // Find the current version (use the first one we find)
    let current_version = files
        .iter()
        .find_map(|f| f.version.as_ref())
        .context("No version found in any file")?
        .clone();

    let new_version = match bump_type {
        BumpType::Major => Version::new(current_version.major + 1, 0, 0),
        BumpType::Minor => Version::new(current_version.major, current_version.minor + 1, 0),
        BumpType::Patch => Version::new(
            current_version.major,
            current_version.minor,
            current_version.patch + 1,
        ),
    };

    println!("{}", "Version Bump Summary:".green().bold());
    println!("Current version: {}", current_version.to_string().red());
    println!("New version: {}", new_version.to_string().green());
    println!();

    // Update each file
    for file in &mut files {
        if file.version.is_none() {
            println!("⚠️  Skipping {} (no version found)", file.path.cyan());
            continue;
        }

        println!("Updating {}...", file.path.cyan());

        match file.file_type {
            FileType::CargoToml => {
                update_cargo_toml(&file.path, &new_version)?;
            }
            FileType::PackageJson => {
                update_package_json(&file.path, &new_version)?;
            }
            FileType::TauriConfig => {
                update_tauri_config(&file.path, &new_version)?;
            }
        }

        println!("  ✅ Updated to {}", new_version.to_string().green());
    }

    println!();

    // Commit changes if requested
    if commit {
        println!("Committing changes...");
        run_command("git", &["add", "."])?;
        let commit_msg = format!("chore: bump version from {current_version} to {new_version}");
        run_command("git", &["commit", "-m", &commit_msg])?;
        println!("  ✅ Changes committed");
    }

    // Create tag if requested
    if tag {
        println!("Creating git tag...");
        let tag_name = format!("v{new_version}");
        let tag_msg = format!("Version {new_version}: Version bump");
        run_command("git", &["tag", "-a", &tag_name, "-m", &tag_msg])?;
        println!("  ✅ Tag {} created", tag_name.green());
    }

    println!();
    println!(
        "{}",
        "Version bump completed successfully! 🎉".green().bold()
    );

    Ok(())
}

fn update_cargo_toml(path: &str, new_version: &Version) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let re = regex::Regex::new(r#"version\s*=\s*"([^"]+)""#)?;
    let new_content = re.replace(&content, format!("version = \"{new_version}\""));
    fs::write(path, new_content.as_bytes())?;
    Ok(())
}

fn update_package_json(path: &str, new_version: &Version) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let re = regex::Regex::new(r#""version"\s*:\s*"([^"]+)""#)?;
    let new_content = re.replace(&content, format!("\"version\": \"{new_version}\""));
    fs::write(path, new_content.as_bytes())?;
    Ok(())
}

fn update_tauri_config(path: &str, new_version: &Version) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let re = regex::Regex::new(r#""version"\s*:\s*"([^"]+)""#)?;
    let new_content = re.replace(&content, format!("\"version\": \"{new_version}\""));
    fs::write(path, new_content.as_bytes())?;
    Ok(())
}

fn run_command(program: &str, args: &[&str]) -> Result<()> {
    use std::process::Command;

    let output = Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run {program} {args:?}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        anyhow::bail!(
            "Command failed: {}\nstderr: {}\nstdout: {}",
            program,
            stderr,
            stdout
        );
    }

    Ok(())
}
