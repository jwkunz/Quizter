use axum::{
    extract::{ws::Message, Path, State, WebSocketUpgrade},
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
    fs,
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

const DEFAULT_ADMIN_PASSCODE: &str = "quiztik-admin";
const DEFAULT_ROOM_CODE: &str = "QUIZTIK";

#[derive(Clone)]
struct AppState {
    game: Arc<Mutex<GameState>>,
    clients: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<Message>>>>,
    data_dir: Arc<PathBuf>,
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
    id: String,
    prompt: String,
    options: Vec<String>,
    correct_index: usize,
    points: u32,
    image_url: Option<String>,
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
}

#[derive(Debug, Clone)]
struct PlayerState {
    id: String,
    name: String,
    score: f64,
    connected: bool,
    used_powerups: HashSet<PowerUp>,
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
}

#[derive(Debug)]
struct GameState {
    room_code: String,
    admin_passcode: String,
    admin_id: Option<String>,
    status: GameStatus,
    players: HashMap<String, PlayerState>,
    questions: Vec<Question>,
    shuffled_question_ids: Vec<String>,
    total_rounds: usize,
    current_round: Option<RoundState>,
    completed_rounds: usize,
}

impl GameState {
    fn new(questions: Vec<Question>) -> Self {
        Self {
            room_code: DEFAULT_ROOM_CODE.to_string(),
            admin_passcode: DEFAULT_ADMIN_PASSCODE.to_string(),
            admin_id: None,
            status: GameStatus::Lobby,
            players: HashMap::new(),
            questions,
            shuffled_question_ids: Vec::new(),
            total_rounds: 10,
            current_round: None,
            completed_rounds: 0,
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

#[derive(Deserialize)]
struct CreateRoomRequest {
    admin_passcode: Option<String>,
    total_rounds: Option<usize>,
}

#[derive(Deserialize)]
struct JoinRequest {
    room_code: String,
    display_name: String,
}

#[derive(Deserialize)]
struct AdminLoginRequest {
    room_code: String,
    admin_passcode: String,
}

#[derive(Deserialize)]
struct AddQuestionRequest {
    admin_id: String,
    prompt: String,
    options: Vec<String>,
    correct_index: usize,
    points: u32,
    image_url: Option<String>,
}

#[derive(Deserialize)]
struct ImportQuestionsRequest {
    admin_id: String,
    questions: Vec<Question>,
}

#[derive(Deserialize)]
struct StartGameRequest {
    admin_id: String,
    total_rounds: usize,
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let data_dir = PathBuf::from("data");
    let _ = fs::create_dir_all(&data_dir);
    let questions = load_questions(&data_dir);

    let state = AppState {
        game: Arc::new(Mutex::new(GameState::new(questions))),
        clients: Arc::new(Mutex::new(HashMap::new())),
        data_dir: Arc::new(data_dir),
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/player", get(player_page))
        .route("/admin", get(admin_page))
        .route("/api/admin/create_room", post(create_room))
        .route("/api/admin/login", post(admin_login))
        .route("/api/join", post(join_room))
        .route("/api/admin/questions/add", post(add_question))
        .route("/api/admin/questions/import", post(import_questions))
        .route("/api/admin/start", post(start_game))
        .route("/api/state/:client_id", get(get_state))
        .route("/ws/:client_id", get(ws_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Quiztik server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind listener");
    axum::serve(listener, app)
        .await
        .expect("server execution failed");
}

async fn root() -> Json<Value> {
    Json(json!({"service": "quiztik-server", "version": env!("CARGO_PKG_VERSION")}))
}

async fn player_page() -> Html<String> {
    Html(fs::read_to_string("web/player/player.html").unwrap_or_else(|_| "Missing player.html".to_string()))
}

async fn admin_page() -> Html<String> {
    Html(fs::read_to_string("web/admin/admin.html").unwrap_or_else(|_| "Missing admin.html".to_string()))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "quiztik-server",
        timestamp: Utc::now().to_rfc3339(),
    })
}

async fn create_room(
    State(state): State<AppState>,
    Json(req): Json<CreateRoomRequest>,
) -> Json<Value> {
    let mut game = state.game.lock().await;
    game.room_code = DEFAULT_ROOM_CODE.to_string();
    if let Some(pass) = req.admin_passcode {
        if !pass.trim().is_empty() {
            game.admin_passcode = pass;
        }
    }
    if let Some(rounds) = req.total_rounds {
        game.total_rounds = rounds.max(1).min(game.questions.len().max(1));
    }

    Json(json!({
        "room_code": game.room_code,
        "total_rounds": game.total_rounds,
        "questions_available": game.questions.len()
    }))
}

async fn admin_login(
    State(state): State<AppState>,
    Json(req): Json<AdminLoginRequest>,
) -> impl IntoResponse {
    let mut game = state.game.lock().await;
    if req.room_code != game.room_code || req.admin_passcode != game.admin_passcode {
        return (axum::http::StatusCode::UNAUTHORIZED, Json(json!({"error": "invalid_credentials"})));
    }
    let admin_id = format!("admin-{}", Uuid::new_v4());
    game.admin_id = Some(admin_id.clone());
    drop(game);

    broadcast_state(&state).await;
    (axum::http::StatusCode::OK, Json(json!({"admin_id": admin_id})))
}

async fn join_room(State(state): State<AppState>, Json(req): Json<JoinRequest>) -> impl IntoResponse {
    let mut game = state.game.lock().await;
    if req.room_code != game.room_code {
        return (axum::http::StatusCode::BAD_REQUEST, Json(json!({"error": "invalid_room_code"})));
    }

    let existing = game
        .players
        .values_mut()
        .find(|p| p.name.eq_ignore_ascii_case(req.display_name.trim()));

    let player_id = if let Some(player) = existing {
        player.connected = true;
        player.id.clone()
    } else {
        let id = format!("player-{}", Uuid::new_v4());
        game.players.insert(
            id.clone(),
            PlayerState {
                id: id.clone(),
                name: req.display_name.trim().to_string(),
                score: 0.0,
                connected: true,
                used_powerups: HashSet::new(),
                tutorial_seen: false,
            },
        );
        id
    };

    drop(game);
    broadcast_state(&state).await;
    (axum::http::StatusCode::OK, Json(json!({"player_id": player_id})))
}

async fn add_question(
    State(state): State<AppState>,
    Json(req): Json<AddQuestionRequest>,
) -> impl IntoResponse {
    if !is_admin(&state, &req.admin_id).await {
        return (axum::http::StatusCode::UNAUTHORIZED, Json(json!({"error": "admin_required"})));
    }
    if req.options.len() != 4 || req.correct_index > 3 || req.points == 0 {
        return (axum::http::StatusCode::BAD_REQUEST, Json(json!({"error": "invalid_question"})));
    }

    let question = Question {
        id: Uuid::new_v4().to_string(),
        prompt: req.prompt,
        options: req.options,
        correct_index: req.correct_index,
        points: req.points,
        image_url: req.image_url,
    };

    {
        let mut game = state.game.lock().await;
        game.questions.push(question.clone());
        save_questions(&state.data_dir, &game.questions);
    }

    broadcast_state(&state).await;
    (axum::http::StatusCode::OK, Json(json!({"ok": true, "question": question})))
}

async fn import_questions(
    State(state): State<AppState>,
    Json(req): Json<ImportQuestionsRequest>,
) -> impl IntoResponse {
    if !is_admin(&state, &req.admin_id).await {
        return (axum::http::StatusCode::UNAUTHORIZED, Json(json!({"error": "admin_required"})));
    }

    if req.questions.is_empty() {
        return (axum::http::StatusCode::BAD_REQUEST, Json(json!({"error": "no_questions"})));
    }

    for q in &req.questions {
        if q.options.len() != 4 || q.correct_index > 3 || q.points == 0 {
            return (axum::http::StatusCode::BAD_REQUEST, Json(json!({"error": "invalid_question_in_pack"})));
        }
    }

    {
        let mut game = state.game.lock().await;
        game.questions = req
            .questions
            .into_iter()
            .map(|mut q| {
                if q.id.is_empty() {
                    q.id = Uuid::new_v4().to_string();
                }
                q
            })
            .collect();
        save_questions(&state.data_dir, &game.questions);
    }

    broadcast_state(&state).await;
    (axum::http::StatusCode::OK, Json(json!({"ok": true})))
}

async fn start_game(
    State(state): State<AppState>,
    Json(req): Json<StartGameRequest>,
) -> impl IntoResponse {
    if !is_admin(&state, &req.admin_id).await {
        return (axum::http::StatusCode::UNAUTHORIZED, Json(json!({"error": "admin_required"})));
    }

    {
        let mut game = state.game.lock().await;
        if game.questions.is_empty() {
            return (axum::http::StatusCode::BAD_REQUEST, Json(json!({"error": "no_questions"})));
        }

        game.total_rounds = req.total_rounds.max(1).min(game.questions.len());
        game.completed_rounds = 0;
        game.status = GameStatus::Lobby;
        game.current_round = None;
        for player in game.players.values_mut() {
            player.score = 0.0;
            player.used_powerups.clear();
        }

        game.shuffled_question_ids = game.questions.iter().map(|q| q.id.clone()).collect();
        game.shuffled_question_ids.shuffle(&mut rand::thread_rng());
    }

    start_next_round(state.clone()).await;
    (axum::http::StatusCode::OK, Json(json!({"ok": true})))
}

async fn get_state(State(state): State<AppState>, Path(client_id): Path<String>) -> Json<Value> {
    Json(build_state_snapshot(&state, &client_id).await)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(client_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, client_id))
}

async fn handle_socket(stream: axum::extract::ws::WebSocket, state: AppState, client_id: String) {
    let (mut sender, mut receiver) = stream.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    state.clients.lock().await.insert(client_id.clone(), tx);

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
        let mut game = state.game.lock().await;
        if let Some(player) = game.players.get_mut(&client_id) {
            player.connected = false;
        }
    }
    broadcast_state(&state).await;
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
                let mut game = state.game.lock().await;
                if let Some(player) = game.players.get_mut(client_id) {
                    player.tutorial_seen = true;
                }
                drop(game);
                broadcast_state(state).await;
            }
        }
        "admin_next_round" => {
            if is_admin(state, client_id).await {
                start_next_round(state.clone()).await;
            }
        }
        _ => {}
    }
}

