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
    Terraform,
    Ansible,
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
            ProjectType::Terraform => "terraform",
            ProjectType::Ansible => "ansible",
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
        (ProjectType::Terraform, &["main.tf", "variables.tf", "outputs.tf"][..]),
        (ProjectType::Ansible, &[]), // Special case - handled below
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

    // Special detection for Ansible
    let ansible_dir = dir.join("ansible");
    if ansible_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&ansible_dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "yml" || ext == "yaml" {
                        return Some(ProjectType::Ansible);
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
