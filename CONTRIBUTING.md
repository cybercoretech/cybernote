# 🤝 Contributing to Cybernote

Thanks for helping build a notebook people can trust with their own words.

## Before changing the vault

- Keep Markdown as the canonical note content and SQLite as rebuildable derived state.
- Add a migration story before changing the note format.
- Preserve older versions when saving or moving notes to trash.
- Never let a search-index failure silently rewrite or discard a note.
- Avoid network calls and telemetry in the local-only path.
- Do not add cryptographic primitives or protocols of our own. Security changes need a written design review first.

## Local checks

```sh
cargo fmt --all -- --check
cargo test --workspace --offline
cargo clippy --workspace --all-targets -- -D warnings
```

## Pull requests

Describe user-visible behavior, data-format changes, security implications, and how recovery was tested. Include terminal screenshots only when they help demonstrate layout; never include real private notes or vault data.
