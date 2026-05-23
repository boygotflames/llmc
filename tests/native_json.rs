use llm_format::ast::{Document, MappingEntry, Node};
use llm_format::diagnostics::Span;
use llm_format::transpile::{self, Target};

// ── helpers ──────────────────────────────────────────────────────────────

fn scalar_node(value: &str) -> Node {
    Node::Scalar {
        value: value.to_string(),
        span: Span::new(0, 0),
    }
}

fn sequence_node(items: Vec<&str>) -> Node {
    Node::Sequence {
        values: items.iter().map(|s| scalar_node(s)).collect(),
        span: Span::new(0, 0),
    }
}

fn mapping_entry(key: &str, value: &str) -> MappingEntry {
    MappingEntry::new(key, scalar_node(value), Span::new(0, 0))
}

// ── OpenAI Chat ──────────────────────────────────────────────────────────

#[test]
fn openai_chat_emits_messages_with_system_and_user() {
    let doc = Document {
        agent: Some(scalar_node("TestAgent")),
        system: Some(scalar_node("You are helpful.")),
        user: Some(scalar_node("Hello")),
        ..Default::default()
    };
    let out = transpile::transpile(&doc, Target::OpenAiChat);
    assert!(out.contains("\"messages\""), "must contain messages");
    assert!(
        out.contains("\"role\": \"system\""),
        "must have system role"
    );
    assert!(out.contains("\"role\": \"user\""), "must have user role");
    assert!(
        out.contains("You are helpful."),
        "must contain system content"
    );
    assert!(out.contains("Hello"), "must contain user content");
}

#[test]
fn openai_chat_mapping_system_becomes_plain_text() {
    let doc = Document {
        agent: Some(scalar_node("TestAgent")),
        system: Some(Node::Mapping {
            entries: vec![
                mapping_entry("role", "assistant"),
                mapping_entry("tone", "friendly"),
            ],
            span: Span::new(0, 0),
        }),
        user: Some(scalar_node("Hi")),
        ..Default::default()
    };
    let out = transpile::transpile(&doc, Target::OpenAiChat);
    assert!(out.contains("role: assistant"), "mapping key must appear");
    assert!(out.contains("tone: friendly"), "mapping value must appear");
}

#[test]
fn openai_chat_includes_simplified_tools() {
    let doc = Document {
        agent: Some(scalar_node("TestAgent")),
        system: Some(scalar_node("You help.")),
        tools: Some(sequence_node(vec!["web_search", "calculator"])),
        ..Default::default()
    };
    let out = transpile::transpile(&doc, Target::OpenAiChat);
    assert!(
        out.contains("\"type\": \"function\""),
        "must have function type"
    );
    assert!(
        out.contains("\"name\": \"web_search\""),
        "must have web_search"
    );
    assert!(
        out.contains("\"name\": \"calculator\""),
        "must have calculator"
    );
}

#[test]
fn openai_chat_omits_tools_when_absent() {
    let doc = Document {
        agent: Some(scalar_node("TestAgent")),
        system: Some(scalar_node("You help.")),
        user: Some(scalar_node("Hello")),
        ..Default::default()
    };
    let out = transpile::transpile(&doc, Target::OpenAiChat);
    assert!(!out.contains("\"tools\""), "no tools key when absent");
}

#[test]
fn openai_chat_omits_agent_memory_constraints_output() {
    let doc = Document {
        agent: Some(scalar_node("SecretAgent")),
        system: Some(scalar_node("You help.")),
        memory: Some(sequence_node(vec!["past_turn"])),
        constraints: Some(sequence_node(vec!["be concise"])),
        output: Some(Node::Mapping {
            entries: vec![mapping_entry("format", "json")],
            span: Span::new(0, 0),
        }),
        ..Default::default()
    };
    let out = transpile::transpile(&doc, Target::OpenAiChat);
    assert!(!out.contains("SecretAgent"), "agent must not appear");
    assert!(!out.contains("past_turn"), "memory must not appear");
    assert!(!out.contains("be concise"), "constraints must not appear");
    assert!(!out.contains("\"format\""), "output must not appear");
}

