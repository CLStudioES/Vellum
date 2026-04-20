# Vellum

## 1. Problems to Solve

- **Fragmentation:** Dispersed .env files without centralized supervision.
- **Desynchronization:** Lack of parity between .env and .env.example files.
- **Technical Errors:** Duplicates, spaces, and invalid formats.
- **Privacy:** Accidental exposure of secrets on screen during demos or recordings.

## 2. Tech Stack

- **Backend:** Rust (Tauri v2).
- **Frontend:** TypeScript + React/Svelte.
- **Styling:** Tailwind CSS.
- **Fonts:** Geist Sans (UI) and IBM Plex Mono (Data).

## 3. Features (MVP)

- **Scanner:** Automatic indexing of .env files in local directories.
- **Watcher:** Real-time UI updates via file system events.
- **Auditing:** Structure comparison between real/example files and duplicate detection.
- **Obfuscation:** Dynamic blur for sensitive values.
- **Generator:** Cryptographic string creation for new variables.

## 4. Architecture

- **Sovereignty:** Direct access to local files; no external secrets database.
- **Persistence:** Local storage for project paths and user configuration.
- **Security:** Isolated IPC via Tauri and usage of the OS native Keychain/Credential Manager.

## 5. License

Distributed under the GPL-3.0 License. See `LICENSE` for more information.
