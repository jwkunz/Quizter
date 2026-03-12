# Quiztik

Quiztik is a LAN multiplayer trivia game with a Rust authoritative server and single-file web clients for players and admins.

## Run locally

```bash
cd server
cargo run --release
```

Open:
- Admin UI: `http://localhost:8080/admin`
- Player UI: `http://localhost:8080/player`

## Build release archives

```bash
scripts/build_release.sh "$(cat VERSION)" local
```

The script deletes old archives in `dist/` and creates fresh versioned zips:
- `quiztik-server-<target>-vX.Y.Z.zip`
- `quiztik-player-vX.Y.Z.zip`
- `quiztik-admin-vX.Y.Z.zip`

## Game highlights

- Room code + display name join
- Admin passcode controls
- Sequential random questions with 15-second timeout
- Correct points + speed bonus scoring
- Six one-time power-ups with global activation notifications
- Guided admin question builder + JSON import
- Player quick tutorial wizard
