# Changelog

## [Unreleased] - Feature Branch: comprehensive-improvements

### Added
- **Configuration file support** - Create and use `~/.config/shapshap/config.toml` for default settings
- **Patch validation** - Validate patches against kernel standards before submission
- **Commit checking** - Check commits for proper format before generating patches
- **Enhanced statistics** - Detailed patch statistics with file-by-file breakdown
- **Test apply feature** - Test patch application in a temporary branch
- **Colored output** - Better visual feedback with colored terminal output
- **Comprehensive error handling** - Better error messages and validation
- **Assembly optimizations** - Enhanced assembly code for performance-critical operations

### Changed
- **Improved diff generation** - Better statistics and formatting
- **Enhanced CLI** - More commands and options
- **Better validation** - Comprehensive checks for patch format compliance

### Commands Added
- `shapshap validate <path>` - Validate a patch file
- `shapshap check [--range]` - Check commits before patching
- `shapshap init [--force]` - Initialize configuration file
- `shapshap stats <path>` - Show patch statistics
- `shapshap test-apply <path>` - Test applying a patch

### Configuration Options
- Default author name and email
- Default output directory
- Default prefix (PATCH, RFC, etc.)
- Validation settings
- Email sending configuration (for future use)

### Validation Checks
- Line length validation
- Trailing whitespace detection
- Tab indentation checking
- Sign-off requirement
- Commit message format validation
- Patch header validation
