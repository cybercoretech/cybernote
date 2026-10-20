# ☁️ Sync design

Cybernote will allow users to choose a sync route. Providers are adapters; they must not define note semantics or encryption policy.

## Planned routes

| Route | What it means | Status |
| --- | --- | --- |
| Local vault | One device or a user-managed folder | Available in the first TUI milestone |
| WebDAV | User supplies a server URL and credentials | Planned adapter |
| S3-compatible | User supplies endpoint, bucket, and narrowly scoped credentials | Planned adapter |
| Self-hosted over Tailscale | Cybernote sync service on the user's own machine, reachable privately on the tailnet | Planned; Tailscale is transport/access, not storage |
| More providers | Add only when the adapter can preserve version and conflict guarantees | Open extension point |

## Requirements

- Offline edits remain usable and queue for later synchronization.
- Every uploaded object is encrypted client-side by default once E2EE ships.
- Remote writes use revision checks or immutable versions; concurrent edits must not be silently overwritten.
- Deletions become tombstones and are not immediately purged from backup history.
- Credentials are stored in OS-protected storage, are scoped to the selected destination, and can be revoked.
- Provider failures are visible and retryable. A stale or disconnected destination is never shown as “backed up.”
- Export and restore work without a Cybernote-hosted account.

## Tailscale mode

The self-hosted service should bind only to the intended interface or sit behind Tailscale Serve. Tailnet ACLs restrict who can reach it; Cybernote still authenticates each user/device and encrypts note payloads end-to-end. The mobile app should use the installed Tailscale client rather than trying to own Android VPN lifecycle in the first release.

## Conflict rules

The core tracks revision ancestry. Independent edits to the same note create a visible conflict entry that preserves both versions. A merge action is explicit, creates a recovery point, and records the resulting revision. Provider timestamps are not trusted as the sole ordering source.
