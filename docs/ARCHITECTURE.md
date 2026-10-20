# 🧩 Cybernote architecture

## Product principles

1. **Local-first:** capture and search work without a network connection.
2. **Markdown-native:** readable files are the durable source of note content; databases are rebuildable indexes.
3. **One domain core:** Rust owns note identity, metadata, indexing, links, history, conflict rules, and storage abstractions. The Rust TUI and Android client share these rules; Android presentation remains native Kotlin.
4. **Provider-neutral:** the core sync contract describes versions, encrypted objects, acknowledgements, and tombstones. WebDAV, S3-compatible storage, and a self-hosted service implement adapters around that contract.
5. **Recoverable:** an edit, deletion, sync conflict, or failed migration must leave a route to the prior version.

## Current workspace

- `crates/cybernote-core`: local vault operations, Markdown frontmatter, SQLite FTS5 index, tags, `[[links]]` backlinks, attachments, recovery snapshots, and trash.
- `crates/cybernote-tui`: Ratatui interface, keyboard interaction, editor handoff, palette chooser, and preview.
- `docs/`: project site and contributor/product documentation.

## Planned shared core boundary

The domain API should accept explicit vault paths and return typed results. It should not depend on a UI runtime, Android lifecycle, a particular cloud provider, or a global singleton. Android bindings should expose a small stable API through UniFFI or an equivalent maintained bridge, with Kotlin responsible for lifecycle, Compose UI, SAF permissions, notifications, and platform key storage.

## Data boundaries

- `notes/*.md`: canonical note text and portable metadata.
- `attachments/<note-id>/`: attachment bytes referenced by relative Markdown links.
- `.cybernote/index.sqlite3`: disposable local search/index state.
- `.cybernote/history/`: local previous versions; future remote backups have independent retention.
- `.cybernote/trash/`: recoverable local removals.
- Remote provider: encrypted objects and only the metadata required to address, version, and synchronize them.

The exact encrypted remote envelope and conflict protocol are not implemented yet. They require a reviewed design and compatibility/version strategy before the first synced vault is created.

## Sync model to design

Each note and attachment needs a stable identifier, content hash, revision/parent information, and deletion tombstone. A provider adapter stores immutable or conditionally updated objects; the core detects concurrent parents and creates a recoverable conflict copy or a user-guided merge. Last-writer-wins alone is not acceptable for note bodies.
