# ShapShap - Linux Kernel Patch Generator

A tool written in Rust and Assembly for generating Linux kernel patches in the proper format for email submission to kernel maintainers.

## Features

- Generates patches in the exact format required by Linux kernel maintainers
- Follows kernel patch submission standards (plain text, proper headers, sign-off)
- Supports single patches and patch series with numbering
- Generates cover letters for patch series
- Assembly-optimized operations for performance
- Compatible with Rust for Linux kernel development requirements

## Requirements

- Rust 1.70+ (2021 edition)
- Git repository with commits to patch
- Linux/macOS/Windows (with appropriate assembly support)

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd ShapShap

# Build the project
cargo build --release

# Install globally (optional)
cargo install --path .
```

## Usage

### Basic Usage

Generate a patch from the latest commit (HEAD):

```bash
shapshap patch
```

This will create a patch file in the current directory.

### Initialize Configuration

Set up a configuration file with your default settings:

```bash
shapshap init
```

This creates `~/.config/shapshap/config.toml` with default settings that you can customize.

### Generate Patch from Commit Range

```bash
# Generate patches from last 3 commits
shapshap patch --range HEAD~3..HEAD

# Generate patch from specific commit range
shapshap patch --range abc123..def456
```

### Specify Output Directory

```bash
shapshap patch --output ./patches
```

### Custom Author Information

```bash
shapshap patch --author "Your Name" --email "your.email@example.com"
```

### Patch Series

For multiple patches in a series:

```bash
# Generate patches with numbering [PATCH 1/3], [PATCH 2/3], etc.
shapshap patch --range HEAD~2..HEAD --total 3

# Include cover letter
shapshap patch --range HEAD~2..HEAD --total 3 --cover-letter
```

### Custom Prefix

For RFC patches or other types:

```bash
shapshap patch --prefix "RFC"
```

### Validate Patches

Before sending patches, validate them:

```bash
shapshap validate path/to/patch.patch
```

### Check Commits

Validate commits before generating patches:

```bash
shapshap check --range HEAD~3..HEAD
```

### Test Patch Application

Test if a patch applies cleanly:

```bash
shapshap test-apply path/to/patch.patch
```

This creates a temporary branch, applies the patch, and cleans up automatically.

### Show Statistics

Get detailed statistics about a patch:

```bash
shapshap patch --stats
```

## Patch Format

The generated patches follow the Linux kernel patch format standard:

```
From <commit-hash> Mon Sep 17 00:00:00 2001
From: Author Name <author@example.com>
Date: Mon, 26 Jan 2025 12:00:00 +0000
Subject: [PATCH] Patch subject line

Patch description body...

---
diff --git a/file.c b/file.c
index 1234567..abcdefg 100644
--- a/file.c
+++ b/file.c
@@ -10,6 +10,7 @@
 ...
+new line
 ...
 
Signed-off-by: Your Name <your.email@example.com>
```

## Linux Kernel Patch Requirements

This tool ensures patches meet all kernel submission requirements:

1. **Plain text format** - No MIME, HTML, or attachments
2. **Proper subject line** - Includes `[PATCH]` prefix
3. **From line** - Identifies the patch author
4. **Sign-off** - Developer's Certificate of Origin (DCO) signature
5. **Proper diff format** - Unified diff with context lines
6. **One problem per patch** - Each patch solves a single issue

## Rust for Linux Compatibility

For Rust kernel code submissions:

- Patches are formatted according to kernel standards
- Code should be formatted with `rustfmt` before creating patches
- Use `make LLVM=1 rustfmtcheck` to verify formatting
- All Rust-specific guidelines apply (see kernel documentation)

## Assembly Optimizations

The tool includes assembly-optimized functions for:

- String sanitization (filename generation)
- Checksum calculation (patch validation)
- Line counting (performance-critical operations)

Assembly code supports:
- x86_64 architecture
- AArch64 architecture
- Fallback to Rust implementation for other architectures

## Examples

### Example 1: Single Patch

```bash
# Make your changes and commit
git add .
git commit -m "drivers: Add new feature to driver X"

# Generate patch
shapshap patch --output ./patches

# Result: patches/Add-new-feature-to-driver-X.patch
```

### Example 2: Patch Series with Cover Letter

```bash
# Generate 3 patches with cover letter
shapshap patch \
  --range HEAD~3..HEAD \
  --total 3 \
  --cover-letter \
  --output ./patches

# Result:
# - patches/0000-cover-letter.patch
# - patches/0001-First-patch.patch
# - patches/0002-Second-patch.patch
# - patches/0003-Third-patch.patch
```

### Example 3: Rust Kernel Module Patch

```bash
# After formatting with rustfmt
make LLVM=1 rustfmtcheck

# Generate patch
shapshap patch \
  --author "Rust Developer" \
  --email "rust@kernel.org" \
  --output ./patches
```

## Sending Patches

After generating patches, send them via email:

```bash
# Using git send-email (recommended)
git send-email --to=maintainer@kernel.org patches/*.patch

# Or manually via email client (plain text only!)
```

## Contributing

Contributions are welcome! Please ensure:

- Code follows Rust style guidelines
- Assembly code is well-documented
- Tests pass: `cargo test`
- Format code: `cargo fmt`

## License

MIT OR Apache-2.0

## References

- [Linux Kernel Patch Submission Guide](https://www.kernel.org/doc/html/latest/process/submitting-patches.html)
- [Rust for Linux Coding Guidelines](https://docs.kernel.org/rust/coding-guidelines.html)
- [git-format-patch Documentation](https://git-scm.com/docs/git-format-patch)
