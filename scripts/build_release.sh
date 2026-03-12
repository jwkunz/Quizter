#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

VERSION="${1:-$(cat VERSION)}"
VERSION="${VERSION#v}"
TARGET="${2:-local}"
DIST_DIR="$ROOT_DIR/dist"
STAGE_DIR="$DIST_DIR/stage"

rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR" "$STAGE_DIR/server" "$STAGE_DIR/player" "$STAGE_DIR/admin"

if [[ "$TARGET" == "local" ]]; then
  (cd server && cargo build --release)
  if [[ -f server/target/release/quiztik-server ]]; then
    cp server/target/release/quiztik-server "$STAGE_DIR/server/quiztik-server"
  fi
else
  BIN_PATH="server/target/$TARGET/release/quiztik-server"
  BIN_PATH_WIN="server/target/$TARGET/release/quiztik-server.exe"
  if [[ -f "$BIN_PATH" ]]; then
    cp "$BIN_PATH" "$STAGE_DIR/server/quiztik-server"
  elif [[ -f "$BIN_PATH_WIN" ]]; then
    cp "$BIN_PATH_WIN" "$STAGE_DIR/server/quiztik-server.exe"
  else
    cat > "$STAGE_DIR/server/README.txt" <<TXT
No server binary found for target '$TARGET'.
Expected one of:
- $BIN_PATH
- $BIN_PATH_WIN
TXT
  fi
fi

cp web/player/player.html "$STAGE_DIR/player/player.html"
cp web/admin/admin.html "$STAGE_DIR/admin/admin.html"

(cd "$STAGE_DIR/server" && zip -qr "$DIST_DIR/quiztik-server-${TARGET}-v${VERSION}.zip" .)
(cd "$STAGE_DIR/player" && zip -qr "$DIST_DIR/quiztik-player-v${VERSION}.zip" .)
(cd "$STAGE_DIR/admin" && zip -qr "$DIST_DIR/quiztik-admin-v${VERSION}.zip" .)

rm -rf "$STAGE_DIR"
ls -1 "$DIST_DIR"
