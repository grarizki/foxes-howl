/// Token and cost estimate for an LLM call
#[derive(Debug, Clone, serde::Serialize)]
pub struct TokenEstimate {
    pub input_tokens: u32,
    pub estimated_output_tokens: u32,
    pub estimated_cost_usd: f64,
}

/// Estimate tokens and cost for a prompt pair.
/// Heuristic: ~4 chars per token for English text.
pub fn estimate(system: &str, user: &str, model: &str) -> TokenEstimate {
    let input_chars = (system.len() + user.len()) as f64;
    let input_tokens = (input_chars / 4.0).ceil() as u32;

    // Estimate output as ~30% of input, min 200, max 2048
    let estimated_output_tokens = ((input_tokens as f64 * 0.3) as u32).clamp(200, 2048);

    let (input_price, output_price) = model_pricing(model);
    let estimated_cost_usd =
        (input_tokens as f64 * input_price) + (estimated_output_tokens as f64 * output_price);

    TokenEstimate {
        input_tokens,
        estimated_output_tokens,
        estimated_cost_usd,
    }
}

/// Price per token (not per million) for known models
fn model_pricing(model: &str) -> (f64, f64) {
    match model {
        // OpenAI
        "gpt-4o" => (2.50 / 1_000_000.0, 10.00 / 1_000_000.0),
        "gpt-4o-mini" => (0.15 / 1_000_000.0, 0.60 / 1_000_000.0),
        "gpt-4-turbo" => (10.00 / 1_000_000.0, 30.00 / 1_000_000.0),
        "gpt-3.5-turbo" => (0.50 / 1_000_000.0, 1.50 / 1_000_000.0),
        // Anthropic
        "claude-sonnet-4-20250514" => (3.00 / 1_000_000.0, 15.00 / 1_000_000.0),
        "claude-3-5-sonnet-20241022" => (3.00 / 1_000_000.0, 15.00 / 1_000_000.0),
        "claude-3-5-haiku-20241022" => (0.80 / 1_000_000.0, 4.00 / 1_000_000.0),
        "claude-3-opus-20240229" => (15.00 / 1_000_000.0, 75.00 / 1_000_000.0),
        // Default: conservative estimate
        _ => (5.00 / 1_000_000.0, 15.00 / 1_000_000.0),
    }
}

/// Format estimate for CLI display
pub fn format_estimate(est: &TokenEstimate) -> String {
    format!(
        "Estimated: ~{} input tokens, ~{} output tokens, ~${:.4}",
        est.input_tokens, est.estimated_output_tokens, est.estimated_cost_usd
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_basic() {
        let est = estimate("You are helpful", "Analyze this repo", "gpt-4o");
        assert!(est.input_tokens > 0);
        assert!(est.estimated_output_tokens >= 200);
        assert!(est.estimated_cost_usd > 0.0);
    }

    #[test]
    fn test_estimate_output_clamped() {
        let short = estimate("Hi", "Hi", "gpt-4o");
        assert_eq!(short.estimated_output_tokens, 200); // min

        let long_input = "x".repeat(50000);
        let long = estimate(&long_input, &long_input, "gpt-4o");
        assert_eq!(long.estimated_output_tokens, 2048); // max
    }

    #[test]
    fn test_model_pricing_known() {
        let (inp, out) = model_pricing("gpt-4o");
        assert!(inp > 0.0);
        assert!(out > inp); // output costs more

        let (inp2, out2) = model_pricing("claude-sonnet-4-20250514");
        assert!(inp2 > 0.0);
        assert!(out2 > inp2);
    }

    #[test]
    fn test_model_pricing_unknown() {
        let (inp, out) = model_pricing("unknown-model");
        assert!(inp > 0.0);
        assert!(out > 0.0);
    }

    #[test]
    fn test_format_estimate() {
        let est = TokenEstimate {
            input_tokens: 1000,
            estimated_output_tokens: 300,
            estimated_cost_usd: 0.007,
        };
        let formatted = format_estimate(&est);
        assert!(formatted.contains("1000"));
        assert!(formatted.contains("300"));
        assert!(formatted.contains("0.0070"));
    }

    #[test]
    fn test_estimate_different_models() {
        let gpt4o = estimate("test system", "test user", "gpt-4o");
        let mini = estimate("test system", "test user", "gpt-4o-mini");
        // Same input, but gpt-4o costs more per token
        assert!(gpt4o.estimated_cost_usd > mini.estimated_cost_usd);
    }
}
