# ✦ Cybernote

**A local-first, Markdown-native notebook with a sharp terminal interface.** Notes stay in your vault as ordinary files. Cybernote adds a fast search index, tags, backlinks, attachments, recovery history, and a keyboard-first workspace around them.

> 🚧 **Early development.** The first milestone is the Rust TUI and shared Rust core. Android, encrypted sync, provider adapters, and automated backup are planned and are not implemented yet. Do not use this build as the only copy of important data.

## 🌌 What Cybernote is becoming

- 📝 Markdown-first notes you can inspect and export without Cybernote
- ⚡ Local full-text search powered by a rebuildable SQLite FTS index
- 🏷 Tags, `[[wiki links]]`, backlinks, and file attachments
- 🕰 Per-note recovery snapshots and recoverable trash
- ⌨️ A polished Ratatui terminal app with a real editor handoff
- 📱 A Kotlin Android app for offline reading, editing, and capture
- 🔐 End-to-end encrypted sync and backups, with keys controlled by the user
- ☁️ Optional WebDAV and S3-compatible storage adapters
- 🕸 An optional self-hosted sync service reachable over Tailscale

## 🚀 Try the TUI

Requirements: Rust 1.85 or newer and a terminal that supports Unicode. Body editing uses `$VISUAL`, then `$EDITOR`, then the first installed editor among Neovim, Nano, and Vi.

```sh
cargo run -p cybernote-tui -- ~/Documents/Cybernote
```

The vault path can also be set with `CYBERNOTE_VAULT`. Cybernote creates `notes/`, `attachments/`, and `.cybernote/` inside the selected vault. The SQLite index is derived data and is rebuilt at launch; Markdown files remain the source of truth. On the default vault only, the first launch imports entries from the previous Python TUI's `~/Documents/darkstar/notes/{Notes,Todo,Projects,Timeline}.md` files without changing those originals.

### ⌨️ Key map

| Key | Action |
| --- | --- |
| `j` / `k`, arrows | Move through notes |
| `/` | Search title, body, and tags |
| `n` | Create a note, then open your preferred editor |
| `e` | Open the edit studio: edit body, rename, change tags, or append Markdown templates |
| `Enter` | Quick edit the body in your preferred editor |
| `a` | Attach a file to the selected note |
| `s` | Capture a screen region and attach it to the selected note (Omarchy) |
| `f` | Create a folder inside the current vault |
| `m` | Move the selected note to a folder; blank moves it to the vault root |
| `v` | Open the vault switcher; press `a` there to add an existing folder, `x` to remove a saved entry |
| `d` | Move a note to recoverable Cybernote trash |
| `u` | Restore the most recently trashed note |
| `p` | Open the palette chooser |
| `t` | Cycle Cybercore palettes |
| `r` | Re-index the vault |
| `?` | Open the command popup |
| `q` | Quit |

The edit studio offers direct Neovim and Nano choices, plus daily, meeting, project-brief, and checklist templates. Editing and metadata changes create a recoverable history snapshot.

### 🎨 Built-in palettes

Neon Cyan, Plasma, Ember, Biohazard, Aurora, Rose Quartz, Arctic, and Solar Flare. Press `p` to browse and apply palettes instantly, or `t` to cycle them. The choice persists between launches and only changes the interface; no theme touches vault data.

## 🗃️ Vault layout

```text
Cybernote/
├── notes/                 # Canonical Markdown files, optionally grouped in folders
│   └── Projects/           # User-created folders; note IDs remain stable
├── attachments/           # Files attached to notes
└── .cybernote/
    ├── index.sqlite3      # Rebuildable search index
    ├── history/<note-id>/ # Previous Markdown versions
    └── trash/             # Notes removed from the active list
```

Notes use a small YAML-like frontmatter header and ordinary Markdown body. Notes can be moved into nested folders under `notes/`; the `folder` metadata and relative path preserve that organization. The vault switcher stores a list of folder roots in the local app config and never moves or deletes vault data when an entry is removed.

## 🧱 Architecture

The workspace separates `cybernote-core` from `cybernote-tui`. The core owns the vault format, indexing, note operations, links, attachments, and local history. The TUI owns terminal presentation and key handling. The same core is intended to be exposed to Android through a narrow, tested binding layer; Android's UI will be native Kotlin.

See [Architecture](docs/ARCHITECTURE.md), [Vault format](docs/VAULT_FORMAT.md), [Security](docs/SECURITY.md), [Sync](docs/SYNC.md), [Backups](docs/BACKUPS.md), [Themes](docs/THEMES.md), and the [Roadmap](docs/ROADMAP.md).

## 🛡️ Security status

Security is a design goal, not a claim that this early build is secure for sensitive material. Current notes and local history are plaintext Markdown. Encrypted sync, encrypted backups, device/key management, and restore verification must be implemented and reviewed before Cybernote describes a vault as end-to-end encrypted. See [SECURITY.md](SECURITY.md) for the current status and reporting process.

## 🧪 Development

```sh
cargo fmt --all -- --check
cargo test --workspace --offline
cargo run -p cybernote-tui -- /tmp/cybernote-demo
```

The project site is published from [`docs/`](docs/index.html) by GitHub Pages Actions. Its Cybercore palette chooser is implemented in plain HTML, CSS, and JavaScript, with reduced-motion support and no analytics. Screenshot capture uses the Omarchy region-capture flow when available; it is copied into the selected note as an ordinary attachment.

## 🤝 Contributing

Start with [CONTRIBUTING.md](CONTRIBUTING.md). Changes to vault format or security boundaries need a design note and migration or recovery story before implementation.

## 📄 License

MIT. See [LICENSE](LICENSE).
