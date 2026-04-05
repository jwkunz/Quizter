use axum::{
    extract::{ws::Message, Path as AxPath, Query, State, WebSocketUpgrade},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    env,
    fs,
    net::{SocketAddr, UdpSocket},
    path::{Path as FsPath, PathBuf},
    process::Command,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, Mutex};
use tower_http::services::ServeDir;
use uuid::Uuid;

const ROOM_CODE_LENGTH: usize = 4;
const ROOM_CODE_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const ROOM_TITLE_MAX_CHARS: usize = 80;
const PLAYER_NAME_MAX_CHARS: usize = 32;
const ROOM_INACTIVITY_TIMEOUT_SECS: u64 = 30 * 60;
const ROOM_CLEANUP_INTERVAL_SECS: u64 = 60;

#[derive(Clone)]
struct AppState {
    rooms: Arc<Mutex<HashMap<String, RoomState>>>,
    owner_index: Arc<Mutex<HashMap<String, String>>>,
    clients: Arc<Mutex<HashMap<String, ClientConnection>>>,
    data_dir: Arc<PathBuf>,
    public_base_url: Arc<String>,
    player_join_url: Arc<String>,
    host_ip: Arc<String>,
    port: u16,
    runtime_root: Arc<PathBuf>,
}

#[derive(Debug)]
struct RoomState {
    room_title: String,
    owner_token: String,
    launched: bool,
    clear_blocked_names_on_new_game: bool,
    blocked_names: HashSet<String>,
    last_activity_at: Instant,
    game: GameState,
}