async fn submit_answer(state: &AppState, client_id: &str, choice_index: usize) {
    let mut should_finalize = false;

    {
        let mut game = state.game.lock().await;
        if game.status != GameStatus::InRound {
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

    broadcast_state(state).await;
    if should_finalize {
        finalize_round(state.clone()).await;
    }
}

async fn activate_powerup(state: &AppState, client_id: &str, powerup: PowerUp) {
    let (notify, powerup_payload);

    {
        let mut game = state.game.lock().await;
        if game.status != GameStatus::InRound {
            return;
        }

        let player = match game.players.get_mut(client_id) {
            Some(p) => p,
            None => return,
        };

        if player.used_powerups.contains(&powerup) {
            return;
        }

        player.used_powerups.insert(powerup.clone());

        let round = match game.current_round.as_mut() {
            Some(r) => r,
            None => return,
        };

        match powerup {
            PowerUp::MixMaster => {
                round.mix_master_owner = Some(client_id.to_string());
                powerup_payload = Some(json!({"active": true}));
            }
            PowerUp::SpeedSearcher => {
                round.speed_searcher_owner = Some(client_id.to_string());
                round.answer_window_secs = 30;
                round.started_at = Instant::now();
                round.deadline = round.started_at + Duration::from_secs(30);
                powerup_payload = Some(json!({"owner": client_id, "seconds": 30}));
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
                round
                    .super_spliter_targets
                    .insert(client_id.to_string(), (round.question.correct_index, random_incorrect));
                powerup_payload = Some(json!({"target": client_id}));
            }
            PowerUp::GreatGambler => {
                if round.great_gambler_factor.is_none() {
                    let mut rng = rand::thread_rng();
                    let factor = rng.gen_range(-1.0f64..=3.0f64);
                    round.great_gambler_factor = Some(factor);
                }
                powerup_payload = Some(json!({"factor": round.great_gambler_factor}));
            }
        }

        notify = true;
    }

    if notify {
        let message = json!({
            "event": "powerup_activated",
            "payload": {
                "player_id": client_id,
                "powerup": powerup,
                "details": powerup_payload
            }
        });
        broadcast_json(state, message).await;
    }

    broadcast_state(state).await;
}

async fn start_next_round(state: AppState) {
    let mut round_started = false;
    {
        let mut game = state.game.lock().await;
        if game.completed_rounds >= game.total_rounds || game.questions.is_empty() {
            game.status = GameStatus::Ended;
            game.current_round = None;
            let history = HistoryEntry {
                finished_at: Utc::now().to_rfc3339(),
                rounds_played: game.completed_rounds,
                leaderboard: game.leaderboard(),
            };
            append_history(&state.data_dir, history);
            drop(game);
            broadcast_state(&state).await;
            return;
        }

        let next_id = game.shuffled_question_ids.get(game.completed_rounds).cloned();
        if let Some(question_id) = next_id {
            if let Some(question) = game.questions.iter().find(|q| q.id == question_id).cloned() {
                let started_at = Instant::now();
                let deadline = started_at + Duration::from_secs(15);
                game.current_round = Some(RoundState {
                    round_number: game.completed_rounds + 1,
                    question,
                    started_at,
                    deadline,
                    answer_window_secs: 15,
                    answers: HashMap::new(),
                    speed_searcher_owner: None,
                    great_gambler_factor: None,
                    double_downers: HashSet::new(),
                    clone_commanders: HashSet::new(),
                    super_spliter_targets: HashMap::new(),
                    mix_master_owner: None,
                });
                game.status = GameStatus::InRound;
                round_started = true;
            }
        }
    }

    if round_started {
        broadcast_state(&state).await;
        spawn_round_timer(state).await;
    }
}

async fn spawn_round_timer(state: AppState) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let (status, remaining) = {
                let game = state.game.lock().await;
                if game.status != GameStatus::InRound {
                    break;
                }
                if let Some(round) = &game.current_round {
                    let now = Instant::now();
                    let rem = if round.deadline > now {
                        (round.deadline - now).as_secs()
                    } else {
                        0
                    };
                    (game.status.clone(), rem)
                } else {
                    break;
                }
            };

            if status == GameStatus::InRound {
                broadcast_json(&state, json!({"event": "timer_tick", "payload": {"seconds_left": remaining}})).await;
            }

            if remaining == 0 {
                finalize_round(state.clone()).await;
                break;
            }
        }
    });
}

