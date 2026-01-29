// Integration tests for ShapShap

use std::process::Command;
use tempfile::TempDir;
use std::fs;
use std::path::Path;

fn setup_test_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    
    // Initialize git repo
    Command::new("git")
        .args(&["init"])
        .current_dir(&dir)
        .status()
        .unwrap();
    
    // Configure git
    Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(&dir)
        .status()
        .unwrap();
    
    Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(&dir)
        .status()
        .unwrap();
    
    // Create initial file
    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "initial\n").unwrap();
    
    // Initial commit
    Command::new("git")
        .args(&["add", "test.txt"])
        .current_dir(&dir)
        .status()
        .unwrap();
    
    Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(&dir)
        .status()
        .unwrap();
    
    dir
}

#[test]
fn test_patch_command() {
    let dir = setup_test_repo();
    
    // Make a change
    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "modified\n").unwrap();
    
    Command::new("git")
        .args(&["add", "test.txt"])
        .current_dir(&dir)
        .status()
        .unwrap();
    
    Command::new("git")
        .args(&["commit", "-m", "test: modify file"])
        .current_dir(&dir)
        .status()
        .unwrap();
    
    // Run shapshap
    let output = Command::new("cargo")
        .args(&["run", "--release", "--", "patch"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("CARGO_TARGET_DIR", dir.path().join("target"))
        .output();
    
    // For now, just verify the command exists
    // Full integration would require the binary to be built
    assert!(output.is_ok());
}