#[derive(Debug)]
struct ClientConnection {
    room_code: String,
    tx: mpsc::UnboundedSender<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Role {
    Admin,
    Player,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum GameStatus {
    Lobby,
    InRound,
    RoundResult,
    Ended,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
enum PowerUp {
    MixMaster,
    SpeedSearcher,
    DoubleDowner,
    CloneCommander,
    SuperSpliter,
    GreatGambler,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Question {
    #[serde(default = "new_question_id", skip_serializing)]
    id: String,
    #[serde(default = "default_question_category")]
    category: String,
    prompt: String,
    options: Vec<String>,
    correct_index: usize,
    points: u32,
    image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QuestionPack {
    #[serde(default)]
    category: Option<String>,
    questions: Vec<Question>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum QuestionPackFile {
    Pack(QuestionPack),
    Questions(Vec<Question>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoryEntry {
    finished_at: String,
    rounds_played: usize,
    leaderboard: Vec<LeaderboardEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LeaderboardEntry {
    player_id: String,
    name: String,
    score: f64,
    last_delta: f64,
}

#[derive(Debug, Clone)]
struct PlayerState {
    id: String,
    name: String,
    score: f64,
    last_score_delta: f64,
    connected: bool,
    eligible_from_round: usize,
    used_powerups: HashSet<PowerUp>,
    pending_powerup: Option<PowerUp>,
    tutorial_seen: bool,
}

#[derive(Debug, Clone)]
struct AnswerState {
    choice_index: usize,
    submitted_at: Instant,
}

#[derive(Debug, Clone)]
struct RoundState {
    round_number: usize,
    question: Question,
    started_at: Instant,
    deadline: Instant,
    answer_window_secs: u64,
    answers: HashMap<String, AnswerState>,
    speed_searcher_owner: Option<String>,
    great_gambler_factor: Option<f64>,
    double_downers: HashSet<String>,
    clone_commanders: HashSet<String>,
    super_spliter_targets: HashMap<String, (usize, usize)>,
    mix_master_owner: Option<String>,
    option_order: Vec<usize>,
}

#[derive(Debug, Clone)]
struct GameState {
    room_code: String,
    status: GameStatus,
    speed_bonus_enabled: bool,
    hide_scores_until_end: bool,
    powerups_enabled: bool,
    response_seconds: u64,
    auto_issue_enabled: bool,
    auto_issue_delay_secs: u64,
    players: HashMap<String, PlayerState>,
    manual_questions: Vec<Question>,
    file_question_banks: HashMap<String, Vec<Question>>,
    selected_bank_files: HashSet<String>,
    questions: Vec<Question>,
    shuffled_question_ids: Vec<String>,
    total_rounds: usize,
    current_round: Option<RoundState>,
    completed_rounds: usize,
}

impl GameState {
    fn new(
        manual_questions: Vec<Question>,
        file_question_banks: HashMap<String, Vec<Question>>,
        selected_bank_files: HashSet<String>,
    ) -> Self {
        let mut game = Self {
            room_code: String::new(),
            status: GameStatus::Lobby,
            speed_bonus_enabled: true,
            hide_scores_until_end: false,
            powerups_enabled: true,
            response_seconds: 15,
            auto_issue_enabled: true,
            auto_issue_delay_secs: 15,
            players: HashMap::new(),
            manual_questions,
            file_question_banks,
            selected_bank_files,
            questions: Vec::new(),
            shuffled_question_ids: Vec::new(),
            total_rounds: 10,
            current_round: None,
            completed_rounds: 0,
        };
        game.rebuild_effective_question_pool();
        game
    }

    fn rebuild_effective_question_pool(&mut self) {
        let mut effective = self.manual_questions.clone();
        let mut selected_files: Vec<String> = self.selected_bank_files.iter().cloned().collect();
        selected_files.sort();
        for file in selected_files {
            if let Some(questions) = self.file_question_banks.get(&file) {
                effective.extend(questions.clone());
            }
        }
        self.questions = effective;
        if self.questions.is_empty() {
            self.total_rounds = 0;
        } else if self.total_rounds == 0 {
            self.total_rounds = 1;
        } else {
            self.total_rounds = self.total_rounds.min(self.questions.len());
        }
    }

    fn available_bank_files(&self) -> Vec<String> {
        let mut files: Vec<String> = self.file_question_banks.keys().cloned().collect();
        files.sort();
        files
    }

    fn question_bank_tree(&self) -> Vec<Value> {
        let mut grouped: HashMap<String, Vec<Value>> = HashMap::new();
        for file_name in self.available_bank_files() {
            if let Some(questions) = self.file_question_banks.get(&file_name) {
                let mut categories: Vec<String> = questions
                    .iter()
                    .map(|q| q.category.clone())
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect();
                categories.sort();
                let category_name = if categories.len() == 1 {
                    categories[0].clone()
                } else {
                    "Mixed".to_string()
                };
                grouped.entry(category_name).or_default().push(json!({
                    "file": file_name,
                    "question_count": questions.len(),
                    "selected": self.selected_bank_files.contains(&file_name),
                }));
            }
        }

        let mut category_names: Vec<String> = grouped.keys().cloned().collect();
        category_names.sort();
        category_names
            .into_iter()
            .map(|name| {
                let mut files = grouped.remove(&name).unwrap_or_default();
                files.sort_by(|a, b| {
                    a["file"]
                        .as_str()
                        .unwrap_or_default()
                        .cmp(b["file"].as_str().unwrap_or_default())
                });
                let question_count: usize = files
                    .iter()
                    .map(|file| file["question_count"].as_u64().unwrap_or(0) as usize)
                    .sum();
                json!({
                    "name": name,
                    "question_count": question_count,
                    "files": files,
                })
            })
            .collect()
    }

    fn total_available_questions(&self) -> usize {
        let file_count: usize = self.file_question_banks.values().map(|v| v.len()).sum();
        self.manual_questions.len() + file_count
    }

    fn reflow_future_rounds_after_pool_change(&mut self) {
        let mut preserved_ids: Vec<String> = self
            .shuffled_question_ids
            .iter()
            .take(self.completed_rounds)
            .cloned()
            .collect();

        if let Some(current) = &self.current_round {
            preserved_ids.push(current.question.id.clone());
        }

        let preserved_set: HashSet<String> = preserved_ids.iter().cloned().collect();
        let mut remaining_ids: Vec<String> = self
            .questions
            .iter()
            .map(|q| q.id.clone())
            .filter(|id| !preserved_set.contains(id))
            .collect();
        remaining_ids.shuffle(&mut rand::thread_rng());
        preserved_ids.extend(remaining_ids);
        self.shuffled_question_ids = preserved_ids;

        if self.questions.is_empty() {
            self.total_rounds = 0;
        } else {
            let min_rounds = self.completed_rounds + usize::from(self.current_round.is_some());
            self.total_rounds = self.total_rounds.max(min_rounds).min(self.questions.len());
        }
    }

    fn leaderboard(&self) -> Vec<LeaderboardEntry> {
        let mut entries: Vec<LeaderboardEntry> = self
            .players
            .values()
            .map(|p| LeaderboardEntry {
                player_id: p.id.clone(),
                name: p.name.clone(),
                score: (p.score * 100.0).round() / 100.0,
                last_delta: (p.last_score_delta * 100.0).round() / 100.0,
            })
            .collect();
        entries.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.name.cmp(&b.name))
        });
        entries
    }
}

fn reset_players_for_new_game(game: &mut GameState) {
    for player in game.players.values_mut() {
        player.score = 0.0;
        player.last_score_delta = 0.0;
        player.used_powerups.clear();
        player.pending_powerup = None;
    }
}

fn prepare_game_for_start(game: &mut GameState, total_rounds: usize) -> Result<(), &'static str> {
    if game.questions.is_empty() {
        return Err("no_questions");
    }

    game.total_rounds = total_rounds.max(1).min(game.questions.len());
    game.completed_rounds = 0;
    game.status = GameStatus::Lobby;
    game.current_round = None;
    reset_players_for_new_game(game);
    game.shuffled_question_ids = game.questions.iter().map(|q| q.id.clone()).collect();
    game.shuffled_question_ids.shuffle(&mut rand::thread_rng());
    Ok(())
}

fn mark_game_ended(game: &mut GameState) -> bool {
    if game.status == GameStatus::Ended {
        return false;
    }
    game.status = GameStatus::Ended;
    game.current_round = None;
    true
}

fn generate_room_code(existing_codes: &HashSet<String>) -> Option<String> {
    let mut rng = rand::thread_rng();
    for _ in 0..512 {
        let code: String = (0..ROOM_CODE_LENGTH)
            .map(|_| {
                let idx = rng.gen_range(0..ROOM_CODE_ALPHABET.len());
                ROOM_CODE_ALPHABET[idx] as char
            })
            .collect();
        if !existing_codes.contains(&code) {
            return Some(code);
        }
    }
    None
}

fn new_owner_token() -> String {
    Uuid::new_v4().to_string()
}

fn normalize_room_title(raw: &str) -> Result<String, &'static str> {
    let title = raw.trim();
    if title.is_empty() {
        return Err("room_title_required");
    }
    if title.chars().count() > ROOM_TITLE_MAX_CHARS {
        return Err("room_title_too_long");
    }
    Ok(title.to_string())
}

fn normalize_player_name(raw: &str) -> Result<String, &'static str> {
    let name = raw.trim();
    if name.is_empty() {
        return Err("display_name_required");
    }
    if name.chars().count() > PLAYER_NAME_MAX_CHARS {
        return Err("display_name_too_long");
    }
    Ok(name.to_string())
}

fn eligible_from_round_for_new_player(game: &GameState) -> usize {
    game.current_round
        .as_ref()
        .map(|round| round.round_number + 1)
        .unwrap_or(game.completed_rounds + 1)
}

fn player_can_participate_in_current_round(player: Option<&PlayerState>, round: Option<&RoundState>) -> bool {
    match (player, round) {
        (_, None) => true,
        (Some(player), Some(round)) => player.eligible_from_round <= round.round_number,
        (None, Some(_)) => false,
    }
}

fn should_show_player_leaderboard(status: &GameStatus, completed_rounds: usize) -> bool {
    matches!(status, GameStatus::InRound | GameStatus::RoundResult)
        || (*status == GameStatus::Ended && completed_rounds > 0)
}

fn normalized_public_base_url(raw: Option<String>, port: u16, detected_lan_ip: &str) -> String {
    if let Some(value) = raw {
        let trimmed = value.trim().trim_end_matches('/');
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    format!("http://{}:{}", detected_lan_ip, port)
}

fn host_label_from_base_url(base_url: &str) -> String {
    let without_scheme = base_url
        .strip_prefix("https://")
        .or_else(|| base_url.strip_prefix("http://"))
        .unwrap_or(base_url);
    without_scheme
        .split('/')
        .next()
        .unwrap_or(without_scheme)
        .to_string()
}

fn room_from_template(
    template: &GameState,
    room_code: String,
    room_title: String,
    owner_token: String,
) -> RoomState {
    let mut game = template.clone();
    game.room_code = room_code;
    game.status = GameStatus::Lobby;
    game.players.clear();
    game.selected_bank_files.clear();
    game.questions.clear();
    game.shuffled_question_ids.clear();
    game.current_round = None;
    game.completed_rounds = 0;
    game.total_rounds = 10;
    game.rebuild_effective_question_pool();

    RoomState {
        room_title,
        owner_token,
        launched: false,
        clear_blocked_names_on_new_game: false,
        blocked_names: HashSet::new(),
        last_activity_at: Instant::now(),
        game,
    }
}

#[derive(Deserialize)]
struct CreateHostedRoomRequest {
    room_title: String,
}

#[derive(Deserialize)]
struct ResumeRoomRequest {
    room_code: String,
    owner_token: String,
}

#[derive(Deserialize)]
struct CloseRoomRequest {
    room_code: String,
    owner_token: String,
}

#[derive(Deserialize)]
struct OwnerRoomQuery {
    room_code: String,
    owner_token: String,
}

#[derive(Deserialize)]
struct JoinRequest {
    room_code: String,
    display_name: String,
}

#[derive(Deserialize)]
struct OwnerStartGameRequest {
    room_code: String,
    owner_token: String,
    total_rounds: usize,
}

#[derive(Deserialize)]
struct OwnerLaunchRoomRequest {
    room_code: String,
    owner_token: String,
}

#[derive(Deserialize)]
struct OwnerEndGameRequest {
    room_code: String,
    owner_token: String,
}

#[derive(Deserialize)]
struct OwnerKickPlayerRequest {
    room_code: String,
    owner_token: String,
    player_id: String,
}

#[derive(Deserialize)]
struct OwnerUnbanNameRequest {
    room_code: String,
    owner_token: String,
    player_name: String,
}

#[derive(Deserialize)]
struct OwnerSetBankSelectionRequest {
    room_code: String,
    owner_token: String,
    selected_files: Vec<String>,
}

#[derive(Deserialize)]
struct OwnerUpdateSettingsRequest {
    room_code: String,
    owner_token: String,
    speed_bonus_enabled: Option<bool>,
    hide_scores_until_end: Option<bool>,
    powerups_enabled: Option<bool>,
    clear_blocked_names_on_new_game: Option<bool>,
    response_seconds: Option<u64>,
    auto_issue_enabled: Option<bool>,
    auto_issue_delay_secs: Option<u64>,
}

#[derive(Deserialize)]
struct WsClientMessage {
    action: String,
    choice_index: Option<usize>,
    powerup: Option<PowerUp>,
    tutorial_seen: Option<bool>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    timestamp: String,
}

#[derive(Serialize)]
struct ServerInfoResponse {
    version: &'static str,
    host_ip: String,
    port: u16,
    home_url: String,
    player_url: String,
}

#[derive(Deserialize)]
struct QrQuery {
    text: String,
}

#[tokio::main]
async fn main() {
    if maybe_relaunch_in_terminal() {
        return;
    }

    tracing_subscriber::fmt().with_env_filter("info").init();

    let host = env::var("QUIZTER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("QUIZTER_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(8080);
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 8080)));
    let detected_lan_ip = detect_lan_ip().unwrap_or_else(|| "127.0.0.1".to_string());
    let public_base_url = normalized_public_base_url(
        env::var("QUIZTER_PUBLIC_BASE_URL").ok(),
        port,
        &detected_lan_ip,
    );
    let host_ip = host_label_from_base_url(&public_base_url);
    let player_join_url = format!("{}/player", public_base_url);

    let runtime_root = runtime_root_dir();
    let data_dir = runtime_root.join("data");
    let _ = fs::create_dir_all(&data_dir);
    let state = AppState {
        rooms: Arc::new(Mutex::new(HashMap::new())),
        owner_index: Arc::new(Mutex::new(HashMap::new())),
        clients: Arc::new(Mutex::new(HashMap::new())),
        data_dir: Arc::new(data_dir),
        public_base_url: Arc::new(public_base_url.clone()),
        player_join_url: Arc::new(player_join_url),
        host_ip: Arc::new(host_ip),
        port,
        runtime_root: Arc::new(runtime_root.clone()),
    };

    let assets_dir = runtime_root.join("assets");
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/player", get(player_page))
        .route("/api/server_info", get(server_info))
        .route("/api/qr.svg", get(qr_svg))
        .route("/api/rooms/create", post(create_hosted_room))
        .route("/api/rooms/resume", post(resume_hosted_room))
        .route("/api/rooms/close", post(close_hosted_room))
        .route("/api/rooms/status", get(get_owner_room_status))
        .route("/api/rooms/question_banks", get(get_owner_question_banks))
        .route("/api/rooms/question_banks/selection", post(set_owner_question_bank_selection))
        .route("/api/rooms/settings", post(update_owner_room_settings))
        .route("/api/rooms/launch", post(launch_owner_room))
        .route("/api/rooms/start", post(start_owner_game))
        .route("/api/rooms/end_game", post(end_owner_game))
        .route("/api/rooms/kick", post(kick_owner_player))
        .route("/api/rooms/unban", post(unban_owner_name))
        .route("/api/join", post(join_room))
        .route("/api/state/:client_id", get(get_state))
        .route("/ws/:client_id", get(ws_handler))
        .nest_service("/assets", ServeDir::new(assets_dir))
        .with_state(state.clone());

    tracing::info!("Quizter server listening on {}", addr);
    tracing::info!("Player join URL: {}/player", public_base_url);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind listener");

    spawn_room_cleanup_task(state.clone());
    maybe_open_home_browser(port);

    axum::serve(listener, app)
        .await
        .expect("server execution failed");
}

async fn root(State(state): State<AppState>) -> Html<String> {
    Html(read_web_html(&state.runtime_root, "home"))
}

async fn player_page(State(state): State<AppState>) -> Html<String> {
    Html(read_web_html(&state.runtime_root, "player"))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "quizter-server",
        timestamp: Utc::now().to_rfc3339(),
    })
}

async fn server_info(State(state): State<AppState>) -> Json<ServerInfoResponse> {
    let version = env!("CARGO_PKG_VERSION");
    let home_url = state.public_base_url.as_ref().clone();
    let player_url = state.player_join_url.as_ref().clone();
    let host_ip = state.host_ip.as_ref().clone();
    let port = state.port;

    Json(ServerInfoResponse {
        version,
        host_ip,
        port,
        home_url,
        player_url,
    })
}

async fn qr_svg(Query(query): Query<QrQuery>) -> impl IntoResponse {
    let Ok(code) = qrcode::QrCode::new(query.text.as_bytes()) else {
        return (
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            "Failed to generate QR".to_string(),
        );
    };
    let image = code
        .render::<qrcode::render::svg::Color<'_>>()
        .min_dimensions(280, 280)
        .dark_color(qrcode::render::svg::Color("#00f6ff"))
        .light_color(qrcode::render::svg::Color("#090909"))
        .build();
    ([(header::CONTENT_TYPE, "image/svg+xml")], image)
}

