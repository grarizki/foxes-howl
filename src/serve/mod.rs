pub mod routes;

use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::Router;
use serde::Serialize;
use std::net::SocketAddr;

/// Start the HTTP server with bearer auth
pub async fn run_server(
    port: u16,
    token_env: &str,
    config: crate::config::Config,
) -> anyhow::Result<()> {
    let token = std::env::var(token_env).map_err(|_| {
        anyhow::anyhow!("Set {} in your environment to start the server", token_env)
    })?;

    // Show token hash (last 4 chars)
    let token_hash = format!("****{}", &token[token.len().saturating_sub(4)..]);

    let app = Router::new()
        .route("/health", get(routes::health))
        .route("/tools", get(routes::tools))
        .route("/profile", get(routes::profile))
        .route("/call", post(routes::call_tool))
        .route("/ai/analyze", post(routes::ai_analyze))
        .route("/ai/recommend", post(routes::ai_recommend))
        .route("/ai/difficulty", post(routes::ai_difficulty))
        .route("/security", post(routes::security_check))
        .layer(axum::Extension(token))
        .layer(axum::Extension(config));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("gh-opp server");
    println!("  Listening: {}", addr);
    println!("  Token:     {}", token_hash);
    println!("  Tools:     GET  /tools");
    println!("  Call:      POST /call");
    println!("  AI:        POST /ai/analyze | /ai/recommend | /ai/difficulty");
    println!("  Security:  POST /security");
    println!("  Profile:   GET  /profile");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Check bearer token. Returns Ok(()) or error status.
pub fn check_auth(headers: &HeaderMap, expected_token: &str) -> Result<(), StatusCode> {
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if auth == format!("Bearer {}", expected_token) {
        Ok(())
    } else if auth.is_empty() {
        Err(StatusCode::UNAUTHORIZED)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_check_auth_valid() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer test-token-1234"),
        );
        assert!(check_auth(&headers, "test-token-1234").is_ok());
    }

    #[test]
    fn test_check_auth_invalid() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer wrong-token"),
        );
        assert_eq!(
            check_auth(&headers, "correct-token"),
            Err(StatusCode::FORBIDDEN)
        );
    }

    #[test]
    fn test_check_auth_missing() {
        let headers = HeaderMap::new();
        assert_eq!(
            check_auth(&headers, "any-token"),
            Err(StatusCode::UNAUTHORIZED)
        );
    }

    #[test]
    fn test_token_hash_display() {
        let token = "abcdefghijklmnop";
        let hash = format!("****{}", &token[token.len().saturating_sub(4)..]);
        assert_eq!(hash, "****mnop");
    }
}
