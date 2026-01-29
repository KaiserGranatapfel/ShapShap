mod asm_utils;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use git2::{Commit, DiffOptions, Repository};
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "shapshap")]
#[command(about = "Generate Linux kernel patches in the proper format", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a patch from git commits
    Patch {
        /// Git commit range (e.g., HEAD~1..HEAD or commit hash)
        #[arg(short, long)]
        range: Option<String>,
        /// Output directory for patch files
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
        /// Author name for sign-off
        #[arg(short, long)]
        author: Option<String>,
        /// Author email for sign-off
        #[arg(short = 'e', long)]
        email: Option<String>,
        /// Subject prefix (default: PATCH)
        #[arg(short = 'p', long, default_value = "PATCH")]
        prefix: String,
        /// Number of patches in series (for [PATCH n/m] format)
        #[arg(short = 'n', long)]
        number: Option<usize>,
        /// Total patches in series (for [PATCH n/m] format)
        #[arg(short = 'm', long)]
        total: Option<usize>,
        /// Include cover letter
        #[arg(short, long)]
        cover_letter: bool,
        /// Output patch to stdout instead of file
        #[arg(long)]
        stdout: bool,
        /// Suppress informational messages (quiet mode)
        #[arg(short = 'q', long)]
        quiet: bool,
        /// Show what would be generated without writing files
        #[arg(long)]
        dry_run: bool,
        /// Overwrite existing patch files without prompting
        #[arg(long)]
        force: bool,
        /// Custom output filename (e.g., patch.txt)
        #[arg(short = 'f', long)]
        filename: Option<String>,
        /// Use RFC prefix (shorthand for --prefix "RFC PATCH")
        #[arg(long)]
        rfc: bool,
        /// Patch version number (e.g., 2 for [PATCH v2])
        #[arg(short = 'v', long = "reroll-count")]
        version: Option<usize>,
        /// Skip adding Signed-off-by line
        #[arg(long)]
        no_signoff: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Patch {
            range,
            output,
            author,
            email,
            prefix,
            number,
            total,
            cover_letter,
            stdout,
            quiet,
            dry_run,
            force,
            filename,
            rfc,
            version,
            no_signoff,
        } => {
            let repo = Repository::open(".")
                .context("Failed to open git repository. Make sure you're in a git repo.")?;

            let commits = if let Some(range_str) = range {
                parse_commit_range(&repo, &range_str)?
            } else {
                // Default to HEAD
                vec![get_head_commit(&repo)?]
            };

            if commits.is_empty() {
                anyhow::bail!("No commits found");
            }

            // Build the effective prefix (handle --rfc and --version flags)
            let effective_prefix = build_prefix(&prefix, rfc, version);

            let total_patches = total.unwrap_or(commits.len());
            let mut patch_number = number.unwrap_or(1);

            // Generate cover letter if requested (skip in stdout/dry-run mode)
            if cover_letter && !stdout && !dry_run {
                generate_cover_letter(&output, &effective_prefix, total_patches, &commits, quiet)?;
            } else if cover_letter && dry_run && !quiet {
                println!("[dry-run] Would generate: {}", output.join("0000-cover-letter.patch").display());
            }

            // Generate patch for each commit
            for commit in commits.iter() {
                let patch_content = generate_patch(
                    &repo,
                    commit,
                    &effective_prefix,
                    patch_number,
                    total_patches,
                    &author,
                    &email,
                    !no_signoff,
                )?;

                // Determine output filename
                let out_filename = if let Some(ref custom_name) = filename {
                    if total_patches > 1 {
                        // For multiple patches with custom filename, add number prefix
                        format!("{:04}-{}", patch_number, custom_name)
                    } else {
                        custom_name.clone()
                    }
                } else {
                    generate_patch_filename(commit, patch_number, total_patches)
                };
                let filepath = output.join(&out_filename);

                if stdout {
                    // Output to stdout
                    io::stdout().write_all(patch_content.as_bytes())?;
                    if total_patches > 1 {
                        // Add separator between patches
                        println!("\n---\n");
                    }
                } else if dry_run {
                    // Dry run - just show what would be generated
                    if !quiet {
                        println!("[dry-run] Would generate: {}", filepath.display());
                        println!("[dry-run] Patch size: {} bytes", patch_content.len());
                    }
                } else {
                    // Check if file exists and handle --force flag
                    if filepath.exists() && !force {
                        anyhow::bail!(
                            "File already exists: {}. Use --force to overwrite.",
                            filepath.display()
                        );
                    }

                    let mut file = File::create(&filepath)
                        .with_context(|| format!("Failed to create patch file: {}", filepath.display()))?;
                    file.write_all(patch_content.as_bytes())
                        .with_context(|| format!("Failed to write patch file: {}", filepath.display()))?;

                    if !quiet {
                        println!("Generated: {}", filepath.display());
                    }
                }

                patch_number += 1;
            }

            if !quiet && !stdout {
                if dry_run {
                    println!("\n[dry-run] Would generate {} patch(es)", commits.len());
                } else {
                    println!("\n✓ Successfully generated {} patch(es)", commits.len());
                }
            }
        }
    }

    Ok(())
}

