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

## v0.5.0 - 2026-03-11
- Completed:
  - Added archive verification script to validate versioned zip outputs and expected content.
  - Added acceptance checklist covering packaging, smoke tests, core rules, and CI matrix.
- Build artifacts:
  - `quiztik-server-local-v0.5.0.zip`
  - `quiztik-player-v0.5.0.zip`
  - `quiztik-admin-v0.5.0.zip`
- Test status:
  - `scripts/verify_artifacts.sh` passes for local build.
- Known issues:
  - Load testing for full 20-player scenario is still a manual validation item.
- Next stage:
  - Feature extension and production hardening.

## v0.6.0 - 2026-03-11
- Completed:
  - Changed server startup behavior to always rebuild the runtime question bank from `assets/questions/*.json` first.
  - Persists rebuilt merged bank to `data/questions.json` after load.
- Build artifacts:
  - `quiztik-server-local-v0.6.0.zip`
  - `quiztik-player-v0.6.0.zip`
  - `quiztik-admin-v0.6.0.zip`
- Test status:
  - `cargo check` passes.
- Known issues:
  - None specific to this change.
- Next stage:
  - Additional bank tooling and import UX enhancements.

## v0.7.0 - 2026-03-11
- Completed:
  - Added UI image integration for logo, power-ups, and round result feedback.
  - Added static asset serving from server (`/assets`).
  - Updated packaging so player/admin zips include images and admin zip includes question banks.
  - Added and committed current image and question-bank resources.
- Build artifacts:
  - `quiztik-server-local-v0.7.0.zip`
  - `quiztik-player-v0.7.0.zip`
  - `quiztik-admin-v0.7.0.zip`
- Test status:
  - `cargo check` passes.
  - Packaging content verified for player/admin images and admin question banks.
- Known issues:
  - None specific to this stage.
- Next stage:
  - Continued gameplay polish and feature expansion.

## v0.8.0 - 2026-03-11
- Completed:
  - Added admin-side Player Join QR panel that encodes the LAN player URL.
  - Added server endpoints: `/api/server_info` and `/api/qr.svg`.
  - Added host IP detection and env-configurable host/port for LAN URL generation.
  - Ensured phones can open `/player` directly from QR URL without manual web app download.
- Build artifacts:
  - `quiztik-server-local-v0.8.0.zip`
  - `quiztik-player-v0.8.0.zip`
  - `quiztik-admin-v0.8.0.zip`
- Test status:
  - `cargo check` passes.
- Known issues:
  - Host IP detection may require manual override on unusual network setups.
- Next stage:
  - UX polish and broader integration coverage.

## v0.9.0 - 2026-03-11
- Completed:
  - Switched release model to server-only distribution zips.
  - Updated server zip contents to include hosted admin/player web files plus image and question assets.
  - Added automatic admin browser launch from server executable startup.
  - Made web server URL fields dynamic so they default to the running server address.
  - Removed `integration_test_launch.sh`.
- Build artifacts:
  - `quiztik-server-local-v0.9.0.zip`
- Test status:
  - `cargo check` passes.
  - `scripts/verify_artifacts.sh` passes for local build.
- Known issues:
  - None specific to this stage.
- Next stage:
  - Documentation expansion for non-technical server admins.

## v0.10.0 - 2026-03-11
- Completed:
  - Rewrote README with verbose non-technical setup and hosting instructions for GitHub users.
  - Added step-by-step admin workflow, player onboarding, troubleshooting, and network guidance.
- Build artifacts:
  - `quiztik-server-local-v0.10.0.zip`
- Test status:
  - `scripts/verify_artifacts.sh` passes for local build.
- Known issues:
  - None specific to documentation stage.
- Next stage:
  - Optional UX/tutorial enhancements and production polish.

## v1.0.0 - 2026-03-11
- Completed:
  - Added MIT license.
  - Added GitHub release workflow that builds server artifacts for Linux, Windows, and macOS on version tags.
  - Workflow now creates a GitHub Release and attaches built zip assets.
- Build artifacts:
  - `quiztik-server-local-v1.0.0.zip`
- Test status:
  - `scripts/verify_artifacts.sh` passes for local build.
- Known issues:
  - None blocking v1.0.0 release.
- Next stage:
  - Post-release maintenance and feature enhancements.

## v1.0.2 - 2026-03-11
- Completed:
  - Fixed admin UI API-base handling to prevent create/update room request hangs.
  - Fixed player UI server URL handling to avoid incorrect automatic endpoint rewrites.
  - Replaced invalid empty question banks with valid JSON content for:
    - `assets/questions/electronics.json`
    - `assets/questions/us_states.json`
  - Validated all question bank JSON files.
- Build artifacts:
  - `quiztik-server-local-v1.0.2.zip`
- Test status:
  - `cargo check` passes.
  - `scripts/verify_artifacts.sh` passes for local build.
- Known issues:
  - None specific to this patch.
- Next stage:
  - Continue gameplay and UX improvements.

