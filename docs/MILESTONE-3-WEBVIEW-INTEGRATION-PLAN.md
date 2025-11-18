# Milestone 3: WebView Integration Implementation Plan

**Date**: 2025-11-18
**Status**: Planning Phase
**Target Version**: v0.6.0
**Priority**: CRITICAL - Required for functional browser

---

## Current State (v0.5.0)

### ✅ What's Implemented

The project has successfully implemented:

- **Milestone 1** (Basic Shell): ✅ COMPLETE
  - Window manager (platform abstraction for Linux/Windows/macOS)
  - Tab manager (tab lifecycle management)
  - Message bus (inter-component communication)
  - Browser shell (integration orchestrator)
  - Shell app (CLI and application entry point)

- **Milestone 2** (Browser Chrome): ✅ COMPLETE
  - UI chrome with egui (address bar, toolbar, tab bar, menus)
  - eframe application framework for GUI
  - Keyboard shortcuts
  - Settings UI
  - Theme system

- **Milestone 4** (Advanced Features): ✅ COMPLETE
  - Downloads manager (download tracking and control)
  - Bookmarks manager (bookmark storage and organization)
  - History manager (SQLite-based browsing history with FTS5 search)
  - Find in page (text search with regex, case-sensitivity, whole word)
  - Security manager (XSS prevention, CSP, URL validation, permissions)
  - Content area (content display management)

### ❌ What's Missing

**Milestone 3 (Component Integration)**: NOT IMPLEMENTED
- ❌ No actual network stack integration
- ❌ No actual render engine integration
- ❌ No HTML/CSS/DOM processing
- ❌ **Cannot load or render web pages**

**Critical Gap**: The browser has all infrastructure components but **cannot display websites**.

Current `webview_integration` component is a coordination layer only - it doesn't integrate with an actual rendering engine.

---

## Architectural Challenge: egui + wry Integration

### The Problem

The current architecture uses:
- **eframe**: Application framework that manages window creation and event loop
- **egui**: Immediate-mode GUI for browser chrome (address bar, tabs, toolbar)
- **wry**: Native WebView library (requires own window management)

These are **architecturally incompatible** without significant refactoring:

| Requirement | eframe/egui | wry |
|-------------|-------------|-----|
| Window management | eframe creates window | wry needs window handle |
| Event loop | eframe runs event loop | wry needs event loop access |
| Rendering | egui renders to GPU | wry renders to native WebView |
| Integration | Complete control | Needs embedding strategy |

### Why This Is Complex

1. **Window Ownership**: eframe owns the window, but wry needs direct access to window handles
2. **Event Loop Coordination**: Both need to process events; must be carefully coordinated
3. **Rendering Composition**: egui renders UI, wry renders web content - must composite correctly
4. **Platform Differences**: Different approaches needed for Linux (GTK+WebKit) vs Windows (WebView2) vs macOS (WKWebView)
5. **Lifecycle Management**: Must coordinate initialization, shutdown, resizing, focus between both systems

---

## Implementation Options

### Option 1: Tauri Architecture (RECOMMENDED)

**Approach**: Use Tauri framework which solves the integration problem.

**How It Works**:
- Tauri uses wry for window management and WebView rendering
- Browser chrome implemented in HTML/CSS/JavaScript instead of egui
- Rust backend communicates via IPC with JavaScript frontend
- Tauri handles all platform-specific integration

**Pros**:
- ✅ Battle-tested integration of wry with web UI
- ✅ Cross-platform support built-in
- ✅ Active development and community
- ✅ Solves all architectural conflicts
- ✅ Can be implemented incrementally

**Cons**:
- ❌ Requires rewriting UI chrome from egui to web technologies
- ❌ Different paradigm from current implementation
- ❌ More complex build system

**Effort Estimate**: 2-3 weeks

**Specification Alignment**: This matches Phase 1 of the specification ("Tauri-based Shell")

### Option 2: Hybrid Window Architecture

**Approach**: egui for chrome, separate wry windows for web content.

**How It Works**:
- eframe window shows browser chrome (address bar, tabs, toolbar)
- Each tab creates a separate native window with wry WebView
- Windows positioned to appear embedded
- Custom window management to coordinate both

**Pros**:
- ✅ Keeps existing egui chrome implementation
- ✅ Uses wry for actual web rendering
- ✅ Incremental migration path

**Cons**:
- ❌ Complex window positioning and coordination
- ❌ Visual glitches during resize/movement
- ❌ Platform-specific challenges (window stacking, focus)
- ❌ Not a typical browser architecture

**Effort Estimate**: 3-4 weeks

### Option 3: Custom Event Loop with winit + egui + wry

