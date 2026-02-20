use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::genetic::Genome;
use crate::genome_decoder;

/// Configuration for the LLM mutation oracle.
#[derive(Debug, Clone, Deserialize)]
pub struct LlmOracleConfig {
    /// Feature flag — oracle is only invoked when enabled.
    pub enabled: bool,
    /// LLM provider: "anthropic", "openai", or "bedrock".
    pub provider: String,
    /// API endpoint URL (not used for bedrock).
    #[serde(default)]
    pub endpoint: String,
    /// API key (not used for bedrock — uses AWS env credentials).
    #[serde(default)]
    pub api_key: String,
    /// Model identifier.
    /// Bedrock: e.g. "us.anthropic.claude-sonnet-4-20250514-v1:0"
    /// Anthropic: e.g. "claude-sonnet-4-20250514"
    pub model: String,
    /// AWS region for Bedrock (default: us-east-1).
    #[serde(default = "default_region")]
    pub region: String,
    /// Number of genomes to request per invocation.
    #[serde(default = "default_genomes_per_invocation")]
    pub genomes_per_invocation: usize,
    /// Maximum tokens in the LLM response.
    #[serde(default = "default_max_response_tokens")]
    pub max_response_tokens: usize,
    /// Minimum generation before oracle can trigger.
    #[serde(default = "default_min_generation")]
    pub min_generation: usize,
    /// L0→L1 promotion rate below this triggers oracle.
    #[serde(default = "default_promotion_rate_threshold")]
    pub promotion_rate_threshold: f64,
    /// TFT rate above this triggers oracle (requires tft_min_generation).
    #[serde(default = "default_tft_rate_threshold")]
    pub tft_rate_threshold: f64,
    /// Minimum generation for TFT trigger (secondary, higher than min_generation).
    #[serde(default = "default_tft_min_generation")]
    pub tft_min_generation: usize,
    /// Minimum generations between invocations.
    #[serde(default = "default_cooldown_gens")]
    pub cooldown_gens: usize,
    /// Minimum seconds between invocations.
    #[serde(default = "default_cooldown_seconds")]
    pub cooldown_seconds: u64,
}

fn default_region() -> String {
    "us-east-1".to_string()
}
fn default_genomes_per_invocation() -> usize {
    10
}
fn default_max_response_tokens() -> usize {
    1024
}
fn default_min_generation() -> usize {
    100
}
fn default_promotion_rate_threshold() -> f64 {
    0.70
}
fn default_tft_rate_threshold() -> f64 {
    0.40
}
fn default_tft_min_generation() -> usize {
    200
}
fn default_cooldown_gens() -> usize {
    50
}
fn default_cooldown_seconds() -> u64 {
    600
}

impl Default for LlmOracleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: "bedrock".to_string(),
            endpoint: String::new(),
            api_key: String::new(),
            model: "us.anthropic.claude-sonnet-4-20250514-v1:0".to_string(),
            region: "us-east-1".to_string(),
            genomes_per_invocation: 10,
            max_response_tokens: 1024,
            min_generation: 100,
            promotion_rate_threshold: 0.70,
            tft_rate_threshold: 0.40,
            tft_min_generation: 200,
            cooldown_gens: 50,
            cooldown_seconds: 600,
        }
    }
}

/// Elite genome context passed to the LLM for mutation guidance.
#[derive(Debug, Clone, Serialize)]
pub struct EliteContext {
    pub formula: String,
    pub fitness: f64,
    pub oos_psr: f64,
    pub layer: usize,
    pub age: usize,
}

/// Top-performing formula from another symbol, for cross-symbol learning.
#[derive(Debug, Clone, Serialize)]
pub struct CrossSymbolElite {
    pub symbol: String,
    pub formula: String,
    pub is_psr: f64,
    pub oos_psr: f64,
}

/// Full context for an LLM oracle invocation.
#[derive(Debug)]
pub struct OracleContext {
    pub symbol: String,
    pub mode: String,
    pub generation: usize,
    pub feat_offset: usize,
    pub factor_names: Vec<String>,
    pub best_oos_psr: f64,
    pub tft_rate: f64,
    pub elites: Vec<EliteContext>,
    pub genomes_requested: usize,
    pub cross_symbol_elites: Vec<CrossSymbolElite>,
}

