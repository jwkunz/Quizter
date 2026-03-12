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

## v0.2.0 - 2026-03-11
- Completed:
  - Implemented Rust authoritative LAN game server core: room join/admin auth, websocket sync, sequential rounds, 15s timeout, random question selection, scoring (correct + speed bonus, incorrect = 0), reconnect-by-name, and game history persistence.
  - Implemented six one-time power-ups with activation broadcasts.
  - Added full single-file player and admin web apps with neon-on-black styling.
  - Added player join tutorial wizard and admin guided manual question builder + JSON question import.
  - Fixed repository hygiene by removing tracked build output and strengthening ignore patterns.
- Build artifacts:
  - `quiztik-server-local-v0.2.0.zip`
  - `quiztik-player-v0.2.0.zip`
  - `quiztik-admin-v0.2.0.zip`
- Test status:
  - `cargo check` passes.
- Known issues:
  - Local packaging currently produces one local server target; CI multi-OS packaging added in next stage.
- Next stage:
  - v0.3.0 packaging workflow and release automation.

## v0.3.0 - 2026-03-11
- Completed:
  - Added GitHub Actions workflow to build server binaries for Linux/Windows/macOS Intel/macOS Apple Silicon.
  - Updated build script for target-aware packaging with optional build skipping in CI.
  - Ensured build script always removes previous archives and outputs versioned zip names.
  - Added tracked asset directories for upcoming graphics/audio.
- Build artifacts:
  - `quiztik-server-local-v0.3.0.zip`
  - `quiztik-player-v0.3.0.zip`
  - `quiztik-admin-v0.3.0.zip`
- Test status:
  - `cargo check` passes; local packaging run complete.
- Known issues:
  - No automated integration tests yet; current validation is manual and compile-time.
- Next stage:
  - v0.4.0 test harness and release-quality cleanup.

## v0.4.0 - 2026-03-11
- Completed:
  - Added scoring unit tests for base + speed bonus and double-down behavior.
  - Added project README with local run and packaging instructions.
  - Kept release script/version flow aligned with commit-stage process.
- Build artifacts:
  - `quiztik-server-local-v0.4.0.zip`
  - `quiztik-player-v0.4.0.zip`
  - `quiztik-admin-v0.4.0.zip`
- Test status:
  - `cargo test` passes.
- Known issues:
  - Full end-to-end browser automation tests are not yet included.
- Next stage:
  - v0.5.0 final polish and acceptance checklist updates.
