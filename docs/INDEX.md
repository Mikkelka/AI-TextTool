# Documentation Index

Welcome to the AI-TextTool documentation. This folder contains project documentation organized by topic.

## Main Documentation

### [../CLAUDE.md](../CLAUDE.md)
**Complete developer guide for the AI-TextTool project**
- Project overview and architecture
- Frontend (Vue 3) and backend (Rust) structure
- Development commands and workflows
- Feature descriptions and implementations
- Data storage and configuration management
- Tauri-specific solutions and learnings
- Project structure and module organization

**Best for**: Understanding project structure, setup, and architecture-level decisions.

---

### [TAURI_REFERENCE.md](TAURI_REFERENCE.md)
**Comprehensive Tauri 2.0 usage guide with battle-tested solutions**
- Clipboard Manager plugin setup and usage
- Global Shortcut plugin for system-wide hotkeys
- File System plugin for file operations
- Opener plugin for opening URLs
- Process and executable path handling
- Tray icon configuration and management
- Window management patterns and solutions
- Commands for frontend-backend communication

**Best for**: Tauri plugin work, common Tauri issue solving, and window management patterns.

---

### [CODE_REVIEW.md](CODE_REVIEW.md)
**Comprehensive code review with optimization opportunities**
- Executive summary of code quality assessment
- Implementation status tracking
- Frontend and backend issues
- Configuration/build issues
- Prioritized recommendations

**Best for**: Historical code quality findings and optimization tracking.

---

### [CODE_OPTIMIZATION_2026-02-12.md](CODE_OPTIMIZATION_2026-02-12.md)
**Prioritized optimization roadmap with implementation status (2026-02-12)**
- Verified baseline from lint/test/build commands
- Priority buckets: P0, P1, P2 with current status
- P0 marked as completed; remaining focus moved to selected P1/P2 items
- Concrete file references and acceptance criteria
- Quick wins (first 48 hours) and next sprint guidance
- Risk/compatibility notes and definition of done

**Best for**: Executing focused, stepwise quality improvements with measurable outcomes.

---

### [CLAUDE.local.md](CLAUDE.local.md)
**Local/private project instructions**
- Development workflow preferences
- Local setup notes
- Environment-specific configuration

**Best for**: Understanding local development preferences.

---

## Quick Start

1. **First time setup?** -> Read [../CLAUDE.md](../CLAUDE.md)
2. **Building a feature?** -> Check [../CLAUDE.md](../CLAUDE.md) for architecture
3. **Using Tauri plugins?** -> Reference [TAURI_REFERENCE.md](TAURI_REFERENCE.md)
4. **Code quality focus?** -> Review [CODE_REVIEW.md](CODE_REVIEW.md)
5. **Current optimization roadmap?** -> Start with [CODE_OPTIMIZATION_2026-02-12.md](CODE_OPTIMIZATION_2026-02-12.md)

## Organization Strategy

The documentation is organized to serve different purposes:

- **../CLAUDE.md** (root) - Main project documentation with architecture and implementation guidance
- **TAURI_REFERENCE.md** - Plugin-specific documentation and solutions
- **CODE_REVIEW.md** - Historical quality metrics and findings
- **CODE_OPTIMIZATION_2026-02-12.md** - Current prioritized optimization roadmap
- **../CLAUDE.local.md** (root) - Local user preferences and configuration
- **INDEX.md** - Documentation navigation guide

Each document is self-contained and can be read independently based on current task context.

---

**Last Updated**: 2026-02-12
