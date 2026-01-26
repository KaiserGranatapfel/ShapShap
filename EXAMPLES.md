# ShapShap Usage Examples

## Example 1: Basic Single Patch

```bash
# Make your kernel changes
git add drivers/char/my_driver.c
git commit -m "drivers/char: Add new feature to my_driver"

# Generate patch
shapshap patch

# Output: patch.txt or Add-new-feature-to-my_driver.patch
```

## Example 2: Patch Series

```bash
# Make multiple commits
git commit -m "drivers/char: Add initial support"
git commit -m "drivers/char: Add interrupt handling"
git commit -m "drivers/char: Add power management"

# Generate patch series with cover letter
shapshap patch \
  --range HEAD~3..HEAD \
  --total 3 \
  --cover-letter \
  --output ./patches

# Output:
# - patches/0000-cover-letter.patch
# - patches/0001-Add-initial-support.patch
# - patches/0002-Add-interrupt-handling.patch
# - patches/0003-Add-power-management.patch
```

## Example 3: Rust for Linux Patch

```bash
# After making Rust kernel changes
make LLVM=1 rustfmtcheck  # Format and check

# Generate patch with custom author
shapshap patch \
  --author "Rust Developer" \
  --email "rust@kernel.org" \
  --output ./patches

# Send via git send-email
git send-email --to=rust-for-linux@vger.kernel.org patches/*.patch
```

## Example 4: RFC Patch

```bash
# Generate RFC patch
shapshap patch \
  --prefix "RFC" \
  --author "Your Name" \
  --email "your.email@example.com"

# Subject will be: [RFC] Your patch subject
```

## Example 5: Specific Commit Range

```bash
# Generate patch from specific commits
shapshap patch \
  --range abc123..def456 \
  --output ./patches
```

## Example 6: Custom Patch Numbering

```bash
# Start numbering from 2 (useful for resubmissions)
shapshap patch \
  --range HEAD~2..HEAD \
  --number 2 \
  --total 5 \
  --output ./patches

# Generates: [PATCH 2/5], [PATCH 3/5]
```

## Sending Patches

After generating patches, send them to kernel maintainers:

```bash
# Using git send-email (recommended)
git send-email \
  --to=maintainer@kernel.org \
  --cc=linux-kernel@vger.kernel.org \
  patches/*.patch

# Or manually via email client (plain text only!)
# Copy the patch content and paste into email body
```

## Patch Format Verification

The generated patches follow Linux kernel standards:

- ✅ Plain text format (no MIME/HTML)
- ✅ Proper From: line with author
- ✅ [PATCH] prefix in subject
- ✅ Signed-off-by line (DCO)
- ✅ Unified diff format
- ✅ Proper date formatting

## Tips

1. **Always test patches**: Apply with `git am` before sending
   ```bash
   git am patches/*.patch
   ```

2. **Check formatting**: For Rust code, use `rustfmt` first
   ```bash
   make LLVM=1 rustfmtcheck
   ```

3. **One change per patch**: Each patch should solve one problem

4. **Write good commit messages**: First line is the subject, body explains why

5. **Include cover letter**: For patch series, always use `--cover-letter`
