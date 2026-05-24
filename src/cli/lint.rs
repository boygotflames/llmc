use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::Args;

use crate::lint::lint_document;
use crate::parser::parse_str;
use crate::validator::validate_document;

#[derive(Debug, Args)]
pub struct LintArgs {
    /// Path to the .llm file to lint.
    pub input: PathBuf,
}

pub fn run(args: LintArgs) -> Result<()> {
    let source = fs::read_to_string(&args.input)
        .with_context(|| format!("failed to read {}", args.input.display()))?;

    let document = parse_str(&source).map_err(|diagnostics| {
        eprintln!("{diagnostics}");
        anyhow!("parse failed")
    })?;

    let (document, compose_diags) = crate::composer::compose(document, &args.input, &[]);
    if compose_diags.has_errors() {
        eprintln!("{compose_diags}");
        return Err(anyhow!("include composition failed"));
    }

    let diagnostics = validate_document(&document);
    if diagnostics.has_errors() {
        eprintln!("{diagnostics}");
        return Err(anyhow!("validation failed"));
    }

    let warnings = lint_document(&document);

    if warnings.is_empty() {
        println!("✓ lint clean  {}", args.input.display());
        Ok(())
    } else {
        for warning in &warnings {
            eprintln!("[{}] {}", warning.code, warning.message);
        }
        eprintln!(
            "✗ lint  {}  ({} warning(s))",
            args.input.display(),
            warnings.len()
        );
        std::process::exit(1);
    }
}