async fn touch_room(state: &AppState, room_code: &str) {
    if let Some(room) = state.rooms.lock().await.get_mut(room_code) {
        room.last_activity_at = Instant::now();
    }
}

async fn with_room_mut<T>(
    state: &AppState,
    room_code: &str,
    f: impl FnOnce(&mut RoomState) -> T,
) -> Option<T> {
    let mut rooms = state.rooms.lock().await;
    rooms.get_mut(room_code).map(f)
}

async fn with_room<T>(
    state: &AppState,
    room_code: &str,
    f: impl FnOnce(&RoomState) -> T,
) -> Option<T> {
    let rooms = state.rooms.lock().await;
    rooms.get(room_code).map(f)
}

async fn room_code_for_join_request(state: &AppState, room_code: &str) -> Option<String> {
    with_room(state, room_code, |_| room_code.to_string()).await
}

async fn room_code_for_owner_token(state: &AppState, owner_token: &str) -> Option<String> {
    state.owner_index.lock().await.get(owner_token).cloned()
}

async fn validate_owner_room_access(
    state: &AppState,
    room_code: &str,
    owner_token: &str,
) -> Option<String> {
    let owner_room_code = room_code_for_owner_token(state, owner_token).await?;
    if owner_room_code != room_code {
        return None;
    }
    let valid = with_room(state, room_code, |room| room.owner_token == owner_token)
        .await
        .unwrap_or(false);
    if valid {
        Some(room_code.to_string())
    } else {
        None
    }
}

async fn remove_room_and_clients(
    state: &AppState,
    room_code: &str,
    owner_token: &str,
    event_name: &str,
) -> Option<String> {
    let removed_room = {
        let mut rooms = state.rooms.lock().await;
        match rooms.get(room_code) {
            Some(room) if room.owner_token == owner_token => {}
            Some(_) => return None,
            None => return None,
        }
        rooms.remove(room_code)
    }?;

    state.owner_index.lock().await.remove(owner_token);

    let client_ids = {
        let clients = state.clients.lock().await;
        clients
            .iter()
            .filter_map(|(client_id, client)| {
                if client.room_code == room_code {
                    Some(client_id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };

    for client_id in &client_ids {
        let _ = send_to_client(
            state,
            client_id,
            json!({"event": event_name, "payload": {"room_code": room_code}}),
        )
        .await;
    }

    {
        let mut clients = state.clients.lock().await;
        for client_id in client_ids {
            clients.remove(&client_id);
        }
    }

    Some(removed_room.room_title)
}

fn spawn_room_cleanup_task(state: AppState) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(ROOM_CLEANUP_INTERVAL_SECS)).await;

            let expired_rooms = {
                let rooms = state.rooms.lock().await;
                let now = Instant::now();
                rooms
                    .iter()
                    .filter_map(|(room_code, room)| {
                        if now.duration_since(room.last_activity_at).as_secs()
                            >= ROOM_INACTIVITY_TIMEOUT_SECS
                        {
                            Some((room_code.clone(), room.owner_token.clone()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            };

            for (room_code, owner_token) in expired_rooms {
                let _ = remove_room_and_clients(&state, &room_code, &owner_token, "room_expired").await;
            }
        }
    });
}

async fn owner_room_payload(state: &AppState, room_code: &str) -> Option<Value> {
    with_room_mut(state, room_code, |room| {
        room.last_activity_at = Instant::now();
        let players = room
            .game
            .players
            .values()
            .map(|player| {
                json!({
                    "id": player.id,
                    "name": player.name,
                    "score": (player.score * 100.0).round() / 100.0,
                    "connected": player.connected,
                })
            })
            .collect::<Vec<_>>();
        let mut blocked_names = room.blocked_names.iter().cloned().collect::<Vec<_>>();
        blocked_names.sort();
        json!({
            "room_code": room.game.room_code,
            "room_title": room.room_title,
            "launched": room.launched,
            "status": room.game.status,
            "available_questions": room.game.total_available_questions(),
            "questions_in_play": room.game.questions.len(),
            "total_rounds": room.game.total_rounds,
            "completed_rounds": room.game.completed_rounds,
            "speed_bonus_enabled": room.game.speed_bonus_enabled,
            "hide_scores_until_end": room.game.hide_scores_until_end,
            "powerups_enabled": room.game.powerups_enabled,
            "clear_blocked_names_on_new_game": room.clear_blocked_names_on_new_game,
            "response_seconds": room.game.response_seconds,
            "auto_issue_enabled": room.game.auto_issue_enabled,
            "auto_issue_delay_secs": room.game.auto_issue_delay_secs,
            "players": players,
            "blocked_names": blocked_names,
            "leaderboard": room.game.leaderboard(),
            "player_url": if room.launched {
                format!("{}?room={}", state.player_join_url, room.game.room_code)
            } else {
                String::new()
            },
        })
    })
    .await
}

async fn room_code_for_known_client(state: &AppState, client_id: &str) -> Option<String> {
    {
        let clients = state.clients.lock().await;
        if let Some(client) = clients.get(client_id) {
            return Some(client.room_code.clone());
        }
    }

    let rooms = state.rooms.lock().await;
    for (room_code, room) in rooms.iter() {
        if room.game.players.contains_key(client_id) {
            return Some(room_code.clone());
        }
    }
    None
}

async fn create_hosted_room(
    State(state): State<AppState>,
    Json(req): Json<CreateHostedRoomRequest>,
) -> impl IntoResponse {
    let room_title = match normalize_room_title(&req.room_title) {
        Ok(title) => title,
        Err(error) => return (StatusCode::BAD_REQUEST, Json(json!({"error": error}))),
    };

    let room_info = {
        let mut rooms = state.rooms.lock().await;
        let existing_codes: HashSet<String> = rooms.keys().cloned().collect();
        let Some(room_code) = generate_room_code(&existing_codes) else {
            return (
                StatusCode::CONFLICT,
                Json(json!({"error": "room_code_unavailable"})),
            );
        };

        let hosted_template =
            GameState::new(Vec::new(), load_file_question_banks(&state.runtime_root), HashSet::new());
        let owner_token = new_owner_token();
        let room = room_from_template(
            &hosted_template,
            room_code.clone(),
            room_title.clone(),
            owner_token.clone(),
        );
        rooms.insert(room_code.clone(), room);
        (room_code, room_title, owner_token)
    };
    state
        .owner_index
        .lock()
        .await
        .insert(room_info.2.clone(), room_info.0.clone());

    let Some(mut payload) = owner_room_payload(&state, &room_info.0).await else {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "invalid_room_code"})));
    };

    if let Some(object) = payload.as_object_mut() {
        object.insert("owner_token".to_string(), json!(room_info.2));
    }

    (StatusCode::OK, Json(payload))
}

async fn resume_hosted_room(
    State(state): State<AppState>,
    Json(req): Json<ResumeRoomRequest>,
) -> impl IntoResponse {
    let Some(owner_room_code) = room_code_for_owner_token(&state, &req.owner_token).await else {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid_owner_token"})));
    };
    if owner_room_code != req.room_code {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid_owner_token"})));
    }

    let Some(valid_room_code) =
        validate_owner_room_access(&state, &req.room_code, &req.owner_token).await
    else {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid_owner_token"})));
    };

    let Some(payload) = owner_room_payload(&state, &valid_room_code).await else {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "invalid_room_code"})));
    };

    (StatusCode::OK, Json(payload))
}

async fn close_hosted_room(
    State(state): State<AppState>,
    Json(req): Json<CloseRoomRequest>,
) -> impl IntoResponse {
    let Some(owner_room_code) = room_code_for_owner_token(&state, &req.owner_token).await else {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid_owner_token"})));
    };
    if owner_room_code != req.room_code {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid_owner_token"})));
    }

    let Some(room_title) =
        remove_room_and_clients(&state, &owner_room_code, &req.owner_token, "room_closed").await
    else {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "invalid_room_code"})));
    };

    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "room_code": owner_room_code,
            "room_title": room_title
        })),
    )
}

async fn get_owner_room_status(
    State(state): State<AppState>,
    Query(query): Query<OwnerRoomQuery>,
) -> impl IntoResponse {
    let Some(room_code) =
        validate_owner_room_access(&state, &query.room_code, &query.owner_token).await
    else {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid_owner_token"})));
    };
    let Some(payload) = owner_room_payload(&state, &room_code).await else {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "invalid_room_code"})));
    };
    (StatusCode::OK, Json(payload))
}

async fn get_owner_question_banks(
    State(state): State<AppState>,
    Query(query): Query<OwnerRoomQuery>,
) -> impl IntoResponse {
    let Some(room_code) =
        validate_owner_room_access(&state, &query.room_code, &query.owner_token).await
    else {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid_owner_token"})));
    };

    let payload = with_room(&state, &room_code, |room| {
        let game = &room.game;
        let mut selected: Vec<String> = game.selected_bank_files.iter().cloned().collect();
        selected.sort();
        json!({
            "room_code": room.game.room_code,
            "room_title": room.room_title,
            "available_files": game.available_bank_files(),
            "selected_files": selected,
            "category_tree": game.question_bank_tree(),
            "effective_question_count": game.questions.len(),
            "available_question_count": game.total_available_questions(),
        })
    })
    .await
    .unwrap_or_else(|| json!({"error": "invalid_room_code"}));

    (StatusCode::OK, Json(payload))
}

