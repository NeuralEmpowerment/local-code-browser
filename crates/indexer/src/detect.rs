use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    Rust,
    NodeJs,
    Python,
    Go,
    Java,
    DotNet,
    Other,
}

impl ProjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProjectType::Rust => "rust",
            ProjectType::NodeJs => "node",
            ProjectType::Python => "python",
            ProjectType::Go => "go",
            ProjectType::Java => "java",
            ProjectType::DotNet => ".net",
            ProjectType::Other => "other",
        }
    }
}

pub fn detect_project_type(dir: &Path) -> Option<ProjectType> {
    // Markers per language/ecosystem
    let candidates = [
        (ProjectType::Rust, &["Cargo.toml"][..]),
        (ProjectType::NodeJs, &["package.json"][..]),
        (ProjectType::Python, &["pyproject.toml", "requirements.txt"]),
        (ProjectType::Go, &["go.mod"][..]),
        (ProjectType::Java, &["pom.xml", "build.gradle", "gradlew"]),
        (ProjectType::DotNet, &["global.json"][..]),
    ];

    for (ptype, files) in candidates.iter() {
        if files.iter().any(|f| dir.join(f).exists()) {
            return Some(*ptype);
        }
        // .NET: also check for *.csproj
        if matches!(ptype, ProjectType::DotNet) {
            if let Ok(rd) = fs::read_dir(dir) {
                for entry in rd.flatten() {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "csproj" {
                            return Some(ProjectType::DotNet);
                        }
                    }
                }
            }
        }
    }
    None
}

pub fn is_git_repo(dir: &Path) -> bool {
    dir.join(".git").is_dir()
}
