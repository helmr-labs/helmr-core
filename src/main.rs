use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Clone)]
struct AgentState {
    state: String,
    mission: String,
}

#[derive(Clone)]
struct AppState {
    used_tokens: Arc<Mutex<HashSet<String>>>,
    tomb_registry: Arc<Mutex<HashSet<String>>>,
    mission_spend: Arc<Mutex<HashMap<String, u32>>>,
    mission_limits: Arc<Mutex<HashMap<String, u32>>>,
    agents: Arc<Mutex<HashMap<String, AgentState>>>,
    activity: Arc<Mutex<Vec<String>>>,
}

#[derive(Deserialize)]
struct AuthorizeRequest {
    agent_id: String,
    mission_id: String,
    action_type: String,
    target: String,
}

#[derive(Serialize)]
struct AuthorizeResponse {
    decision: String,
    reason: String,
    token: Option<String>,
}

#[derive(Deserialize)]
struct AirlockWriteRequest {
    agent_id: String,
    mission_id: String,
    token: String,
    target: String,
    content: String,
}

#[derive(Deserialize)]
struct MissionCreateRequest {
    mission_id: String,
    spend_limit: u32,
}

#[derive(Deserialize)]
struct TerminateRequest {
    agent_id: String,
}

#[derive(Serialize)]
struct DecisionResponse {
    decision: String,
    reason: String,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn bar(count: u32, limit: u32) -> String {
    let safe_limit = if limit == 0 { 1 } else { limit };
    let width: u32 = 20;
    let filled = (count * width) / safe_limit;

    let mut result = String::new();

    for i in 0..width {
        if i < filled {
            result.push('█');
        } else {
            result.push('░');
        }
    }

    result
}

async fn mission_create(
    State(state): State<AppState>,
    Json(req): Json<MissionCreateRequest>,
) -> Json<DecisionResponse> {
    let mut limits = state.mission_limits.lock().unwrap();
    limits.insert(req.mission_id.clone(), req.spend_limit);
    drop(limits);

    let mut spend = state.mission_spend.lock().unwrap();
    spend.insert(req.mission_id.clone(), 0);
    drop(spend);

    let mut activity = state.activity.lock().unwrap();
    activity.push(format!("{} operator {} SUCCESS", now(), req.mission_id));

    Json(DecisionResponse {
        decision: "SUCCESS".into(),
        reason: "MISSION_CREATED".into(),
    })
}

async fn authorize(
    State(state): State<AppState>,
    Json(req): Json<AuthorizeRequest>,
) -> Json<AuthorizeResponse> {
    let tomb = state.tomb_registry.lock().unwrap();
    if tomb.contains(&req.agent_id) {
        return Json(AuthorizeResponse {
            decision: "BLOCKED".into(),
            reason: "AGENT_TERMINATED".into(),
            token: None,
        });
    }
    drop(tomb);

    let limits = state.mission_limits.lock().unwrap();
    if !limits.contains_key(&req.mission_id) {
        return Json(AuthorizeResponse {
            decision: "BLOCKED".into(),
            reason: "UNKNOWN_MISSION".into(),
            token: None,
        });
    }

    let limit = *limits.get(&req.mission_id).unwrap();
    drop(limits);

    let mut spend = state.mission_spend.lock().unwrap();
    let count = spend.entry(req.mission_id.clone()).or_insert(0);

    if *count >= limit {
        return Json(AuthorizeResponse {
            decision: "BLOCKED".into(),
            reason: "MISSION_SPEND_LIMIT".into(),
            token: None,
        });
    }

    *count += 1;
    drop(spend);

    let token = Uuid::new_v4().to_string();

    let mut agents = state.agents.lock().unwrap();
    agents.insert(
        req.agent_id.clone(),
        AgentState {
            state: "AUTHORIZED".into(),
            mission: req.mission_id.clone(),
        },
    );
    drop(agents);

    let mut activity = state.activity.lock().unwrap();
    activity.push(format!("{} {} {} ALLOWED", now(), req.agent_id, req.mission_id));

    let _ = &req.action_type;
    let _ = &req.target;

    Json(AuthorizeResponse {
        decision: "ALLOWED".into(),
        reason: "TOKEN_ISSUED".into(),
        token: Some(token),
    })
}

async fn airlock_write(
    State(state): State<AppState>,
    Json(req): Json<AirlockWriteRequest>,
) -> Json<DecisionResponse> {
    let tomb = state.tomb_registry.lock().unwrap();
    if tomb.contains(&req.agent_id) {
        return Json(DecisionResponse {
            decision: "BLOCKED".into(),
            reason: "AGENT_TERMINATED".into(),
        });
    }
    drop(tomb);

    let mut used = state.used_tokens.lock().unwrap();
    if used.contains(&req.token) {
        return Json(DecisionResponse {
            decision: "BLOCKED".into(),
            reason: "TOKEN_ALREADY_USED".into(),
        });
    }

    used.insert(req.token.clone());
    drop(used);

    let path = PathBuf::from(&req.target);

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let _ = fs::write(&path, req.content);

    let mut agents = state.agents.lock().unwrap();
    agents.insert(
        req.agent_id.clone(),
        AgentState {
            state: "EXECUTING".into(),
            mission: req.mission_id.clone(),
        },
    );
    drop(agents);

    let mut activity = state.activity.lock().unwrap();
    activity.push(format!("{} {} {} EXECUTED", now(), req.agent_id, req.mission_id));

    Json(DecisionResponse {
        decision: "ALLOWED".into(),
        reason: "TOKEN_EXECUTED".into(),
    })
}

async fn terminate(
    State(state): State<AppState>,
    Json(req): Json<TerminateRequest>,
) -> Json<DecisionResponse> {
    let mut tomb = state.tomb_registry.lock().unwrap();
    tomb.insert(req.agent_id.clone());
    drop(tomb);

    let mut agents = state.agents.lock().unwrap();
    agents.insert(
        req.agent_id.clone(),
        AgentState {
            state: "TERMINATED".into(),
            mission: "-".into(),
        },
    );
    drop(agents);

    let mut activity = state.activity.lock().unwrap();
    activity.push(format!("{} operator {} TERMINATED", now(), req.agent_id));

    Json(DecisionResponse {
        decision: "SUCCESS".into(),
        reason: "AGENT_TERMINATED".into(),
    })
}

async fn board(State(state): State<AppState>) -> Html<String> {
    let activity = state.activity.lock().unwrap();
    let missions = state.mission_spend.lock().unwrap();
    let limits = state.mission_limits.lock().unwrap();
    let agents = state.agents.lock().unwrap();

    let mut mission_rows = String::new();

    for (mission, count) in missions.iter() {
        if let Some(limit) = limits.get(mission) {
            let gauge = bar(*count, *limit);
            mission_rows.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}/{}</td></tr>",
                mission, gauge, count, limit
            ));
        }
    }

    let mut agent_rows = String::new();

    for (name, info) in agents.iter() {
        agent_rows.push_str(&format!(
            "<tr><td style='color:#ff4fd8'>{}</td><td>{}</td><td>{}</td></tr>",
            name, info.state, info.mission
        ));
    }

    let mut rows = String::new();

    for line in activity.iter().rev().take(50) {
        let parts: Vec<&str> = line.split(' ').collect();

        if parts.len() >= 4 {
            rows.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                parts[0], parts[1], parts[2], parts[3]
            ));
        }
    }

    let html = format!(
        r#"
<html>
<head>
<title>HELmR CONTROL BOARD</title>
<meta http-equiv="refresh" content="2">
<style>
body {{ background:#000; color:#00ffa6; font-family:monospace; padding:20px; }}
.panel {{ border:1px solid #00ffa6; padding:12px; margin-bottom:20px; }}
table {{ width:100%; border-collapse:collapse; }}
td, th {{ padding:6px; text-align:left; }}
</style>
</head>
<body>

<h1>HELmR CONTROL BOARD</h1>

<div class="panel">
<b>MISSION BOARD</b>
<table>
<tr>
<th>MISSION</th>
<th>BUDGET</th>
<th>USAGE</th>
</tr>
{}
</table>
</div>

<div class="panel">
<b>AGENT STATUS</b>
<table>
<tr>
<th>AGENT</th>
<th>STATE</th>
<th>MISSION</th>
</tr>
{}
</table>
</div>

<div class="panel">
<b>RECENT EVENTS</b>
<table>
<tr>
<th>TIME</th>
<th>AGENT</th>
<th>MISSION</th>
<th>RESULT</th>
</tr>
{}
</table>
</div>

</body>
</html>
"#,
        mission_rows, agent_rows, rows
    );

    Html(html)
}

#[tokio::main]
async fn main() {
    let state = AppState {
        used_tokens: Arc::new(Mutex::new(HashSet::new())),
        tomb_registry: Arc::new(Mutex::new(HashSet::new())),
        mission_spend: Arc::new(Mutex::new(HashMap::new())),
        mission_limits: Arc::new(Mutex::new(HashMap::new())),
        agents: Arc::new(Mutex::new(HashMap::new())),
        activity: Arc::new(Mutex::new(Vec::new())),
    };

    let app = Router::new()
        .route("/mission/create", post(mission_create))
        .route("/authorize", post(authorize))
        .route("/airlock/write_file", post(airlock_write))
        .route("/control/terminate", post(terminate))
        .route("/console/board", get(board))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:7070").await.unwrap();

    println!("HELmR running on http://127.0.0.1:7070");

    axum::serve(listener, app).await.unwrap();
}