## v1.0.3 - 2026-03-11
- Completed:
  - Fixed CI portability issues in release scripts:
    - removed `rg`/`unzip` dependency from artifact verification.
    - added Python-based zip creation fallback when `zip` is unavailable.
  - Updated release workflow to ensure Python is available on runners.
- Build artifacts:
  - `quiztik-server-local-v1.0.3.zip`
- Test status:
  - `cargo check` passes.
  - `scripts/verify_artifacts.sh` passes for local build.
- Known issues:
  - None specific to this patch.
- Next stage:
  - Continue post-release reliability and UX improvements.

## v1.1.0 - 2026-03-11
- Completed:
  - Added full-screen 1-second round result flash (Correct/Incorrect/No Answer) using result graphics.
  - Kept panel feedback as secondary while making the flash the primary end-of-round scoring signal.
  - De-emphasized player feed panel and kept it bottom-priority in layout.
- Build artifacts:
  - `quiztik-server-local-v1.1.0.zip`
- Test status:
  - Build and artifact verification scripts pass.
- Known issues:
  - None specific to this milestone.
- Next stage:
  - Single-page instruction overhaul with detailed power-up descriptions.

## v1.2.0 - 2026-03-11
- Completed:
  - Replaced step-by-step tutorial with a single-page instruction panel.
  - Added detailed power-up descriptions for all six power-ups.
  - Kept simple `Continue` and `Skip` actions with tutorial-seen persistence behavior.
- Build artifacts:
  - `quiztik-server-local-v1.2.0.zip`
- Test status:
  - Build and artifact verification scripts pass.
- Known issues:
  - None specific to this milestone.
- Next stage:
  - Speed Searcher 60-second update and affected-player red push alerts.

## v1.3.0 - 2026-03-11
- Completed:
  - Increased Speed Searcher exclusive answer window from 30s to 60s.
  - Added affected-player alert metadata to `powerup_activated` events.
  - Added red top push-banner alerts in player UI for affected players.
  - Added affected-player alert coverage for Mix Master, Speed Searcher, Super Spliter, and Great Gambler.
- Build artifacts:
  - `quiztik-server-local-v1.3.0.zip`
- Test status:
  - `cargo check` passes.
  - Build and artifact verification scripts pass.
- Known issues:
  - None specific to this milestone.
- Next stage:
  - Backend question bank selection API with persisted selection state.

## v1.4.0 - 2026-03-11
- Completed:
  - Added backend question-bank file selection APIs:
    - `GET /api/admin/question_banks`
    - `POST /api/admin/question_banks/selection`
  - Added persisted selected-bank-file state in runtime data (`selected_bank_files.json`).
  - Introduced separate manual question pool vs file-bank question pool.
  - Added effective pool rebuild and future-round reflow logic so mid-game bank updates apply from next round onward.
  - Set first-run file-bank selection default to all-off.
- Build artifacts:
  - `quiztik-server-local-v1.4.0.zip`
- Test status:
  - `cargo check` passes.
  - Build and artifact verification scripts pass.
- Known issues:
  - Admin UI for selecting banks is pending next milestone.
- Next stage:
  - Admin bank selector checklist UI and live controls.

## v1.5.0 - 2026-03-11
- Completed:
  - Added admin question-bank selector UI with alphabetical checklist.
  - Added `Add All Banks` and `Clear All Banks` controls.
  - Added live selection updates to backend selection API.
  - Added selected-bank summary and empty-selection warning.
- Build artifacts:
  - `quiztik-server-local-v1.5.0.zip`
- Test status:
  - Build and artifact verification scripts pass.
- Known issues:
  - Lobby top fields still need explicit labels and start-game guardrails.
- Next stage:
  - Lobby labeling and empty-pool guardrails.

## v1.6.0 - 2026-03-11
- Completed:
  - Added explicit labels to admin lobby fields: Server URL, Room Code, Admin Passcode, Rounds.
  - Added start-game guardrail behavior when no playable questions are available.
  - Updated room creation round-limit handling for empty pools.
  - Removed built-in manual-question defaults to honor all-off file-bank start state.
- Build artifacts:
  - `quiztik-server-local-v1.6.0.zip`
- Test status:
  - `cargo check` passes.
  - Build and artifact verification scripts pass.
- Known issues:
  - Integration pass pending for full multi-scenario stability validation.
- Next stage:
  - Integration hardening for mid-game bank updates and alert behavior.

## v1.7.0 - 2026-03-11
- Completed:
  - Hardened round pipeline when live bank selection changes shrink/alter future queue.
  - Added safe end-of-game fallback if the next round cannot resolve a valid question.
  - Extended bank-selection response with effective question count metadata.
  - Verified affected-player alert targeting remains scoped to players listed by server payload.
- Build artifacts:
  - `quiztik-server-local-v1.7.0.zip`
- Test status:
  - `cargo check` passes.
  - Build and artifact verification scripts pass.
- Known issues:
  - None specific to this milestone.
- Next stage:
  - v2.0.0 release docs and finalization.
