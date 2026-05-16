use {
    crate::prompts::generic::INTENT_DETECTION_PROMPT,
    anyhow::Result,
    serde::{Deserialize, Serialize},
};

/// The classified intent of a single user message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum AgentIntent {
    DirectAnswer,
    ToolCall {
        tool: String,
        #[serde(default)]
        args: serde_json::Value,
    },
    TaskPlan,
}

/// Parsed JSON shape from the LLM for intent detection.
#[derive(Debug, Deserialize)]
struct IntentResponse {
    intent: String,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    args: Option<serde_json::Value>,
}

/// Type alias for the LLM generator function.
pub type GenerateFn =
    dyn FnMut(&str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String>> + Send>>;

/// Classifies the user's message into one of three execution modes.
pub async fn classify_intent(
    prompt: &str,
    workspace_snapshot: &str,
    mcp_tools: &str,
    generate_fn: &mut GenerateFn,
) -> AgentIntent {
    let full = INTENT_DETECTION_PROMPT
        .replace("{USER_PROMPT}", prompt)
        .replace("{WORKSPACE}", workspace_snapshot)
        .replace("{MCP_TOOLS}", mcp_tools);

    let raw = match generate_fn(&full).await {
        Ok(r) => r,
        Err(_) => return AgentIntent::TaskPlan,
    };

    let clean = crate::common::utils::strip_code_blocks(&raw);
    let parsed: IntentResponse = match serde_json::from_str(clean.trim()) {
        Ok(p) => p,
        Err(_) => return AgentIntent::TaskPlan,
    };

    match parsed.intent.as_str() {
        "direct_answer" => AgentIntent::DirectAnswer,
        "tool_call" => AgentIntent::ToolCall {
            tool: parsed.tool.unwrap_or_else(|| "list_dir".to_string()),
            args: parsed.args.unwrap_or(serde_json::Value::Null),
        },
        _ => AgentIntent::TaskPlan,
    }
}
