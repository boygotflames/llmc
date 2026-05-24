use super::{quote, vars};
use crate::ast::{Document, Node};

/// Emit a JSON payload compatible with the OpenAI Chat Completions API.
///
/// Mapped fields:
/// - `system` → messages[] with role "system"
/// - `user`   → messages[] with role "user"
/// - `tools`  → tools[] with simplified function objects
///
/// Omitted (honest limitations):
/// - `agent`       — metadata, not part of the API payload
/// - `memory`      — role ambiguity; deferred
/// - `constraints` — semantic mapping undefined; deferred
/// - `output`      — mapped to response_format in future release; deferred
/// - `vars`        — expanded at transpile time, not emitted
/// - `model`       — intentionally omitted; caller must inject it
pub fn emit(document: &Document) -> String {
    let expanded = vars::expand_document(document);
    let mut fields: Vec<String> = Vec::new();

    // Messages array
    let mut messages = Vec::new();
    if let Some(system) = expanded.system.as_ref() {
        messages.push(format!(
            "    {{\n      \"role\": \"system\",\n      \"content\": {}\n    }}",
            quote(&node_to_content(system))
        ));
    }
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
                    "    {{\n      \"type\": \"function\",\n      \"function\": {{\n        \"name\": {}\n      }}\n    }}",
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
