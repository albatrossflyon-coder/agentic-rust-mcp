use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::io::{stdin, stdout, AsyncBufReadExt, BufReader, AsyncWriteExt};
use tracing::{info, warn, error};
use chrono::Local;
use agentic_rust_mcp::gmail_sender;

// ============================================================================
// DATA STRUCTURES
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeploymentStatus {
    pub name: String,
    pub url: String,
    pub status: String,
    pub last_updated: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BufferPost {
    pub channel: String,
    pub scheduled_posts: u32,
    pub pending_approval: u32,
    pub next_post_time: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FirestoreLead {
    pub id: String,
    pub name: String,
    pub email: String,
    pub created_at: String,
}

// Render API response
#[derive(Debug, Serialize, Deserialize)]
pub struct RenderService {
    pub id: String,
    pub name: String,
    pub status: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

// Vercel API response
#[derive(Debug, Serialize, Deserialize)]
pub struct VercelDeploymentsResponse {
    pub deployments: Vec<VercelDeployment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VercelDeployment {
    pub id: String,
    pub name: String,
    pub state: String,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
}

// Buffer API response
#[derive(Debug, Serialize, Deserialize)]
pub struct BufferProfilesResponse {
    pub profiles: Vec<BufferProfile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BufferProfile {
    pub id: String,
    pub service: String,
    pub service_username: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BufferScheduleResponse {
    pub updates: Vec<BufferUpdate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BufferUpdate {
    pub id: String,
    pub text: String,
    pub due_at: i64,
    pub status: String,
}

// ============================================================================
// TOOL IMPLEMENTATIONS (REAL API CALLS)
// ============================================================================

pub async fn agency_pulse() -> Result<DeploymentStatus> {
    dotenv::dotenv().ok();
    
    let render_key = std::env::var("RENDER_API_KEY").unwrap_or_default();
    let vercel_token = std::env::var("VERCEL_TOKEN").unwrap_or_default();
    
    let mut render_status = "offline".to_string();
    if !render_key.is_empty() {
        match query_render_services(&render_key).await {
            Ok(services) => {
                if !services.is_empty() && services[0].status == "live" {
                    render_status = "live".to_string();
                } else {
                    render_status = "deploying".to_string();
                }
            }
            Err(e) => {
                warn!("Render API error: {}", e);
                render_status = "failed".to_string();
            }
        }
    }

    let mut vercel_status = "offline".to_string();
    if !vercel_token.is_empty() {
        match query_vercel_deployments(&vercel_token).await {
            Ok(deploys) => {
                if !deploys.is_empty() && deploys[0].state == "READY" {
                    vercel_status = "live".to_string();
                } else {
                    vercel_status = "deploying".to_string();
                }
            }
            Err(e) => {
                warn!("Vercel API error: {}", e);
                vercel_status = "failed".to_string();
            }
        }
    }

    let status = if render_status == "live" && vercel_status == "live" {
        "live".to_string()
    } else if render_status == "failed" || vercel_status == "failed" {
        "failed".to_string()
    } else {
        "deploying".to_string()
    };

    Ok(DeploymentStatus {
        name: "Agentic Platform".to_string(),
        url: "https://albatrossai.online".to_string(),
        status,
        last_updated: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}

async fn query_render_services(api_key: &str) -> Result<Vec<RenderService>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let resp = client
        .get("https://api.render.com/v1/services")
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;
    
    let body: Value = resp.json().await?;
    let services = serde_json::from_value(body["services"].clone())
        .unwrap_or_default();
    Ok(services)
}

async fn query_vercel_deployments(token: &str) -> Result<Vec<VercelDeployment>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let resp = client
        .get("https://api.vercel.com/v9/deployments")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    
    let body: VercelDeploymentsResponse = resp.json().await?;
    Ok(body.deployments)
}

pub async fn content_check() -> Result<BufferPost> {
    dotenv::dotenv().ok();
    
    let buffer_key = std::env::var("BUFFER_API_KEY").unwrap_or_default();
    
    let mut scheduled_posts = 0u32;
    let mut pending_approval = 0u32;
    let next_post_time = "N/A".to_string();
    
    if !buffer_key.is_empty() {
        match query_buffer_profiles(&buffer_key).await {
            Ok(profiles) => {
                for profile in profiles {
                    if let Ok(updates) = query_buffer_schedule(&buffer_key, &profile.id).await {
                        scheduled_posts += updates.len() as u32;
                        for update in updates {
                            if update.status == "pending" {
                                pending_approval += 1;
                            }
                        }
                    }
                }
            }
            Err(e) => warn!("Buffer API error: {}", e),
        }
    }

    Ok(BufferPost {
        channel: "Social Media".to_string(),
        scheduled_posts,
        pending_approval,
        next_post_time,
    })
}

async fn query_buffer_profiles(token: &str) -> Result<Vec<BufferProfile>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let resp = client
        .get("https://api.bufferapp.com/1/profiles.json")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    
    let body: BufferProfilesResponse = resp.json().await?;
    Ok(body.profiles)
}

async fn query_buffer_schedule(token: &str, profile_id: &str) -> Result<Vec<BufferUpdate>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let resp = client
        .get(format!(
            "https://api.bufferapp.com/1/profiles/{}/schedules.json",
            profile_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    
    let body: BufferScheduleResponse = resp.json().await?;
    Ok(body.updates)
}

pub async fn data_vault() -> Result<Vec<FirestoreLead>> {
    dotenv::dotenv().ok();
    
    let project_id = std::env::var("FIREBASE_PROJECT_ID").unwrap_or_default();
    let api_key = std::env::var("FIREBASE_API_KEY").unwrap_or_default();
    
    let mut leads = Vec::new();
    
    if !project_id.is_empty() && !api_key.is_empty() {
        match query_firestore(&project_id, &api_key, "leads").await {
            Ok(docs) => {
                leads = docs;
            }
            Err(e) => warn!("Firestore API error: {}", e),
        }
    }

    Ok(leads)
}

async fn query_firestore(project_id: &str, api_key: &str, collection: &str) -> Result<Vec<FirestoreLead>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let url = format!(
        "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/{}",
        project_id, collection
    );

    let resp = client
        .get(&url)
        .header("x-goog-api-key", api_key)
        .send()
        .await?;
    
    let body: Value = resp.json().await?;
    let mut leads = Vec::new();
    
    if let Some(documents) = body.get("documents").and_then(|d| d.as_array()) {
        for doc in documents {
            if let Some(fields) = doc.get("fields").and_then(|f| f.as_object()) {
                let id = doc.get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let name = fields.get("name")
                    .and_then(|n| n.get("stringValue"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("Unknown")
                    .to_string();
                let email = fields.get("email")
                    .and_then(|e| e.get("stringValue"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown@example.com")
                    .to_string();
                
                leads.push(FirestoreLead {
                    id,
                    name,
                    email,
                    created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                });
            }
        }
    }

    Ok(leads)
}

// ============================================================================
// MCP JSON-RPC 2.0 PROTOCOL
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<serde_json::Number>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<serde_json::Number>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

async fn handle_request(req: JsonRpcRequest) -> JsonRpcResponse {
    match req.method.as_str() {
        "initialize" => {
            info!("🔗 Initialize handshake received");
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                result: Some(json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "agentic-rust-mcp",
                        "version": "0.4.0"
                    }
                })),
                error: None,
            }
        }
        "tools/list" => {
            info!("📋 Tools list requested");
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                result: Some(json!({
                    "tools": [
                        {
                            "name": "agency_pulse",
                            "description": "Check live deployment status on Render and Vercel",
                            "inputSchema": {
                                "type": "object",
                                "properties": {},
                                "required": []
                            }
                        },
                        {
                            "name": "content_check",
                            "description": "Check Buffer scheduled posts across channels",
                            "inputSchema": {
                                "type": "object",
                                "properties": {},
                                "required": []
                            }
                        },
                        {
                            "name": "data_vault",
                            "description": "Query Firestore for leads/data",
                            "inputSchema": {
                                "type": "object",
                                "properties": {},
                                "required": []
                            }
                        },
                        {
                            "name": "send_gmail",
                            "description": "Send an email via Gmail SMTP",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "to": {
                                        "type": "string",
                                        "description": "Recipient email address"
                                    },
                                    "subject": {
                                        "type": "string",
                                        "description": "Email subject line"
                                    },
                                    "body": {
                                        "type": "string",
                                        "description": "Plain text email body"
                                    }
                                },
                                "required": ["to", "subject", "body"]
                            }
                        }
                    ]
                })),
                error: None,
            }
        }
        "tools/call" => {
            let params = req.params.as_ref().and_then(|p| p.as_object());
            let tool_name = params.and_then(|p| p.get("name")).and_then(|n| n.as_str());
            
            match tool_name {
                Some("agency_pulse") => {
                    info!("🚀 Calling agency_pulse");
                    match agency_pulse().await {
                        Ok(result) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: req.id,
                            result: Some(serde_json::to_value(result).unwrap_or(json!({}))),
                            error: None,
                        },
                        Err(e) => {
                            error!("agency_pulse failed: {}", e);
                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: req.id,
                                result: None,
                                error: Some(JsonRpcError {
                                    code: -32603,
                                    message: format!("Internal error: {}", e),
                                }),
                            }
                        }
                    }
                }
                Some("content_check") => {
                    info!("📮 Calling content_check");
                    match content_check().await {
                        Ok(result) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: req.id,
                            result: Some(serde_json::to_value(result).unwrap_or(json!({}))),
                            error: None,
                        },
                        Err(e) => {
                            error!("content_check failed: {}", e);
                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: req.id,
                                result: None,
                                error: Some(JsonRpcError {
                                    code: -32603,
                                    message: format!("Internal error: {}", e),
                                }),
                            }
                        }
                    }
                }
                Some("data_vault") => {
                    info!("🔓 Calling data_vault");
                    match data_vault().await {
                        Ok(result) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: req.id,
                            result: Some(serde_json::to_value(result).unwrap_or(json!([]))),
                            error: None,
                        },
                        Err(e) => {
                            error!("data_vault failed: {}", e);
                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: req.id,
                                result: None,
                                error: Some(JsonRpcError {
                                    code: -32603,
                                    message: format!("Internal error: {}", e),
                                }),
                            }
                        }
                    }
                }
                Some("send_gmail") => {
                    info!("📧 Calling send_gmail");
                    let to = req.params.as_ref()
                        .and_then(|p| p.get("arguments"))
                        .and_then(|a| a.get("to"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let subject = req.params.as_ref()
                        .and_then(|p| p.get("arguments"))
                        .and_then(|a| a.get("subject"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("(no subject)")
                        .to_string();
                    let body_text = req.params.as_ref()
                        .and_then(|p| p.get("arguments"))
                        .and_then(|a| a.get("body"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    if to.is_empty() {
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: req.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32602,
                                message: "Missing required parameter: to".to_string(),
                            }),
                        }
                    } else {
                        match gmail_sender::send_email(&to, &subject, &body_text).await {
                            Ok(()) => JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: req.id,
                                result: Some(json!({
                                    "success": true,
                                    "to": to,
                                    "subject": subject
                                })),
                                error: None,
                            },
                            Err(e) => {
                                error!("send_gmail failed: {}", e);
                                JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    id: req.id,
                                    result: None,
                                    error: Some(JsonRpcError {
                                        code: -32603,
                                        message: format!("Email send failed: {}", e),
                                    }),
                                }
                            }
                        }
                    }
                }
                _ => {
                    warn!("Unknown tool: {:?}", tool_name);
                    JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32601,
                            message: "Method not found".to_string(),
                        }),
                    }
                }
            }
        }
        _ => {
            warn!("Unknown method: {}", req.method);
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: "Method not found".to_string(),
                }),
            }
        }
    }
}

