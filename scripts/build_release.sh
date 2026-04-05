#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

VERSION="${1:-$(cat VERSION)}"
VERSION="${VERSION#v}"
TARGET="${2:-local}"
DIST_DIR="$ROOT_DIR/dist"
STAGE_DIR="$DIST_DIR/stage"
SKIP_SERVER_BUILD="${SKIP_SERVER_BUILD:-0}"
PYTHON_BIN="${PYTHON_BIN:-}"

if [[ -z "$PYTHON_BIN" ]]; then
  if command -v python3 >/dev/null 2>&1; then
    PYTHON_BIN="python3"
  elif command -v python >/dev/null 2>&1; then
    PYTHON_BIN="python"
  fi
fi

zip_dir() {
  local src_dir="$1"
  local out_zip="$2"
  if command -v zip >/dev/null 2>&1; then
    (cd "$src_dir" && zip -qr "$out_zip" .)
    return
  fi

  if [[ -n "$PYTHON_BIN" ]]; then
    "$PYTHON_BIN" - "$src_dir" "$out_zip" <<'PY'
import os
import sys
import zipfile

src = sys.argv[1]
out = sys.argv[2]
with zipfile.ZipFile(out, "w", compression=zipfile.ZIP_DEFLATED) as zf:
    for root, _, files in os.walk(src):
        for name in files:
            full_path = os.path.join(root, name)
            arcname = os.path.relpath(full_path, src)
            zf.write(full_path, arcname)
PY
    return
  fi

  echo "No zip tool found (zip/python3/python). Cannot package artifact."
  exit 1
}

rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR" "$STAGE_DIR/server"

if [[ "$TARGET" == "local" ]]; then
  if [[ "$SKIP_SERVER_BUILD" != "1" ]]; then
    (cd server && cargo build --release)
  fi
  if [[ -f server/target/release/quizter-server ]]; then
    cp server/target/release/quizter-server "$STAGE_DIR/server/quizter-server"
  else
    echo "Missing local server binary" > "$STAGE_DIR/server/README.txt"
  fi
else
  if [[ "$SKIP_SERVER_BUILD" != "1" ]]; then
    (cd server && cargo build --release --target "$TARGET")
  fi

  BIN_PATH="server/target/$TARGET/release/quizter-server"
  BIN_PATH_WIN="server/target/$TARGET/release/quizter-server.exe"
  if [[ -f "$BIN_PATH" ]]; then
    cp "$BIN_PATH" "$STAGE_DIR/server/quizter-server"
  elif [[ -f "$BIN_PATH_WIN" ]]; then
    cp "$BIN_PATH_WIN" "$STAGE_DIR/server/quizter-server.exe"
  else
    cat > "$STAGE_DIR/server/README.txt" <<TXT
No server binary found for target '$TARGET'.
Expected one of:
- $BIN_PATH
- $BIN_PATH_WIN
TXT
  fi
fi

mkdir -p "$STAGE_DIR/server/web/player" "$STAGE_DIR/server/web/home"
cp web/player/player.html "$STAGE_DIR/server/web/player/player.html"
cp web/home/home.html "$STAGE_DIR/server/web/home/home.html"
mkdir -p "$STAGE_DIR/server/assets"
cp -R assets/. "$STAGE_DIR/server/assets/" 2>/dev/null || true

zip_dir "$STAGE_DIR/server" "$DIST_DIR/quizter-server-${TARGET}-v${VERSION}.zip"

rm -rf "$STAGE_DIR"
ls -1 "$DIST_DIR"
