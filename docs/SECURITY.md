# 🔐 Security policy and current status

Cybernote is being designed for private notes, but the current early TUI stores notes, attachments, history, and trash as ordinary files. **This version does not encrypt a vault.** Do not treat local app lock, disk permissions, HTTPS, or Tailscale as end-to-end encryption.

## Intended security properties

- Encryption and authentication use established, maintained libraries and protocols; Cybernote will not invent cryptography.
- Sync and backup providers receive ciphertext by default. The user's key is not uploaded to the provider or self-hosted sync service.
- Local indexes and caches are derived from the vault and can be rebuilt. They must be protected consistently with the note contents.
- A recovery mechanism is designed and tested before encrypted sync is enabled. Losing all device keys and the recovery secret must be described honestly as unrecoverable.
- Tailscale provides private network reachability and access controls for a self-hosted endpoint. It does not replace application authorization or payload encryption.
- Backups must be versioned so synced deletions and ransomware-like changes do not immediately erase prior restore points.

## Release gate for encrypted sync

Before release, document the threat model, key derivation and rotation, device enrollment and revocation, metadata exposed to each provider, conflict behavior, recovery workflow, and backup restoration. Add independent cryptographic review and tests for tampering, wrong keys, interrupted writes, stale devices, and concurrent edits.

## Reporting a vulnerability

Please do not post sensitive exploit details in a public issue. Contact the Cybercore maintainers privately through the GitHub organization and include affected version, reproduction steps, and impact. We will coordinate a fix and disclosure timeline.
