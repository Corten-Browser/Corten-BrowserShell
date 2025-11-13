# Ui Chrome

**Type**: feature
**Tech Stack**: Rust, egui/iced
**Version**: 0.17.0
**Estimated LOC**: 10,000-12,000

## Responsibility

Browser UI widgets including address bar, tab bar, toolbar, menu system, and theming

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

## Usage

This component is ready for development via Task tool orchestration.

**Through Orchestrator:**
Tell the orchestrator to work on this component, and it will launch an agent using the Task tool.

**Direct Work:**
```bash
cd components/ui_chrome
claude code
# Claude Code reads local CLAUDE.md and you work directly
```

## Development

See CLAUDE.md for detailed development instructions, quality standards, and TDD requirements.

---
**Last Updated**: 2025-11-13
**Project**: Corten-BrowserShell
