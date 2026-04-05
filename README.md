# Quizter

Quizter is a browser-based multiplayer quiz game server.

One person runs the server, opens the Quizter homepage, creates a room, and lets players join from their phone or computer browser by room code or QR code.

![Quizter Logo](assets/images/Quizter_logo.png)

**Software Version:** `v4.0.0`

Copyright 2026 Numerius Engineering LLC.  
Distributed under the terms of the MIT License  
Contact numerius.engineering@gmail.com for support

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
- hosted home page
- legacy admin web page
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
5. Quizter will try to launch in a visible terminal window so the server process is easy to see and stop.
6. Your browser should open automatically to the Quizter homepage.
7. On the homepage:
   - click **Create Room**
   - enter a room title
   - load and select question packs from the server
   - choose rounds and settings
   - click **Launch Room**
   - ask players to scan the QR code or enter the 4-character room code
   - click **Start Game** when players are ready
8. Players open on phones and join with room code plus display name.
9. When you are finished, use **End Game** to stop the current game or **Close Room** to fully release the room.

## Local Hosting Notes

If you are running Quizter from your own computer for a local gathering:

- Host computer and player devices must be on the same Wi-Fi/LAN.
- If prompted by firewall, **allow** the server on private/local networks.
- If browser does not auto-open, manually visit:
  - `http://127.0.0.1:8080/` (on host computer)

The legacy admin console still exists at:

- `http://127.0.0.1:8080/admin`

Use it only if you specifically want the older admin-login workflow.

## First-Time Hosted Walkthrough

After opening the homepage:

1. Enter a room title.
2. Click **Create Room**.
3. Use **Resume Room** later from the same browser if you reload or come back.
4. Load the available server-side question packs.
5. Choose at least one pack and set the number of rounds.
6. Configure game settings:
   - response speed bonus
   - hide scores until end of game
   - powerups
   - response time
   - automatic issue
   - automatic issue delay
7. Click **Launch Room** to make the player QR code and join link live.
8. Let players join by QR code or room code.
9. Click **Start Game** when the room is ready to begin.
10. Monitor connected players, leaderboard, and blocked names from the homepage.
11. Use **End Game** to stop the current game but keep the room open, or **Close Room** to invalidate the room and remove players.

Game settings:

- `Enable response speed bonus to player's score`
  - Default: on
  - Faster correct answers gain bonus points on top of base question points.
- `Hide player scores until end of game`
  - Default: off
  - Player leaderboards show hidden scores until the game ends, while the admin can still monitor progress.
- `Enable power ups`
  - Default: on
  - Disables all player power-up use when turned off.
- `Seconds allowed for responses`
  - Default: 15
  - Sets the normal answer timer for each question.
- `Enable automatic timed question issue`
  - Default: on
  - Automatically starts the next question after each round result.
- `Seconds between automatic question issue`
  - Default: 15
  - Delay between one round result and the next question when automatic issue is enabled.

## Player Experience

Players open the player page from QR/URL and can:

- join by room code + name
- answer timed questions
- keep the live leaderboard at the top of the page
- see each question's category while answering
- use one-time power-ups
- see a full instruction page with detailed power-up explanations
- see a large centered power-up graphic flash when a power-up is activated
- get red top alerts when other players trigger power-ups that affect them
- see score updates and round progress
- see the correct answer beneath the incorrect result graphic

No install needed on player devices; modern mobile browsers are enough.

## Question Banks

Question bank JSON files live under:

- `assets/questions/`

Question-bank behavior:

- File-bank list is read from `assets/questions/*.json`.
- The hosted room flow loads available pack files from `assets/questions/*.json`.
- Each room chooses its own subset of pack files from the server library.
- Effective playable pool = selected file-bank questions for that room.

Question pack format:

```json
{
  "category": "History",
  "questions": [
    {
      "prompt": "Question text",
      "options": ["A", "B", "C", "D"],
      "correct_index": 1,
      "points": 100,
      "image_url": null
    }
  ]
}
```

Notes:

- The root `category` field is optional.
- If the root `category` is missing, Quizter assigns the pack to `Generic`.
- The root `category` applies to every question in that pack.

Easy question generation with an LLM:

- You can ask ChatGPT or another LLM to generate a pack for you.
- Useful prompt template:

```text
Please generate { } questions about { } using the JSON format provided in this example:
{
  "category": "History",
  "questions": [
    {
      "prompt": "Which document first established the principle that government derives its authority from the consent of the governed?",
      "options": [
        "The Articles of Confederation",
        "The Declaration of Independence",
        "The Bill of Rights",
        "The Federalist Papers"
      ],
      "correct_index": 1,
      "points": 100,
      "image_url": null
    }
  ]
}
```

- Replace the first `{ }` with the number of questions.
- Replace the second `{ }` with the topic.
- After generation, save the JSON as a `.json` file under `assets/questions/`, restart the server if needed, and enable it from the category filter.

Rules:

- `options` must have exactly 4 entries
- `correct_index` must be 0 to 3
- `points` must be > 0
- `category` can be any label you want; the server will build the admin filter from the categories it finds

## If Browser Does Not Open Automatically

Open the hosted homepage manually on the host machine:

- `http://127.0.0.1:8080/`

Then create a room and use the generated QR code to let players join.

If Quizter could not relaunch itself into a separate terminal window on your platform, keep the original terminal window open while you host the game.

## Gameplay Notes

- End-of-round result graphics flash full-screen for 2 seconds.
- Incorrect or missed rounds show the correct answer beneath the result graphic during the result flash.
- Incorrect result flashes remain on screen longer than correct flashes for easier reading.
- `Speed Searcher` now provides a 60-second exclusive answer window.
- Affected-player power-up alerts appear as red push banners at top of player screen.

## Troubleshooting

### Players cannot connect during local hosting

- Confirm host and players are on same network.
- Confirm firewall permission for the server executable.
- Confirm QR URL uses host LAN IP (not localhost).
- Try refreshing player page.

### Server keeps running after browser closes

- Closing only the browser may leave the Quizter server running.
- Stop the running server process when you are done hosting.
- If needed, close the terminal window that launched Quizter or stop the `quizter-server` process manually.

### Admin page says missing files

- Make sure you are running the executable from the extracted server zip contents.
- Keep `web/` and `assets/` folders next to the executable.

### Port 8080 already in use

Run with a different port:

- `QUIZTER_PORT=9090 ./quizter-server` (macOS/Linux)
- `set QUIZTER_PORT=9090 && quizter-server.exe` (Windows cmd)

Then open `http://127.0.0.1:9090/admin`.
Or open `http://127.0.0.1:9090/` for the hosted homepage.

## For Developers

Build and package locally:

```bash
scripts/build_release.sh "$(cat VERSION)" local
scripts/verify_artifacts.sh "$(cat VERSION)" local
scripts/smoke_hosted_flow.sh
```

Current release model is server-only artifacts.
