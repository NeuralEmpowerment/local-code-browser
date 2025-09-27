use indexer::{
    detect::{detect_project_type, ProjectType},
    scan_roots, AppConfig, Db, ScanOptions,
};
use std::fs;
use std::io::Write;

#[test]
fn scans_minimal_node_project() {
    let dir = tempfile::tempdir().unwrap();
    let proj = dir.path().join("my-node");
    fs::create_dir_all(&proj).unwrap();
    fs::write(proj.join("package.json"), "{\"name\":\"x\"}").unwrap();
    // add a file to contribute size and mtime
    let mut f = fs::File::create(proj.join("index.js")).unwrap();
    writeln!(f, "console.log('hi');").unwrap();

    let db_path = dir.path().join("db.sqlite");
    let db = Db::open(&db_path).unwrap();

    let cfg = AppConfig {
        roots: vec![dir.path().to_path_buf()],
        ..Default::default()
    };

    let n = scan_roots(&db, &cfg, &ScanOptions { dry_run: false }).unwrap();
    assert_eq!(n, 1);

    let rows = db.list_projects(indexer::SortKey::Recent, 10).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "my-node");
    assert_eq!(rows[0].project_type.as_deref(), Some("node"));
    assert!(rows[0].size_bytes.unwrap_or(0) > 0);
}

#[test]
fn detects_terraform_project() {
    let dir = tempfile::tempdir().unwrap();
    let proj = dir.path().join("terraform-project");
    fs::create_dir_all(&proj).unwrap();

    // Create a main.tf file
    fs::write(
        proj.join("main.tf"),
        "resource \"aws_instance\" \"example\" {}",
    )
    .unwrap();

    let project_type = detect_project_type(&proj);
    assert_eq!(project_type, Some(ProjectType::Terraform));
}

#[test]
fn detects_ansible_project() {
    let dir = tempfile::tempdir().unwrap();
    let proj = dir.path().join("ansible-project");
    fs::create_dir_all(&proj).unwrap();

    // Create an ansible directory with a playbook
    fs::create_dir_all(proj.join("ansible")).unwrap();
    fs::write(
        proj.join("ansible").join("playbook.yml"),
        "- hosts: all\n  tasks:\n    - debug: msg=hello",
    )
    .unwrap();

    let project_type = detect_project_type(&proj);
    assert_eq!(project_type, Some(ProjectType::Ansible));
}
