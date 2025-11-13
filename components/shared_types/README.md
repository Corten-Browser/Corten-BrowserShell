# Shared Types

**Type**: base
**Tech Stack**: Rust
**Version**: 0.17.0
**Estimated LOC**: 3,000-5,000

## Responsibility

Common types, interfaces, and message protocol definitions for all browser components

## Structure

```
├── src/           # Source code (Rust)
├── tests/         # Tests (unit, integration)
├── CLAUDE.md      # Component-specific instructions for Claude Code
└── README.md      # This file
```

## Dependencies

None (base component)

## Usage

This component is ready for development via Task tool orchestration.

**Through Orchestrator:**
Tell the orchestrator to work on this component, and it will launch an agent using the Task tool.

**Direct Work:**
```bash
cd components/shared_types
claude code
# Claude Code reads local CLAUDE.md and you work directly
```

## Development

See CLAUDE.md for detailed development instructions, quality standards, and TDD requirements.

---
**Last Updated**: 2025-11-13
**Project**: Corten-BrowserShell
