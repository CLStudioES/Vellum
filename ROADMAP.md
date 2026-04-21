# Vellum Roadmap

## Phase 1: Foundation & Discovery

- [x] Project scaffolding (Tauri v2 + React + Tailwind v4).
- [x] Basic directory scanning via Rust bridge.
- [x] Engine: Content parsing with Key=Value extraction, multiline support, and sensitivity heuristics.
- [x] UI: Dashboard layout with scan results, audit cards, and obfuscation toggles.
- [x] Native directory picker via `tauri-plugin-dialog`.
- [x] Unified `scan_directory` command: single IPC call scans and parses all .env files at once.

## Phase 2: Identity & Access Control (Current)

- [ ] Auth Module: Local registration and login system.
- [ ] Key Derivation: Replace raw passphrase with Argon2-based key derivation from master password.
- [ ] Session Management: Vault key held in memory during app lifecycle. Cleared on close.
- [ ] User Profile: Encrypted local storage for user identity.
- [ ] Ownership: Every project linked to its creator via `owner_id`.
- [ ] Roles: `owner`, `editor`, `viewer` per project member.
- [ ] UI: Login/Register screen as app entry point. Dashboard gated behind auth.

## Phase 3: The Vault (Persistence & Security)

- [x] Encryption Layer: AES-GCM for sensitive data at rest in the Vault.
- [x] Project struct with multi-file support and vault persistence.
- [ ] Feature: "Snapshot" — Save the current state of all environments in a project.
- [ ] Project Dashboard: List owned and shared projects with role indicators.
- [ ] Selective Sync: Per-variable and per-file checkboxes before committing to the Vault.

## Phase 4: Synchronization & Portability

- [ ] Feature: "Export Bundle" — Generate a single encrypted `.vlm` file with role metadata attached.
- [ ] Feature: "Import Bundle" — Reconstruct `.env` files on a new machine and register as project member.
- [ ] Invitation Flow: Owner exports a Bundle with a target role; recipient imports and gains access.
- [ ] Audit Tool: Compare local files vs. Vault snapshots to detect drift.

## Phase 5: Refinement

- [ ] OS Keychain Integration: Store session tokens in Windows Credential Manager / macOS Keychain.
- [ ] .env.example Watcher: Alert when keys in example don't match local files.
- [ ] Generator: Cryptographic string creation for new variables inline.
- [ ] Granular Permissions: Per-variable visibility rules for viewer role.