/// Result of a single oracle invocation.
#[derive(Debug)]
pub struct OracleResult {
    /// Successfully parsed and validated genomes.
    pub genomes: Vec<Genome>,
    /// Number of formulas returned by the LLM.
    pub raw_count: usize,
    /// The prompt sent to the LLM.
    pub prompt: String,
    /// Raw text returned by the LLM.
    pub response_text: String,
    /// All formulas parsed from the LLM response.
    pub parsed_formulas: Vec<String>,
    /// Formulas that passed validation and became genomes.
    pub accepted_formulas: Vec<String>,
    /// Formulas that failed validation: (formula, rejection reason).
    pub rejected_details: Vec<(String, String)>,
}

/// Build the prompt for the LLM.
pub fn build_prompt(ctx: &OracleContext) -> String {
    let mut prompt = String::with_capacity(2048);

    prompt.push_str(
        "You are a quantitative finance researcher designing alpha factors for US equities.\n\n",
    );

    // Context section
    prompt.push_str("## Context\n");
    prompt.push_str(&format!("- Symbol: {} ({} mode)\n", ctx.symbol, ctx.mode));
    prompt.push_str(&format!("- Generation: {}\n", ctx.generation));
    prompt.push_str(&format!("- Best OOS PSR: {:.3}\n", ctx.best_oos_psr));
    prompt.push_str(&format!(
        "- too_few_trades rate: {:.1}%\n",
        ctx.tft_rate * 100.0
    ));
    prompt.push('\n');

    // Available features
    prompt.push_str("## Available Features\n");
    for (i, name) in ctx.factor_names.iter().enumerate() {
        prompt.push_str(&format!("- {} (id: {})\n", name, i));
    }
    prompt.push('\n');

    // Available operators
    prompt.push_str("## Available Operators (RPN / postfix notation)\n");
    for (name, arity, desc) in genome_decoder::operator_descriptions() {
        prompt.push_str(&format!("- {} ({}, {})\n", name, arity, desc));
    }
    prompt.push('\n');

    // RPN explanation
    prompt.push_str("## RPN Notation\n");
    prompt.push_str("Formulas use Reverse Polish Notation (postfix). ");
    prompt.push_str("Features push values onto the stack, operators consume stack values.\n");
    prompt.push_str("- Unary ops consume 1 value, push 1 result\n");
    prompt.push_str("- Binary ops consume 2 values, push 1 result\n");
    prompt.push_str("- A valid formula ends with exactly 1 value on the stack\n");
    prompt.push_str("Examples:\n");
    prompt.push_str("- `return ABS` → absolute return (unary on 1 feature)\n");
    prompt.push_str(
        "- `momentum volume_ratio MUL` → momentum × volume_ratio (binary on 2 features)\n",
    );
    prompt.push_str("- `return TS_MEAN return SUB` → return - TS_MEAN(return) (mean reversion)\n");
    prompt.push('\n');

    // Current elites
    if !ctx.elites.is_empty() {
        prompt.push_str("## Current Elite Formulas (by OOS PSR)\n");
        for (i, elite) in ctx.elites.iter().enumerate() {
            prompt.push_str(&format!(
                "{}. `{}` → PSR: {:.3} (L{}, age {})\n",
                i + 1,
                elite.formula,
                elite.fitness,
                elite.layer,
                elite.age,
            ));
        }
        prompt.push('\n');
    }

    // Cross-symbol successful formulas
    if !ctx.cross_symbol_elites.is_empty() {
        prompt.push_str("## Cross-Symbol Successful Formulas\n");
        prompt.push_str(
            "These formulas achieved positive OOS PSR on similar stocks. Consider adapting these patterns:\n",
        );
        for (i, elite) in ctx.cross_symbol_elites.iter().enumerate() {
            prompt.push_str(&format!(
                "{}. {}: `{}` → OOS PSR: {:.3}, IS PSR: {:.3}\n",
                i + 1,
                elite.symbol,
                elite.formula,
                elite.oos_psr,
                elite.is_psr,
            ));
        }
        prompt.push('\n');
    }

    // Task
    prompt.push_str("## Task\n");
    prompt.push_str(&format!(
        "Generate {} new alpha factor formulas that:\n",
        ctx.genomes_requested
    ));
    prompt.push_str("1. Are DIFFERENT from the elites above\n");
    prompt.push_str("2. Combine features in financially meaningful ways\n");
    prompt.push_str("3. Produce trading signals that vary over time (not constants)\n");
    prompt
        .push_str("4. Use the RPN notation with feature and operator names separated by spaces\n");
    prompt.push_str("5. Have at most 15 tokens and end with exactly 1 value on the stack\n");
    prompt.push('\n');
    prompt.push_str("Output ONLY a JSON array of strings, each string is an RPN formula.\n");
    prompt.push_str("Example output:\n");
    prompt.push_str("[\"return momentum SUB ABS\", \"volume_ratio TS_MEAN close_position MUL\"]\n");

    prompt
}

