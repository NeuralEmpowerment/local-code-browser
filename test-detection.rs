use std::path::Path;
use indexer::detect::{detect_project_type, ProjectType};

fn main() {
    let test_dirs = vec![
        "test-repo",
        "test-repo/terraform",
        "test-repo/terraform/aws",
        "test-repo/ansible",
    ];

    for dir in test_dirs {
        let path = Path::new(dir);
        if let Some(project_type) = detect_project_type(path) {
            println!("{}: {:?}", dir, project_type.as_str());
        } else {
            println!("{}: None", dir);
        }
    }
}
