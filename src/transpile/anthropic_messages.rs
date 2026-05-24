use super::{quote, vars};
use crate::ast::{Document, Node};

/// Emit a JSON payload compatible with the Anthropic Messages API.
///
/// Mapped fields:
/// - `system` → top-level "system" string
/// - `user`   → messages[] with role "user"
/// - `tools`  → tools[] with placeholder description and input_schema
///
/// Omitted (honest limitations):
/// - `agent`       — metadata, not part of the API payload
/// - `memory`      — role ambiguity; deferred
/// - `constraints` — semantic mapping undefined; deferred
/// - `output`      — mapped to Anthropic output schema in future; deferred
/// - `vars`        — expanded at transpile time, not emitted
/// - `model`       — intentionally omitted; caller must inject it
/// - `max_tokens`  — intentionally omitted; caller must inject it
pub fn emit(document: &Document) -> String {
    let expanded = vars::expand_document(document);
    let mut fields: Vec<String> = Vec::new();

    // System (top-level string, not in messages)
    if let Some(system) = expanded.system.as_ref() {
        fields.push(format!("  \"system\": {}", quote(&node_to_content(system))));
    }

    // Messages array
    let mut messages = Vec::new();
    if let Some(user) = expanded.user.as_ref() {
        messages.push(format!(
            "    {{\n      \"role\": \"user\",\n      \"content\": {}\n    }}",
            quote(&node_to_content(user))
        ));
    }
    let messages_str = if messages.is_empty() {
        "  \"messages\": []".to_string()
    } else {
        format!("  \"messages\": [\n{}\n  ]", messages.join(",\n"))
    };
    fields.push(messages_str);

    // Tools
    if let Some(Node::Sequence { values, .. }) = expanded.tools.as_ref()
        && !values.is_empty()
    {
        let tool_lines: Vec<String> = values
            .iter()
            .map(|item| {
                let name = item.as_scalar().unwrap_or("unknown");
                format!(
                    "    {{\n      \"name\": {},\n      \"description\": \"\",\n      \"input_schema\": {{\n        \"type\": \"object\"\n      }}\n    }}",
                    quote(name)
                )
            })
            .collect();
        fields.push(format!("  \"tools\": [\n{}\n  ]", tool_lines.join(",\n")));
    }

    format!("{{\n{}\n}}", fields.join(",\n"))
}

fn node_to_content(node: &Node) -> String {
    match node {
        Node::Scalar { value, .. } => value.clone(),
        Node::Mapping { entries, .. } => entries
            .iter()
            .map(|e| format!("{}: {}", e.key, node_to_content(&e.value)))
            .collect::<Vec<_>>()
            .join("\n"),
        Node::Sequence { values, .. } => values
            .iter()
            .map(|v| format!("- {}", node_to_content(v)))
            .collect::<Vec<_>>()
            .join("\n"),
    }
}
