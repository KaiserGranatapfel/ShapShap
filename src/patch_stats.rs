// Enhanced patch statistics and diff analysis

use git2::Diff;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FileStats {
    pub path: String,
    pub insertions: usize,
    pub deletions: usize,
    pub hunks: usize,
}

#[derive(Debug, Clone)]
pub struct PatchStats {
    pub total_files: usize,
    pub total_insertions: usize,
    pub total_deletions: usize,
    pub total_hunks: usize,
    pub file_stats: Vec<FileStats>,
    pub binary_files: Vec<String>,
}

impl PatchStats {
    pub fn new() -> Self {
        PatchStats {
            total_files: 0,
            total_insertions: 0,
            total_deletions: 0,
            total_hunks: 0,
            file_stats: Vec::new(),
            binary_files: Vec::new(),
        }
    }

    pub fn from_diff(diff: &Diff) -> Result<Self, git2::Error> {
        let mut stats = PatchStats::new();
        let mut file_stats_map: HashMap<String, FileStats> = HashMap::new();
        let mut line_stats: HashMap<String, (usize, usize, usize)> = HashMap::new();

        // Collect file information and line statistics in one pass
        diff.print(git2::DiffFormat::Patch, |delta, hunk, line| {
            let old_path = delta.old_file().path()
                .and_then(|p| p.to_str())
                .unwrap_or("unknown")
                .to_string();
            let new_path = delta.new_file().path()
                .and_then(|p| p.to_str())
                .unwrap_or("unknown")
                .to_string();

            let path = if old_path == new_path {
                old_path.clone()
            } else {
                format!("{} => {}", old_path, new_path)
            };

            // Initialize file stat if needed
            if !file_stats_map.contains_key(&path) {
                let old_id = delta.old_file().id();
                let new_id = delta.new_file().id();
                if old_id.is_zero() || new_id.is_zero() {
                    stats.binary_files.push(path.clone());
                }
                
                file_stats_map.insert(path.clone(), FileStats {
                    path: path.clone(),
                    insertions: 0,
                    deletions: 0,
                    hunks: 0,
                });
                
                line_stats.insert(path.clone(), (0, 0, 0));
            }

            // Count hunks (when we see a hunk header)
            if let Some(_) = hunk {
                if let Some(file_stat) = file_stats_map.get_mut(&path) {
                    file_stat.hunks += 1;
                    stats.total_hunks += 1;
                }
            }

            // Count lines
            if let Some((ins, del, _)) = line_stats.get_mut(&path) {
                match line.origin() {
                    '+' => {
                        *ins += 1;
                        stats.total_insertions += 1;
                    }
                    '-' => {
                        *del += 1;
                        stats.total_deletions += 1;
                    }
                    _ => {}
                }
            }
            
            true
        })?;

        // Merge line stats into file stats
        for (path, (insertions, deletions, _)) in line_stats {
            if let Some(file_stat) = file_stats_map.get_mut(&path) {
                file_stat.insertions = insertions;
                file_stat.deletions = deletions;
            }
        }

        stats.file_stats = file_stats_map.into_values().collect();
        stats.total_files = stats.file_stats.len();

        Ok(stats)
    }

    pub fn format_stat_line(&self) -> String {
        let mut output = String::new();

        for file_stat in &self.file_stats {
            let changes = file_stat.insertions + file_stat.deletions;
            output.push_str(&format!(
                " {} | {} +{} -{}\n",
                file_stat.path,
                changes,
                file_stat.insertions,
                file_stat.deletions
            ));
        }

        if !self.file_stats.is_empty() {
            output.push_str(&format!(
                " {} file{} changed, {} insertion{}(+), {} deletion{}(-)\n",
                self.total_files,
                if self.total_files == 1 { "" } else { "s" },
                self.total_insertions,
                if self.total_insertions == 1 { "" } else { "s" },
                self.total_deletions,
                if self.total_deletions == 1 { "" } else { "s" }
            ));
        }

        output
    }

    pub fn print_summary(&self) {
        println!("\n📊 Patch Statistics:");
        println!("  Files changed: {}", self.total_files);
        println!("  Insertions:    +{}", self.total_insertions);
        println!("  Deletions:     -{}", self.total_deletions);
        println!("  Hunks:         {}", self.total_hunks);

        if !self.binary_files.is_empty() {
            println!("\n  Binary files:");
            for file in &self.binary_files {
                println!("    - {}", file);
            }
        }

        if !self.file_stats.is_empty() {
            println!("\n  File details:");
            for file_stat in &self.file_stats {
                println!(
                    "    {}: +{} -{} ({} hunks)",
                    file_stat.path,
                    file_stat.insertions,
                    file_stat.deletions,
                    file_stat.hunks
                );
            }
        }
    }
}
