use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::{Args, ValueEnum};
use reqwest::blocking::Client;
use serde_json::Value;

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

    // 6. Resolve API key
    let key = resolve_api_key(&args)?;

    // 7. Send HTTP request
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

    // 8. Handle response
    let status = response.status();
    let response_text = response
        .text()
        .with_context(|| "failed to read response body")?;

    if status.is_success() {
        let json: Value = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("API returned invalid JSON: {}", e))?;
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
        return Ok(key.clone());
    }

    let env_var = match args.provider {
        RunProviderArg::Openai => "OPENAI_API_KEY",
        RunProviderArg::Anthropic => "ANTHROPIC_API_KEY",
    };

    env::var(env_var).with_context(|| {
        format!(
            "no API key provided. Use --key or set {} environment variable",
            env_var
        )
    })
}