**Approach**: Manual coordination of all three systems.

**How It Works**:
- Use raw winit for window creation
- Manually integrate egui rendering
- Manually integrate wry WebViews
- Write custom event loop that coordinates all three
- Handle all platform-specific details

**Pros**:
- ✅ Complete control over integration
- ✅ Can optimize for specific use cases
- ✅ Matches specification's Phase 3 ("Pure Rust Shell")

**Cons**:
- ❌ VERY complex implementation
- ❌ High maintenance burden
- ❌ Platform-specific code for Linux/Windows/macOS
- ❌ Requires deep understanding of all three systems

**Effort Estimate**: 6-8 weeks

### Option 4: Pure egui with Custom Rendering

**Approach**: Render web content within egui using a custom engine.

**How It Works**:
- Implement HTML/CSS/JS rendering entirely in Rust
- Render web content as egui widgets
- No external WebView dependency

**Pros**:
- ✅ Complete Rust implementation
- ✅ Perfect egui integration

**Cons**:
- ❌ Requires building an entire browser engine
- ❌ Months/years of development
- ❌ Unlikely to match native WebView quality
- ❌ Not practical for this project

**Effort Estimate**: Not realistic

---

## Recommended Approach

**Adopt Option 1: Tauri Architecture**

### Rationale

1. **Specification Alignment**: The specification describes Phase 1 as "Tauri-based Shell", which is what we should implement first
2. **Proven Solution**: Tauri has already solved the wry integration problem
3. **Fastest Path**: Can implement in 2-3 weeks vs 3-8 weeks for other options
4. **Production Quality**: Tauri is production-ready, used by many applications
5. **Incremental Migration**: Can migrate egui components to web incrementally

### Migration Strategy

**Phase 1: Tauri Foundation** (Week 1)
- Add Tauri framework to project
- Create Tauri configuration
- Migrate application entry point to Tauri
- Basic window creation with wry WebView
- Keep existing Rust components as backend

**Phase 2: Web UI Migration** (Week 2)
- Implement address bar in HTML/CSS/TypeScript
- Implement tab bar in web UI
- Implement toolbar and navigation buttons
- Create IPC bridge to Rust backend (window_manager, tab_manager)

**Phase 3: Component Integration** (Week 3)
- Connect web UI to bookmarks_manager
- Connect web UI to history_manager
- Connect web UI to downloads_manager
- Connect web UI to settings_manager
- Full navigation working (load actual web pages)

**Phase 4: Testing & Validation** (Week 4)
- Test actual web page loading
- Test all browser features end-to-end
- Performance optimization
- Cross-platform testing (Linux/Windows/macOS)

---

## v0.6.0 Scope Definition

**Goal**: Achieve Milestone 3 - Component Integration

**Core Deliverables**:
- [ ] Tauri framework integrated
- [ ] Basic web UI for browser chrome
- [ ] WebView can load and render actual web pages
- [ ] Navigation works (address bar → web content)
- [ ] Tabs can display different websites
- [ ] All existing components integrated with web UI

**Success Criteria** (from Specification):
- [x] Load and render web pages
- [ ] Component communication stable
- [ ] 50% of integration tests pass

**Version**: v0.6.0
**Timeline**: 3-4 weeks
**Effort**: ~120-160 hours

---

## Alternative: Quick Proof of Concept

If full Tauri migration is not desired, we can create a quick proof-of-concept:

**Standalone wry Example** (1-2 days):
- Create simple standalone application using wry
- Demonstrate loading web pages
- Show basic navigation (forward/back)
- Prove wry works on target platforms
- Document integration requirements

This would prove the concept without committing to full integration, allowing informed decision on architecture.

---

## Decision Required

The project is at a critical decision point:

1. **Implement Milestone 3 with Tauri** → Full functional browser in v0.6.0 (3-4 weeks)
2. **Create proof-of-concept** → Validate approach before committing (1-2 days)
3. **Defer Milestone 3** → Keep v0.5.0 as component library, schedule Milestone 3 later

**Current Blocker**: Cannot proceed with Milestone 3 implementation without architectural decision.

**Recommendation**: Option 1 (Tauri) for fastest path to functional browser matching specification Phase 1.

---

## References

- **Specification**: `browser-shell-specification.md` - Phases 1-4, Milestones 1-6
- **Tauri Documentation**: https://tauri.app/
- **wry Documentation**: https://github.com/tauri-apps/wry
- **Current Implementation**: `components/` directory (all components)
- **v0.5.0 Completion Report**: `PROJECT-COMPLETION-REPORT-v0.5.0.md`

---

**Status**: This document defines the path forward for Milestone 3. Implementation awaits architectural decision and resource allocation.
