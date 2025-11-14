# Browser Shell Component Contracts

This directory contains the API contracts for all browser shell components.

## Contracts

- **shared_types.yaml** - Common data structures and type definitions
- **message_bus.yaml** - Async message routing API
- **platform_abstraction.yaml** - Platform-specific window implementations
- **window_manager.yaml** - Window lifecycle management API
- **tab_manager.yaml** - Tab lifecycle and navigation API
- **ui_chrome.yaml** - Browser UI elements API
- **settings_manager.yaml** - Settings and preferences API
- **downloads_manager.yaml** - Download management API
- **bookmarks_manager.yaml** - Bookmark storage API
- **browser_shell.yaml** - Main orchestrator API
- **shell_app.yaml** - Application entry point API

## Contract Format

Contracts are defined in YAML format with:
- **contract_name**: Unique identifier
- **version**: Semver version
- **description**: Contract purpose
- **dependencies**: Other contracts this depends on
- **api**: Public interface definitions
- **types**: Custom type definitions

## Usage

Component agents use these contracts to:
1. Understand the public API they must implement
2. Identify dependencies on other components
3. Ensure type compatibility across components
4. Generate contract compliance tests
