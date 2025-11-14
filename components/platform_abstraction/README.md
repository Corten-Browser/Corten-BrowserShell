# platform_abstraction

**Type**: core
**Tech Stack**: Rust, x11rb/wayland (Linux), windows crate (Windows), cocoa (macOS)
**Estimated LOC**: 12,000

## Responsibility

Platform-specific window management implementations for Linux, Windows, and macOS

## Dependencies

- `shared_types`

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