async fn set_owner_question_bank_selection(
    State(state): State<AppState>,
    Json(req): Json<OwnerSetBankSelectionRequest>,
) -> impl IntoResponse {
    let Some(room_code) =
        validate_owner_room_access(&state, &req.room_code, &req.owner_token).await
    else {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid_owner_token"})));
    };

    let counts = with_room_mut(&state, &room_code, |room| {
        room.last_activity_at = Instant::now();
        let game = &mut room.game;
        let available: HashSet<String> = game.available_bank_files().into_iter().collect();
        let selected: HashSet<String> = req
            .selected_files
            .into_iter()
            .filter(|name| available.contains(name))
            .collect();
        game.selected_bank_files = selected;
        game.rebuild_effective_question_pool();
        game.reflow_future_rounds_after_pool_change();
        (game.questions.len(), game.total_available_questions())
    })
    .await;

    let Some((effective_question_count, available_question_count)) = counts else {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "invalid_room_code"})));
    };

    broadcast_room_state(&state, &room_code).await;
    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "effective_question_count": effective_question_count,
            "available_question_count": available_question_count
        })),
    )
}

async fn update_owner_room_settings(
    State(state): State<AppState>,
    Json(req): Json<OwnerUpdateSettingsRequest>,
) -> impl IntoResponse {
    let Some(room_code) =
        validate_owner_room_access(&state, &req.room_code, &req.owner_token).await
    else {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid_owner_token"})));
    };

    let updated = with_room_mut(&state, &room_code, |room| {
        room.last_activity_at = Instant::now();
        let game = &mut room.game;
        if let Some(enabled) = req.speed_bonus_enabled {
            game.speed_bonus_enabled = enabled;
        }
        if let Some(hidden) = req.hide_scores_until_end {
            game.hide_scores_until_end = hidden;
        }
        if let Some(enabled) = req.powerups_enabled {
            game.powerups_enabled = enabled;
        }
        if let Some(enabled) = req.clear_blocked_names_on_new_game {
            room.clear_blocked_names_on_new_game = enabled;
        }
        if let Some(seconds) = req.response_seconds {
            game.response_seconds = seconds.clamp(1, 300);
        }
        if let Some(enabled) = req.auto_issue_enabled {
            game.auto_issue_enabled = enabled;
        }
        if let Some(delay) = req.auto_issue_delay_secs {
            game.auto_issue_delay_secs = delay.clamp(1, 300);
        }
        true
    })
    .await;

    if updated != Some(true) {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "invalid_room_code"})));
    }

    let Some(payload) = owner_room_payload(&state, &room_code).await else {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "invalid_room_code"})));
    };

    broadcast_room_state(&state, &room_code).await;
    (StatusCode::OK, Json(payload))
}

async fn launch_owner_room(
    State(state): State<AppState>,
    Json(req): Json<OwnerLaunchRoomRequest>,
) -> impl IntoResponse {
    let Some(room_code) =
        validate_owner_room_access(&state, &req.room_code, &req.owner_token).await
    else {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid_owner_token"})));
    };

    let launched = with_room_mut(&state, &room_code, |room| {
        room.last_activity_at = Instant::now();
        if room.game.questions.is_empty() {
            return false;
        }
        room.launched = true;
        true
    })
    .await;

    match launched {
        Some(true) => {}
        Some(false) => {
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "no_questions"})));
        }
        None => {
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "invalid_room_code"})));
        }
    }

    let Some(payload) = owner_room_payload(&state, &room_code).await else {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "invalid_room_code"})));
    };

    broadcast_room_state(&state, &room_code).await;
    (StatusCode::OK, Json(payload))
}

async fn start_owner_game(
    State(state): State<AppState>,
    Json(req): Json<OwnerStartGameRequest>,
) -> impl IntoResponse {
    let Some(room_code) =
        validate_owner_room_access(&state, &req.room_code, &req.owner_token).await
    else {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid_owner_token"})));
    };

    let startable = with_room_mut(&state, &room_code, |room| {
        room.last_activity_at = Instant::now();
        if !room.launched {
            return Err("room_not_open");
        }
        let game = &mut room.game;
        if room.clear_blocked_names_on_new_game {
            room.blocked_names.clear();
        }
        prepare_game_for_start(game, req.total_rounds)
    })
    .await;

    match startable {
        Some(Ok(())) => {}
        Some(Err(error)) => return (StatusCode::BAD_REQUEST, Json(json!({"error": error}))),
        None => {
            return (StatusCode::BAD_REQUEST, Json(json!({"error": "invalid_room_code"})));
        }
    }

    start_next_round_in_room(state.clone(), &room_code).await;
    (StatusCode::OK, Json(json!({"ok": true})))
}

async fn end_owner_game(
    State(state): State<AppState>,
    Json(req): Json<OwnerEndGameRequest>,
) -> impl IntoResponse {
    let Some(room_code) =
        validate_owner_room_access(&state, &req.room_code, &req.owner_token).await
    else {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid_owner_token"})));
    };

    let ended = with_room_mut(&state, &room_code, |room| {
        room.last_activity_at = Instant::now();
        mark_game_ended(&mut room.game)
    })
    .await;

    let Some(ended) = ended else {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "invalid_room_code"})));
    };

    broadcast_room_state(&state, &room_code).await;
    (
        StatusCode::OK,
        Json(json!({
            "ok": true,
            "ended": ended
        })),
    )
}

async fn kick_owner_player(
    State(state): State<AppState>,
    Json(req): Json<OwnerKickPlayerRequest>,
) -> impl IntoResponse {
    let Some(room_code) =
        validate_owner_room_access(&state, &req.room_code, &req.owner_token).await
    else {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid_owner_token"})));
    };

    let kicked = with_room_mut(&state, &room_code, |room| {
        room.last_activity_at = Instant::now();
        let game = &mut room.game;
        let Some(player) = game.players.remove(&req.player_id) else {
            return None;
        };
        room.blocked_names.insert(player.name.clone());
        Some(player.name)
    })
    .await;

    let Some(Some(player_name)) = kicked else {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "invalid_player_id"})));
    };

    let _ = send_to_client(
        &state,
        &req.player_id,
        json!({"event": "player_kicked", "payload": {"room_code": room_code, "player_name": player_name}}),
    )
    .await;

    {
        let mut clients = state.clients.lock().await;
        clients.remove(&req.player_id);
    }

    broadcast_room_state(&state, &room_code).await;
    (StatusCode::OK, Json(json!({"ok": true, "player_name": player_name})))
}

async fn unban_owner_name(
    State(state): State<AppState>,
    Json(req): Json<OwnerUnbanNameRequest>,
) -> impl IntoResponse {
    let Some(room_code) =
        validate_owner_room_access(&state, &req.room_code, &req.owner_token).await
    else {
        return (StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid_owner_token"})));
    };

    let removed = with_room_mut(&state, &room_code, |room| {
        room.last_activity_at = Instant::now();
        let existing = room
            .blocked_names
            .iter()
            .find(|name| name.eq_ignore_ascii_case(req.player_name.trim()))
            .cloned();
        if let Some(name) = existing {
            room.blocked_names.remove(&name);
            Some(name)
        } else {
            None
        }
    })
    .await;

    let Some(unbanned_name) = removed.flatten() else {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "player_name_not_blocked"})));
    };

    broadcast_room_state(&state, &room_code).await;
    (StatusCode::OK, Json(json!({"ok": true, "player_name": unbanned_name})))
}

async fn join_room(State(state): State<AppState>, Json(req): Json<JoinRequest>) -> impl IntoResponse {
    let display_name = match normalize_player_name(&req.display_name) {
        Ok(name) => name,
        Err(error) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(json!({"error": error})),
            )
        }
    };
    let Some(room_code) = room_code_for_join_request(&state, &req.room_code).await else {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_room_code"})),
        );
    };
    let player_id = {
        with_room_mut(&state, &room_code, |room| {
            room.last_activity_at = Instant::now();
            if !room.launched {
                return Err((
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(json!({"error": "room_not_open"})),
                ));
            }
            let game = &mut room.game;
            if req.room_code != game.room_code {
                return Err((
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(json!({"error": "invalid_room_code"})),
                ));
            }

            let existing = game
                .players
                .values_mut()
                .find(|p| p.name.eq_ignore_ascii_case(&display_name));

            if room
                .blocked_names
                .iter()
                .any(|name| name.eq_ignore_ascii_case(&display_name))
            {
                return Err((
                    axum::http::StatusCode::BAD_REQUEST,
                    Json(json!({"error": "player_blocked"})),
                ));
            }

            let player_id = if let Some(player) = existing {
                player.connected = true;
                player.id.clone()
            } else {
                let id = format!("player-{}", Uuid::new_v4());
                game.players.insert(
                    id.clone(),
                    PlayerState {
                        id: id.clone(),
                        name: display_name.clone(),
                        score: 0.0,
                        last_score_delta: 0.0,
                        connected: true,
                        eligible_from_round: eligible_from_round_for_new_player(game),
                        used_powerups: HashSet::new(),
                        pending_powerup: None,
                        tutorial_seen: false,
                    },
                );
                id
            };

            Ok(player_id)
        })
        .await
    };
    let player_id = match player_id {
        Some(Ok(player_id)) => player_id,
        Some(Err(err)) => return err,
        None => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(json!({"error": "invalid_room_code"})),
            )
        }
    };
    broadcast_room_state(&state, &room_code).await;
    (axum::http::StatusCode::OK, Json(json!({"player_id": player_id})))
}

async fn get_state(State(state): State<AppState>, AxPath(client_id): AxPath<String>) -> Json<Value> {
    Json(build_state_snapshot(&state, &client_id).await)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    AxPath(client_id): AxPath<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, client_id))
}

