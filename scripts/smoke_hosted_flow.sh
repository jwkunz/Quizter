#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORT="${1:-18080}"
BASE_URL="http://127.0.0.1:${PORT}"
SERVER_LOG="$(mktemp)"
SERVER_PID=""

cleanup() {
  if [[ -n "${SERVER_PID}" ]] && kill -0 "${SERVER_PID}" 2>/dev/null; then
    kill "${SERVER_PID}" 2>/dev/null || true
    wait "${SERVER_PID}" 2>/dev/null || true
  fi
  rm -f "${SERVER_LOG}"
}
trap cleanup EXIT

json_get() {
  local field="$1"
  python3 -c 'import json,sys; print(json.load(sys.stdin).get(sys.argv[1], ""))' "${field}"
}

json_first_list_item() {
  local field="$1"
  python3 -c 'import json,sys; data=json.load(sys.stdin).get(sys.argv[1], []); print(data[0] if data else "")' "${field}"
}

json_has_player_name() {
  local player_name="$1"
  python3 -c 'import json,sys; payload=json.load(sys.stdin); target=sys.argv[1].lower(); print("yes" if any((player.get("name","").lower()==target) for player in payload.get("players", [])) else "no")' "${player_name}"
}

json_get_nested() {
  local path="$1"
  python3 -c 'import json,sys; data=json.load(sys.stdin); value=data
for part in sys.argv[1].split("."):
    if isinstance(value, dict):
        value=value.get(part)
    else:
        value=None
        break
if isinstance(value, bool):
    print("true" if value else "false")
elif value is None:
    print("")
else:
    print(value)' "${path}"
}

post_json() {
  local path="$1"
  local body="$2"
  local response_file
  response_file="$(mktemp)"
  local status
  status="$(curl -sS -o "${response_file}" -w "%{http_code}" -H "Content-Type: application/json" -d "${body}" "${BASE_URL}${path}")"
  local response
  response="$(cat "${response_file}")"
  rm -f "${response_file}"
  printf '%s\n%s\n' "${status}" "${response}"
}

get_json() {
  local path="$1"
  local response_file
  response_file="$(mktemp)"
  local status
  status="$(curl -sS -o "${response_file}" -w "%{http_code}" "${BASE_URL}${path}")"
  local response
  response="$(cat "${response_file}")"
  rm -f "${response_file}"
  printf '%s\n%s\n' "${status}" "${response}"
}

expect_status() {
  local actual="$1"
  local expected="$2"
  local label="$3"
  if [[ "${actual}" != "${expected}" ]]; then
    echo "FAIL: ${label} returned HTTP ${actual}, expected ${expected}"
    exit 1
  fi
}

echo "Starting Quizter server on ${BASE_URL}"
(
  cd "${ROOT_DIR}/server"
  QUIZTER_PORT="${PORT}" QUIZTER_OPEN_BROWSER=0 QUIZTER_SPAWN_TERMINAL=0 cargo run >"${SERVER_LOG}" 2>&1
) &
SERVER_PID="$!"

for _ in $(seq 1 60); do
  if curl -fsS "${BASE_URL}/health" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

if ! curl -fsS "${BASE_URL}/health" >/dev/null 2>&1; then
  echo "FAIL: server did not become healthy"
  echo "Server log:"
  cat "${SERVER_LOG}"
  exit 1
fi

echo "Creating hosted room"
create_result="$(post_json "/api/rooms/create" '{"room_title":"Smoke Test Room"}')"
create_status="$(printf '%s' "${create_result}" | sed -n '1p')"
create_body="$(printf '%s' "${create_result}" | sed -n '2p')"
expect_status "${create_status}" "200" "create room"

ROOM_CODE="$(printf '%s' "${create_body}" | json_get room_code)"
OWNER_TOKEN="$(printf '%s' "${create_body}" | json_get owner_token)"
if [[ -z "${ROOM_CODE}" || -z "${OWNER_TOKEN}" ]]; then
  echo "FAIL: create room did not return room ownership data"
  exit 1
fi

echo "Verifying hosted room title validation"
empty_room_result="$(post_json "/api/rooms/create" '{"room_title":"   "}')"
empty_room_status="$(printf '%s' "${empty_room_result}" | sed -n '1p')"
empty_room_body="$(printf '%s' "${empty_room_result}" | sed -n '2p')"
expect_status "${empty_room_status}" "400" "empty room title"
if [[ "$(printf '%s' "${empty_room_body}" | json_get error)" != "room_title_required" ]]; then
  echo "FAIL: empty room title did not return room_title_required"
  exit 1
fi

long_room_title="$(python3 -c 'print("X" * 81)')"
long_room_result="$(post_json "/api/rooms/create" "{\"room_title\":\"${long_room_title}\"}")"
long_room_status="$(printf '%s' "${long_room_result}" | sed -n '1p')"
long_room_body="$(printf '%s' "${long_room_result}" | sed -n '2p')"
expect_status "${long_room_status}" "400" "oversized room title"
if [[ "$(printf '%s' "${long_room_body}" | json_get error)" != "room_title_too_long" ]]; then
  echo "FAIL: oversized room title did not return room_title_too_long"
  exit 1