/// Build the effective prefix based on flags
fn build_prefix(base_prefix: &str, rfc: bool, version: Option<usize>) -> String {
    let mut prefix = if rfc {
        "RFC PATCH".to_string()
    } else {
        base_prefix.to_string()
    };

    if let Some(v) = version {
        prefix = format!("{} v{}", prefix, v);
    }

    prefix
}

fn parse_commit_range<'a>(repo: &'a Repository, range: &str) -> Result<Vec<Commit<'a>>> {
    let mut commits = Vec::new();
    let mut revwalk = repo.revwalk()?;

    if range.contains("..") {
        // Range like HEAD~3..HEAD or commit1..commit2
        let parts: Vec<&str> = range.split("..").collect();
        if parts.len() == 2 {
            let from = repo.revparse_single(parts[0])?.id();
            let to = repo.revparse_single(parts[1])?.id();
            revwalk.push(to)?;
            revwalk.hide(from)?;
        } else {
            anyhow::bail!("Invalid range format: {}", range);
        }
    } else {
        // Single commit or reference
        let revspec = repo.revparse(range)?;
        if let Some(commit) = revspec.from() {
            revwalk.push(commit.id())?;
        } else {
            anyhow::bail!("Invalid commit reference: {}", range);
        }
    }

    for oid in revwalk {
        let oid = oid?;
        if let Ok(commit) = repo.find_commit(oid) {
            commits.push(commit);
        }
    }

    Ok(commits)
}

fn get_head_commit(repo: &Repository) -> Result<Commit<'_>> {
    let head = repo.head()?;
    let oid = head.target().context("HEAD has no target")?;
    repo.find_commit(oid)
        .context("Failed to find HEAD commit")
}

fn generate_patch(
    repo: &Repository,
    commit: &Commit,
    prefix: &str,
    number: usize,
    total: usize,
    author: &Option<String>,
    email: &Option<String>,
    include_signoff: bool,
) -> Result<String> {
    let mut patch = String::new();

    // Generate From line with fixed timestamp (kernel standard)
    let author_sig = commit.author();
    let commit_time = commit.time();
    let timestamp = DateTime::<Utc>::from_timestamp(commit_time.seconds(), 0)
        .unwrap_or_else(|| Utc::now());
    
    // Fixed timestamp format: Mon Sep 17 00:00:00 2001 (kernel standard)
    patch.push_str(&format!(
        "From {} Mon Sep 17 00:00:00 2001\n",
        commit.id()
    ));
    patch.push_str(&format!(
        "From: {} <{}>\n",
        author_sig.name().unwrap_or("Unknown"),
        author_sig.email().unwrap_or("unknown@example.com")
    ));
    patch.push_str(&format!("Date: {}\n", timestamp.format("%a, %d %b %Y %H:%M:%S %z")));
    
    // Subject line
    let subject = commit
        .message()
        .and_then(|msg| msg.lines().next())
        .unwrap_or("No subject");
    
    let subject_prefix = if total > 1 {
        format!("[{prefix} {number}/{total}]", prefix = prefix, number = number, total = total)
    } else {
        format!("[{prefix}]", prefix = prefix)
    };
    
    patch.push_str(&format!("Subject: {} {}\n\n", subject_prefix, subject));

    // Commit message body (skip first line, it's the subject)
    if let Some(message) = commit.message() {
        let lines: Vec<&str> = message.lines().collect();
        if lines.len() > 1 {
            for line in lines.iter().skip(1) {
                if line.trim().is_empty() && patch.ends_with('\n') {
                    continue;
                }
                patch.push_str(line);
                patch.push_str("\n");
            }
        }
    }

    // Separator
    patch.push_str("---\n");

    // Generate diff
    let diff = generate_diff(repo, commit)?;
    patch.push_str(&diff);
    patch.push_str("\n");

    // Sign-off line (optional)
    if include_signoff {
        let sign_off_name = author
            .as_deref()
            .or_else(|| author_sig.name())
            .unwrap_or("Unknown");
        let sign_off_email = email
            .as_deref()
            .or_else(|| author_sig.email())
            .unwrap_or("unknown@example.com");

        patch.push_str(&format!(
            "Signed-off-by: {} <{}>\n",
            sign_off_name, sign_off_email
        ));
    }

    Ok(patch)
}