async fn handle_socket(stream: axum::extract::ws::WebSocket, state: AppState, client_id: String) {
    let (mut sender, mut receiver) = stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    let Some(room_code) = room_code_for_known_client(&state, &client_id).await else {
        return;
    };

    state.clients.lock().await.insert(
        client_id.clone(),
        ClientConnection {
            room_code: room_code.clone(),
            tx,
        },
    );
    touch_room(&state, &room_code).await;

    let init = build_state_snapshot(&state, &client_id).await;
    let _ = send_to_client(&state, &client_id, json!({"event": "state", "payload": init})).await;

    let write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            if let Ok(client_msg) = serde_json::from_str::<WsClientMessage>(&text) {
                handle_client_action(&state, &client_id, client_msg).await;
            }
        }
    }

    write_task.abort();
    state.clients.lock().await.remove(&client_id);

    {
        let _ = with_room_mut(&state, &room_code, |room| {
            room.last_activity_at = Instant::now();
            let game = &mut room.game;
            if let Some(player) = game.players.get_mut(&client_id) {
                player.connected = false;
            }
        })
        .await;
    }
    broadcast_room_state(&state, &room_code).await;
}

async fn handle_client_action(state: &AppState, client_id: &str, msg: WsClientMessage) {
    match msg.action.as_str() {
        "submit_answer" => {
            if let Some(choice_index) = msg.choice_index {
                submit_answer(state, client_id, choice_index).await;
            }
        }
        "activate_powerup" => {
            if let Some(powerup) = msg.powerup {
                activate_powerup(state, client_id, powerup).await;
            }
        }
        "tutorial_seen" => {
            if msg.tutorial_seen.unwrap_or(false) {
                let Some(room_code) = room_code_for_known_client(state, client_id).await else {
                    return;
                };
                {
                    let mut rooms = state.rooms.lock().await;
                    let room = match rooms.get_mut(&room_code) {
                        Some(room) => room,
                        None => return,
                    };
                    room.last_activity_at = Instant::now();
                    let game = &mut room.game;
                    if let Some(player) = game.players.get_mut(client_id) {
                        player.tutorial_seen = true;
                    }
                }
                broadcast_room_state(state, &room_code).await;
            }
        }
        _ => {}
    }
}

async fn submit_answer(state: &AppState, client_id: &str, choice_index: usize) {
    let Some(room_code) = room_code_for_known_client(state, client_id).await else {
        return;
    };
    let mut should_finalize = false;

    {
        let mut rooms = state.rooms.lock().await;
        let room = match rooms.get_mut(&room_code) {
            Some(room) => room,
            None => return,
        };
        room.last_activity_at = Instant::now();
        let game = &mut room.game;
        if game.status != GameStatus::InRound {
            return;
        }

        if game
            .players
            .get(client_id)
            .map(|player| !player_can_participate_in_current_round(Some(player), game.current_round.as_ref()))
            .unwrap_or(true)
        {
            return;
        }

        let connected_players = game.players.values().filter(|p| p.connected).count();
        let round = match game.current_round.as_mut() {
            Some(r) => r,
            None => return,
        };

        if choice_index > 3 || round.answers.contains_key(client_id) {
            return;
        }

        if let Some(lock_owner) = &round.speed_searcher_owner {
            if lock_owner != client_id {
                return;
            }
        }

        round.answers.insert(
            client_id.to_string(),
            AnswerState {
                choice_index,
                submitted_at: Instant::now(),
            },
        );

        if let Some(lock_owner) = &round.speed_searcher_owner {
            if lock_owner == client_id {
                should_finalize = true;
            }
        } else {
            if round.answers.len() >= connected_players.max(1) {
                should_finalize = true;
            }
        }
    }

    broadcast_room_state(state, &room_code).await;
    if should_finalize {
        finalize_round_in_room(state.clone(), &room_code).await;
    }
}

async fn activate_powerup(state: &AppState, client_id: &str, powerup: PowerUp) {
    let Some(room_code) = room_code_for_known_client(state, client_id).await else {
        return;
    };
    let mut activation_message = None;
    let mut queued = false;

    {
        let mut rooms = state.rooms.lock().await;
        let room = match rooms.get_mut(&room_code) {
            Some(room) => room,
            None => return,
        };
        room.last_activity_at = Instant::now();
        let game = &mut room.game;
        if !game.powerups_enabled {
            return;
        }
        let status = game.status.clone();
        let player = match game.players.get_mut(client_id) {
            Some(p) => p,
            None => return,
        };

        if player.used_powerups.contains(&powerup) {
            return;
        }

        if status == GameStatus::Ended {
            return;
        }

        if status != GameStatus::InRound {
            if player.pending_powerup.is_some() {
                return;
            }
            player.used_powerups.insert(powerup.clone());
            player.pending_powerup = Some(powerup.clone());
            queued = true;
        } else {
            player.used_powerups.insert(powerup.clone());
            player.pending_powerup = None;
            activation_message = apply_powerup_to_current_round(game, client_id, powerup.clone());
        }
    }

    if let Some(message) = activation_message {
        broadcast_room_json(state, &room_code, message).await;
    }

    if queued {
        let queued_notice = json!({
            "event": "powerup_queued",
            "payload": {
                "player_id": client_id,
                "powerup": powerup
            }
        });
        let _ = send_to_client(state, client_id, queued_notice).await;
    }

    broadcast_room_state(state, &room_code).await;
}

fn apply_powerup_to_current_round(
    game: &mut GameState,
    client_id: &str,
    powerup: PowerUp,
) -> Option<Value> {
    let player_name = game
        .players
        .get(client_id)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| client_id.to_string());
    let connected_other_players: Vec<String> = game
        .players
        .values()
        .filter(|p| p.connected && p.id != client_id)
        .map(|p| p.id.clone())
        .collect();

    let mut affected_players: Vec<String> = Vec::new();
    let mut alert_message: Option<String> = None;
    let powerup_payload;

    let round = game.current_round.as_mut()?;
    match powerup {
        PowerUp::MixMaster => {
            round.mix_master_owner = Some(client_id.to_string());
            powerup_payload = Some(json!({"active": true}));
            affected_players = connected_other_players;
            alert_message = Some("Mix Master is active: your answer text/order may be scrambled.".to_string());
        }
        PowerUp::SpeedSearcher => {
            round.speed_searcher_owner = Some(client_id.to_string());
            round.answer_window_secs = 60;
            round.started_at = Instant::now();
            round.deadline = round.started_at + Duration::from_secs(60);
            powerup_payload = Some(json!({"owner": client_id, "seconds": 60}));
            affected_players = connected_other_players;
            alert_message = Some("Speed Searcher activated: you are locked out until the 60s window ends.".to_string());
        }
        PowerUp::DoubleDowner => {
            round.double_downers.insert(client_id.to_string());
            powerup_payload = Some(json!({"active": true}));
        }
        PowerUp::CloneCommander => {
            round.clone_commanders.insert(client_id.to_string());
            powerup_payload = Some(json!({"active": true}));
        }
        PowerUp::SuperSpliter => {
            let mut rng = rand::thread_rng();
            let incorrects: Vec<usize> = (0..4)
                .filter(|i| *i != round.question.correct_index)
                .collect();
            let random_incorrect = *incorrects.choose(&mut rng).unwrap_or(&incorrects[0]);
            for target_id in &connected_other_players {
                round
                    .super_spliter_targets
                    .insert(target_id.clone(), (round.question.correct_index, random_incorrect));
            }
            powerup_payload = Some(json!({"targets": connected_other_players}));
            affected_players = connected_other_players;
            alert_message = Some("Super Spliter activated: your choices were reduced this round.".to_string());
        }
        PowerUp::GreatGambler => {
            if round.great_gambler_factor.is_none() {
                let mut rng = rand::thread_rng();
                let factor = rng.gen_range(-1.0f64..=3.0f64);
                round.great_gambler_factor = Some(factor);
            }
            powerup_payload = Some(json!({"factor": round.great_gambler_factor}));
            affected_players = connected_other_players;
            alert_message = Some("Great Gambler activated: round scoring will be multiplied.".to_string());
        }
    }

    Some(json!({
        "event": "powerup_activated",
        "payload": {
            "player_id": client_id,
            "player_name": player_name,
            "powerup": powerup,
            "details": powerup_payload,
            "affected_players": affected_players,
            "alert_message": alert_message
        }
    }))
}

async fn start_next_round_in_room(state: AppState, room_code: &str) {
    let mut round_started = false;
    let mut should_end_game = false;
    let mut queued_activations = Vec::new();
    let room_code = room_code.to_string();
    {
        let Some(()) = with_room_mut(&state, &room_code, |room| {
            room.last_activity_at = Instant::now();
            let game = &mut room.game;
        if game.status == GameStatus::InRound || game.status == GameStatus::Ended {
            return;
        }
        if game.current_round.is_some() {
            return;
        }
        if game.completed_rounds >= game.total_rounds || game.questions.is_empty() {
            should_end_game = true;
        } else {
            let next_id = game.shuffled_question_ids.get(game.completed_rounds).cloned();
            if let Some(question_id) = next_id {
                if let Some(question) = game.questions.iter().find(|q| q.id == question_id).cloned() {
                    let started_at = Instant::now();
                    let deadline = started_at + Duration::from_secs(game.response_seconds);
                    let mut option_order = vec![0, 1, 2, 3];
                    option_order.shuffle(&mut rand::thread_rng());
                    game.current_round = Some(RoundState {
                        round_number: game.completed_rounds + 1,
                        question,
                        started_at,
                        deadline,
                        answer_window_secs: game.response_seconds,
                        answers: HashMap::new(),
                        speed_searcher_owner: None,
                        great_gambler_factor: None,
                        double_downers: HashSet::new(),
                        clone_commanders: HashSet::new(),
                        super_spliter_targets: HashMap::new(),
                        mix_master_owner: None,
                        option_order,
                    });
                    let queued: Vec<(String, PowerUp)> = game
                        .players
                        .iter()
                        .filter_map(|(player_id, player)| {
                            player
                                .pending_powerup
                                .clone()
                                .map(|powerup| (player_id.clone(), powerup))
                        })
                        .collect();
                    for (player_id, powerup) in queued {
                        if let Some(player) = game.players.get_mut(&player_id) {
                            player.pending_powerup = None;
                        }
                        if let Some(message) =
                            apply_powerup_to_current_round(game, &player_id, powerup)
                        {
                            queued_activations.push(message);
                        }
                    }
                    game.status = GameStatus::InRound;
                    round_started = true;
                } else {
                    should_end_game = true;
                }
            } else {
                should_end_game = true;
            }
        }
        })
        .await else {
            return;
        };
    }

    if should_end_game {
        {
            let _ = with_room_mut(&state, &room_code, |room| {
                room.last_activity_at = Instant::now();
                let game = &mut room.game;
                game.status = GameStatus::Ended;
                game.current_round = None;
                let history = HistoryEntry {
                    finished_at: Utc::now().to_rfc3339(),
                    rounds_played: game.completed_rounds,
                    leaderboard: game.leaderboard(),
                };
                append_history(&state.data_dir, history);
            })
            .await;
        }
        broadcast_room_state(&state, &room_code).await;
        return;
    }

    if round_started {
        broadcast_room_state(&state, &room_code).await;
        for message in queued_activations {
            broadcast_room_json(&state, &room_code, message).await;
        }
        spawn_round_timer(state, room_code).await;
    }
}