/// Parse the LLM response text into a list of RPN formula strings.
///
/// Handles common LLM output patterns:
/// - Raw JSON array
/// - JSON wrapped in markdown code fences
/// - JSON with surrounding text
pub fn parse_response(response_text: &str) -> Vec<String> {
    // Try to find a JSON array in the response
    let text = response_text.trim();

    // Strip markdown code fences if present
    let json_str = if text.contains("```") {
        text.split("```")
            .nth(1)
            .map(|s| {
                // Remove optional language tag (e.g., ```json)
                s.strip_prefix("json").unwrap_or(s)
            })
            .unwrap_or(text)
            .trim()
    } else {
        text
    };

    // Find the first [ and last ] to extract the array
    let start = json_str.find('[');
    let end = json_str.rfind(']');

    if let (Some(s), Some(e)) = (start, end) {
        if s < e {
            let array_str = &json_str[s..=e];
            match serde_json::from_str::<Vec<String>>(array_str) {
                Ok(formulas) => return formulas,
                Err(e) => {
                    warn!("Failed to parse LLM response as JSON array: {}", e);
                }
            }
        }
    }

    warn!(
        "Could not extract JSON array from LLM response (length={})",
        response_text.len()
    );
    Vec::new()
}

/// Validate and convert parsed formula strings into Genome structs.
///
/// Filters out:
/// - Formulas with unrecognized tokens
/// - Stack-invalid formulas
/// - Duplicate formulas (within this batch)
/// - Formulas identical to existing elites
pub fn validate_formulas(
    formulas: Vec<String>,
    feat_offset: usize,
    factor_names: &[String],
    existing_tokens: &[Vec<usize>],
) -> ValidatedFormulas {
    let raw_count = formulas.len();
    let mut genomes = Vec::new();
    let mut accepted_formulas = Vec::new();
    let mut rejected_details = Vec::new();
    let mut seen: std::collections::HashSet<Vec<usize>> = std::collections::HashSet::new();

    // Pre-populate seen with existing elite tokens
    for tokens in existing_tokens {
        seen.insert(tokens.clone());
    }

    for formula in &formulas {
        // Step 1: Parse name → tokens
        let tokens = match genome_decoder::encode_formula(formula, feat_offset, factor_names) {
            Some(t) => t,
            None => {
                debug!("LLM formula rejected (unknown token): {}", formula);
                rejected_details.push((formula.clone(), "unknown token".to_string()));
                continue;
            }
        };

        // Step 2: Stack validation
        if let Err(e) = genome_decoder::validate_stack(&tokens, feat_offset) {
            debug!("LLM formula rejected (stack: {}): {}", e, formula);
            rejected_details.push((formula.clone(), format!("stack: {}", e)));
            continue;
        }

        // Step 3: Dedup
        if !seen.insert(tokens.clone()) {
            debug!("LLM formula rejected (duplicate): {}", formula);
            rejected_details.push((formula.clone(), "duplicate".to_string()));
            continue;
        }

        accepted_formulas.push(formula.clone());
        genomes.push(Genome {
            tokens,
            fitness: 0.0,
            age: 0,
        });
    }

    let rejected_count = raw_count - genomes.len();
    ValidatedFormulas {
        genomes,
        raw_count,
        rejected_count,
        accepted_formulas,
        rejected_details,
    }
}

