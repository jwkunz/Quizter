# Quiztik

Quiztik is a local-network (LAN) multiplayer quiz game.

One person (the admin/host) runs the server on their computer. Everyone else joins from their phone or computer browser by scanning a QR code or opening a URL.

## What You Download

You only need **one zip file**:

- `quiztik-server-<your-platform>-vX.Y.Z.zip`

This server zip already includes:

- the server executable
- admin web page
- player web page
- images and question banks

No separate player or admin app download is needed.

## Quick Start (Non-Technical)

1. Download the latest server zip from GitHub Releases.
2. Extract/unzip it to a folder (for example: Desktop `Quiztik`).
3. Open that folder.
4. Run the server executable:
   - Windows: double-click `quiztik-server.exe`
   - macOS/Linux: double-click `quiztik-server` (or run from terminal)
5. Your browser should open automatically to the admin page.
6. In admin page:
   - create/update room
   - login as admin
   - start game
7. Ask players to scan the **Player Join QR** shown at top of admin page.
8. Players open on phones and join with room code + display name.

## Important Network Requirements

For phones to join:

- Host/admin computer and player devices must be on the **same Wi-Fi/LAN**.
- If prompted by firewall, **allow** the server on private/local networks.
- If browser does not auto-open, manually visit:
  - `http://127.0.0.1:8080/admin` (on host computer)

## First-Time Admin Walkthrough

After opening admin page:

1. Leave default server URL unless you know you need a different one.
2. Choose/confirm:
   - room code
   - admin passcode
   - number of rounds
3. Click **Create/Update Room**.
4. Click **Admin Login**.
5. Add questions (guided form) or import JSON question packs.
6. Click **Start Game**.
7. Watch player join activity and leaderboard live.

## Player Experience

Players open the hosted player page from QR/URL and can:

- join by room code + name
- answer timed questions
- use one-time power-ups
- see score updates and round progress

No install needed on player devices; modern mobile browsers are enough.

## Question Banks

Question bank JSON files live under:

- `assets/questions/`

On server startup, Quiztik rebuilds the active runtime bank from `assets/questions/*.json`.

Question format per item:

```json
{
  "id": "optional-source-id",
  "prompt": "Question text",
  "options": ["A", "B", "C", "D"],
  "correct_index": 1,
  "points": 100,
  "image_url": null
}
```

Rules:

- `options` must have exactly 4 entries
- `correct_index` must be 0 to 3
- `points` must be > 0
- source `id` values are ignored during merge/import; server assigns fresh unique IDs

## If Browser Does Not Open Automatically

Open admin page manually on host machine:

- `http://127.0.0.1:8080/admin`

Then use the QR code at the top of admin page to let players join.

## Troubleshooting

### Players cannot connect

- Confirm host and players are on same network.
- Confirm firewall permission for the server executable.
- Confirm QR URL uses host LAN IP (not localhost).
- Try refreshing player page.

### Admin page says missing files

- Make sure you are running the executable from the extracted server zip contents.
- Keep `web/` and `assets/` folders next to the executable.

### Port 8080 already in use

Run with a different port:

- `QUIZTIK_PORT=9090 ./quiztik-server` (macOS/Linux)
- `set QUIZTIK_PORT=9090 && quiztik-server.exe` (Windows cmd)

Then open `http://127.0.0.1:9090/admin`.

## For Developers

Build and package locally:

```bash
scripts/build_release.sh "$(cat VERSION)" local
scripts/verify_artifacts.sh "$(cat VERSION)" local
```

Current release model is server-only artifacts.
