#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use std::process::Command;

    fn setup_test_repo() -> (TempDir, git2::Repository) {
        let dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(&dir).unwrap();
        
        // Create initial commit
        let mut index = repo.index().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "initial content\n").unwrap();
        index.add_path(&file_path.strip_prefix(dir.path()).unwrap()).unwrap();
        index.write().unwrap();
        
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[]).unwrap();
        
        (dir, repo)
    }

    #[test]
    fn test_patch_generation() {
        let (_dir, repo) = setup_test_repo();
        
        // Make a change
        let file_path = _dir.path().join("test.txt");
        fs::write(&file_path, "modified content\n").unwrap();
        
        let mut index = repo.index().unwrap();
        index.add_path(&file_path.strip_prefix(_dir.path()).unwrap()).unwrap();
        index.write().unwrap();
        
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        let parent = repo.head().unwrap().peel_to_commit().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "test: modify file", &tree, &[&parent]).unwrap();
        
        let commit = repo.head().unwrap().peel_to_commit().unwrap();
        let patch = generate_patch(&repo, &commit, "PATCH", 1, 1, &None, &None).unwrap();
        
        assert!(patch.contains("From:"));
        assert!(patch.contains("Subject:"));
        assert!(patch.contains("diff --git"));
        assert!(patch.contains("Signed-off-by:"));
    }

    #[test]
    fn test_validation() {
        use validation::PatchValidator;
        
        let validator = PatchValidator::new();
        let patch_content = r#"From abc123 Mon Sep 17 00:00:00 2001
From: Test <test@example.com>
Date: Mon, 01 Jan 2024 12:00:00 +0000
Subject: [PATCH] test: add feature

This is a test patch.

---
diff --git a/test.txt b/test.txt
index 1234567..abcdefg 100644
--- a/test.txt
+++ b/test.txt
@@ -1 +1,2 @@
 line1
+line2

Signed-off-by: Test <test@example.com>
"#;
        
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), patch_content).unwrap();
        
        let result = validator.validate_patch_file(temp_file.path()).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_asm_utils() {
        use asm_utils::{fast_sanitize_string, calculate_patch_checksum, fast_count_lines};
        
        // Test string sanitization
        let input = "test/file-name.txt";
        let output = fast_sanitize_string(input);
        assert_eq!(output, "test/file-name.txt");
        
        // Test checksum
        let data = b"test data";
        let checksum = calculate_patch_checksum(data);
        assert!(checksum > 0);
        
        // Test line counting
        let data = b"line1\nline2\nline3";
        let count = fast_count_lines(data);
        assert_eq!(count, 2);
    }
}
