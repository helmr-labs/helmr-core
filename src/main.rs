use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    fs,
    io::{BufRead, BufReader},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    used_tokens: Arc<Mutex<HashSet<String>>>,
    frozen_agents: Arc<Mutex<HashSet<String>>>,
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
    token: String,
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
struct FreezeRequest {
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

fn write_trace(entry: serde_json::Value) {
    let trace_dir = PathBuf::from("traces");
    let _ = fs::create_dir_all(&trace_dir);

    let trace_path = trace_dir.join("trace.log");

    let line = format!("{}\n", entry.to_string());

    let _ = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(trace_path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
}

async fn authorize(
    State(state): State<AppState>,
    Json(req): Json<AuthorizeRequest>,
) -> Json<AuthorizeResponse> {

    let frozen = state.frozen_agents.lock().unwrap();

    if frozen.contains(&req.agent_id) {
        return Json(AuthorizeResponse {
            decision: "BLOCKED".to_string(),
            reason: "AGENT_FROZEN".to_string(),
            token: "".to_string(),
        });
    }

    let token_id = Uuid::new_v4().to_string();

    write_trace(json!({
        "timestamp": now(),
        "token_id": token_id,
        "agent_id": req.agent_id,
        "mission_id": req.mission_id,
        "action_type": req.action_type,
        "target": req.target,
        "decision": "ALLOWED",
        "reason": "TOKEN_ISSUED"
    }));

    Json(AuthorizeResponse {
        decision: "ALLOWED".to_string(),
        reason: "ALLOW_WORKSPACE_ONLY".to_string(),
        token: token_id,
    })
}

async fn airlock_write_file(
    State(state): State<AppState>,
    Json(req): Json<AirlockWriteRequest>,
) -> Json<DecisionResponse> {

    let frozen = state.frozen_agents.lock().unwrap();

    if frozen.contains(&req.agent_id) {
        return Json(DecisionResponse {
            decision: "BLOCKED".to_string(),
            reason: "AGENT_FROZEN".to_string(),
        });
    }

    if !req.target.starts_with("workspace/") {
        return Json(DecisionResponse {
            decision: "BLOCKED".to_string(),
            reason: "TARGET_OUTSIDE_WORKSPACE".to_string(),
        });
    }

    let mut used = state.used_tokens.lock().unwrap();

    if used.contains(&req.token) {

        write_trace(json!({
            "timestamp": now(),
            "token_id": req.token,
            "agent_id": req.agent_id,
            "mission_id": req.mission_id,
            "action_type": "write_file",
            "target": req.target,
            "decision": "BLOCKED",
            "reason": "TOKEN_ALREADY_USED"
        }));

        return Json(DecisionResponse {
            decision: "BLOCKED".to_string(),
            reason: "TOKEN_ALREADY_USED".to_string(),
        });
    }

    used.insert(req.token.clone());

    let path = PathBuf::from(&req.target);

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let _ = fs::write(&path, req.content);

    write_trace(json!({
        "timestamp": now(),
        "token_id": req.token,
        "agent_id": req.agent_id,
        "mission_id": req.mission_id,
        "action_type": "write_file",
        "target": req.target,
        "decision": "ALLOWED",
        "reason": "TOKEN_VERIFIED_EXECUTED"
    }));

    Json(DecisionResponse {
        decision: "ALLOWED".to_string(),
        reason: "TOKEN_VERIFIED_EXECUTED".to_string(),
    })
}

async fn freeze_agent(
    State(state): State<AppState>,
    Json(req): Json<FreezeRequest>,
) -> Json<DecisionResponse> {

    let mut frozen = state.frozen_agents.lock().unwrap();
    frozen.insert(req.agent_id.clone());

    write_trace(json!({
        "timestamp": now(),
        "agent_id": req.agent_id,
        "decision": "FREEZE",
        "reason": "AGENT_FROZEN"
    }));

    Json(DecisionResponse {
        decision: "SUCCESS".to_string(),
        reason: "AGENT_FROZEN".to_string(),
    })
}

async fn console_events() -> Json<Vec<Value>> {

    let trace_path = PathBuf::from("traces/trace.log");

    if !trace_path.exists() {
        return Json(vec![]);
    }

    let file = fs::File::open(trace_path).unwrap();
    let reader = BufReader::new(file);

    let mut lines: Vec<String> = reader.lines().flatten().collect();
    lines.reverse();

    let mut events: Vec<Value> = Vec::new();

    for line in lines.into_iter().take(50) {
        if let Ok(v) = serde_json::from_str::<Value>(&line) {
            events.push(v);
        }
    }

    Json(events)
}

async fn console_board() -> Html<String> {

let html = r#"
<html>
<head>
<title>HELmR Activity Board</title>

<style>
body { font-family: monospace; background:#111; color:#0f0; padding:20px }
table { border-collapse: collapse; width:100% }
th, td { border-bottom:1px solid #333; padding:6px }
button { background:#900; color:white; border:none; padding:4px 8px }
</style>

</head>

<body>

<h2>HELmR ACTIVITY BOARD</h2>

<table id="board">
<tr>
<th>TIME</th>
<th>AGENT</th>
<th>ACTION</th>
<th>TARGET</th>
<th>RESULT</th>
<th>CONTROL</th>
</tr>
</table>

<script>

async function freezeAgent(agent) {

await fetch('/freeze_agent', {
method: 'POST',
headers: {'Content-Type':'application/json'},
body: JSON.stringify({agent_id:agent})
});

}

async function refresh() {

const res = await fetch('/console/events');
const data = await res.json();

const table = document.getElementById('board');

table.innerHTML = `
<tr>
<th>TIME</th>
<th>AGENT</th>
<th>ACTION</th>
<th>TARGET</th>
<th>RESULT</th>
<th>CONTROL</th>
</tr>
`;

data.forEach(e => {

const agent = e.agent_id || "";

const row = document.createElement('tr');

row.innerHTML = `
<td>${e.timestamp || ""}</td>
<td>${agent}</td>
<td>${e.action_type || ""}</td>
<td>${e.target || ""}</td>
<td>${e.decision || ""}</td>
<td><button onclick="freezeAgent('${agent}')">FREEZE</button></td>
`;

table.appendChild(row);

});

}

setInterval(refresh,1000);
refresh();

</script>

</body>
</html>
"#;

Html(html.to_string())

}

#[tokio::main]
async fn main() {

let state = AppState {
used_tokens: Arc::new(Mutex::new(HashSet::new())),
frozen_agents: Arc::new(Mutex::new(HashSet::new())),
};

let app = Router::new()
.route("/authorize", post(authorize))
.route("/airlock/write_file", post(airlock_write_file))
.route("/freeze_agent", post(freeze_agent))
.route("/console/events", get(console_events))
.route("/console/board", get(console_board))
.with_state(state);

let listener = TcpListener::bind("127.0.0.1:7070")
.await
.unwrap();

println!("HELmR listening on http://127.0.0.1:7070");

axum::serve(listener, app)
.await
.unwrap();
}