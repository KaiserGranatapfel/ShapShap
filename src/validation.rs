// Patch validation and checking utilities

use anyhow::{Context, Result};
use git2::Commit;
use regex::Regex;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, msg: String) {
        self.is_valid = false;
        self.errors.push(msg);
    }

    pub fn add_warning(&mut self, msg: String) {
        self.warnings.push(msg);
    }

    pub fn print(&self) {
        if self.is_valid && self.warnings.is_empty() {
            println!("✓ Validation passed");
            return;
        }

        if !self.errors.is_empty() {
            eprintln!("✗ Validation errors:");
            for error in &self.errors {
                eprintln!("  - {}", error);
            }
        }

        if !self.warnings.is_empty() {
            println!("⚠ Warnings:");
            for warning in &self.warnings {
                println!("  - {}", warning);
            }
        }
    }
}

pub struct PatchValidator {
    check_line_length: bool,
    max_line_length: usize,
    check_trailing_whitespace: bool,
    check_tab_indentation: bool,
    require_sign_off: bool,
    check_commit_message_format: bool,
}

impl PatchValidator {
    pub fn new() -> Self {
        PatchValidator {
            check_line_length: true,
            max_line_length: 80,
            check_trailing_whitespace: true,
            check_tab_indentation: true,
            require_sign_off: true,
            check_commit_message_format: true,
        }
    }

    pub fn with_line_length_check(mut self, check: bool, max: usize) -> Self {
        self.check_line_length = check;
        self.max_line_length = max;
        self
    }

    pub fn with_trailing_whitespace_check(mut self, check: bool) -> Self {
        self.check_trailing_whitespace = check;
        self
    }

    pub fn with_tab_indentation_check(mut self, check: bool) -> Self {
        self.check_tab_indentation = check;
        self
    }

    pub fn with_sign_off_requirement(mut self, require: bool) -> Self {
        self.require_sign_off = require;
        self
    }

    pub fn with_commit_message_check(mut self, check: bool) -> Self {
        self.check_commit_message_format = check;
        self
    }

    pub fn validate_commit(&self, commit: &Commit) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Check commit message format
        if self.check_commit_message_format {
            if let Some(message) = commit.message() {
                let lines: Vec<&str> = message.lines().collect();
                
                // Check subject line length
                if let Some(subject) = lines.first() {
                    if subject.len() > 72 {
                        result.add_warning(format!(
                            "Subject line is {} characters (recommended: <= 72)",
                            subject.len()
                        ));
                    }
                    
                    // Check for proper prefix
                    if !subject.contains(':') {
                        result.add_warning(
                            "Subject line should follow 'subsystem: description' format".to_string()
                        );
                    }
                }

                // Check for empty line after subject
                if lines.len() > 1 && !lines[1].trim().is_empty() {
                    result.add_warning(
                        "Commit message should have empty line after subject".to_string()
                    );
                }
            }
        }

        // Check for sign-off
        if self.require_sign_off {
            if let Some(message) = commit.message() {
                if !message.contains("Signed-off-by:") {
                    result.add_error("Missing Signed-off-by line (required for kernel patches)".to_string());
                }
            }
        }

        result
    }

    pub fn validate_patch_file(&self, patch_path: &Path) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();
        let content = std::fs::read_to_string(patch_path)
            .with_context(|| format!("Failed to read patch file: {}", patch_path.display()))?;

        let lines: Vec<&str> = content.lines().collect();

        // Check for required headers
        let has_from = content.contains("From:");
        let has_subject = content.contains("Subject:");
        let has_diff = content.contains("diff --git") || content.contains("---");

        if !has_from {
            result.add_error("Missing 'From:' header".to_string());
        }
        if !has_subject {
            result.add_error("Missing 'Subject:' header".to_string());
        }
        if !has_diff {
            result.add_error("Missing diff content".to_string());
        }

        // Check line lengths
        if self.check_line_length {
            for (line_num, line) in lines.iter().enumerate() {
                // Allow longer lines in diff hunks
                if !line.starts_with("diff --git")
                    && !line.starts_with("index ")
                    && !line.starts_with("---")
                    && !line.starts_with("+++")
                    && !line.starts_with("@@")
                    && !line.starts_with("+")
                    && !line.starts_with("-")
                    && !line.starts_with(" ")
                {
                    if line.len() > self.max_line_length {
                        result.add_warning(format!(
                            "Line {} exceeds {} characters (length: {})",
                            line_num + 1,
                            self.max_line_length,
                            line.len()
                        ));
                    }
                }
            }
        }

        // Check for trailing whitespace
        if self.check_trailing_whitespace {
            for (line_num, line) in lines.iter().enumerate() {
                if *line != line.trim_end() {
                    result.add_warning(format!(
                        "Line {} has trailing whitespace",
                        line_num + 1
                    ));
                }
            }
        }

        // Check for sign-off
        if self.require_sign_off {
            if !content.contains("Signed-off-by:") {
                result.add_error("Missing Signed-off-by line".to_string());
            } else {
                // Validate sign-off format
                let sign_off_re = Regex::new(r"Signed-off-by:\s*[^<]+\s*<[^>]+>")
                    .expect("Invalid regex");
                if !sign_off_re.is_match(&content) {
                    result.add_error("Invalid Signed-off-by format".to_string());
                }
            }
        }

        // Check for proper diff format
        if has_diff {
            let diff_start = content.find("---").unwrap_or(0);
            let before_diff = &content[..diff_start];
            
            if !before_diff.contains("---\n") && !before_diff.contains("---\r\n") {
                result.add_warning("Missing '---' separator before diff".to_string());
            }
        }

        Ok(result)
    }

    pub fn validate_diff_content(&self, diff_content: &str) -> ValidationResult {
        let mut result = ValidationResult::new();
        let lines: Vec<&str> = diff_content.lines().collect();

        // Check for tab indentation in C/Rust files
        if self.check_tab_indentation {
            let c_file_extensions = ["c", "h", "rs"];
            let mut in_c_file = false;

            for line in &lines {
                if line.starts_with("diff --git") {
                    in_c_file = c_file_extensions.iter().any(|ext| line.contains(&format!(".{}", ext)));
                }

                if in_c_file && (line.starts_with("+") || line.starts_with("-")) {
                    let code_line = &line[1..];
                    if code_line.starts_with("    ") && !code_line.starts_with("\t") {
                        result.add_warning(
                            "C/Rust files should use tabs for indentation, not spaces".to_string()
                        );
                    }
                }
            }
        }

        result
    }
}

impl Default for PatchValidator {
    fn default() -> Self {
        Self::new()
    }
}
