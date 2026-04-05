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
  - `quizter-server-local-v0.1.0.zip`
  - `quizter-player-v0.1.0.zip`
  - `quizter-admin-v0.1.0.zip`
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
  - `quizter-server-local-v0.2.0.zip`
  - `quizter-player-v0.2.0.zip`
  - `quizter-admin-v0.2.0.zip`
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
  - `quizter-server-local-v0.3.0.zip`
  - `quizter-player-v0.3.0.zip`
  - `quizter-admin-v0.3.0.zip`
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
  - `quizter-server-local-v0.4.0.zip`
  - `quizter-player-v0.4.0.zip`
  - `quizter-admin-v0.4.0.zip`
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
  - `quizter-server-local-v0.5.0.zip`
  - `quizter-player-v0.5.0.zip`
  - `quizter-admin-v0.5.0.zip`
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
  - `quizter-server-local-v0.6.0.zip`
  - `quizter-player-v0.6.0.zip`
  - `quizter-admin-v0.6.0.zip`
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
  - `quizter-server-local-v0.7.0.zip`
  - `quizter-player-v0.7.0.zip`
  - `quizter-admin-v0.7.0.zip`
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
  - `quizter-server-local-v0.8.0.zip`
  - `quizter-player-v0.8.0.zip`
  - `quizter-admin-v0.8.0.zip`
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
  - `quizter-server-local-v0.9.0.zip`
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
  - `quizter-server-local-v0.10.0.zip`
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
  - `quizter-server-local-v1.0.0.zip`
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
  - `quizter-server-local-v1.0.2.zip`
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
  - `quizter-server-local-v1.0.3.zip`
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
  - `quizter-server-local-v1.1.0.zip`
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
  - `quizter-server-local-v1.2.0.zip`
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
  - `quizter-server-local-v1.3.0.zip`
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
  - `quizter-server-local-v1.4.0.zip`
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
  - `quizter-server-local-v1.5.0.zip`
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
  - `quizter-server-local-v1.6.0.zip`
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
  - `quizter-server-local-v1.7.0.zip`
- Test status:
  - `cargo check` passes.
  - Build and artifact verification scripts pass.
- Known issues:
  - None specific to this milestone.
- Next stage:
  - v2.0.0 release docs and finalization.

## v2.0.0 - 2026-03-11
- Completed:
  - Implemented player UX feedback package:
    - 1-second full-screen round result flash.
    - Single-page detailed instruction panel with full power-up documentation.
    - De-emphasized bottom game feed.
    - Red top push alerts for affected players on impacting power-ups.
  - Updated power-up gameplay behavior:
    - Speed Searcher extended to 60 seconds.
  - Implemented admin bank-selection system:
    - persisted file-bank selection state.
    - backend bank list/selection APIs.
    - live alphabetical checkbox selector with Add All / Clear All.
    - next-round-only application of mid-game selection changes.
  - Added admin lobby usability guardrails:
    - explicit top-field labels.
    - no-question start guardrails.
  - Hardened round pipeline handling for live question-pool updates.
  - Expanded README with beginner GitHub Releases download instructions and updated gameplay/admin docs.
- Build artifacts:
  - `quizter-server-local-v2.0.0.zip`
- Test status:
  - `cargo check` passes.
  - Build and artifact verification scripts pass.
- Known issues:
  - None release-blocking.
- Next stage:
  - Post-2.0 polish and additional gameplay features.

## v2.1.0 - 2026-03-12
- Completed:
  - Replaced question-pack JSON field usage from `id` to `category` across all files in `assets/questions/`.
  - Sorted built-in question packs into five categories:
    - `History`
    - `Religion`
    - `STEM`
    - `Literature`
    - `Geography`
  - Updated server question handling to preserve an internal runtime ID while exposing category-driven pack data.
  - Added category metadata to player question state so the live player UI shows the question category during each round.
  - Reworked the admin question pool filter into a category tree with per-category bulk add/clear controls plus per-file checkboxes.
  - Updated README and admin help text to describe the new `category` field and category-based filter flow.
- Build artifacts:
  - Not run for this milestone.
- Test status:
  - `cargo check` passes.
  - Verified all `assets/questions/*.json` files now use `category` and no longer use `id`.
