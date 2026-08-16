use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{stdin, stdout, AsyncBufReadExt, BufReader, AsyncWriteExt};
use tracing::{info, warn, error};
use agentic_rust_mcp::tools::{agency_pulse, content_check, data_vault, send_gmail_tool, SendGmailRequest};

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
                    let args = req.params.as_ref()
                        .and_then(|p| p.get("arguments"))
                        .cloned()
                        .unwrap_or(json!({}));

                    match serde_json::from_value::<SendGmailRequest>(args) {
                        Err(e) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: req.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32602,
                                message: format!("Invalid arguments: {}", e),
                            }),
                        },
                        Ok(gmail_req) if gmail_req.to.is_empty() => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: req.id,
                            result: None,
                            error: Some(JsonRpcError {
                                code: -32602,
                                message: "Missing required parameter: to".to_string(),
                            }),
                        },
                        Ok(gmail_req) => {
                            match send_gmail_tool(&gmail_req.to, &gmail_req.subject, &gmail_req.body).await {
                                Ok(result) => JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    id: req.id,
                                    result: Some(serde_json::to_value(result).unwrap_or(json!({}))),
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
