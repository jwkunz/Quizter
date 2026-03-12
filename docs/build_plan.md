# Quizter Build and Release Plan (Finalized Commit + Archive Rules)

## Summary
Implement Quizter in staged minor versions starting at `v0.0.0`, with one git commit per minor bump.
At every commit, run the build script, which must remove old archives and generate fresh versioned zip artifacts.

## Stage and Version Policy
1. Start at `v0.0.0` with repo init and docs setup.
2. Bump minor each stage (`v0.1.0`, `v0.2.0`, …).
3. For each stage:
   - complete stage work,
   - update `docs/build_notes.md`,
   - run build script,
   - verify artifacts,
   - commit with versioned message/tag convention.

## Required Docs
- `docs/build_plan.md`: canonical implementation plan/spec.
- `docs/build_notes.md`: append-only stage log including version/date/completed items/build artifacts/test status/known issues/next stage.

## Build Script and Artifact Contract
- Add a build script (repo-local, runnable in CI and locally) that:
  - accepts/derives the current version,
  - deletes previous archives before packaging,
  - builds and outputs fresh zips only for current version.
- Produce separate zips:
  - `quizter-server-<target>-vX.Y.Z.zip`
  - `quizter-player-vX.Y.Z.zip`
  - `quizter-admin-vX.Y.Z.zip`
- Server targets built by GitHub workflow:
  - `x86_64-unknown-linux-gnu`
  - `x86_64-pc-windows-msvc`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`

## Implementation Scope (Functional)
- LAN Rust authoritative server, room code join, admin passcode, reconnect.
- Sequential random rounds, 15s timeout, progress display.
- Scoring: correct = base points + speed bonus; incorrect = 0.
- Six one-time power-ups with broadcast activation and defined behaviors.
- Admin guided manual question panel + JSON pack support.
- Player join flow includes short wizard tutorial.
- Neon-on-black visual style and asset folders.

## Test and Acceptance
- Per-stage validation includes functional checks plus packaging checks.
- At every commit, confirm old archives removed, new archives created with versioned names, and expected zip contents present.
- Stabilization includes 20-player LAN reliability checks and role/security verification.