#[test]
fn openai_chat_empty_messages_when_no_system_or_user() {
    let doc = Document {
        agent: Some(scalar_node("TestAgent")),
        ..Default::default()
    };
    let out = transpile::transpile(&doc, Target::OpenAiChat);
    assert!(out.contains("\"messages\": []"), "empty messages array");
}

// ── Anthropic Messages ───────────────────────────────────────────────────

#[test]
fn anthropic_messages_system_is_top_level_string() {
    let doc = Document {
        agent: Some(scalar_node("TestAgent")),
        system: Some(scalar_node("You are helpful.")),
        user: Some(scalar_node("Hello")),
        ..Default::default()
    };
    let out = transpile::transpile(&doc, Target::AnthropicMessages);
    assert!(
        out.contains("\"system\": \"You are helpful.\""),
        "system at top level"
    );
    assert!(out.contains("\"role\": \"user\""), "user in messages");
    assert!(out.contains("Hello"), "user content present");
}

#[test]
fn anthropic_messages_mapping_system_becomes_plain_text() {
    let doc = Document {
        agent: Some(scalar_node("TestAgent")),
        system: Some(Node::Mapping {
            entries: vec![
                mapping_entry("role", "analyst"),
                mapping_entry("tone", "formal"),
            ],
            span: Span::new(0, 0),
        }),
        user: Some(scalar_node("Hi")),
        ..Default::default()
    };
    let out = transpile::transpile(&doc, Target::AnthropicMessages);
    assert!(out.contains("role: analyst"), "mapping key must appear");
    assert!(out.contains("tone: formal"), "mapping value must appear");
}

#[test]
fn anthropic_messages_includes_tools_with_placeholder_schema() {
    let doc = Document {
        agent: Some(scalar_node("TestAgent")),
        system: Some(scalar_node("You help.")),
        tools: Some(sequence_node(vec!["web_search"])),
        ..Default::default()
    };
    let out = transpile::transpile(&doc, Target::AnthropicMessages);
    assert!(
        out.contains("\"name\": \"web_search\""),
        "must have tool name"
    );
    assert!(out.contains("\"input_schema\""), "must have input_schema");
    assert!(
        out.contains("\"type\": \"object\""),
        "placeholder schema present"
    );
    assert!(
        out.contains("\"description\": \"\""),
        "empty description placeholder"
    );
}

#[test]
fn anthropic_messages_omits_tools_when_absent() {
    let doc = Document {
        agent: Some(scalar_node("TestAgent")),
        system: Some(scalar_node("You help.")),
        user: Some(scalar_node("Hello")),
        ..Default::default()
    };
    let out = transpile::transpile(&doc, Target::AnthropicMessages);
    assert!(!out.contains("\"tools\""), "no tools key when absent");
}

#[test]
fn anthropic_messages_omits_agent_memory_constraints_output() {
    let doc = Document {
        agent: Some(scalar_node("SecretAgent")),
        system: Some(scalar_node("You help.")),
        memory: Some(sequence_node(vec!["past_turn"])),
        constraints: Some(sequence_node(vec!["be concise"])),
        output: Some(Node::Mapping {
            entries: vec![mapping_entry("format", "json")],
            span: Span::new(0, 0),
        }),
        ..Default::default()
    };
    let out = transpile::transpile(&doc, Target::AnthropicMessages);
    assert!(!out.contains("SecretAgent"), "agent must not appear");
    assert!(!out.contains("past_turn"), "memory must not appear");
    assert!(!out.contains("be concise"), "constraints must not appear");
    assert!(!out.contains("\"format\""), "output must not appear");
}

#[test]
fn anthropic_messages_empty_messages_when_user_absent() {
    let doc = Document {
        agent: Some(scalar_node("TestAgent")),
        system: Some(scalar_node("You help.")),
        ..Default::default()
    };
    let out = transpile::transpile(&doc, Target::AnthropicMessages);
    assert!(out.contains("\"messages\": []"), "empty messages array");
}
