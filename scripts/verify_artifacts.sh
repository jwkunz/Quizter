#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

VERSION="${1:-$(cat VERSION)}"
VERSION="${VERSION#v}"
TARGET="${2:-local}"
PYTHON_BIN="${PYTHON_BIN:-}"

if [[ -z "$PYTHON_BIN" ]]; then
  if command -v python3 >/dev/null 2>&1; then
    PYTHON_BIN="python3"
  elif command -v python >/dev/null 2>&1; then
    PYTHON_BIN="python"
  else
    echo "python3/python is required for artifact verification."
    exit 1
  fi
fi

SERVER_ZIP="dist/quizster-server-${TARGET}-v${VERSION}.zip"

for f in "$SERVER_ZIP"; do
  if [[ ! -f "$f" ]]; then
    echo "Missing artifact: $f"
    exit 1
  fi
done

"$PYTHON_BIN" - "$SERVER_ZIP" <<'PY'
import sys
import zipfile

zip_path = sys.argv[1]

with zipfile.ZipFile(zip_path, "r") as zf:
    names = zf.namelist()

def has_any(*candidates):
    return any(name in names for name in candidates)

def has_prefix(prefix):
    return any(name.startswith(prefix) for name in names)

if not has_any("quizster-server", "quizster-server.exe", "README.txt"):
    print("server executable/readme missing from server zip")
    sys.exit(1)

if "web/player/player.html" not in names:
    print("player html missing from server zip")
    sys.exit(1)

if "web/home/home.html" not in names:
    print("home html missing from server zip")
    sys.exit(1)

if not has_prefix("assets/questions/"):
    print("question banks missing from server zip")
    sys.exit(1)

if not has_prefix("assets/images/"):
    print("image assets missing from server zip")
    sys.exit(1)

if not has_prefix("assets/music/"):
    print("music assets missing from server zip")
    sys.exit(1)

if not has_prefix("assets/sfx/"):
    print("sfx assets missing from server zip")
    sys.exit(1)
PY

echo "Artifacts verified for v${VERSION} target=${TARGET}"
