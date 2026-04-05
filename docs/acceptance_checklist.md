# Acceptance Checklist

Use this checklist before tagging or sharing a release build.

## Packaging
- Run: `scripts/build_release.sh "$(cat VERSION)" local`
- Run: `scripts/verify_artifacts.sh "$(cat VERSION)" local`
- Confirm `dist/` only includes current version archives.

## Functional Smoke Test
- Start server: `cd server && cargo run --release`
- Run hosted room smoke test: `scripts/smoke_hosted_flow.sh`
- For hosted manual flow, confirm players cannot join before the host launches the room.
- Open `/admin`, create room, login, add at least 2 questions manually.
- Open `/player` from one or more browsers and join room.
- Start game, answer questions, trigger power-ups, verify round results and leaderboard.

## Key Rules
- 15s timeout when unanswered.
- Correct score = base + speed bonus.
- Incorrect score = 0.
- Each power-up is usable once per player per game.
- Player tutorial wizard is accessible during join flow.

## CI Matrix
- GitHub workflow builds targets:
  - `x86_64-unknown-linux-gnu`
  - `x86_64-pc-windows-msvc`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
