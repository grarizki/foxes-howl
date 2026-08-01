pub mod discover;
pub mod issues;

use octocrab::Octocrab;

pub fn build_client() -> anyhow::Result<Octocrab> {
    let token = std::env::var("GITHUB_TOKEN").ok();

    let builder = octocrab::OctocrabBuilder::new();
    let client = match token {
        Some(t) => builder.personal_token(t),
        None => {
            tracing::warn!("GITHUB_TOKEN not set; rate limit = 60 req/hr");
            builder
        }
    }
    .build()?;

    Ok(client)
}
