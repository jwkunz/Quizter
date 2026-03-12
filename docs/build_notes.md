# Build Notes

## v0.0.0 - 2026-03-11
- Completed:
  - Initialized git repository and baseline project structure.
  - Added build plan and build notes documents.
  - Added initial build script scaffold and version file.
- Build artifacts:
  - Pending first build script run for baseline.
- Test status:
  - N/A (baseline docs/setup).
- Known issues:
  - Application code not started yet.
- Next stage:
  - v0.1.0 scaffolding.

## v0.1.0 - 2026-03-11
- Completed:
  - Added Rust server scaffold with health endpoint.
  - Added single-file player and admin HTML app shells.
  - Upgraded build script to build local server and package role-specific artifacts.
- Build artifacts:
  - `quiztik-server-local-v0.1.0.zip`
  - `quiztik-player-v0.1.0.zip`
  - `quiztik-admin-v0.1.0.zip`
- Test status:
  - `cargo build --release` passes for server.
- Known issues:
  - Gameplay and realtime protocol not implemented yet.
- Next stage:
  - v0.2.0 session and role system.
