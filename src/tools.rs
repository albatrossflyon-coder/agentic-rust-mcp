use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use chrono::Local;

use crate::gmail_sender;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GmailSendResult {
    pub success: bool,
    pub to: String,
    pub subject: String,
    pub demo_mode: bool,
}

// Shared send_gmail request shape, used by both the stdio MCP server
// (main.rs, parsed from JSON-RPC "arguments") and the web demo
// (web_server.rs, parsed from a POST body) — one typed contract instead of
// two independent raw-JSON/struct definitions for the same tool call.
#[derive(Debug, Deserialize)]
pub struct SendGmailRequest {
    #[serde(default)]
    pub to: String,
    #[serde(default = "default_gmail_subject")]
    pub subject: String,
    #[serde(default)]
    pub body: String,
}

fn default_gmail_subject() -> String {
    "(no subject)".to_string()
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
    #[serde(rename = "uid")]
    pub id: String,
    pub name: String,
    // ponytail: Vercel's schema marks `state` as legacy/optional and
    // `readyState` as the guaranteed field; keep both and prefer `state`
    // when present rather than requiring a field the API doesn't promise.
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default, rename = "readyState")]
    pub ready_state: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
}

impl VercelDeployment {
    pub fn effective_state(&self) -> Option<&str> {
        self.state.as_deref().or(self.ready_state.as_deref())
    }
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
// DEMO MODE
// ============================================================================

// ponytail: one env var gates all fixture data, checked once per call — no
// separate demo-data files/config layer needed for 4 tools.
pub fn demo_mode() -> bool {
    std::env::var("DEMO_MODE").map(|v| v == "true" || v == "1").unwrap_or(false)
}

// ============================================================================
// TOOL IMPLEMENTATIONS (REAL API CALLS)
// ============================================================================

pub async fn agency_pulse() -> Result<DeploymentStatus> {
    dotenv::dotenv().ok();

    if demo_mode() {
        return Ok(DeploymentStatus {
            name: "Agentic Platform (Demo)".to_string(),
            url: "https://albatrossai.online".to_string(),
            status: "live".to_string(),
            last_updated: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        });
    }

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
                tracing::warn!("Render API error: {}", e);
                render_status = "failed".to_string();
            }
        }
    }

    let mut vercel_status = "offline".to_string();
    if !vercel_token.is_empty() {
        match query_vercel_deployments(&vercel_token).await {
            Ok(deploys) => {
                if !deploys.is_empty() && deploys[0].effective_state() == Some("READY") {
                    vercel_status = "live".to_string();
                } else {
                    vercel_status = "deploying".to_string();
                }
            }
            Err(e) => {
                tracing::warn!("Vercel API error: {}", e);
                vercel_status = "failed".to_string();
            }
        }
    }

    let status = if render_status == "live" && vercel_status == "live" {
        "live".to_string()
    } else if render_status == "failed" || vercel_status == "failed" {
        "failed".to_string()
    } else if render_key.is_empty() && vercel_token.is_empty() {
        // ponytail: neither key is set, so this isn't "deploying" — nothing
        // was ever configured to check. DeploymentStatus has one flat status
        // field; a real per-provider (render vs vercel) breakdown would need
        // a wire-format change, out of scope for this pass.
        "configuration_error".to_string()
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
        .get("https://api.vercel.com/v6/deployments")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    let body: VercelDeploymentsResponse = resp.json().await?;
    Ok(body.deployments)
}

pub async fn content_check() -> Result<BufferPost> {
    dotenv::dotenv().ok();

    if demo_mode() {
        return Ok(BufferPost {
            channel: "Social Media (Demo)".to_string(),
            scheduled_posts: 12,
            pending_approval: 3,
            next_post_time: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        });
    }

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
            Err(e) => tracing::warn!("Buffer API error: {}", e),
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

    if demo_mode() {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        return Ok(vec![
            FirestoreLead {
                id: "demo-lead-1".to_string(),
                name: "Jordan Lee".to_string(),
                email: "jordan.lee@example.com".to_string(),
                created_at: now.clone(),
            },
            FirestoreLead {
                id: "demo-lead-2".to_string(),
                name: "Priya Nair".to_string(),
                email: "priya.nair@example.com".to_string(),
                created_at: now,
            },
        ]);
    }


    let project_id = std::env::var("FIREBASE_PROJECT_ID").unwrap_or_default();
    let api_key = std::env::var("FIREBASE_API_KEY").unwrap_or_default();

    let mut leads = Vec::new();

    if !project_id.is_empty() && !api_key.is_empty() {
        match query_firestore(&project_id, &api_key, "leads").await {
            Ok(docs) => {
                leads = docs;
            }
            Err(e) => tracing::warn!("Firestore API error: {}", e),
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

pub async fn send_gmail_tool(to: &str, subject: &str, body: &str) -> Result<GmailSendResult> {
    if demo_mode() {
        // ponytail: public demo never sends real mail — no rate limiter/audit
        // log needed when the side effect itself never happens.
        tokio::time::sleep(Duration::from_millis(400)).await;
        return Ok(GmailSendResult {
            success: true,
            to: to.to_string(),
            subject: subject.to_string(),
            demo_mode: true,
        });
    }

    gmail_sender::send_email(to, subject, body).await?;
    Ok(GmailSendResult {
        success: true,
        to: to.to_string(),
        subject: subject.to_string(),
        demo_mode: false,
    })
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

    // ponytail: one test, one set/remove of the process-global DEMO_MODE env
    // var — two separate #[tokio::test] fns mutating it would race under the
    // default parallel test runner.
    #[tokio::test]
    async fn test_demo_mode_short_circuits_every_tool() {
        std::env::set_var("DEMO_MODE", "true");

        let mail = send_gmail_tool("nobody@example.com", "subject", "body").await.unwrap();
        assert!(mail.demo_mode);
        assert!(mail.success);

        let status = agency_pulse().await.unwrap();
        assert_eq!(status.status, "live");
        assert!(status.name.contains("Demo"));

        let post = content_check().await.unwrap();
        assert!(post.channel.contains("Demo"));

        let leads = data_vault().await.unwrap();
        assert_eq!(leads.len(), 2);

        std::env::remove_var("DEMO_MODE");
    }
}
