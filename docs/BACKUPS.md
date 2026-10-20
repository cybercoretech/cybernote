# 🧯 Backup and restore

Sync keeps devices aligned; backup protects against deletion, corruption, account loss, and mistakes. They are separate systems.

## Planned backup behavior

- Create dated, versioned snapshots independently of the sync cursor.
- Encrypt snapshots on the client before uploading to any remote target.
- Support local removable storage and at least one remote object destination; WebDAV and S3-compatible storage are planned adapters.
- Offer retention presets and show the age/result of the last successful backup.
- Preserve deleted notes and prior attachment versions for the configured retention window.
- Include a manifest with format version, checksums, note/attachment IDs, and restore instructions.
- Provide a dry-run restore and a complete restore into a new vault; never overwrite the only source by default.

## Release quality gate

A backup feature is not complete until CI or a documented local test creates a vault, makes edits and deletions, creates an encrypted backup, restores it with the recovery secret, verifies checksums and attachments, and proves that a wrong key or damaged archive fails safely. A restore drill should run for every backup format migration.

## Current status

The TUI keeps local per-note history and a recoverable trash. These are useful recovery aids, not independent backups. Automated snapshots, encryption, remote destinations, retention management, and restore verification are not implemented yet.
