# Browser Shell

**Type**: integration
**Tech Stack**: Rust
**Version**: 0.17.0
**Estimated LOC**: 6,000-8,000

## Responsibility

Main browser orchestration, component lifecycle coordination, and public API exposure

## Structure

```
├── src/           # Source code (Rust)
├── tests/         # Tests (unit, integration)
├── CLAUDE.md      # Component-specific instructions for Claude Code
└── README.md      # This file
```

## Dependencies

- shared_types
- message_bus
- platform_abstraction
- window_manager
- tab_manager
- ui_chrome
- user_data

## Usage

This component is ready for development via Task tool orchestration.

**Through Orchestrator:**
Tell the orchestrator to work on this component, and it will launch an agent using the Task tool.

**Direct Work:**
```bash
cd components/browser_shell
claude code
# Claude Code reads local CLAUDE.md and you work directly
```

## Development

See CLAUDE.md for detailed development instructions, quality standards, and TDD requirements.

---
**Last Updated**: 2025-11-13
**Project**: Corten-BrowserShell
