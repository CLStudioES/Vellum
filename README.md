# Vellum

## 1. Problems to Solve

- **Fragmentation:** Dispersed .env files without centralized supervision.
- **Desynchronization:** Lack of parity between environments (local, staging, prod) and .env.example files.
- **Technical Errors:** Duplicates, spaces, and invalid formats.
- **Privacy:** Accidental exposure of secrets on screen during demos or recordings.
- **Portability:** Difficulty syncing sensitive variables between multiple workstations without using the cloud or Git.

## 2. Tech Stack

- **Backend:** Rust (Tauri v2).
- **Frontend:** TypeScript + React.
- **Styling:** Tailwind CSS v4.
- **Fonts:** Geist Sans (UI), IBM Plex Mono (Data), and Source Serif 4 (Accents).

## 3. Features (MVP)

- **Scanner:** Automatic indexing and parsing of .env files in local directories.
- **The Vault:** Encrypted local database to store project snapshots.
- **Vellum Bundles:** Password-protected encrypted exports to sync variables between PCs without GitHub.
- **Auditing:** Structure comparison between real/example files and duplicate detection.
- **Obfuscation:** Dynamic blur for sensitive values.
- **Generator:** Cryptographic string creation for new variables.

## 4. Architecture

- **Sovereignty:** Direct access to local files and local encrypted storage; no cloud dependency.
- **Persistence:** Local storage for project paths and encrypted snapshots.
- **Security:** AES-GCM encryption for the Vault and usage of the OS native Keychain/Credential Manager.

## 5. License

Distributed under the GPL-3.0 License. See `LICENSE` for more information.
