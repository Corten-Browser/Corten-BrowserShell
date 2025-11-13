# Platform Abstraction

**Type**: core
**Tech Stack**: Rust, X11rb, Win32 API, Cocoa
**Version**: 0.17.0
**Estimated LOC**: 8,000-10,000

## Responsibility

Platform-specific window operations, clipboard, notifications, and system integration

## Structure

```
├── src/           # Source code (Rust)
├── tests/         # Tests (unit, integration)
├── CLAUDE.md      # Component-specific instructions for Claude Code
└── README.md      # This file
```

## Dependencies

- shared_types

## Usage

This component is ready for development via Task tool orchestration.

**Through Orchestrator:**
Tell the orchestrator to work on this component, and it will launch an agent using the Task tool.

**Direct Work:**
```bash
cd components/platform_abstraction
claude code
# Claude Code reads local CLAUDE.md and you work directly
```

## Development

See CLAUDE.md for detailed development instructions, quality standards, and TDD requirements.

---
**Last Updated**: 2025-11-13
**Project**: Corten-BrowserShell
