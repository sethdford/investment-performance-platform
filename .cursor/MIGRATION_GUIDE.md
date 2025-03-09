# Migration Guide: .cursorrules to Project Rules

This guide explains the migration from the custom `.cursorrules` format to Cursor's native project rules system.

## What Changed

The `.cursorrules` file was a custom configuration format with extensive settings for Rust code analysis, error handling, and formatting. Cursor now supports a native project rules system that provides similar functionality in a more standardized format.

## Migration Summary

1. **Rules Directory**: Created `.cursor/rules/` directory
2. **Project Rules**: Converted key rules to `.cursor/rules/rust-project-rules.json`
3. **Settings**: Added `.cursor/settings.json` to enable and configure the rules
4. **Documentation**: Added README and this migration guide

## What Was Preserved

The following aspects of the original `.cursorrules` were preserved in the migration:

- Core coding standards and best practices
- Rust naming conventions
- Documentation requirements
- Error handling patterns
- Async programming guidelines
- Safety considerations

## What Was Modified

The following aspects were adapted to fit Cursor's project rules format:

- Rule definitions now use regex patterns instead of custom detection logic
- Severity levels are standardized (error, warning, info, suggestion)
- Configuration is split between rules and settings files

## What Was Not Migrated

Some features of the original `.cursorrules` couldn't be directly migrated:

- Complex error pattern detection and fix strategies
- Detailed output formatting templates
- Advanced analysis capabilities
- Memory management configurations
- Custom execution commands

## Using the New Rules

The project rules are automatically applied when you open the project in Cursor. You'll see warnings and suggestions in the editor based on the enabled rules.

## Extending the Rules

To add more rules from the original `.cursorrules`:

1. Identify a rule or pattern from `.cursorrules` you want to implement
2. Convert it to a regex-based pattern in the project rules format
3. Add it to `.cursor/rules/rust-project-rules.json`
4. Enable it in `.cursor/settings.json`

## Future Improvements

As Cursor's project rules system evolves, more features from the original `.cursorrules` may become implementable. The current migration focuses on the most important and directly applicable rules.

## Reference

- Original `.cursorrules` file (preserved in the project root)
- `.cursor/rules/README.md` for details on the current rules
- Cursor documentation on project rules 