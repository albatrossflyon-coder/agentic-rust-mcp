use axum::{
    extract::Json,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::json;
use tracing::info;

use agentic_rust_mcp::tools::{agency_pulse, content_check, data_vault, send_gmail_tool};

const INDEX_HTML: &str = include_str!("../../static/index.html");

#[derive(Debug, Deserialize)]
struct SendGmailRequest {
    #[serde(default)]
    to: String,
    #[serde(default)]
    subject: String,
    #[serde(default)]
    body: String,
}

async fn index() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], INDEX_HTML)
}

async fn health() -> impl IntoResponse {
    "ok"
}

fn respond<T: serde::Serialize>(result: anyhow::Result<T>) -> Response {
    match result {
        Ok(value) => Json(value).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn handle_agency_pulse() -> Response {
    respond(agency_pulse().await)
}

async fn handle_content_check() -> Response {
    respond(content_check().await)
}

async fn handle_data_vault() -> Response {
    respond(data_vault().await)
}

async fn handle_send_gmail(Json(req): Json<SendGmailRequest>) -> Response {
    if req.to.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Missing required field: to" })),
        )
            .into_response();
    }

    respond(send_gmail_tool(&req.to, &req.subject, &req.body).await)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    dotenv::dotenv().ok();

    // This binary is the public-facing demo surface — it has no auth, so it
    // must never be able to reach real accounts. The stdio server (main.rs)
    // is the one to run against real credentials; refuse to start rather
    // than silently serve live data/side effects to anonymous traffic.
    if !agentic_rust_mcp::tools::demo_mode() {
        eprintln!("FATAL: web_server requires DEMO_MODE=true. This binary is public-facing and must never reach real Render/Vercel/Buffer/Firestore/Gmail accounts. Refusing to start.");
        std::process::exit(1);
    }
    info!("🌐 agentic-rust-mcp web demo starting (DEMO_MODE=true)");

    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/api/agency_pulse", post(handle_agency_pulse))
        .route("/api/content_check", post(handle_content_check))
        .route("/api/data_vault", post(handle_data_vault))
        .route("/api/send_gmail", post(handle_send_gmail));

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("📡 Listening on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
