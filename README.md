# Quizter

Quizter is a local-network (LAN) multiplayer quiz game.

One person (the admin/host) runs the server on their computer. Everyone else joins from their phone or computer browser by scanning a QR code or opening a URL.

## New to GitHub? How to Download Quizter

If you have never used GitHub before, follow these exact steps:

1. Go to the Quizter repository page in your browser.
2. On the right side, find the **Releases** section and click the latest release version.
3. Scroll to **Assets** and click to expand it.
4. Download the zip that matches your computer:
   - Windows: `quizter-server-x86_64-pc-windows-msvc-vX.Y.Z.zip`
   - macOS (Apple Silicon): `quizter-server-aarch64-apple-darwin-vX.Y.Z.zip`
   - Linux: `quizter-server-x86_64-unknown-linux-gnu-vX.Y.Z.zip`
5. Wait for download to finish, then unzip/extract the file.
6. Open the extracted folder and run the server executable inside.

## What You Download

You only need **one zip file**:

- `quizter-server-<your-platform>-vX.Y.Z.zip`

This server zip already includes:

- the server executable
- admin web page
- player web page
- images and question banks

No separate player or admin app download is needed.

## Quick Start (Non-Technical)

1. Download the latest server zip from GitHub Releases.
2. Extract/unzip it to a folder (for example: Desktop `Quizter`).
3. Open that folder.
4. Run the server executable:
   - Windows: double-click `quizter-server.exe`
   - macOS/Linux: double-click `quizter-server` (or run from terminal)
5. Your browser should open automatically to the admin page.
6. In admin page:
   - create/update room
   - login as admin
   - select question bank files (if desired)
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
5. Optional: choose question banks in **Question Bank Files**.
   - Default is all bank files off.
   - Use checkboxes, **Add All Banks**, or **Clear All Banks**.
   - Changes during a game apply from the next round onward.
6. Add manual questions (guided form) or import JSON packs (manual pool).
7. Click **Start Game**.
8. Watch player join activity and leaderboard live.

## Player Experience

Players open the hosted player page from QR/URL and can:

- join by room code + name
- answer timed questions
- use one-time power-ups
- see a full instruction page with detailed power-up explanations
- get red top alerts when other players trigger power-ups that affect them
- see score updates and round progress

No install needed on player devices; modern mobile browsers are enough.

## Question Banks

Question bank JSON files live under:

- `assets/questions/`

File-bank behavior:

- File-bank list is read from `assets/questions/*.json`.
- Bank file selection is persisted on the server.
- Default first-run bank selection is all-off.
- Effective playable pool = selected file-bank questions + manual/imported questions.

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

## Gameplay Notes

- End-of-round result graphics flash full-screen for 1 second.
- `Speed Searcher` now provides a 60-second exclusive answer window.
- Affected-player power-up alerts appear as red push banners at top of player screen.

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

- `QUIZTER_PORT=9090 ./quizter-server` (macOS/Linux)
- `set QUIZTER_PORT=9090 && quizter-server.exe` (Windows cmd)

Then open `http://127.0.0.1:9090/admin`.

## For Developers

Build and package locally:

```bash
scripts/build_release.sh "$(cat VERSION)" local
scripts/verify_artifacts.sh "$(cat VERSION)" local
```

Current release model is server-only artifacts.
