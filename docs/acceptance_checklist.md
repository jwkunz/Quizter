# Acceptance Checklist

Use this checklist before tagging or sharing a release build.

## Packaging
- Run: `scripts/build_release.sh "$(cat VERSION)" local`
- Run: `scripts/verify_artifacts.sh "$(cat VERSION)" local`
- Confirm `dist/` only includes current version archives.

## Functional Smoke Test
- Start server: `cd server && cargo run --release`
- Run hosted room smoke test: `scripts/smoke_hosted_flow.sh`
- Confirm the hosted homepage at `/` is the primary host entry point.
- Confirm players cannot join before the host launches the room.
- Create a room from `/`.
- Select at least one server-side question pack.
- Launch the room and confirm QR/link become visible only after launch.
- Join from `/player` in one or more browsers or phones.
- Start a game, answer questions, verify round results and leaderboard behavior.
- Confirm late joiners wait until the next question.
- End the game and confirm the room can start another game without being closed.
- Close the room and confirm players are removed.

## Key Rules
- 15s timeout when unanswered.
- Correct score = base + speed bonus.
- Incorrect score = 0.
- Each power-up is usable once per player per game.
- Player tutorial wizard is accessible during join flow.
- Hosted room links and QR codes use `QUIZTER_PUBLIC_BASE_URL` when configured.

## CI Matrix
- GitHub workflow builds targets:
  - `x86_64-unknown-linux-gnu`
  - `x86_64-pc-windows-msvc`
  - `x86_64-apple-darwin`
  - `aarch64-apple-darwin`