/// Intermediate result from validate_formulas, before full OracleResult is assembled.
#[derive(Debug)]
pub struct ValidatedFormulas {
    pub genomes: Vec<Genome>,
    pub raw_count: usize,
    pub rejected_count: usize,
    pub accepted_formulas: Vec<String>,
    pub rejected_details: Vec<(String, String)>,
}

/// Call the LLM API and return the raw response text.
///
/// Supports Anthropic Messages API, OpenAI Chat Completions, and AWS Bedrock Converse.
pub async fn call_llm(config: &LlmOracleConfig, prompt: &str) -> Result<String, LlmError> {
    match config.provider.as_str() {
        "bedrock" => call_bedrock(config, prompt).await,
        "anthropic" | "openai" => {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| LlmError::HttpClient(e.to_string()))?;
            match config.provider.as_str() {
                "anthropic" => call_anthropic(&client, config, prompt).await,
                "openai" => call_openai(&client, config, prompt).await,
                _ => unreachable!(),
            }
        }
        other => Err(LlmError::UnsupportedProvider(other.to_string())),
    }
}

async fn call_anthropic(
    client: &reqwest::Client,
    config: &LlmOracleConfig,
    prompt: &str,
) -> Result<String, LlmError> {
    let body = serde_json::json!({
        "model": config.model,
        "max_tokens": config.max_response_tokens,
        "messages": [
            {"role": "user", "content": prompt}
        ]
    });

    let resp = client
        .post(&config.endpoint)
        .header("x-api-key", &config.api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| LlmError::Request(e.to_string()))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(LlmError::ApiError {
            status: status.as_u16(),
            body: text,
        });
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| LlmError::ParseResponse(e.to_string()))?;

    // Extract text from Anthropic's response format
    json["content"]
        .as_array()
        .and_then(|blocks| blocks.first())
        .and_then(|block| block["text"].as_str())
        .map(String::from)
        .ok_or_else(|| LlmError::ParseResponse("no text in response content".to_string()))
}

async fn call_openai(
    client: &reqwest::Client,
    config: &LlmOracleConfig,
    prompt: &str,
) -> Result<String, LlmError> {
    let body = serde_json::json!({
        "model": config.model,
        "max_tokens": config.max_response_tokens,
        "messages": [
            {"role": "user", "content": prompt}
        ]
    });

    let resp = client
        .post(&config.endpoint)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| LlmError::Request(e.to_string()))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(LlmError::ApiError {
            status: status.as_u16(),
            body: text,
        });
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| LlmError::ParseResponse(e.to_string()))?;

    json["choices"]
        .as_array()
        .and_then(|choices| choices.first())
        .and_then(|choice| choice["message"]["content"].as_str())
        .map(String::from)
        .ok_or_else(|| LlmError::ParseResponse("no content in response choices".to_string()))
}

/// Call AWS Bedrock Converse API.
/// Auth via environment variables: AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY,
/// and optionally AWS_SESSION_TOKEN.
async fn call_bedrock(config: &LlmOracleConfig, prompt: &str) -> Result<String, LlmError> {
    use aws_sdk_bedrockruntime::types::{
        ContentBlock, ConversationRole, InferenceConfiguration, Message as BrMessage,
    };

    let region = aws_config::Region::new(config.region.clone());
    let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region)
        .load()
        .await;

    let client = aws_sdk_bedrockruntime::Client::new(&aws_config);

    let message = BrMessage::builder()
        .role(ConversationRole::User)
        .content(ContentBlock::Text(prompt.to_string()))
        .build()
        .map_err(|e| LlmError::Request(format!("failed to build message: {}", e)))?;

    let max_tokens: i32 = config
        .max_response_tokens
        .try_into()
        .map_err(|_| LlmError::Request("max_response_tokens exceeds i32::MAX".to_string()))?;
    let inference_config = InferenceConfiguration::builder()
        .max_tokens(max_tokens)
        .build();

    let response = client
        .converse()
        .model_id(&config.model)
        .messages(message)
        .inference_config(inference_config)
        .send()
        .await
        .map_err(|e| LlmError::Request(format!("Bedrock Converse failed: {}", e)))?;

    // Extract text from Converse response
    let output = response
        .output()
        .ok_or_else(|| LlmError::ParseResponse("no output in Bedrock response".to_string()))?;

    if let aws_sdk_bedrockruntime::types::ConverseOutput::Message(msg) = output {
        for block in msg.content() {
            if let ContentBlock::Text(text) = block {
                return Ok(text.clone());
            }
        }
        Err(LlmError::ParseResponse(
            "no text block in Bedrock response".to_string(),
        ))
    } else {
        Err(LlmError::ParseResponse(
            "unexpected Bedrock output type".to_string(),
        ))
    }
}

