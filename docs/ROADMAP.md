# 🗺️ Cybernote roadmap

This roadmap is directional; milestones ship only when their data-safety checks pass.

## M1 — Rust TUI and local core

- [x] Workspace split between core and terminal UI
- [x] Markdown note creation/editing, tags, links/backlinks, attachments
- [x] Rebuildable FTS search index
- [x] Per-note history and recoverable trash
- [x] Keyboard-first panels, editor handoff, themes, and popups
- [ ] Robust editor conflict detection and recovery UX
- [ ] Import/export, templates, sorting/filtering, and richer Markdown preview
- [ ] Cross-platform packaging and signed release artifacts

## M2 — Android offline client

- Native Kotlin interface with shared Rust domain rules
- Offline vault, fast search, edit/capture, share-sheet intake, attachments
- Theme chooser, accessible type scaling, widgets/quick capture
- Device-protected key material and lifecycle-safe background work

## M3 — Encrypted synchronization

- Reviewed threat model, key hierarchy, recovery, and device revocation
- Versioned encrypted object protocol and conflict-safe sync engine
- WebDAV adapter, then S3-compatible adapter
- Self-hosted endpoint reachable over Tailscale with documented ACL setup

## M4 — Independent encrypted backups

- Retention policy, local and remote targets, backup health indicators
- Restore to a new vault, integrity verification, and recovery-key drills
- Automated regression tests for deletes, conflicts, interruption, and corruption

## M5 — Polish and ecosystem

- Android and desktop release channels, migration/import tools, translations
- More provider adapters based on user demand
- Optional web viewer only if its privacy and key model can match the native apps
