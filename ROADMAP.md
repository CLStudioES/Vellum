# Vellum Roadmap

## Phase 1: Foundation & Discovery

- [x] Project scaffolding (Tauri v2 + React + Tailwind v4).
- [x] Directory scanning via Rust bridge.
- [x] Content parsing: Key=Value extraction, multiline support, sensitivity heuristics.
- [x] UI: Dashboard layout with scan results, audit cards, and obfuscation toggles.
- [x] Native directory picker via `tauri-plugin-dialog`.
- [x] Unified `scan_directory` command: single IPC call scans and parses all .env files.
- [x] Local auth module with Argon2 key derivation and in-memory session.
- [x] AES-GCM encrypted local vault with project CRUD and role-based members.

## Phase 2: API & Remote Vault (Current)

- [ ] Axum REST API: project scaffolding, routing, error handling.
- [ ] Neon PostgreSQL: schema design and migrations (users, projects, members, entries).
- [ ] Auth endpoints: register, login, JWT issuance and validation.
- [ ] Project endpoints: create, list (owned + shared), update, delete.
- [ ] Membership endpoints: invite user by username with role, remove member, update role.
- [ ] Entry endpoints: store and retrieve E2E encrypted variable blobs.
- [ ] Tauri client: replace local vault calls with API requests. Keep scanner local.
- [ ] Role enforcement: server-side validation on every mutating endpoint.

## Phase 3: Collaboration & Access Control

- [ ] Invitation flow: owner invites collaborator by username, assigns role.
- [ ] Project dashboard: list owned and shared projects with role indicators.
- [ ] Editor capabilities: modify variables, add/remove env files within a project.
- [ ] Viewer restrictions: read-only access, values always obfuscated, no export.
- [ ] Ownership transfer: owner can promote an editor to owner.

## Phase 4: Auditing & Sync

- [ ] Snapshot: save the current state of all environments in a project.
- [ ] Drift detection: compare local .env files against the remote vault.
- [ ] .env.example watcher: alert when keys in example don't match local files.
- [ ] Selective sync: per-variable and per-file checkboxes before pushing to the vault.

## Phase 5: Refinement

- [ ] Offline mode: local cache for read access when the API is unreachable.
- [ ] Generator: cryptographic string creation for new variables inline.
- [ ] Granular permissions: per-variable visibility rules for viewer role.
- [ ] Activity log: track who changed what and when within a project.