fn generate_diff(repo: &Repository, commit: &Commit) -> Result<String> {
    let tree = commit.tree()?;
    let parent = commit.parent(0).ok();
    let parent_tree = parent.as_ref().and_then(|p| p.tree().ok());

    let mut diff_opts = DiffOptions::new();
    diff_opts.context_lines(3);
    diff_opts.interhunk_lines(3);
    diff_opts.show_binary(false);

    let diff = repo.diff_tree_to_tree(
        parent_tree.as_ref(),
        Some(&tree),
        Some(&mut diff_opts),
    )?;

    let mut output = String::new();
    
    // Generate diff stats - collect file information
    let mut files_info = Vec::new();
    
    diff.foreach(
        &mut |delta, _| {
            if let (Some(old_path), Some(new_path)) = (delta.old_file().path(), delta.new_file().path()) {
                let old_path_str = old_path.to_string_lossy().to_string();
                let new_path_str = new_path.to_string_lossy().to_string();
                files_info.push((old_path_str, new_path_str));
            }
            true
        },
        None,
        None,
        None,
    )?;

    // Note: Detailed stats would require iterating through all lines
    // For now, we'll just show file names in the diff output itself

    // Generate the actual diff
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let content = std::str::from_utf8(line.content()).unwrap_or("");
        match line.origin() {
            '+' => {
                if !content.is_empty() {
                    output.push_str("+");
                    output.push_str(content);
                }
            }
            '-' => {
                if !content.is_empty() {
                    output.push_str("-");
                    output.push_str(content);
                }
            }
            ' ' => {
                output.push_str(" ");
                output.push_str(content);
            }
            'F' => {
                // File header
                output.push_str(content);
            }
            'H' => {
                // Hunk header
                output.push_str(content);
            }
            _ => {
                output.push_str(content);
            }
        }
        true
    })?;

    Ok(output)
}

fn generate_patch_filename(commit: &Commit, number: usize, total: usize) -> String {
    let subject = commit
        .message()
        .and_then(|msg| msg.lines().next())
        .unwrap_or("patch");

    // Sanitize subject for filename using assembly-optimized function
    let sanitized = asm_utils::fast_sanitize_string(subject);
    let sanitized = sanitized.trim_matches('-');
    let sanitized = if sanitized.len() > 50 {
        &sanitized[..50]
    } else {
        sanitized
    };

    if total > 1 {
        format!("{:04}-{}.patch", number, sanitized)
    } else {
        format!("{}.patch", sanitized)
    }
}

fn generate_cover_letter(
    output: &PathBuf,
    prefix: &str,
    total: usize,
    commits: &[Commit],
    quiet: bool,
) -> Result<()> {
    let mut cover = String::new();

    cover.push_str(&format!("Subject: [{} 0/{total}] Cover letter\n\n", prefix, total = total));
    cover.push_str("This patch series includes the following changes:\n\n");

    for (idx, commit) in commits.iter().enumerate() {
        let subject = commit
            .message()
            .and_then(|msg| msg.lines().next())
            .unwrap_or("No subject");
        cover.push_str(&format!("{}. {}\n", idx + 1, subject));
    }

    cover.push_str("\n---\n");
    cover.push_str("\n");

    let filepath = output.join("0000-cover-letter.patch");
    let mut file = File::create(&filepath)
        .context("Failed to create cover letter file")?;
    file.write_all(cover.as_bytes())
        .context("Failed to write cover letter")?;

    if !quiet {
        println!("Generated: {}", filepath.display());
    }
    Ok(())
}