/// End-to-end oracle invocation: build prompt → call LLM → parse → validate.
pub async fn generate_mutations(
    config: &LlmOracleConfig,
    ctx: &OracleContext,
    existing_tokens: &[Vec<usize>],
) -> Result<OracleResult, LlmError> {
    let prompt = build_prompt(ctx);
    info!(
        "[{}:{}] LLM oracle invoked: gen={}, elites={}, prompt_len={}",
        ctx.symbol,
        ctx.mode,
        ctx.generation,
        ctx.elites.len(),
        prompt.len()
    );

    let response_text = call_llm(config, &prompt).await?;
    debug!(
        "[{}:{}] LLM response ({}B): {}",
        ctx.symbol,
        ctx.mode,
        response_text.len(),
        &response_text[..response_text.len().min(200)]
    );

    let parsed_formulas = parse_response(&response_text);
    info!(
        "[{}:{}] LLM returned {} formulas",
        ctx.symbol,
        ctx.mode,
        parsed_formulas.len()
    );

    let validated = validate_formulas(
        parsed_formulas.clone(),
        ctx.feat_offset,
        &ctx.factor_names,
        existing_tokens,
    );
    info!(
        "[{}:{}] LLM oracle result: {} valid, {} rejected (of {} raw)",
        ctx.symbol,
        ctx.mode,
        validated.genomes.len(),
        validated.rejected_count,
        validated.raw_count
    );

    Ok(OracleResult {
        genomes: validated.genomes,
        raw_count: validated.raw_count,
        prompt,
        response_text,
        parsed_formulas,
        accepted_formulas: validated.accepted_formulas,
        rejected_details: validated.rejected_details,
    })
}