fi

echo "Resuming hosted room ${ROOM_CODE}"
resume_result="$(post_json "/api/rooms/resume" "{\"room_code\":\"${ROOM_CODE}\",\"owner_token\":\"${OWNER_TOKEN}\"}")"
resume_status="$(printf '%s' "${resume_result}" | sed -n '1p')"
resume_body="$(printf '%s' "${resume_result}" | sed -n '2p')"
expect_status "${resume_status}" "200" "resume room"

echo "Verifying joins are blocked before launch"
prelaunch_join_result="$(post_json "/api/join" "{\"room_code\":\"${ROOM_CODE}\",\"display_name\":\"TooEarly\"}")"
prelaunch_join_status="$(printf '%s' "${prelaunch_join_result}" | sed -n '1p')"
prelaunch_join_body="$(printf '%s' "${prelaunch_join_result}" | sed -n '2p')"
expect_status "${prelaunch_join_status}" "400" "prelaunch join"
if [[ "$(printf '%s' "${prelaunch_join_body}" | json_get error)" != "room_not_open" ]]; then
  echo "FAIL: prelaunch join did not return room_not_open"
  exit 1
fi

echo "Loading question banks"
banks_result="$(get_json "/api/rooms/question_banks?room_code=${ROOM_CODE}&owner_token=${OWNER_TOKEN}")"
banks_status="$(printf '%s' "${banks_result}" | sed -n '1p')"
banks_body="$(printf '%s' "${banks_result}" | sed -n '2p')"
expect_status "${banks_status}" "200" "load question banks"

FIRST_PACK="$(printf '%s' "${banks_body}" | json_first_list_item available_files)"
if [[ -z "${FIRST_PACK}" ]]; then
  echo "FAIL: no question pack files are available for the hosted smoke test"
  exit 1
fi

echo "Selecting question pack ${FIRST_PACK}"
selection_result="$(post_json "/api/rooms/question_banks/selection" "{\"room_code\":\"${ROOM_CODE}\",\"owner_token\":\"${OWNER_TOKEN}\",\"selected_files\":[\"${FIRST_PACK}\"]}")"
selection_status="$(printf '%s' "${selection_result}" | sed -n '1p')"
selection_body="$(printf '%s' "${selection_result}" | sed -n '2p')"
expect_status "${selection_status}" "200" "apply question bank selection"

QUESTIONS_IN_PLAY="$(printf '%s' "${selection_body}" | json_get effective_question_count)"
if [[ -z "${QUESTIONS_IN_PLAY}" || "${QUESTIONS_IN_PLAY}" == "0" ]]; then
  echo "FAIL: selected pack did not produce playable questions"
  exit 1
fi

echo "Launching hosted room"
launch_result="$(post_json "/api/rooms/launch" "{\"room_code\":\"${ROOM_CODE}\",\"owner_token\":\"${OWNER_TOKEN}\"}")"
launch_status="$(printf '%s' "${launch_result}" | sed -n '1p')"
expect_status "${launch_status}" "200" "launch room"

echo "Joining player SmokePlayer"
join_result="$(post_json "/api/join" "{\"room_code\":\"${ROOM_CODE}\",\"display_name\":\"SmokePlayer\"}")"
join_status="$(printf '%s' "${join_result}" | sed -n '1p')"
join_body="$(printf '%s' "${join_result}" | sed -n '2p')"
expect_status "${join_status}" "200" "join player"
PLAYER_ID="$(printf '%s' "${join_body}" | json_get player_id)"
if [[ -z "${PLAYER_ID}" ]]; then
  echo "FAIL: player join did not return a player id"
  exit 1
fi

echo "Verifying lobby state hides leaderboard before the game starts"
lobby_state_result="$(get_json "/api/state/${PLAYER_ID}")"
lobby_state_status="$(printf '%s' "${lobby_state_result}" | sed -n '1p')"
lobby_state_body="$(printf '%s' "${lobby_state_result}" | sed -n '2p')"
expect_status "${lobby_state_status}" "200" "lobby state"
if [[ "$(printf '%s' "${lobby_state_body}" | json_get_nested show_leaderboard)" != "false" ]]; then
  echo "FAIL: lobby state should hide leaderboard between games"
  exit 1
fi

echo "Verifying player-name validation"
long_player_name="$(python3 -c 'print("Y" * 33)')"
long_name_join_result="$(post_json "/api/join" "{\"room_code\":\"${ROOM_CODE}\",\"display_name\":\"${long_player_name}\"}")"
long_name_join_status="$(printf '%s' "${long_name_join_result}" | sed -n '1p')"
long_name_join_body="$(printf '%s' "${long_name_join_result}" | sed -n '2p')"
expect_status "${long_name_join_status}" "400" "oversized player name"
if [[ "$(printf '%s' "${long_name_join_body}" | json_get error)" != "display_name_too_long" ]]; then
  echo "FAIL: oversized player name did not return display_name_too_long"
  exit 1
