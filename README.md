# Vellum

## 1. Problems to Solve

- **Fragmentation:** Dispersed .env files without centralized supervision.
- **Desynchronization:** Lack of parity between environments (local, staging, prod) and .env.example files.
- **Technical Errors:** Duplicates, spaces, and invalid formats.
- **Privacy:** Accidental exposure of secrets on screen during demos or recordings.
- **Portability:** Difficulty syncing sensitive variables between multiple workstations without using the cloud or Git.
- **Collaboration:** No safe way to share environment configs with teammates without exposing raw secrets.

## 2. Tech Stack

- **Backend:** Rust (Tauri v2).
- **Frontend:** TypeScript + React.
- **Styling:** Tailwind CSS v4.
- **Fonts:** Geist Sans (UI), IBM Plex Mono (Data), and Source Serif 4 (Accents).
- **Crypto:** AES-GCM (data at rest), Argon2 (key derivation from master password).

## 3. Features (MVP)

- **Identity:** Local auth system with registration and login. Master password derives the encryption key via Argon2. Session lives in memory; dies when the app closes.
- **Scanner:** Automatic indexing and parsing of .env files in local directories. Handles multiline values, variable expansion (`${VAR}`), and sensitivity heuristics.
- **The Vault:** Encrypted local database to store project snapshots. Each project tracks its owner and access members.
- **Access Control:** Role-based permissions per project — `owner`, `editor`, `viewer`. Owners invite collaborators via Bundles with a specific role attached.
- **Vellum Bundles:** Password-protected encrypted exports to sync variables between PCs without GitHub. Also used as the invitation mechanism for shared projects.
- **Auditing:** Structure comparison between real/example files and duplicate detection.
- **Obfuscation:** Dynamic blur for sensitive values. Viewer role enforces obfuscation by default.
- **Generator:** Cryptographic string creation for new variables.

## 4. Architecture

- **Sovereignty:** Direct access to local files and local encrypted storage; no cloud dependency.
- **Identity Layer:** User profile stored encrypted locally. Master password never persisted in plain text. Argon2 derives the vault key at login.
- **Ownership Model:** Every project has an `owner_id`. Users only see their own projects plus those shared with them.
- **Persistence:** Local storage for project paths, encrypted snapshots, and user profiles.
- **Security:** AES-GCM encryption for the Vault, Argon2 for key derivation, and OS native Keychain/Credential Manager for session convenience.

## 5. License

Distributed under the GPL-3.0 License. See `LICENSE` for more information.
