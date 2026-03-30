use serde::{Deserialize, Serialize};

/// A perspective template that transforms a base prompt through an analytical frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Perspective {
    pub id: String,
    pub label: String,
    pub instructions: String,
}

/// Built-in perspectives embedded at compile time.
const BUILT_INS: &[(&str, &str, &str)] = &[
    (
        "default",
        "Default",
        "Analyze this directly and thoroughly. Provide a clear, balanced assessment covering \
         strengths, weaknesses, risks, and recommendations. Do not ask follow-up questions — \
         make explicit assumptions and continue.",
    ),
    (
        "creative",
        "Creative",
        "Approach this from an unconventional angle. Look for non-obvious connections, \
         alternative framings, and creative solutions that a conventional analysis might miss. \
         Challenge standard assumptions. Do not ask follow-up questions — make explicit \
         assumptions and continue.",
    ),
    (
        "adversarial",
        "Adversarial",
        "Take a critical, adversarial stance. Actively search for flaws, vulnerabilities, \
         hidden risks, unstated assumptions, and failure modes. Assume the worst-case scenario \
         is plausible and explain why. Do not ask follow-up questions — make explicit assumptions \
         and continue.",
    ),
    (
        "performance",
        "Performance",
        "Focus on efficiency, scalability, and performance implications. Identify bottlenecks, \
         resource constraints, and optimization opportunities. Quantify where possible. Do not \
         ask follow-up questions — make explicit assumptions and continue.",
    ),
    (
        "devils-advocate",
        "Devil's Advocate",
        "Argue against the premise. Reframe the task from the strongest skeptical perspective. \
         Search for weak assumptions, hidden risks, edge cases, and likely failure modes. \
         Present the strongest case for why the proposed approach might be wrong or insufficient. \
         Do not ask follow-up questions — make explicit assumptions and continue.",
    ),
];

/// Load all built-in perspectives.
pub fn load_builtin_perspectives() -> Vec<Perspective> {
    BUILT_INS
        .iter()
        .map(|(id, label, instructions)| Perspective {
            id: id.to_string(),
            label: label.to_string(),
            instructions: instructions.to_string(),
        })
        .collect()
}

/// Get a specific perspective by ID from the built-in set.
pub fn get_perspective(id: &str) -> Option<Perspective> {
    load_builtin_perspectives().into_iter().find(|p| p.id == id)
}

/// Expand a base prompt with perspective instructions and optional context.
pub fn assemble_prompt(
    base_prompt: &str,
    _perspective: &Perspective,
    context: Option<&str>,
) -> String {
    let mut parts = Vec::new();

    if let Some(ctx) = context {
        if !ctx.is_empty() {
            parts.push(format!("<context>\n{ctx}\n</context>\n"));
        }
    }

    parts.push(base_prompt.to_string());

    // Perspective instructions go into the system prompt for Claude/Gemini,
    // or developer_instructions for Codex. This function assembles the user prompt only.
    // The perspective is injected by the provider adapter.
    parts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_perspectives_load() {
        let perspectives = load_builtin_perspectives();
        assert_eq!(perspectives.len(), 5);
        assert_eq!(perspectives[0].id, "default");
        assert_eq!(perspectives[4].id, "devils-advocate");
    }

    #[test]
    fn test_get_perspective_found() {
        let p = get_perspective("adversarial").unwrap();
        assert_eq!(p.label, "Adversarial");
        assert!(p.instructions.contains("critical"));
    }

    #[test]
    fn test_get_perspective_not_found() {
        assert!(get_perspective("nonexistent").is_none());
    }

    #[test]
    fn test_assemble_prompt_with_context() {
        let p = get_perspective("default").unwrap();
        let result = assemble_prompt("What is Rust?", &p, Some("Rust is a language."));
        assert!(result.contains("<context>"));
        assert!(result.contains("Rust is a language."));
        assert!(result.contains("What is Rust?"));
    }

    #[test]
    fn test_assemble_prompt_without_context() {
        let p = get_perspective("default").unwrap();
        let result = assemble_prompt("What is Rust?", &p, None);
        assert!(!result.contains("<context>"));
        assert!(result.contains("What is Rust?"));
    }
}