- Known issues:
  - Existing legacy manual question files may still deserialize without a category and will default to `Uncategorized`.
- Next stage:
  - Optional release build and broader category-aware admin/player polish.

## v2.2.0 - 2026-03-12
- Completed:
  - Changed built-in question packs to use a root JSON object with:
    - optional `category`
    - `questions`
  - Removed repeated per-question `category` values from `assets/questions/*.json` to reduce duplication.
  - Updated server loading to support:
    - new root object packs
    - legacy plain arrays for backward compatibility
  - Added default internal category assignment of `Generic` when a pack does not specify a root category.
  - Updated pack export/import serialization paths to emit the root pack format.
  - Made the admin category tree fully data-driven from discovered pack categories instead of fixed category assumptions.
  - Updated README and admin help text to document the new root pack schema and dynamic category discovery.
- Build artifacts:
  - Not run for this milestone.
- Test status:
  - `cargo check` passes.
  - Verified all built-in question packs now use root `category` + `questions` structure.
- Known issues:
  - Legacy manual question storage still uses per-question records internally and defaults missing categories to `Generic`.
- Next stage:
  - Optional release build and additional category-management polish.

## v2.3.0 - 2026-03-12
- Completed:
  - Added admin support for manual or automatic question issue flow.
  - Added `Enable Automatic Question Issue` with configurable seconds between questions.
  - Renamed admin `Force Next Round` action to `Issue Question`.
  - Implemented server-side delayed round start after round results when automatic issue is enabled.
  - Updated README and admin help text to explain manual question issue versus automatic timed issue.
- Build artifacts:
  - Not run for this milestone.
- Test status:
  - `cargo check` passes.
- Known issues:
  - Automatic issue uses a simple server-side timer and does not currently display a separate admin countdown between rounds.
- Next stage:
  - Consolidated admin game settings and additional rule toggles.

## v2.4.0 - 2026-03-12
- Completed:
  - Consolidated gameplay controls into a dedicated admin `Game Settings` panel.
  - Added configurable game settings:
    - response speed bonus enable/disable
    - hide player scores until end of game
    - power-up enable/disable
    - seconds allowed for responses
    - automatic timed question issue enable/disable
    - seconds between automatic question issue
  - Updated server round handling to honor configurable response time and automatic issue delay.
  - Updated scoring to optionally remove the response speed bonus while preserving base scoring.
  - Updated player snapshots so score visibility can be hidden until game end for players only.
  - Updated player UI to hide the power-up panel when power-ups are disabled.
  - Updated README and admin help instructions to explain the consolidated settings and both question-issue flows.
  - Changed bank selection startup behavior so no question packs are selected when the server boots.
- Build artifacts:
  - Not run for this milestone.
- Test status:
  - `cargo check` passes.
- Known issues:
  - Manual round issue remains available even when automatic issue is enabled, which allows an admin to issue the next question early if desired.
- Next stage:
  - Optional release build and more advanced admin orchestration polish.

## v2.5.0 - 2026-03-12
- Completed:
  - Increased displayed PNG art sizing across admin/player UI while leaving the QR code unchanged.
  - Doubled the full-screen round-result flash duration from 1 second to 2 seconds.
- Build artifacts:
  - Not run for this milestone.
- Test status:
  - UI-only change; no server logic changes required.
- Known issues:
  - Additional local question-pack edits remain outside this milestone.
- Next stage:
  - Optional build, art polish, and question-pack updates.

## v2.6.0 - 2026-03-12
- Completed:
  - Removed the admin `Apply Filter` step so question-pack filtering now follows the live checked state immediately.
  - Updated admin filter actions so category toggles and add-all/clear-all push the selection to the server right away.
  - Moved the player leaderboard above the question panel for easier score visibility during gameplay.
  - Reduced power-up button size while keeping the art visible.
  - Added a centered power-up activation flash using the power-up artwork.
  - Updated incorrect and missed-round result flashes to show the correct answer.
  - Updated README and admin help text to match the live filter and player feedback behavior.
- Build artifacts:
  - Not run for this milestone.
- Test status:
  - `cargo check` passes.
