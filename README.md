vault-keeper
------------

A privacy-focused, offline-first personal vault application for managing accounts, bookmarks, notes, and media — all encrypted locally on your device.

[Why vault-keeper?](https://github.com/AI-Star-Dev/vaultkeeper-releases/issues/1)

## Features

- **Account Management** — store and organize credentials with encrypted fields
- **Bookmarks** — save and categorize web links with notes
- **Notes** — write and organize markdown notes, with optional per-note secondary-password encryption
- **Media** — manage and reference local media files
- **Reports** — generate summaries across your vault data
- **Tags** — tag and filter content across all categories
- **Workspace** — multi-workspace support with isolated storage
- **Lock Screen** — master-password protection with Argon2 hashing and AES-GCM encryption

## Security Architecture

VaultKeeper uses a layered encryption model:

### Layer 1 — SQLCipher (database-level)
All workspace databases (`.db` files) are encrypted at rest using **SQLCipher** with AES-256-CBC.  The encryption key is the 32-byte master key derived from your master password via **Argon2id** (64 MB memory, 3 iterations).  Without the correct master password the database file is unreadable — it appears as random bytes to any external tool.

The meta database (`vault_meta.db`) is not SQLCipher-encrypted because it must be readable before the master key is known (to verify the password on startup).  Its sensitive rows are protected at the application level via an Argon2id verify hash.

### Layer 2 — Application-level field encryption (AES-256-GCM)
Even inside the encrypted database, sensitive account fields are individually encrypted with the master key:

| Field | Storage |
|---|---|
| `accounts.password` | AES-256-GCM ciphertext (`nonce:ciphertext`, hex) |
| `accounts.username` | AES-256-GCM ciphertext |
| `accounts.notes` | AES-256-GCM ciphertext |
| `accounts.url`, `accounts.title` | Plaintext (low sensitivity) |

### Layer 3 — Per-note secondary encryption (AES-256-GCM)
Individual notes can be locked with a **secondary password** independent of the master password.  The note content is encrypted with a key derived from that secondary password via Argon2id with a per-note random salt.  Editing an encrypted note in the editor re-encrypts the content with a fresh salt on save; the plaintext is never written back to the database.

### Key management
- Master key lives only in memory (cleared on lock / app exit)
- Argon2id parameters: 64 MB memory · 3 time cost · 4 parallelism
- Nonces: 12-byte random (OS RNG) per encryption operation
- **There is no password recovery mechanism** — forgetting the master password means permanent loss of access to all workspace data

## Tech Stack

- [Tauri 2](https://v2.tauri.app/) — native desktop shell (Rust backend)
- [Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/) — frontend
- [Vite](https://vitejs.dev/) — build tooling
- [Pinia](https://pinia.vuejs.org/) — state management
- [Tailwind CSS](https://tailwindcss.com/) — styling
- [SQLCipher](https://www.zetetic.net/sqlcipher/) (via rusqlite bundled-sqlcipher) — encrypted local storage
- [Argon2](https://github.com/RustCrypto/password-hashes/tree/master/argon2) + [AES-GCM](https://github.com/RustCrypto/AEADs/tree/master/aes-gcm) — encryption

## Prerequisites

- [Rust](https://rust-lang.org/tools/install/)
- [Node.js](https://nodejs.org/en/download)
- [pnpm](https://pnpm.io/installation)
- [Strawberry Perl](https://strawberryperl.com/) *(Windows only — required to compile OpenSSL for SQLCipher)*

## Getting Started

```bash
# Install frontend dependencies
pnpm install

# Check Rust backend compiles
cd src-tauri && cargo check && cd ..

# Run in development mode
pnpm tauri dev
```

## Build

```bash
# Production build
pnpm tauri build
```

Artifacts are placed in `src-tauri/target/release/bundle/`.

## Project Structure

```
src/                  # Vue frontend
  views/              # Page-level components (accounts, bookmarks, notes, media, reports, tags)
  components/         # Shared UI components
  stores/             # Pinia stores
  composables/        # Reusable Vue composables
  router/             # Vue Router config
src-tauri/            # Rust backend
  src/                # Tauri commands and business logic
  tauri.conf.json     # Tauri configuration
```

## IDE Setup

[VS Code](https://code.visualstudio.com/) with the following extensions:
- [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
