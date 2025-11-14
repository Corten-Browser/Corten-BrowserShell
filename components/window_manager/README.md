# window_manager

**Type**: feature
**Tech Stack**: Rust, tokio
**Estimated LOC**: 8,000

## Responsibility

Browser window lifecycle management including creation, resizing, focus, and multi-window support

## Dependencies

- `shared_types`
- `message_bus`
- `platform_abstraction`

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