- Known issues:
  - Additional local question-pack edits remain outside this milestone.
- Next stage:
  - Optional build, question-pack updates, and further gameplay polish.

## v2.7.0 - 2026-03-12
- Completed:
  - Reorganized the admin console into six top-level tabs: `Welcome`, `Lobby`, `Game Settings`, `Question Pool`, `Game Monitor`, and `Help`.
  - Added a Welcome splash screen with logo, software version, and legal/support text.
  - Moved room setup, QR code, and admin login into the Lobby tab.
  - Moved game rules into the Game Settings tab.
  - Renamed `Current Question Pool Filter` to `Question Pool Selection` and placed it in the Question Pool tab.
  - Moved live feed and leaderboard into the Game Monitor tab.
  - Added a Help tab containing setup steps, question-pack instructions, game guidance, troubleshooting notes, LLM prompt guidance, and artwork.
  - Updated the README to mirror the welcome/legal content and new admin tab structure.
- Build artifacts:
  - Not run for this milestone.
- Test status:
  - `cargo check` passes.
- Known issues:
  - Additional local question-pack edits remain outside this milestone.
- Next stage:
  - Optional build, question-pack updates, and further admin polish.

## v2.8.0 - 2026-03-12
- Completed:
  - Reordered the admin tabs so `Question Pool` appears immediately after `Lobby`.
  - Added a large countdown widget to `Game Monitor` showing either answer time remaining or time before the next round starts.
  - Added a manual `Issue Question` button to `Game Monitor` for manual round issue mode.
  - Increased the incorrect-result flash duration on the player UI.
  - Made the correct answer text more prominent beneath the incorrect result graphic.
  - Updated README and admin help text to explain the new `Game Monitor` countdown flow.
- Build artifacts:
  - Not run for this milestone.
- Test status:
  - `cargo check` passes.
- Known issues:
  - Additional local question-pack edits remain outside this milestone.
- Next stage:
  - Optional build, question-pack updates, and additional admin/player polish.

## v2.9.0 - 2026-03-12
- Completed:
  - Added a player-side round countdown widget below the leaderboard.
  - During active rounds, the widget shows the time remaining to answer.
  - During automatic round transitions, the widget shows `Time Until Next Question` with a live countdown.
  - During manual round transitions, the widget shows `Waiting for Host to Issue Question`.
- Build artifacts:
  - Not run for this milestone.
- Test status:
  - `cargo check` passes.
- Known issues:
  - Additional local question-pack edits remain outside this milestone.
- Next stage:
  - Optional build, question-pack updates, and additional admin/player polish.

## v2.10.0 - 2026-03-12
- Completed:
  - Moved the `Start Game` button and main game status line to the top of the `Game Monitor` tab.
  - Kept existing game control behavior intact while making the monitor tab the host's main live-control surface.
- Build artifacts:
  - Not run for this milestone.
- Test status:
  - `cargo check` passes.
- Known issues:
  - Additional local question-pack edits remain outside this milestone.
- Next stage:
  - Optional build, question-pack updates, and additional admin/player polish.

## v3.0.0 - 2026-03-12
- Completed:
  - Finalized the current Quizter gameplay/admin UX milestone set and included the latest question-bank asset updates.
  - Updated the local build script to package the full `assets/` tree, including images, music, sfx, and question packs.
  - Updated artifact verification to confirm the packaged archive contains the full asset bundle.
  - Updated the GitHub release workflow to use `quizter-server-*` artifact names and include both macOS release targets.
- Build artifacts:
  - `quizter-server-local-v3.0.0.zip`
- Test status:
  - `cargo check` passes.
  - `scripts/build_release.sh "$(cat VERSION)" local` passes.
  - `scripts/verify_artifacts.sh "$(cat VERSION)" local` passes.
- Known issues:
  - None recorded for this release pass beyond normal platform-specific runtime validation in CI.
- Next stage:
  - Publish `v3.0.0` release artifacts and monitor workflow output.

