# Vellum

## 1. Problems to Solve

- **Fragmentation:** Dispersed .env files without centralized supervision.
- **Desynchronization:** Lack of parity between environments (local, staging, prod) and .env.example files.
- **Technical Errors:** Duplicates, spaces, and invalid formats.
- **Privacy:** Accidental exposure of secrets on screen during demos or recordings.
- **Collaboration:** No safe way to share environment configs with teammates without exposing raw secrets.
- **Access Control:** No mechanism to grant granular, role-based access to shared configurations.

## 2. Tech Stack

- **Desktop Client:** Rust (Tauri v2) + TypeScript + React.
- **API:** Rust (Axum).
- **Database:** PostgreSQL (Neon serverless).
- **ORM/Driver:** sqlx (compile-time checked queries).
- **Auth:** Argon2 (password hashing), JWT (session tokens).
- **Crypto:** AES-GCM (E2E encryption). The server stores encrypted blobs; it never sees plaintext values.
- **Styling:** Tailwind CSS v4.
- **Fonts:** Geist Sans (UI), IBM Plex Mono (Data), Source Serif 4 (Accents).

## 3. Features (MVP)

- **Identity:** Registration and login against the API. Argon2 hashes the password server-side. JWT tokens manage the session.
- **Scanner:** Automatic indexing and parsing of .env files in local directories. Handles multiline values, variable expansion (`${VAR}`), and sensitivity heuristics.
- **The Vault:** Remote encrypted storage backed by Neon. Each project tracks its owner and access members. All variable values are encrypted client-side before upload.
- **Access Control:** Role-based permissions per project — `owner`, `editor`, `viewer`. Owners invite collaborators by username with a specific role attached.
- **Auditing:** Structure comparison between real/example files and duplicate detection.
- **Obfuscation:** Dynamic blur for sensitive values. Viewer role enforces obfuscation by default.
- **Generator:** Cryptographic string creation for new variables.

## 4. Architecture

- **Data Sovereignty:** Variable values are encrypted client-side with AES-GCM before leaving the device. The API and database only handle ciphertext. Decryption happens exclusively in the Tauri client.
- **API Layer:** Axum REST API handles auth, project CRUD, membership management, and encrypted entry storage. Stateless; all state lives in Neon.
- **Identity:** Argon2 password hashing on the server. JWT access tokens with expiration. No master password needed — the encryption key is derived per-user and stored securely in the client session.
- **Ownership Model:** Every project has an `owner_id`. Users only see their own projects plus those shared with them. Role validation happens server-side.
- **Persistence:** Neon PostgreSQL as the single source of truth. The Tauri client may cache data locally for offline access in future iterations.
- **Security:** E2E encryption (AES-GCM), Argon2 password hashing, JWT auth, role-based access control enforced at the API level.

## 5. Getting Started

```bash
# 1. Clone and install frontend dependencies
bun install

# 2. Set up the API environment
cp vellum-api/.env.example vellum-api/.env
# Edit vellum-api/.env with your Neon connection string and JWT secret

# 3. Run the SQL migration in your Neon console
# Paste the contents of vellum-api/migrations/001_init.sql

# 4. Start the API (terminal 1)
cd vellum-api && cargo run

# 5. Start the desktop client (terminal 2)
bun run tauri dev

# Test credentials (already seeded in Neon)
# Username: test
# Password: testtest123
```

## 6. License

Distributed under the GPL-3.0 License. See `LICENSE` for more information.