async fn spawn_round_timer(state: AppState, room_code: String) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let (status, remaining) = {
                let Some(result) = with_room(&state, &room_code, |room| {
                    let game = &room.game;
                    if game.status != GameStatus::InRound {
                        return None;
                    }
                    if let Some(round) = &game.current_round {
                        let now = Instant::now();
                        let rem = if round.deadline > now {
                            (round.deadline - now).as_secs()
                        } else {
                            0
                        };
                        Some((game.status.clone(), rem))
                    } else {
                        None
                    }
                })
                .await else {
                    break;
                };
                match result {
                    Some(result) => result,
                    None => break,
                }
            };

            if status == GameStatus::InRound {
                broadcast_room_json(&state, &room_code, json!({"event": "timer_tick", "payload": {"seconds_left": remaining}})).await;
            }

            if remaining == 0 {
                finalize_round_in_room(state.clone(), &room_code).await;
                break;
            }
        }
    });
}

fn spawn_auto_issue_timer(state: AppState, room_code: String, delay_secs: u64) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(delay_secs)).await;
        let should_issue = {
            let Some(result) = with_room(&state, &room_code, |room| {
                let game = &room.game;
                game.status == GameStatus::RoundResult
                    && game.current_round.is_none()
                    && game.auto_issue_enabled
            })
            .await else {
                return;
            };
            result
        };
        if should_issue {
            start_next_round_in_room(state.clone(), &room_code).await;
        }
    });
}

async fn finalize_round_in_room(state: AppState, room_code: &str) {
    let room_code = room_code.to_string();

    let Some(Some((result_payload, end_game, auto_issue_enabled, auto_issue_delay_secs))) =
        with_room_mut(&state, &room_code, |room| {
            room.last_activity_at = Instant::now();
            let game = &mut room.game;
            if game.status != GameStatus::InRound {
                return None;
            }

            let round = match game.current_round.take() {
                Some(r) => r,
                None => return None,
            };
            let speed_bonus_enabled = game.speed_bonus_enabled;

            let mut round_scores: HashMap<String, f64> = HashMap::new();
            for player in game.players.values_mut() {
                player.last_score_delta = 0.0;
            }
            for (player_id, ans) in &round.answers {
                let mut score = 0.0;
                let is_correct = ans.choice_index == round.question.correct_index;
                if is_correct {
                    let elapsed_secs = ans.submitted_at.duration_since(round.started_at).as_secs_f64();
                    score = calculate_correct_score(
                        round.question.points,
                        elapsed_secs,
                        round.answer_window_secs as f64,
                        round.double_downers.contains(player_id),
                        speed_bonus_enabled,
                    );
                }
                round_scores.insert(player_id.clone(), score);
            }

            if let Some(multiplier) = round.great_gambler_factor {
                for score in round_scores.values_mut() {
                    *score *= multiplier;
                }
            }

            let top_round_score = round_scores
                .values()
                .copied()
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0);

            for player_id in round.clone_commanders {
                round_scores.insert(player_id, top_round_score);
            }

            for (player_id, gain) in &round_scores {
                if let Some(player) = game.players.get_mut(player_id) {
                    player.score += gain;
                    player.last_score_delta = *gain;
                }
            }

            game.completed_rounds += 1;
            game.status = GameStatus::RoundResult;

            let mut details = serde_json::Map::new();
            for (player_id, answer) in round.answers {
                let gained = round_scores.get(&player_id).copied().unwrap_or(0.0);
                details.insert(
                    player_id,
                    json!({
                        "choice_index": answer.choice_index,
                        "is_correct": answer.choice_index == round.question.correct_index,
                        "score_delta": (gained * 100.0).round() / 100.0,
                    }),
                );
            }

            let result_payload = json!({
                "round_number": round.round_number,
                "correct_index": round.question.correct_index,
                "question_id": round.question.id,
                "scores": details,
                "leaderboard": game.leaderboard(),
                "great_gambler_factor": round.great_gambler_factor,
            });

            Some((
                result_payload,
                game.completed_rounds >= game.total_rounds,
                game.auto_issue_enabled,
                game.auto_issue_delay_secs,
            ))
        })
        .await
    else {
            return;
        };

    broadcast_room_json(&state, &room_code, json!({"event": "round_result", "payload": result_payload})).await;
    broadcast_room_state(&state, &room_code).await;

    if end_game {
        {
            let _ = with_room_mut(&state, &room_code, |room| {
                room.last_activity_at = Instant::now();
                let game = &mut room.game;
                game.status = GameStatus::Ended;
                let history = HistoryEntry {
                    finished_at: Utc::now().to_rfc3339(),
                    rounds_played: game.completed_rounds,
                    leaderboard: game.leaderboard(),
                };
                append_history(&state.data_dir, history);
            })
            .await;
        }
        broadcast_room_state(&state, &room_code).await;
    } else if auto_issue_enabled {
        spawn_auto_issue_timer(state.clone(), room_code, auto_issue_delay_secs);
    }
}

async fn build_state_snapshot(state: &AppState, client_id: &str) -> Value {
    let Some(room_code) = room_code_for_known_client(state, client_id).await else {
        return json!({
            "status": GameStatus::Ended,
            "role": Role::Player
        });
    };
    let snapshot = with_room(state, &room_code, |room| {
        let game = &room.game;
        let mut visible_question = None;
        let mut waiting_for_next_round = false;

        let player_eligible_for_round = player_can_participate_in_current_round(
            game.players.get(client_id),
            game.current_round.as_ref(),
        );

        if let Some(round) = &game.current_round {
            if player_eligible_for_round {
                let mut options: Vec<Value> = round
                    .option_order
                    .iter()
                    .filter_map(|idx| {
                        round
                            .question
                            .options
                            .get(*idx)
                            .map(|text| json!({"index": idx, "text": text}))
                    })
                    .collect();

                if let Some((correct, incorrect)) = round.super_spliter_targets.get(client_id) {
                    options.retain(|o| {
                        let idx = o.get("index").and_then(|v| v.as_u64()).unwrap_or(99) as usize;
                        idx == *correct || idx == *incorrect
                    });
                }

                visible_question = Some(json!({
                    "id": round.question.id,
                    "category": round.question.category,
                    "prompt": round.question.prompt,
                    "image_url": round.question.image_url,
                    "points": round.question.points,
                    "options": options,
                    "round_number": round.round_number,
                    "total_rounds": game.total_rounds,
                    "seconds_left": round.deadline.saturating_duration_since(Instant::now()).as_secs(),
                    "speed_searcher_owner": round.speed_searcher_owner,
                    "mix_master_owner": round.mix_master_owner,
                }));
            } else {
                waiting_for_next_round = true;
            }
        }

        let role = Role::Player;
        let scores_hidden = game.hide_scores_until_end && game.status != GameStatus::Ended;
        let show_leaderboard = should_show_player_leaderboard(&game.status, game.completed_rounds);

        let your_state = game.players.get(client_id).map(|p| {
            json!({
                "id": p.id,
                "name": p.name,
                "score": (p.score * 100.0).round() / 100.0,
                "eligible_from_round": p.eligible_from_round,
                "used_powerups": p.used_powerups,
                "pending_powerup": p.pending_powerup,
                "tutorial_seen": p.tutorial_seen,
            })
        });

        json!({
            "status": game.status,
            "room_code": game.room_code,
            "room_title": room.room_title,
            "role": role,
            "total_rounds": game.total_rounds,
            "completed_rounds": game.completed_rounds,
            "speed_bonus_enabled": game.speed_bonus_enabled,
            "hide_scores_until_end": game.hide_scores_until_end,
            "powerups_enabled": game.powerups_enabled,
            "response_seconds": game.response_seconds,
            "auto_issue_enabled": game.auto_issue_enabled,
            "auto_issue_delay_secs": game.auto_issue_delay_secs,
            "scores_hidden": scores_hidden,
            "show_leaderboard": show_leaderboard,
            "waiting_for_next_round": waiting_for_next_round,
            "questions_available": game.questions.len(),
            "questions_in_play": game.questions.len(),
            "available_questions": game.total_available_questions(),
            "leaderboard": if scores_hidden {
                game.leaderboard()
                    .into_iter()
                    .map(|entry| {
                        json!({
                            "player_id": entry.player_id,
                            "name": entry.name,
                            "score": 0.0,
                            "last_delta": 0.0
                        })
                    })
                    .collect::<Vec<_>>()
            } else {
                game.leaderboard()
                    .into_iter()
                    .map(|entry| serde_json::to_value(entry).unwrap_or_else(|_| json!({})))
                    .collect::<Vec<_>>()
            },
            "current_question": visible_question,
            "you": your_state,
        })
    })
    .await;

    snapshot.unwrap_or_else(|| {
        json!({
            "status": GameStatus::Ended,
            "room_code": room_code,
            "role": Role::Player
        })
    })
}

