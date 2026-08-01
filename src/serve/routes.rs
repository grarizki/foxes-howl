use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use super::check_auth;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

pub async fn tools(Extension(token): Extension<String>, headers: HeaderMap) -> Result<Json<serde_json::Value>, StatusCode> {
    check_auth(&headers, &token)?;
    let tools = crate::ai::tools::definitions();
    Ok(Json(serde_json::json!(tools)))
}

pub async fn profile(Extension(token): Extension<String>, Extension(config): Extension<crate::config::Config>, headers: HeaderMap) -> Result<Json<serde_json::Value>, StatusCode> {
    check_auth(&headers, &token)?;
    Ok(Json(serde_json::json!(config.ai.profile)))
}

#[derive(Deserialize)]
pub struct CallRequest {
    pub tool: String,
    pub args: serde_json::Value,
}

#[derive(Serialize)]
pub struct CallResponse {
    pub result: serde_json::Value,
    pub tokens: Option<crate::ai::estimate::TokenEstimate>,
}

pub async fn call_tool(
    Extension(token): Extension<String>,
    headers: HeaderMap,
    Json(req): Json<CallRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_auth(&headers, &token)?;

    // For now, return tool not implemented message
    // Full dispatch will be wired in Phase 15
    Ok(Json(serde_json::json!({
        "error": format!("Tool '{}' dispatch not yet wired", req.tool),
        "available_tools": ["discover_repos", "scan_issues", "analyze_repo", "ai_recommend"]
    })))
}

#[derive(Deserialize)]
pub struct AiRepoRequest {
    pub repo: String,
}

#[derive(Deserialize)]
pub struct AiRecommendRequest {
    pub repo: String,
    pub skills: Option<Vec<String>>,
    pub hours: Option<u32>,
}

pub async fn ai_analyze(
    Extension(token): Extension<String>,
    Extension(config): Extension<crate::config::Config>,
    headers: HeaderMap,
    Json(req): Json<AiRepoRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_auth(&headers, &token)?;

    let (system, user) = crate::ai::prompts::build_analyze_prompt(
        &req.repo, "[]", 0, 0.0, 0.0,
    );
    let est = crate::ai::estimate::estimate(&system, &user, &config.ai.model);

    // Return estimate only — actual AI call requires API key
    Ok(Json(serde_json::json!({
        "status": "prompt_ready",
        "repo": req.repo,
        "tokens": est,
        "note": "Set API key to execute. Use CLI for full AI analysis."
    })))
}

pub async fn ai_recommend(
    Extension(token): Extension<String>,
    Extension(config): Extension<crate::config::Config>,
    headers: HeaderMap,
    Json(req): Json<AiRecommendRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_auth(&headers, &token)?;

    let profile = crate::config::UserProfile {
        name: config.ai.profile.name.clone(),
        skills: req.skills.unwrap_or_else(|| config.ai.profile.skills.clone()),
        experience: config.ai.profile.experience.clone(),
        hours_per_week: req.hours.unwrap_or(config.ai.profile.hours_per_week),
        interests: config.ai.profile.interests.clone(),
    };

    let (system, user) = crate::ai::prompts::build_recommend_prompt(
        &req.repo, &profile, "[]", "[]",
    );
    let est = crate::ai::estimate::estimate(&system, &user, &config.ai.model);

    Ok(Json(serde_json::json!({
        "status": "prompt_ready",
        "repo": req.repo,
        "tokens": est,
        "note": "Set API key to execute. Use CLI for full AI recommendations."
    })))
}

pub async fn ai_difficulty(
    Extension(token): Extension<String>,
    Extension(config): Extension<crate::config::Config>,
    headers: HeaderMap,
    Json(req): Json<AiRepoRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_auth(&headers, &token)?;

    let (system, user) = crate::ai::prompts::build_difficulty_prompt(&req.repo, "[]");
    let est = crate::ai::estimate::estimate(&system, &user, &config.ai.model);

    Ok(Json(serde_json::json!({
        "status": "prompt_ready",
        "repo": req.repo,
        "tokens": est,
        "note": "Set API key to execute. Use CLI for full difficulty ratings."
    })))
}

pub async fn security_check(
    Extension(token): Extension<String>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    check_auth(&headers, &token)?;

    let report = crate::security::run_all(None, &[]).await;
    Ok(Json(serde_json::json!(report)))
}
