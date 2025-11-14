# browser_shell

**Type**: integration
**Tech Stack**: Rust, tokio
**Estimated LOC**: 8,000

## Responsibility

Main orchestrator coordinating all browser shell components through the message bus

## Dependencies

- `shared_types`
- `message_bus`
- `window_manager`
- `tab_manager`
- `ui_chrome`
- `settings_manager`
- `downloads_manager`
- `bookmarks_manager`

## Structure

```
├── src/           # Source code
├── tests/         # Tests (unit, integration, contracts)
├── CLAUDE.md      # Component-specific instructions for Claude Code
└── README.md      # This file
```

## Development

See CLAUDE.md for detailed development instructions, quality standards, and TDD requirements.

This component is part of the CortenBrowser Browser Shell project.