fi

echo "Checking room status for joined player"
status_result="$(get_json "/api/rooms/status?room_code=${ROOM_CODE}&owner_token=${OWNER_TOKEN}")"
status_code="$(printf '%s' "${status_result}" | sed -n '1p')"
status_body="$(printf '%s' "${status_result}" | sed -n '2p')"
expect_status "${status_code}" "200" "get room status"
PLAYER_PRESENT="$(printf '%s' "${status_body}" | json_has_player_name SmokePlayer)"
if [[ "${PLAYER_PRESENT}" != "yes" ]]; then
  echo "FAIL: joined player is missing from hosted room status"
  exit 1
fi

echo "Kicking player SmokePlayer"
kick_result="$(post_json "/api/rooms/kick" "{\"room_code\":\"${ROOM_CODE}\",\"owner_token\":\"${OWNER_TOKEN}\",\"player_id\":\"${PLAYER_ID}\"}")"
kick_status="$(printf '%s' "${kick_result}" | sed -n '1p')"
expect_status "${kick_status}" "200" "kick player"

echo "Verifying blocked join is rejected"
blocked_join_result="$(post_json "/api/join" "{\"room_code\":\"${ROOM_CODE}\",\"display_name\":\"SmokePlayer\"}")"
blocked_join_status="$(printf '%s' "${blocked_join_result}" | sed -n '1p')"
blocked_join_body="$(printf '%s' "${blocked_join_result}" | sed -n '2p')"
expect_status "${blocked_join_status}" "400" "blocked player join"
if [[ "$(printf '%s' "${blocked_join_body}" | json_get error)" != "player_blocked" ]]; then
  echo "FAIL: blocked join did not return player_blocked"
  exit 1
fi

echo "Unbanning SmokePlayer"
unban_result="$(post_json "/api/rooms/unban" "{\"room_code\":\"${ROOM_CODE}\",\"owner_token\":\"${OWNER_TOKEN}\",\"player_name\":\"SmokePlayer\"}")"
unban_status="$(printf '%s' "${unban_result}" | sed -n '1p')"
expect_status "${unban_status}" "200" "unban player"

echo "Rejoining player SmokePlayer"
rejoin_result="$(post_json "/api/join" "{\"room_code\":\"${ROOM_CODE}\",\"display_name\":\"SmokePlayer\"}")"
rejoin_status="$(printf '%s' "${rejoin_result}" | sed -n '1p')"
expect_status "${rejoin_status}" "200" "rejoin player"

echo "Starting hosted game"
start_result="$(post_json "/api/rooms/start" "{\"room_code\":\"${ROOM_CODE}\",\"owner_token\":\"${OWNER_TOKEN}\",\"total_rounds\":1}")"
start_status="$(printf '%s' "${start_result}" | sed -n '1p')"
expect_status "${start_status}" "200" "start game"

echo "Joining a late player during the active round"
late_join_result="$(post_json "/api/join" "{\"room_code\":\"${ROOM_CODE}\",\"display_name\":\"LatePlayer\"}")"
late_join_status="$(printf '%s' "${late_join_result}" | sed -n '1p')"
late_join_body="$(printf '%s' "${late_join_result}" | sed -n '2p')"
expect_status "${late_join_status}" "200" "late join"
LATE_PLAYER_ID="$(printf '%s' "${late_join_body}" | json_get player_id)"
if [[ -z "${LATE_PLAYER_ID}" ]]; then
  echo "FAIL: late join did not return a player id"
  exit 1
fi

echo "Verifying late join waits until the next round"
late_state_result="$(get_json "/api/state/${LATE_PLAYER_ID}")"
late_state_status="$(printf '%s' "${late_state_result}" | sed -n '1p')"
late_state_body="$(printf '%s' "${late_state_result}" | sed -n '2p')"
expect_status "${late_state_status}" "200" "late join state"
if [[ "$(printf '%s' "${late_state_body}" | json_get_nested waiting_for_next_round)" != "true" ]]; then
  echo "FAIL: late join state should mark waiting_for_next_round"
  exit 1
fi
if [[ -n "$(printf '%s' "${late_state_body}" | json_get_nested current_question.id)" ]]; then
  echo "FAIL: late join state should not expose the current question"
  exit 1
fi

echo "Ending hosted game"
end_result="$(post_json "/api/rooms/end_game" "{\"room_code\":\"${ROOM_CODE}\",\"owner_token\":\"${OWNER_TOKEN}\"}")"
end_status="$(printf '%s' "${end_result}" | sed -n '1p')"
expect_status "${end_status}" "200" "end game"

echo "Closing hosted room"
close_result="$(post_json "/api/rooms/close" "{\"room_code\":\"${ROOM_CODE}\",\"owner_token\":\"${OWNER_TOKEN}\"}")"
close_status="$(printf '%s' "${close_result}" | sed -n '1p')"
expect_status "${close_status}" "200" "close room"

echo "Hosted smoke flow passed on ${BASE_URL}"
