# 🗃️ Vault format

## Canonical files

Each note is a UTF-8 Markdown file in `notes/<folder>/<stable-id>.md` (or directly in `notes/` for the vault root). The first milestone uses a simple header:

```markdown
---
id: 1234abcd
title: A note title
folder: Projects
tags: #ideas #project
created: 1792457712345
updated: 1792457712345
---

Markdown body. Link to another note with `[[note-id]]` or `[[exact note title]]`.
```

Times are Unix epoch milliseconds. IDs are opaque stable identifiers and do not depend on the title, so renaming or moving a note does not invalidate links. The optional `folder` field is a relative path beneath `notes/`; older notes without it are treated as root notes. Tags are plain words. Attachment links are relative paths into `attachments/<note-id>/`.

## Derived state

`.cybernote/index.sqlite3` is a local FTS5 index and can be deleted/rebuilt. It is not the authoritative copy of note text. `.cybernote/history/` contains prior local Markdown versions, and `.cybernote/trash/` holds notes removed from the active list.

## Compatibility commitments

- Keep ordinary text export possible.
- Add format versions before introducing incompatible fields or encrypted envelopes.
- Write migration tools that preserve the source and are safe to retry.
- Test old note fixtures before changing serialization.
- Do not treat a cache, index, or provider listing as the sole source of truth.

The current parser is intentionally small and does not claim full YAML frontmatter compatibility. A formal schema and migration tests are required before external clients depend on it.