## v3.1.0 - 2026-03-12
- Completed:
  - Added a large `Exit and Close` button at the top of the admin UI.
  - Added an admin-authenticated shutdown endpoint so the server can be stopped cleanly from the browser.
  - Updated Help and README text to warn that closing only the browser may leave the server running.
  - Added best-effort terminal relaunch logic so desktop launches try to keep the server visible in a terminal window instead of disappearing into the background.
  - Updated local packaging and verified the release archive after the shutdown/launch changes.
- Build artifacts:
  - `quizter-server-local-v3.1.0.zip`
- Test status:
  - `cargo check` passes.
  - `scripts/build_release.sh v3.1.0 local` passes.
  - `scripts/verify_artifacts.sh v3.1.0 local` passes.
- Known issues:
  - Terminal relaunch depends on platform terminal availability and falls back to normal in-place launch if a supported terminal app is not found.
- Next stage:
  - Monitor the `v3.1.0` GitHub Actions release build and confirm platform-specific shutdown behavior.

## v4.0.0 - 2026-04-04
- Completed:
  - Defined the hosted-product transformation plan for moving Quizter from a
    single-room LAN app to a multi-room public hosted service.
  - Added a Phase 1 backend architecture checklist focused on replacing the
    single global game model with a room registry.
  - Bumped version markers to `v4.0.0` and aligned Cargo package versioning with
    the published app version.
  - Began Phase 1 backend implementation by introducing a room registry and
    room-scoped client connection metadata while preserving the legacy default
    room behavior.
  - Converted server broadcast and state snapshot plumbing to resolve room
    context from connected clients instead of relying only on one global app
    game handle.
  - Added reusable room access helpers and explicit room-scoped broadcast
    helpers to reduce direct default-room lock plumbing in server handlers.
  - Moved round progression, timer dispatch, and round finalization onto
    explicit room-code-based functions so the gameplay engine no longer hard
    codes the legacy default room path internally.
  - Updated admin login, player join, and websocket client attachment to resolve
    room membership by room code or known client identity instead of always
    binding connections to the compatibility default room.
  - Added generated 4-character room-code creation, room-template cloning, and
    a new backend `POST /api/rooms/create` endpoint that inserts real separate
    rooms into the in-memory registry.
  - Added room titles to room state and to the serialized state snapshot for
    future hosted UI work.
  - Updated `start_game` to resolve the admin's actual room membership so
    independently created rooms can run their own game loop instead of only the
    legacy default room being startable.
  - Moved question-pack export, pack listing, and pack selection APIs onto the
    admin's actual room so each created room can inspect and modify its own
    effective content pool independently.
  - Moved manual question add/import and question-bank import APIs onto the
    admin's actual room so question-pool mutations no longer write only to the
    legacy default room.
  - Added owner tokens and an owner-token index for hosted rooms.
  - Updated hosted room creation to return an owner token and added
    `POST /api/rooms/resume` so a hosted room can be resumed without relying on
    the legacy admin passcode flow.
  - Added `POST /api/rooms/close` so a hosted room can be explicitly closed by
    owner token, with owner-index cleanup and room-scoped client disconnect
    cleanup.
  - Added a background room cleanup task that expires non-legacy rooms after 30
    minutes of inactivity using the tracked `last_activity_at` field.
  - Unified explicit close and inactivity expiration through a shared room
    removal helper that cleans room state, owner-token mappings, and
    room-scoped clients.
  - Added owner-token-authenticated hosted room control endpoints for question
    bank inspection, question bank selection, and game start so future hosted UI
    work does not need to depend on the legacy admin login flow.
  - Added a new hosted landing page at `/` with `Create Room`, `Resume Room`,
    and `Join Room` flows backed by the owner-token room APIs.
  - Updated the player page to honor `?room=CODE` query parameters so hosted
    landing-page joins can prefill the room code automatically.
  - Extended the hosted landing page into an initial owner control surface that
    can load question packs, apply pack selection, choose rounds, and start a
    game through the owner-token room APIs.
- Build artifacts:
  - Not run for this planning milestone.
- Test status:
  - `cargo check` passes after adding hosted room pack and start controls to the
    landing page.
- Known issues:
  - Hosted architecture is still only partially implemented.
  - Current routes and UI still operate through the legacy default room.
- Next stage:
  - Continue Phase 1 by introducing explicit room/game session types beyond the
    legacy default room compatibility path.
