# Vellum Roadmap

## Phase 1: Foundation & Discovery (Current)

- [x] Project scaffolding (Tauri v2 + React + Tailwind v4).
- [x] Basic directory scanning via Rust bridge.
- [ ] UI: Dashboard layout with IBM Plex Mono for data visualization.
- [ ] Engine: Content parsing to extract Key=Value pairs from files.

## Phase 2: The Vault (Persistence & Security)

- [ ] Local Database: Implementation of SQLite/JSON storage for snapshots.
- [ ] Encryption Layer: AES-GCM for sensitive data at rest in the Vault.
- [ ] Feature: "Snapshot" - Save the current state of all environments in a project.

## Phase 3: Synchronization & Portability

- [ ] Feature: "Export Bundle" - Generate a single encrypted `.vlm` file.
- [ ] Feature: "Import Bundle" - Reconstruct `.env` files on a new machine.
- [ ] Audit Tool: Compare local files vs. Vault snapshots to detect drift.

## Phase 4: Refinement

- [ ] Dynamic Blur: Toggle visibility for sensitive keys.
- [ ] .env.example Watcher: Alert when keys in example don't match local files.
