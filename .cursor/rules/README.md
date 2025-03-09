# Rust-Eze Project Rules

This directory contains project-specific rules for Rust code quality and consistency in the Cursor IDE.

## Overview

These rules are derived from the `.cursorrules` file and have been migrated to the Cursor project rules format. They enforce best practices for Rust development, with a focus on:

- Code style and naming conventions
- Error handling patterns
- Async/concurrent programming practices
- Documentation requirements
- Safety considerations

## Rules Structure

The rules are defined in `rust-project-rules.json` and are enabled in the project's `.cursor/settings.json` file.

Each rule consists of:
- **name**: A unique identifier for the rule
- **description**: A brief explanation of what the rule enforces
- **pattern**: A regex pattern that matches code that violates the rule
- **message**: The message displayed when the rule is violated
- **severity**: The severity level (error, warning, info, suggestion)
- **languages**: The languages the rule applies to (in this case, "rust")
- **excludePatterns** (optional): Patterns to exclude from matching

## Adding New Rules

To add a new rule:

1. Edit `.cursor/rules/rust-project-rules.json`
2. Add a new rule object to the `rules` array
3. Add the rule name to the `enabledRules` array in `.cursor/settings.json`

## Rule Severity Levels

- **error**: Must be fixed before committing code
- **warning**: Should be fixed but won't block commits
- **info**: Informational only, suggesting improvements
- **suggestion**: Optional improvements to consider

## Customizing Rules

You can customize existing rules by modifying their patterns, messages, or severity levels in the `rust-project-rules.json` file.

## Disabling Rules

To disable a specific rule, remove it from the `enabledRules` array in `.cursor/settings.json`.

## Original .cursorrules

The original `.cursorrules` file contains additional configuration that isn't directly supported by Cursor's project rules format. The most critical rules have been migrated to this format, but you may want to reference the original file for additional guidance.

## Maintenance

These rules should be reviewed and updated periodically to ensure they remain relevant and effective as the project evolves. 