async fn broadcast_room_state(state: &AppState, room_code: &str) {
    let client_ids = state
        .clients
        .lock()
        .await
        .iter()
        .filter_map(|(id, client)| {
            if client.room_code == room_code {
                Some(id.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    for id in client_ids {
        let snapshot = build_state_snapshot(state, &id).await;
        let _ = send_to_client(state, &id, json!({"event": "state", "payload": snapshot})).await;
    }
}

async fn send_to_client(state: &AppState, client_id: &str, payload: Value) -> bool {
    let msg = Message::Text(payload.to_string());
    let clients = state.clients.lock().await;
    if let Some(client) = clients.get(client_id) {
        client.tx.send(msg).is_ok()
    } else {
        false
    }
}

async fn broadcast_room_json(state: &AppState, room_code: &str, payload: Value) {
    let client_ids = state
        .clients
        .lock()
        .await
        .iter()
        .filter_map(|(id, client)| {
            if client.room_code == room_code {
                Some(id.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    for id in client_ids {
        let _ = send_to_client(state, &id, payload.clone()).await;
    }
}

fn load_file_question_banks(runtime_root: &FsPath) -> HashMap<String, Vec<Question>> {
    let dir = runtime_root.join("assets/questions");
    if !dir.exists() {
        return HashMap::new();
    }

    let mut files: Vec<PathBuf> = match fs::read_dir(&dir) {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|p| p.extension().map(|ext| ext == "json").unwrap_or(false))
            .collect(),
        Err(_) => return HashMap::new(),
    };

    files.sort();

    let mut bank_map: HashMap<String, Vec<Question>> = HashMap::new();
    for file in files {
        let file_name = file
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown.json".to_string());
        let raw = match fs::read_to_string(&file) {
            Ok(raw) => raw,
            Err(err) => {
                tracing::warn!("Could not read question file {}: {}", file.display(), err);
                continue;
            }
        };

        let parsed = match serde_json::from_str::<QuestionPackFile>(&raw) {
            Ok(parsed) => parsed,
            Err(err) => {
                tracing::warn!("Invalid JSON in {}: {}", file.display(), err);
                continue;
            }
        };

        let (pack_category, questions) = match parsed {
            QuestionPackFile::Pack(pack) => (pack.category, pack.questions),
            QuestionPackFile::Questions(questions) => (None, questions),
        };

        let pack_category = pack_category
            .filter(|category| !category.trim().is_empty())
            .unwrap_or_else(default_question_category);

        for mut question in questions {
            question.id = Uuid::new_v4().to_string();
            question.category = pack_category.clone();
            if question.options.len() != 4 || question.correct_index > 3 || question.points == 0 {
                tracing::warn!(
                    "Skipping invalid question '{}' from {}",
                    question.prompt,
                    file.display()
                );
                continue;
            }
            bank_map.entry(file_name.clone()).or_default().push(question);
        }
    }

    if !bank_map.is_empty() {
        let count: usize = bank_map.values().map(|v| v.len()).sum();
        tracing::info!(
            "Loaded {} questions across {} files from assets/questions/*.json",
            count,
            bank_map.len()
        );
    }
    bank_map
}

fn new_question_id() -> String {
    Uuid::new_v4().to_string()
}

fn default_question_category() -> String {
    "Generic".to_string()
}

fn append_history(data_dir: &PathBuf, entry: HistoryEntry) {
    let path = data_dir.join("history.json");
    let mut history: Vec<HistoryEntry> = if let Ok(raw) = fs::read_to_string(&path) {
        serde_json::from_str(&raw).unwrap_or_default()
    } else {
        Vec::new()
    };
    history.push(entry);
    if let Ok(serialized) = serde_json::to_string_pretty(&history) {
        let _ = fs::write(path, serialized);
    }
}

fn runtime_root_dir() -> PathBuf {
    fn has_runtime_layout(dir: &FsPath) -> bool {
        dir.join("web").exists() && dir.join("assets").exists()
    }

    fn first_matching_ancestor(start: &FsPath) -> Option<PathBuf> {
        for dir in start.ancestors() {
            if has_runtime_layout(dir) {
                return Some(dir.to_path_buf());
            }
        }
        None
    }

    if let Ok(cwd) = env::current_dir() {
        if let Some(found) = first_matching_ancestor(&cwd) {
            return found;
        }
    }

    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            if let Some(found) = first_matching_ancestor(parent) {
                return found;
            }
            return parent.to_path_buf();
        }
    }

    PathBuf::from(".")
}

fn read_web_html(runtime_root: &FsPath, role: &str) -> String {
    let path = runtime_root.join("web").join(role).join(format!("{}.html", role));
    fs::read_to_string(path).unwrap_or_else(|_| format!("Missing {}.html", role))
}

fn maybe_open_home_browser(port: u16) {
    if env::var("QUIZTER_OPEN_BROWSER")
        .map(|v| v == "0" || v.eq_ignore_ascii_case("false"))
        .unwrap_or(false)
    {
        return;
    }
    let url = format!("http://127.0.0.1:{}/", port);
    let _ = webbrowser::open(&url);
}

fn maybe_relaunch_in_terminal() -> bool {
    use std::io::IsTerminal;

    if env::var("QUIZTER_SPAWN_TERMINAL")
        .map(|v| v == "0" || v.eq_ignore_ascii_case("false"))
        .unwrap_or(false)
    {
        return false;
    }

    if env::var("QUIZTER_TERMINAL_LAUNCHED")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return false;
    }

    if std::io::stdout().is_terminal() || std::io::stderr().is_terminal() {
        return false;
    }

    let exe = match env::current_exe() {
        Ok(path) => path,
        Err(_) => return false,
    };
    let cwd = env::current_dir().ok();

    if spawn_terminal_process(&exe, cwd.as_deref()) {
        return true;
    }

    false
}

fn spawn_terminal_process(exe: &FsPath, cwd: Option<&FsPath>) -> bool {
    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C")
            .arg("start")
            .arg("Quizter Server")
            .arg(exe);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        cmd.env("QUIZTER_TERMINAL_LAUNCHED", "1");
        return cmd.spawn().is_ok();
    }

    #[cfg(target_os = "macos")]
    {
        let escaped_exe = shell_escape(exe);
        let escaped_cwd = cwd.map(shell_escape).unwrap_or_else(|| ".".to_string());
        let script = format!(
            "tell application \"Terminal\" to do script \"cd {} && QUIZTER_TERMINAL_LAUNCHED=1 {}\"",
            escaped_cwd, escaped_exe
        );
        return Command::new("osascript").arg("-e").arg(script).spawn().is_ok();
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let terminal_attempts: [(&str, &[&str]); 5] = [
            ("x-terminal-emulator", &["-e"]),
            ("gnome-terminal", &["--"]),
            ("konsole", &["-e"]),
            ("xfce4-terminal", &["--command"]),
            ("xterm", &["-e"]),
        ];

        for (program, prefix) in terminal_attempts {
            let mut cmd = Command::new(program);
            if let Some(dir) = cwd {
                cmd.current_dir(dir);
            }
            cmd.env("QUIZTER_TERMINAL_LAUNCHED", "1");
            for part in prefix {
                cmd.arg(part);
            }
            if program == "xfce4-terminal" {
                let launch = format!("QUIZTER_TERMINAL_LAUNCHED=1 {}", shell_escape(exe));
                cmd.arg(launch);
            } else {
                cmd.arg(exe);
            }
            if cmd.spawn().is_ok() {
                return true;
            }
        }
    }

    false
}

#[cfg(any(target_os = "macos", all(unix, not(target_os = "macos"))))]
fn shell_escape(path: &FsPath) -> String {
    let text = path.to_string_lossy().replace('\'', "'\"'\"'");
    format!("'{}'", text)
}

fn detect_lan_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let local = socket.local_addr().ok()?;
    Some(local.ip().to_string())
}

fn calculate_correct_score(
    points: u32,
    elapsed_secs: f64,
    total_secs: f64,
    doubled: bool,
    speed_bonus_enabled: bool,
) -> f64 {
    let speed_factor = ((total_secs - elapsed_secs) / total_secs).clamp(0.0, 1.0);
    let speed_bonus = if speed_bonus_enabled {
        points as f64 * 0.5 * speed_factor
    } else {
        0.0
    };
    let mut score = points as f64 + speed_bonus;
    if doubled {
        score *= 2.0;
    }
    score
}

#[cfg(test)]
mod tests {
    use super::{
        calculate_correct_score, eligible_from_round_for_new_player, host_label_from_base_url,
        mark_game_ended, normalize_player_name, normalize_room_title,
        normalized_public_base_url, player_can_participate_in_current_round,
        prepare_game_for_start, remove_room_and_clients, room_from_template,
        should_show_player_leaderboard, validate_owner_room_access, AppState,
        ClientConnection, GameState, GameStatus, PlayerState, RoundState,
    };
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tokio::sync::{mpsc, Mutex};

    fn build_test_state_with_hosted_room(room_code: &str, owner_token: &str) -> AppState {
        let template = GameState::new(Vec::new(), HashMap::new(), HashSet::new());
        let hosted_room = room_from_template(
            &template,
            room_code.to_string(),
            "Hosted Test Room".to_string(),
            owner_token.to_string(),
        );

        let mut rooms = HashMap::new();
        rooms.insert(room_code.to_string(), hosted_room);

        let mut owner_index = HashMap::new();
        owner_index.insert(owner_token.to_string(), room_code.to_string());

        AppState {
            rooms: Arc::new(Mutex::new(rooms)),
            owner_index: Arc::new(Mutex::new(owner_index)),
            clients: Arc::new(Mutex::new(HashMap::new())),
            data_dir: Arc::new(PathBuf::from(".")),
            public_base_url: Arc::new("http://127.0.0.1:8080".to_string()),
            player_join_url: Arc::new("http://127.0.0.1:8080/player".to_string()),
            host_ip: Arc::new("127.0.0.1".to_string()),
            port: 8080,
            runtime_root: Arc::new(PathBuf::from(".")),
        }
    }

    #[test]
    fn score_is_max_at_zero_elapsed() {
        let score = calculate_correct_score(100, 0.0, 15.0, false, true);
        assert!((score - 150.0).abs() < 0.0001);
    }

    #[test]
    fn score_is_base_at_timeout_boundary() {
        let score = calculate_correct_score(100, 15.0, 15.0, false, true);
        assert!((score - 100.0).abs() < 0.0001);
    }

    #[test]
    fn score_doubles_when_double_downer_is_active() {
        let score = calculate_correct_score(100, 3.0, 15.0, true, true);
        assert!(score > 200.0);
    }

    #[test]
    fn score_has_no_speed_bonus_when_disabled() {
        let score = calculate_correct_score(100, 0.0, 15.0, false, false);
        assert!((score - 100.0).abs() < 0.0001);
    }

    #[test]
    fn room_title_is_trimmed_and_validated() {
        assert_eq!(normalize_room_title("  Friday Night  ").unwrap(), "Friday Night");
        assert_eq!(normalize_room_title("   ").unwrap_err(), "room_title_required");
        assert_eq!(
            normalize_room_title(&"X".repeat(81)).unwrap_err(),
            "room_title_too_long"
        );
    }

    #[test]
    fn player_name_is_trimmed_and_validated() {
        assert_eq!(normalize_player_name("  Alice  ").unwrap(), "Alice");
        assert_eq!(normalize_player_name("   ").unwrap_err(), "display_name_required");
        assert_eq!(
            normalize_player_name(&"Y".repeat(33)).unwrap_err(),
            "display_name_too_long"
        );
    }

    #[test]
    fn new_players_join_next_round_if_a_round_is_active() {
        let mut game = GameState::new(Vec::new(), HashMap::new(), HashSet::new());
        game.completed_rounds = 2;
        game.current_round = Some(RoundState {
            round_number: 3,
            question: super::Question {
                id: "q-1".to_string(),
                category: "Test".to_string(),
                prompt: "Prompt".to_string(),
                options: vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()],
                correct_index: 0,
                points: 100,
                image_url: None,
            },
            started_at: Instant::now(),
            deadline: Instant::now() + Duration::from_secs(15),
            answer_window_secs: 15,
            answers: HashMap::new(),
            speed_searcher_owner: None,
            great_gambler_factor: None,
            double_downers: HashSet::new(),
            clone_commanders: HashSet::new(),
            super_spliter_targets: HashMap::new(),
            mix_master_owner: None,
            option_order: vec![0, 1, 2, 3],
        });

        assert_eq!(eligible_from_round_for_new_player(&game), 4);
    }

    #[test]
    fn late_joiner_cannot_participate_in_current_round() {
        let round = RoundState {
            round_number: 2,
            question: super::Question {
                id: "q-1".to_string(),
                category: "Test".to_string(),
                prompt: "Prompt".to_string(),
                options: vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()],
                correct_index: 0,
                points: 100,
                image_url: None,
            },
            started_at: Instant::now(),
            deadline: Instant::now() + Duration::from_secs(15),
            answer_window_secs: 15,
            answers: HashMap::new(),
            speed_searcher_owner: None,
            great_gambler_factor: None,
            double_downers: HashSet::new(),
            clone_commanders: HashSet::new(),
            super_spliter_targets: HashMap::new(),
            mix_master_owner: None,
            option_order: vec![0, 1, 2, 3],
        };
        let player = PlayerState {
            id: "player-1".to_string(),
            name: "Late".to_string(),
            score: 0.0,
            last_score_delta: 0.0,
            connected: true,
            eligible_from_round: 3,
            used_powerups: HashSet::new(),
            pending_powerup: None,
            tutorial_seen: false,
        };

        assert!(!player_can_participate_in_current_round(Some(&player), Some(&round)));
    }

    #[test]
    fn player_leaderboard_is_hidden_between_games() {
        assert!(!should_show_player_leaderboard(&GameStatus::Lobby, 0));
        assert!(should_show_player_leaderboard(&GameStatus::InRound, 0));
        assert!(should_show_player_leaderboard(&GameStatus::RoundResult, 1));
        assert!(should_show_player_leaderboard(&GameStatus::Ended, 1));
        assert!(!should_show_player_leaderboard(&GameStatus::Ended, 0));
    }

    #[test]
    fn public_base_url_defaults_to_lan_when_not_configured() {
        assert_eq!(
            normalized_public_base_url(None, 8080, "192.168.1.10"),
            "http://192.168.1.10:8080"
        );
    }

    #[test]
    fn public_base_url_uses_configured_origin_without_trailing_slash() {
        assert_eq!(
            normalized_public_base_url(Some("https://quizter.example.com/".to_string()), 8080, "192.168.1.10"),
            "https://quizter.example.com"
        );
    }

    #[test]
    fn host_label_is_extracted_from_public_base_url() {
        assert_eq!(
            host_label_from_base_url("https://quizter.example.com/path"),
            "quizter.example.com"
        );
        assert_eq!(host_label_from_base_url("http://127.0.0.1:8080"), "127.0.0.1:8080");
    }

    #[test]
    fn prepare_game_for_start_resets_scores_and_clamps_rounds() {
        let mut game = GameState::new(Vec::new(), HashMap::new(), HashSet::new());
        game.questions = vec![
            super::Question {
                id: "q-1".to_string(),
                category: "Test".to_string(),
                prompt: "One".to_string(),
                options: vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()],
                correct_index: 0,
                points: 100,
                image_url: None,
            },
            super::Question {
                id: "q-2".to_string(),
                category: "Test".to_string(),
                prompt: "Two".to_string(),
                options: vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()],
                correct_index: 1,
                points: 100,
                image_url: None,
            },
        ];
        game.players.insert(
            "player-1".to_string(),
            PlayerState {
                id: "player-1".to_string(),
                name: "Alice".to_string(),
                score: 250.0,
                last_score_delta: 50.0,
                connected: true,
                eligible_from_round: 1,
                used_powerups: HashSet::from([super::PowerUp::MixMaster]),
                pending_powerup: Some(super::PowerUp::DoubleDowner),
                tutorial_seen: true,
            },
        );

        prepare_game_for_start(&mut game, 99).unwrap();

        assert_eq!(game.total_rounds, 2);
        assert_eq!(game.completed_rounds, 0);
        assert_eq!(game.status, GameStatus::Lobby);
        assert!(game.current_round.is_none());
        assert_eq!(game.shuffled_question_ids.len(), 2);
        let player = game.players.get("player-1").unwrap();
        assert_eq!(player.score, 0.0);
        assert_eq!(player.last_score_delta, 0.0);
        assert!(player.used_powerups.is_empty());
        assert!(player.pending_powerup.is_none());
    }

    #[test]
    fn prepare_game_for_start_rejects_empty_question_pool() {
        let mut game = GameState::new(Vec::new(), HashMap::new(), HashSet::new());
        assert_eq!(prepare_game_for_start(&mut game, 3), Err("no_questions"));
    }

    #[test]
    fn mark_game_ended_only_transitions_once() {
        let mut game = GameState::new(Vec::new(), HashMap::new(), HashSet::new());
        game.status = GameStatus::InRound;
        game.current_round = Some(RoundState {
            round_number: 1,
            question: super::Question {
                id: "q-1".to_string(),
                category: "Test".to_string(),
                prompt: "Prompt".to_string(),
                options: vec!["A".to_string(), "B".to_string(), "C".to_string(), "D".to_string()],
                correct_index: 0,
                points: 100,
                image_url: None,
            },
            started_at: Instant::now(),
            deadline: Instant::now() + Duration::from_secs(15),
            answer_window_secs: 15,
            answers: HashMap::new(),
            speed_searcher_owner: None,
            great_gambler_factor: None,
            double_downers: HashSet::new(),
            clone_commanders: HashSet::new(),
            super_spliter_targets: HashMap::new(),
            mix_master_owner: None,
            option_order: vec![0, 1, 2, 3],
        });

        assert!(mark_game_ended(&mut game));
        assert_eq!(game.status, GameStatus::Ended);
        assert!(game.current_round.is_none());
        assert!(!mark_game_ended(&mut game));
    }

    #[tokio::test]
    async fn owner_token_must_match_the_room_it_controls() {
        let state = build_test_state_with_hosted_room("AB12", "owner-123");

        assert_eq!(
            validate_owner_room_access(&state, "AB12", "owner-123").await,
            Some("AB12".to_string())
        );
        assert_eq!(
            validate_owner_room_access(&state, "ZZ99", "owner-123").await,
            None
        );
        assert_eq!(
            validate_owner_room_access(&state, "AB12", "wrong-token").await,
            None
        );
    }

    #[tokio::test]
    async fn removing_a_hosted_room_cleans_up_owner_token_and_clients() {
        let state = build_test_state_with_hosted_room("AB12", "owner-123");
        let (tx, _rx) = mpsc::unbounded_channel();
        state.clients.lock().await.insert(
            "player-1".to_string(),
            ClientConnection {
                room_code: "AB12".to_string(),
                tx,
            },
        );

        let removed_title =
            remove_room_and_clients(&state, "AB12", "owner-123", "room_closed").await;

        assert_eq!(removed_title, Some("Hosted Test Room".to_string()));
        assert!(state.rooms.lock().await.get("AB12").is_none());
        assert!(state.owner_index.lock().await.get("owner-123").is_none());
        assert!(state.clients.lock().await.get("player-1").is_none());
        assert!(state.rooms.lock().await.is_empty());
    }
}
