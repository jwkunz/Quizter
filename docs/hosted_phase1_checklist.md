# Quizster Hosted Phase 1 Checklist

Status:
- Completed.
- The room-registry and hosted owner-token foundation from this checklist has
  already been implemented and carried forward into the current hosted-only
  app.

## Phase Goal

Phase 1 is the backend architecture refactor that replaces the single global
game model with a room registry while preserving the existing gameplay rules as
much as possible.

This phase does not yet try to finish the hosted UI. The goal is to create the
server shape that later phases can build on.

## Success Criteria

- The server can hold multiple isolated rooms in memory.
- Each room owns its own game state.
- Current gameplay logic is routed through room-scoped state.
- WebSocket broadcasts and state snapshots are room-scoped.
- The code is split enough that later phases do not require a full rewrite of
  `server/src/main.rs`.

## Constraints

- Keep the implementation single-process and in-memory.
- Do not add a database.
- Do not add user accounts.
- Keep current scoring and power-up behavior unless a room-scoping change forces
  a local adjustment.
- Prefer incremental migration over a large rewrite.

## Recommended Server Refactor Strategy

## Step 1: Introduce room-scoped domain types

Create explicit types for:

- `RoomState`
- `GameSessionState`
- `RoomSettings`
- `RoomStatus`
- `RoomRegistry`

Goal:
pull room lifecycle apart from game lifecycle before changing public routes.

Notes:
- `GameSessionState` should absorb the fields that only make sense for one game.
- `RoomState` should own settings, player roster, blocked names, pack snapshot,
  owner token, and current game.

## Step 2: Extract pure gameplay helpers from global assumptions

Identify logic in `server/src/main.rs` that can stay mostly unchanged if given a
room-local game object:

- round issue
- answer submission
- scoring
- power-up activation
- round completion
- leaderboard generation
- pack pool rebuild

Goal:
make current game rules reusable inside `RoomState.current_game`.

## Step 3: Replace single global room storage

Replace:

- one `AppState.game`

with:

- `AppState.rooms`
- `AppState.owner_index`

Goal:
store multiple rooms concurrently and prepare for owner-token lookup.

Open design choice:
- keep one global client registry keyed by client id with room metadata
- or nest client registries per room

Recommendation:
- use a global client registry with explicit room code and role metadata to
  minimize the first migration cost

## Step 4: Make every state read and mutation room-aware

Update handlers and internal helpers so each action resolves a room first, then
operates only on that room.

Areas to convert:

- join flow
- owner/admin flow
- state snapshot generation
- WebSocket message handling
- broadcast helpers
- timer tasks

Goal:
remove all hidden assumptions that there is only one active room.

## Step 5: Room-scope WebSocket broadcast and snapshot flow

Current broadcast helpers push global game events. Replace them with room-aware
broadcast functions:

- `broadcast_room_state(room_code)`
- `broadcast_room_json(room_code, payload)`

Goal:
guarantee event isolation between simultaneous rooms.

## Step 6: Add room activity tracking

Every owner/player action and active connection event should update
`last_activity_at` for the room.

Goal:
prepare for inactivity expiration in Phase 2 without needing another major
refactor.

## Step 7: Add room-pack snapshot plumbing

Refactor question-bank loading so that room creation can capture a pack snapshot
for the room instead of relying on app-global selected state.

Goal:
move toward the hosted model without needing the full wizard yet.

Initial approach:
- keep existing file scanning helper patterns
- shift from app-global selection to room-local selection

## File-Level Checklist

## `/home/jwkunz/repos/Quizter/server/src/main.rs`

Primary tasks:

- define new room-oriented types
- isolate current game-only logic from app-global assumptions
- replace global router handlers that assume one room
- make WebSocket handling room-aware
- make timer tasks room-aware
- remove or deprecate server-global selected-bank behavior

## Suggested module split after initial extraction

Even if the first pass remains in one file, target these conceptual sections:

- room models
- gameplay models
- HTTP handlers
- WebSocket handlers
- question-pack loading
- room cleanup/timers
- runtime/environment helpers

If Phase 1 grows too large inside one file, split into:

- `server/src/room.rs`
- `server/src/game.rs`
- `server/src/api.rs`
- `server/src/ws.rs`
- `server/src/packs.rs`

That split is optional, but should be considered if `main.rs` starts resisting
incremental changes.

## Migration Order

Recommended order of actual coding work:

1. Add new room-oriented types without deleting old ones.
2. Move helper logic to work on room-local game/session values.
3. Replace app state storage from single game to room registry.
4. Update broadcast/state snapshot helpers.
5. Update join/admin/player handlers to resolve rooms explicitly.
6. Remove now-dead single-room code paths.

This order reduces the number of simultaneous moving pieces.

## Temporary Compatibility Plan

To reduce risk during Phase 1:

- it is acceptable to keep a temporary compatibility path that still creates one
  default room while the room registry work lands
- it is acceptable for the old admin/player HTML to remain temporarily awkward
  during backend migration

The important outcome is backend room isolation, not polished hosted UX yet.

## Risks To Watch

- timer tasks accidentally mutating the wrong room after room lookup changes
- WebSocket events leaking between rooms
- player reconnect logic still assuming one global player namespace
- question selection or shuffled queues accidentally shared across rooms
- leftover server-global fields creating contradictory sources of truth

## Minimum Validation For Phase 1

Before Phase 1 is considered complete, verify:

- two rooms can exist at once
- actions in one room do not affect the other
- state snapshots are room-specific
- players in room A do not receive events from room B
- question-pool updates stay local to the room

## Phase 1 Deliverable

When Phase 1 code is complete, the repo should be ready for Phase 2:

- owner-token room creation/resume
- room expiration
- room closure
- hosted homepage flow