// ============================================================================
// MAIN
// ============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter("info")
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("🏗️ Agentic Rust MCP Server starting (v0.4.0)");
    info!("📡 MCP JSON-RPC 2.0 over stdio");
    info!("🔧 Tools: agency_pulse, content_check, data_vault, send_gmail");

    dotenv::dotenv().ok();
    
    if std::env::var("RENDER_API_KEY").is_ok() {
        info!("✅ RENDER_API_KEY loaded");
    } else {
        warn!("⚠️  RENDER_API_KEY not found");
    }
    if std::env::var("VERCEL_TOKEN").is_ok() {
        info!("✅ VERCEL_TOKEN loaded");
    } else {
        warn!("⚠️  VERCEL_TOKEN not found");
    }
    if std::env::var("BUFFER_API_KEY").is_ok() {
        info!("✅ BUFFER_API_KEY loaded");
    } else {
        warn!("⚠️  BUFFER_API_KEY not found");
    }
    if std::env::var("FIREBASE_PROJECT_ID").is_ok() {
        info!("✅ FIREBASE_PROJECT_ID loaded");
    } else {
        warn!("⚠️  FIREBASE_PROJECT_ID not found");
    }
    if std::env::var("FIREBASE_API_KEY").is_ok() {
        info!("✅ FIREBASE_API_KEY loaded");
    } else {
        warn!("⚠️  FIREBASE_API_KEY not found");
    }
    if std::env::var("GMAIL_USER").is_ok() {
        info!("✅ GMAIL_USER loaded");
    } else {
        warn!("⚠️  GMAIL_USER not found — send_gmail will fail");
    }
    if std::env::var("GMAIL_APP_PASSWORD").is_ok() {
        info!("✅ GMAIL_APP_PASSWORD loaded");
    } else {
        warn!("⚠️  GMAIL_APP_PASSWORD not found — send_gmail will fail");
    }

    let stdin = stdin();
    let stdout = stdout();
    let mut reader = BufReader::new(stdin);
    let mut writer = stdout;

    info!("📡 Listening for MCP JSON-RPC 2.0 requests...");

    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line).await? == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        info!("📨 Received: {}", trimmed);

        match serde_json::from_str::<JsonRpcRequest>(trimmed) {
            Ok(req) => {
                let response = handle_request(req).await;
                let response_json = serde_json::to_string(&response)?;
                writer.write_all(format!("{}\n", response_json).as_bytes()).await?;
                writer.flush().await?;
            }
            Err(e) => {
                error!("Failed to parse JSON-RPC request: {}", e);
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: "Parse error".to_string(),
                    }),
                };
                let response_json = serde_json::to_string(&error_response)?;
                writer.write_all(format!("{}\n", response_json).as_bytes()).await?;
                writer.flush().await?;
            }
        }
    }

    info!("👋 Shutting down");
    Ok(())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agency_pulse_schema() {
        let status = DeploymentStatus {
            name: "Test".to_string(),
            url: "http://test.com".to_string(),
            status: "live".to_string(),
            last_updated: "2024-01-01".to_string(),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Test"));
    }

    #[test]
    fn test_content_check_schema() {
        let post = BufferPost {
            channel: "Twitter".to_string(),
            scheduled_posts: 5,
            pending_approval: 1,
            next_post_time: "2024-01-01 10:00:00".to_string(),
        };
        let json = serde_json::to_string(&post).unwrap();
        assert!(json.contains("Twitter"));
    }

    #[test]
    fn test_data_vault_schema() {
        let lead = FirestoreLead {
            id: "123".to_string(),
            name: "Test Lead".to_string(),
            email: "test@example.com".to_string(),
            created_at: "2024-01-01".to_string(),
        };
        let json = serde_json::to_string(&lead).unwrap();
        assert!(json.contains("test@example.com"));
    }

    #[test]
    fn test_jsonrpc_response() {
        let resp = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some("1".parse().unwrap()),
            result: Some(json!({"status": "ok"})),
            error: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("2.0"));
        assert!(json.contains("ok"));
    }
}