#[derive(Debug)]
pub enum LlmError {
    HttpClient(String),
    Request(String),
    ApiError { status: u16, body: String },
    ParseResponse(String),
    UnsupportedProvider(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HttpClient(e) => write!(f, "HTTP client error: {}", e),
            Self::Request(e) => write!(f, "request failed: {}", e),
            Self::ApiError { status, body } => {
                write!(f, "API error (status {}): {}", status, body)
            }
            Self::ParseResponse(e) => write!(f, "response parse error: {}", e),
            Self::UnsupportedProvider(p) => write!(f, "unsupported provider: {}", p),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_factors() -> Vec<String> {
        vec![
            "return",
            "vwap_deviation",
            "volume_ratio",
            "mean_reversion",
            "adv_ratio",
            "volatility",
            "momentum",
            "relative_strength",
            "close_position",
            "intraday_range",
            "vol_regime",
            "trend_strength",
            "momentum_regime",
            "atr_pct",
            "obv_pct",
            "mfi",
            "bb_percent_b",
            "macd_hist",
            "sma_200_diff",
            "amihud_illiq",
            "spread_proxy",
            "return_autocorr",
            "spy_corr",
            "spy_beta",
            "spy_rel_strength",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    #[test]
    fn test_build_prompt_contains_key_sections() {
        let ctx = OracleContext {
            symbol: "NVDA".to_string(),
            mode: "long_only".to_string(),
            generation: 500,
            feat_offset: 25,
            factor_names: test_factors(),
            best_oos_psr: 1.8,
            tft_rate: 0.15,
            elites: vec![EliteContext {
                formula: "return momentum SUB ABS".to_string(),
                fitness: 2.1,
                oos_psr: 1.8,
                layer: 3,
                age: 45,
            }],
            genomes_requested: 10,
            cross_symbol_elites: vec![CrossSymbolElite {
                symbol: "TSLA".to_string(),
                formula: "return TS_STD close TS_MEAN DIV".to_string(),
                is_psr: 1.80,
                oos_psr: 1.256,
            }],
        };

        let prompt = build_prompt(&ctx);
        assert!(prompt.contains("NVDA"));
        assert!(prompt.contains("long_only"));
        assert!(prompt.contains("Generation: 500"));
        assert!(prompt.contains("return momentum SUB ABS"));
        assert!(prompt.contains("ADD"));
        assert!(prompt.contains("TS_MEAN"));
        assert!(prompt.contains("JSON array"));
        assert!(prompt.contains("Cross-Symbol Successful Formulas"));
        assert!(prompt.contains("TSLA"));
        assert!(prompt.contains("return TS_STD close TS_MEAN DIV"));
    }

    #[test]
    fn test_parse_response_clean_json() {
        let response = r#"["return ABS", "momentum volume_ratio MUL"]"#;
        let formulas = parse_response(response);
        assert_eq!(formulas.len(), 2);
        assert_eq!(formulas[0], "return ABS");
        assert_eq!(formulas[1], "momentum volume_ratio MUL");
    }

    #[test]
    fn test_parse_response_with_code_fence() {
        let response =
            "Here are the formulas:\n```json\n[\"return ABS\", \"momentum TS_MEAN\"]\n```\n";
        let formulas = parse_response(response);
        assert_eq!(formulas.len(), 2);
    }

    #[test]
    fn test_parse_response_with_surrounding_text() {
        let response =
            "I'll generate some formulas:\n\n[\"return ABS\"]\n\nThese should work well.";
        let formulas = parse_response(response);
        assert_eq!(formulas.len(), 1);
        assert_eq!(formulas[0], "return ABS");
    }

    #[test]
    fn test_parse_response_invalid() {
        let response = "I don't understand the request.";
        let formulas = parse_response(response);
        assert!(formulas.is_empty());
    }

    #[test]
    fn test_validate_formulas_mixed() {
        let factors = test_factors();
        let feat_offset = 25;

        let formulas = vec![
            "return ABS".to_string(),                // valid unary
            "momentum volume_ratio MUL".to_string(), // valid binary
            "BOGUS_FEATURE ABS".to_string(),         // unknown token
            "return return".to_string(),             // stack depth 2
            "return ABS".to_string(),                // duplicate of first
        ];

        let result = validate_formulas(formulas, feat_offset, &factors, &[]);
        assert_eq!(result.raw_count, 5);
        assert_eq!(result.genomes.len(), 2); // only first two are valid
        assert_eq!(result.rejected_count, 3);
        assert_eq!(result.accepted_formulas.len(), 2);
        assert_eq!(result.accepted_formulas[0], "return ABS");
        assert_eq!(result.rejected_details.len(), 3);
        assert_eq!(result.rejected_details[0].1, "unknown token");
    }

    #[test]
    fn test_validate_formulas_dedup_vs_existing() {
        let factors = test_factors();
        let feat_offset = 25;

        // Token representation of "return ABS"
        let existing = vec![vec![0, feat_offset + 5]];

        let formulas = vec!["return ABS".to_string()];

        let result = validate_formulas(formulas, feat_offset, &factors, &existing);
        assert_eq!(result.genomes.len(), 0); // rejected as duplicate of existing
    }

    #[test]
    fn test_validate_formulas_complex_valid() {
        let factors = test_factors();
        let feat_offset = 25;

        let formulas = vec![
            // return TS_MEAN(return) SUB → mean reversion signal
            "return return TS_MEAN SUB".to_string(),
            // momentum * volume_ratio + trend_strength
            "momentum volume_ratio MUL trend_strength ADD".to_string(),
            // TS_RANK(volatility) - TS_RANK(spy_beta)
            "volatility TS_RANK spy_beta TS_RANK SUB".to_string(),
        ];

        let result = validate_formulas(formulas, feat_offset, &factors, &[]);
        assert_eq!(result.genomes.len(), 3);
        assert_eq!(result.rejected_count, 0);
    }
}
