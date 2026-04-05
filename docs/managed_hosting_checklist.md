# Managed Hosting Checklist

Use this checklist when moving Quizter from local development to a managed
hosting provider.

## Goal

Run one hosted Quizter server on a public HTTPS domain so:

- the host uses `/`
- players join through `/player`
- QR codes point at the public domain
- owner-token room control still works in the browser

## Minimum Runtime Expectations

Quizter needs:

- one long-running server process
- HTTP and WebSocket support
- static file serving for `assets/`
- the repo runtime layout with:
  - `server/`
  - `web/`
  - `assets/`

## Required Environment Variables

- `QUIZTER_PUBLIC_BASE_URL`
  - example: `https://quizter.example.com`
  - required for correct player join links and QR codes in hosted mode
- `QUIZTER_OPEN_BROWSER=0`
  - prevents local browser auto-open behavior on the server
- `QUIZTER_SPAWN_TERMINAL=0`
  - prevents local terminal relaunch behavior on the server

Optional:

- `QUIZTER_HOST=0.0.0.0`
- `QUIZTER_PORT`
  - only if your platform requires a specific bind port

## Pre-Deploy Checks

- Run `cargo test`
- Run `scripts/smoke_hosted_flow.sh`
- Run local manual verification with `QUIZTER_PUBLIC_BASE_URL` pointed at the
  expected public origin if you want to confirm QR/link generation before
  deploying.
- Confirm hosted homepage flow works locally:
  - create room
  - choose packs
  - launch room
  - join from player page
  - start game
  - end game
  - close room

## First Hosted Smoke Test

After deployment:

1. Open the public homepage.
2. Create a room.
3. Confirm the room starts in setup mode.
4. Confirm no QR or live player link is exposed before launch.
5. Select at least one question pack.
6. Launch the room.
7. Confirm QR and player link now use the public domain.
8. Join from a separate browser or phone.
9. Start a game.
10. Confirm late joiners wait until the next question.
11. End the game.
12. Close the room.

## Operational Notes

- Hosted room state is still in memory.
- A server restart will remove active rooms.
- Room expiration is currently inactivity-based.
- Legacy `/admin` still exists, but the intended hosted entry point is `/`.
- Managed hosting should expose the service over HTTPS before phone testing so
  browsers and QR joins follow the same origin you intend to ship.

## Recommended Next Checks

- Verify HTTPS on the final public domain.
- Verify WebSocket traffic works through the provider.
- Verify phone clients can join and stay connected for a full game.
- Verify the platform serves the app from a single long-running instance if you
  still depend on in-memory room state.
