use crate::ast::{Document, Node};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LintWarning {
    pub code: &'static str,
    pub message: String,
}

/// Run all lint rules against a validated document.
/// Returns warnings in discovery order.
pub fn lint_document(document: &Document) -> Vec<LintWarning> {
    let mut warnings = Vec::new();

    rule_missing_output(document, &mut warnings);
    rule_missing_user(document, &mut warnings);
    rule_short_system(document, &mut warnings);
    rule_long_system(document, &mut warnings);
    rule_contradictory_constraints(document, &mut warnings);
    rule_duplicate_constraints(document, &mut warnings);

    warnings
}

// L001: Missing output schema
fn rule_missing_output(doc: &Document, warnings: &mut Vec<LintWarning>) {
    if doc.output.is_none() {
        warnings.push(LintWarning {
            code: "L001",
            message: "prompt does not declare expected output schema \
                      (add `output:`)"
                .to_string(),
        });
    }
}

// L002: Missing user turn
fn rule_missing_user(doc: &Document, warnings: &mut Vec<LintWarning>) {
    if doc.user.is_none() {
        warnings.push(LintWarning {
            code: "L002",
            message: "prompt has no user turn; the model may not \
                      produce useful output"
                .to_string(),
        });
    }
}

// L003: Very short system prompt
fn rule_short_system(doc: &Document, warnings: &mut Vec<LintWarning>) {
    let content_len = system_content_length(doc);
    if content_len > 0 && content_len < 20 {
        warnings.push(LintWarning {
            code: "L003",
            message: format!(
                "system prompt is very short ({} chars); \
                 consider adding more context",
                content_len
            ),
        });
    }
}

// L004: Very long system prompt
fn rule_long_system(doc: &Document, warnings: &mut Vec<LintWarning>) {
    let content_len = system_content_length(doc);
    if content_len > 4000 {
        warnings.push(LintWarning {
            code: "L004",
            message: format!(
                "system prompt is very long ({} chars); \
                 may exceed context window",
                content_len
            ),
        });
    }
}

// L005: Contradictory constraints
fn rule_contradictory_constraints(doc: &Document, warnings: &mut Vec<LintWarning>) {
    let constraints = match doc.constraints.as_ref() {
        Some(Node::Sequence { values, .. }) => values,
        _ => return,
    };

    let texts: Vec<String> = constraints
        .iter()
        .filter_map(|n| n.as_scalar())
        .map(|s| s.to_lowercase())
        .collect();

    let concise = texts
        .iter()
        .any(|s| s.contains("concise") || s.contains("brief") || s.contains("short"));
    let verbose = texts.iter().any(|s| {
        s.contains("detailed")
            || s.contains("thorough")
            || s.contains("verbose")
            || s.contains("comprehensive")
    });

    if concise && verbose {
        warnings.push(LintWarning {
            code: "L005",
            message: "contradictory constraints: both concise and \
                      detailed/verbose instructions present"
                .to_string(),
        });
    }
}

// L006: Duplicate constraints
fn rule_duplicate_constraints(doc: &Document, warnings: &mut Vec<LintWarning>) {
    let constraints = match doc.constraints.as_ref() {
        Some(Node::Sequence { values, .. }) => values,
        _ => return,
    };

    let mut seen = HashSet::new();
    for node in constraints {
        if let Some(text) = node.as_scalar() {
            let normalized = text.trim().to_lowercase();
            if !seen.insert(normalized.clone()) {
                warnings.push(LintWarning {
                    code: "L006",
                    message: format!("duplicate constraint: \"{}\"", text),
                });
            }
        }
    }
}

/// Approximate character count of system prompt content.
fn system_content_length(doc: &Document) -> usize {
    match doc.system.as_ref() {
        Some(Node::Scalar { value, .. }) => value.len(),
        Some(Node::Mapping { entries, .. }) => entries
            .iter()
            .map(|e| e.key.len() + e.value.as_scalar().unwrap_or("").len())
            .sum::<usize>(),
        _ => 0,
    }
}