async fn finalize_round(state: AppState) {
    let result_payload;
    let end_game;

    {
        let mut game = state.game.lock().await;
        if game.status != GameStatus::InRound {
            return;
        }

        let round = match game.current_round.take() {
            Some(r) => r,
            None => return,
        };

        let mut round_scores: HashMap<String, f64> = HashMap::new();
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

        result_payload = json!({
            "round_number": round.round_number,
            "correct_index": round.question.correct_index,
            "question_id": round.question.id,
            "scores": details,
            "leaderboard": game.leaderboard(),
            "great_gambler_factor": round.great_gambler_factor,
        });

        end_game = game.completed_rounds >= game.total_rounds;
    }

    broadcast_json(&state, json!({"event": "round_result", "payload": result_payload})).await;
    broadcast_state(&state).await;

    if end_game {
        let mut game = state.game.lock().await;
        game.status = GameStatus::Ended;
        let history = HistoryEntry {
            finished_at: Utc::now().to_rfc3339(),
            rounds_played: game.completed_rounds,
            leaderboard: game.leaderboard(),
        };
        append_history(&state.data_dir, history);
        drop(game);
        broadcast_state(&state).await;
    }
}

async fn build_state_snapshot(state: &AppState, client_id: &str) -> Value {
    let game = state.game.lock().await;
    let mut visible_question = None;

    if let Some(round) = &game.current_round {
        let mut options: Vec<Value> = round
            .question
            .options
            .iter()
            .enumerate()
            .map(|(idx, text)| json!({"index": idx, "text": text}))
            .collect();

        if let Some((correct, incorrect)) = round.super_spliter_targets.get(client_id) {
            options.retain(|o| {
                let idx = o.get("index").and_then(|v| v.as_u64()).unwrap_or(99) as usize;
                idx == *correct || idx == *incorrect
            });
        }

        visible_question = Some(json!({
            "id": round.question.id,
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
    }

    let role = if game.admin_id.as_deref() == Some(client_id) {
        Role::Admin
    } else {
        Role::Player
    };

    let your_state = game.players.get(client_id).map(|p| {
        json!({
            "id": p.id,
            "name": p.name,
            "score": (p.score * 100.0).round() / 100.0,
            "used_powerups": p.used_powerups,
            "tutorial_seen": p.tutorial_seen,
        })
    });

    json!({
        "status": game.status,
        "room_code": game.room_code,
        "role": role,
        "total_rounds": game.total_rounds,
        "completed_rounds": game.completed_rounds,
        "questions_available": game.questions.len(),
        "leaderboard": game.leaderboard(),
        "current_question": visible_question,
        "you": your_state,
    })
}

async fn broadcast_state(state: &AppState) {
    let client_ids = state.clients.lock().await.keys().cloned().collect::<Vec<_>>();
    for id in client_ids {
        let snapshot = build_state_snapshot(state, &id).await;
        let _ = send_to_client(state, &id, json!({"event": "state", "payload": snapshot})).await;
    }
}

async fn send_to_client(state: &AppState, client_id: &str, payload: Value) -> bool {
    let msg = Message::Text(payload.to_string());
    let clients = state.clients.lock().await;
    if let Some(tx) = clients.get(client_id) {
        tx.send(msg).is_ok()
    } else {
        false
    }
}

async fn broadcast_json(state: &AppState, payload: Value) {
    let client_ids = state.clients.lock().await.keys().cloned().collect::<Vec<_>>();
    for id in client_ids {
        let _ = send_to_client(state, &id, payload.clone()).await;
    }
}

async fn is_admin(state: &AppState, client_id: &str) -> bool {
    let game = state.game.lock().await;
    game.admin_id.as_deref() == Some(client_id)
}

fn load_questions(data_dir: &PathBuf) -> Vec<Question> {
    let file = data_dir.join("questions.json");
    if let Ok(raw) = fs::read_to_string(&file) {
        if let Ok(questions) = serde_json::from_str::<Vec<Question>>(&raw) {
            if !questions.is_empty() {
                return questions;
            }
        }
    }

    let defaults = vec![
        Question {
            id: Uuid::new_v4().to_string(),
            prompt: "What language is Quiztik server written in?".to_string(),
            options: vec!["Go".to_string(), "Rust".to_string(), "Python".to_string(), "Java".to_string()],
            correct_index: 1,
            points: 100,
            image_url: None,
        },
        Question {
            id: Uuid::new_v4().to_string(),
            prompt: "How many seconds is the default answer timeout?".to_string(),
            options: vec!["10".to_string(), "15".to_string(), "20".to_string(), "30".to_string()],
            correct_index: 1,
            points: 100,
            image_url: None,
        },
    ];
    save_questions(data_dir, &defaults);
    defaults
}

fn save_questions(data_dir: &PathBuf, questions: &[Question]) {
    let _ = fs::create_dir_all(data_dir);
    let path = data_dir.join("questions.json");
    if let Ok(serialized) = serde_json::to_string_pretty(questions) {
        let _ = fs::write(path, serialized);
    }
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

fn calculate_correct_score(points: u32, elapsed_secs: f64, total_secs: f64, doubled: bool) -> f64 {
    let speed_factor = ((total_secs - elapsed_secs) / total_secs).clamp(0.0, 1.0);
    let speed_bonus = points as f64 * 0.5 * speed_factor;
    let mut score = points as f64 + speed_bonus;
    if doubled {
        score *= 2.0;
    }
    score
}

#[cfg(test)]
mod tests {
    use super::calculate_correct_score;

    #[test]
    fn score_is_max_at_zero_elapsed() {
        let score = calculate_correct_score(100, 0.0, 15.0, false);
        assert!((score - 150.0).abs() < 0.0001);
    }

    #[test]
    fn score_is_base_at_timeout_boundary() {
        let score = calculate_correct_score(100, 15.0, 15.0, false);
        assert!((score - 100.0).abs() < 0.0001);
    }

    #[test]
    fn score_doubles_when_double_downer_is_active() {
        let score = calculate_correct_score(100, 3.0, 15.0, true);
        assert!(score > 200.0);
    }
}
