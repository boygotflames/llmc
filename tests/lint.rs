use llm_format::ast::{Document, MappingEntry, Node};
use llm_format::diagnostics::Span;
use llm_format::lint::lint_document;

fn scalar(value: &str) -> Node {
    Node::Scalar {
        value: value.to_string(),
        span: Span::new(0, 0),
    }
}

fn mapping_entry(key: &str, value: &str) -> MappingEntry {
    MappingEntry::new(key, scalar(value), Span::new(0, 0))
}

fn sequence(items: Vec<&str>) -> Node {
    Node::Sequence {
        values: items.iter().map(|s| scalar(s)).collect(),
        span: Span::new(0, 0),
    }
}

// ── L001: Missing output ─────────────────────────────────────────────────

#[test]
fn lint_warns_when_output_is_missing() {
    let doc = Document {
        agent: Some(scalar("Test")),
        system: Some(scalar("You are helpful.")),
        ..Default::default()
    };
    let warnings = lint_document(&doc);
    assert!(warnings.iter().any(|w| w.code == "L001"));
}

#[test]
fn lint_no_l001_when_output_present() {
    let doc = Document {
        agent: Some(scalar("Test")),
        system: Some(scalar("You are helpful.")),
        output: Some(Node::Mapping {
            entries: vec![mapping_entry("format", "json")],
            span: Span::new(0, 0),
        }),
        ..Default::default()
    };
    let warnings = lint_document(&doc);
    assert!(!warnings.iter().any(|w| w.code == "L001"));
}

// ── L002: Missing user ───────────────────────────────────────────────────

#[test]
fn lint_warns_when_user_is_missing() {
    let doc = Document {
        agent: Some(scalar("Test")),
        system: Some(scalar("You are helpful.")),
        ..Default::default()
    };
    let warnings = lint_document(&doc);
    assert!(warnings.iter().any(|w| w.code == "L002"));
}

#[test]
fn lint_no_l002_when_user_present() {
    let doc = Document {
        agent: Some(scalar("Test")),
        system: Some(scalar("You are helpful.")),
        user: Some(scalar("Hello")),
        ..Default::default()
    };
    let warnings = lint_document(&doc);
    assert!(!warnings.iter().any(|w| w.code == "L002"));
}

// ── L003: Short system ───────────────────────────────────────────────────

#[test]
fn lint_warns_when_system_is_very_short() {
    let doc = Document {
        agent: Some(scalar("Test")),
        system: Some(scalar("Hi")),
        ..Default::default()
    };
    let warnings = lint_document(&doc);
    assert!(warnings.iter().any(|w| w.code == "L003"));
}

#[test]
fn lint_no_l003_when_system_is_reasonable() {
    let doc = Document {
        agent: Some(scalar("Test")),
        system: Some(scalar(
            "You are a helpful assistant that answers questions clearly.",
        )),
        ..Default::default()
    };
    let warnings = lint_document(&doc);
    assert!(!warnings.iter().any(|w| w.code == "L003"));
}

// ── L004: Long system ────────────────────────────────────────────────────

#[test]
fn lint_warns_when_system_is_very_long() {
    let doc = Document {
        agent: Some(scalar("Test")),
        system: Some(scalar(&"a".repeat(4001))),
        ..Default::default()
    };
    let warnings = lint_document(&doc);
    assert!(warnings.iter().any(|w| w.code == "L004"));
}

#[test]
fn lint_no_l004_when_system_is_reasonable_length() {
    let doc = Document {
        agent: Some(scalar("Test")),
        system: Some(scalar("You are a helpful assistant.")),
        ..Default::default()
    };
    let warnings = lint_document(&doc);
    assert!(!warnings.iter().any(|w| w.code == "L004"));
}

// ── L005: Contradictory constraints ──────────────────────────────────────

#[test]
fn lint_warns_on_contradictory_constraints() {
    let doc = Document {
        agent: Some(scalar("Test")),
        system: Some(scalar("You are helpful.")),
        constraints: Some(sequence(vec!["Be concise", "Be thorough and detailed"])),
        ..Default::default()
    };
    let warnings = lint_document(&doc);
    assert!(warnings.iter().any(|w| w.code == "L005"));
}

#[test]
fn lint_no_l005_when_constraints_are_consistent() {
    let doc = Document {
        agent: Some(scalar("Test")),
        system: Some(scalar("You are helpful.")),
        constraints: Some(sequence(vec!["Be concise", "Be polite"])),
        ..Default::default()
    };
    let warnings = lint_document(&doc);
    assert!(!warnings.iter().any(|w| w.code == "L005"));
}

// ── L006: Duplicate constraints ──────────────────────────────────────────

#[test]
fn lint_warns_on_duplicate_constraints() {
    let doc = Document {
        agent: Some(scalar("Test")),
        system: Some(scalar("You are helpful.")),
        constraints: Some(sequence(vec!["Be concise", "Be concise"])),
        ..Default::default()
    };
    let warnings = lint_document(&doc);
    assert!(warnings.iter().any(|w| w.code == "L006"));
}

#[test]
fn lint_no_l006_when_constraints_are_unique() {
    let doc = Document {
        agent: Some(scalar("Test")),
        system: Some(scalar("You are helpful.")),
        constraints: Some(sequence(vec!["Be concise", "Be polite"])),
        ..Default::default()
    };
    let warnings = lint_document(&doc);
    assert!(!warnings.iter().any(|w| w.code == "L006"));
}

// ── Clean document ───────────────────────────────────────────────────────

#[test]
fn lint_clean_document_has_no_warnings() {
    let doc = Document {
        agent: Some(scalar("Test")),
        system: Some(scalar(
            "You are a helpful assistant that answers questions clearly.",
        )),
        user: Some(scalar("What is the weather?")),
        output: Some(Node::Mapping {
            entries: vec![mapping_entry("temperature", "float")],
            span: Span::new(0, 0),
        }),
        ..Default::default()
    };
    let warnings = lint_document(&doc);
    assert!(
        warnings.is_empty(),
        "expected no warnings, got: {:?}",
        warnings
    );
}
