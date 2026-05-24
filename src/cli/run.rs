use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::{Args, ValueEnum};
use reqwest::blocking::Client;
use serde_json::Value;

use crate::ast::Document;
use crate::parser::parse_str;
use crate::provider::Provider;
use crate::transpile::{self, Target};
use crate::validator::validate_document;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RunProviderArg {
    Openai,
    Anthropic,
}

impl From<RunProviderArg> for Provider {
    fn from(value: RunProviderArg) -> Self {
        match value {
            RunProviderArg::Openai => Provider::Openai,
            RunProviderArg::Anthropic => Provider::Anthropic,
        }
    }
}

#[derive(Debug, Args)]
pub struct RunArgs {
    /// Path to the .llm file to execute.
    pub input: PathBuf,

    /// Target provider for the API call.
    #[arg(long, value_enum)]
    pub provider: RunProviderArg,

    /// API key. Falls back to OPENAI_API_KEY or ANTHROPIC_API_KEY env var.
    #[arg(long)]
    pub key: Option<String>,

    /// Model identifier (e.g., gpt-4o, claude-3-5-sonnet-20241022).
    #[arg(long)]
    pub model: String,

    /// Maximum tokens to generate. Required by Anthropic; ignored by OpenAI.
    #[arg(long, default_value_t = 1024)]
    pub max_tokens: u32,

    /// Print the API payload without sending it. Useful for debugging.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

pub fn run(args: RunArgs) -> Result<()> {
    // 1. Read and parse
    let source = fs::read_to_string(&args.input)
        .with_context(|| format!("failed to read {}", args.input.display()))?;

    let document = parse_str(&source).map_err(|diagnostics| {
        eprintln!("{diagnostics}");
        anyhow!("parse failed")
    })?;

    // 2. Compose includes
    let (document, compose_diags) = crate::composer::compose(document, &args.input, &[]);
    if compose_diags.has_errors() {
        eprintln!("{compose_diags}");
        return Err(anyhow!("include composition failed"));
    }

    // 3. Validate
    let diagnostics = validate_document(&document);
    if diagnostics.has_errors() {
        eprintln!("{diagnostics}");
        return Err(anyhow!("validation failed"));
    }

    // 4. Transpile to provider-native JSON
    let target = match args.provider {
        RunProviderArg::Openai => Target::OpenAiChat,
        RunProviderArg::Anthropic => Target::AnthropicMessages,
    };

    let payload = transpile::transpile_with_provider(&document, target, args.provider.into())?;

    // 5. Inject caller-owned fields (model, max_tokens)
    let mut json_payload: Value = serde_json::from_str(&payload)
        .map_err(|e| anyhow!("transpiled payload is not valid JSON: {}", e))?;

    json_payload["model"] = Value::String(args.model.clone());

    if args.provider == RunProviderArg::Anthropic {
        json_payload["max_tokens"] = Value::Number(args.max_tokens.into());
    }

    let body = serde_json::to_string(&json_payload)
        .map_err(|e| anyhow!("failed to serialize request body: {}", e))?;

    // 6. Dry-run: print payload and exit without sending (no API key needed)
    if args.dry_run {
        let json: Value =
            serde_json::from_str(&body).map_err(|e| anyhow!("payload is not valid JSON: {}", e))?;
        let pretty = serde_json::to_string_pretty(&json)
            .map_err(|e| anyhow!("failed to pretty-print payload: {}", e))?;
        let mut stdout = io::stdout().lock();
        stdout
            .write_all(pretty.as_bytes())
            .context("failed to write payload to stdout")?;
        stdout.write_all(b"\n").context("failed to write newline")?;
        return Ok(());
    }

    // 7. Resolve API key
    let key = resolve_api_key(&args)?;

    // 8. Send HTTP request
    let client = Client::new();

    let (url, response) = match args.provider {
        RunProviderArg::Openai => {
            let url = "https://api.openai.com/v1/chat/completions";
            let resp = client
                .post(url)
                .header("Authorization", format!("Bearer {}", key))
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .with_context(|| format!("failed to POST to {}", url))?;
            (url, resp)
        }
        RunProviderArg::Anthropic => {
            let url = "https://api.anthropic.com/v1/messages";
            let resp = client
                .post(url)
                .header("x-api-key", key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .with_context(|| format!("failed to POST to {}", url))?;
            (url, resp)
        }
    };

    // 9. Handle response
    let status = response.status();
    let response_text = response
        .text()
        .with_context(|| "failed to read response body")?;

    if status.is_success() {
        let json: Value = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("API returned invalid JSON: {}", e))?;

        // Validate response against output: schema if declared
        let missing_fields = validate_response_against_schema(&json, &document);
        if !missing_fields.is_empty() {
            eprintln!(
                "warning: response missing declared output field(s): {}",
                missing_fields.join(", ")
            );
        }

        let pretty = serde_json::to_string_pretty(&json)
            .map_err(|e| anyhow!("failed to pretty-print response: {}", e))?;
        let mut stdout = io::stdout().lock();
        stdout
            .write_all(pretty.as_bytes())
            .context("failed to write response to stdout")?;
        stdout.write_all(b"\n").context("failed to write newline")?;
        Ok(())
    } else {
        eprintln!("HTTP {} from {}", status, url);
        eprintln!("{}", response_text);
        Err(anyhow!("API returned error status: {}", status))
    }
}

fn resolve_api_key(args: &RunArgs) -> Result<String> {
    if let Some(ref key) = args.key {
        if key.is_empty() {
            return Err(anyhow!("--key value is empty"));
        }
        return Ok(key.clone());
    }

    let env_var = match args.provider {
        RunProviderArg::Openai => "OPENAI_API_KEY",
        RunProviderArg::Anthropic => "ANTHROPIC_API_KEY",
    };

    let key = env::var(env_var).with_context(|| {
        format!(
            "no API key provided. Use --key or set {} environment variable",
            env_var
        )
    })?;

    if key.is_empty() {
        return Err(anyhow!("{} environment variable is set but empty", env_var));
    }

    Ok(key)
}

/// Validate API response content against the document's `output:` schema.
/// Returns a list of missing field names.
/// If the document has no `output:` key, returns empty vec (no validation).
fn validate_response_against_schema(response_json: &Value, document: &Document) -> Vec<String> {
    use crate::ast::Node;

    let output_node = match document.output.as_ref() {
        Some(n) => n,
        None => return vec![],
    };

    let expected_keys = match output_node {
        Node::Mapping { entries, .. } => entries.iter().map(|e| e.key.clone()).collect::<Vec<_>>(),
        Node::Scalar { .. } | Node::Sequence { .. } => return vec![],
    };

    // Extract the text content from the response
    // OpenAI: response.choices[0].message.content
    // Anthropic: response.content[0].text
    let content_str = response_json
        .pointer("/choices/0/message/content")
        .or_else(|| response_json.pointer("/content/0/text"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Try to parse the content as JSON for field checking
    let content_json: Option<Value> = serde_json::from_str(content_str).ok();

    match content_json {
        Some(json) => {
            // Check each declared output key exists in response JSON
            expected_keys
                .into_iter()
                .filter(|key| json.get(key).is_none())
                .collect()
        }
        None => {
            // Response is plain text, not JSON — cannot validate schema
            // This is not an error; output: may declare intent, not structure
            vec![]
        }
    }
}
