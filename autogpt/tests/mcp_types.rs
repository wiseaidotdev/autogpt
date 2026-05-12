// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # MCP types unit tests.

#![cfg(feature = "cli")]

use autogpt::mcp::types::{
    McpServerInfo, McpServerStatus, McpTool, McpToolCall, McpToolParam, McpToolResult,
};
use serde_json::json;
use std::collections::HashMap;

fn make_tool(name: &str, fqn: &str) -> McpTool {
    let mut params = HashMap::new();
    params.insert(
        "query".to_string(),
        McpToolParam {
            param_type: "string".to_string(),
            description: "Search query".to_string(),
            required: true,
            enum_values: vec![],
        },
    );
    params.insert(
        "limit".to_string(),
        McpToolParam {
            param_type: "integer".to_string(),
            description: "Max results".to_string(),
            required: false,
            enum_values: vec![],
        },
    );
    McpTool {
        name: name.to_string(),
        fqn: fqn.to_string(),
        description: format!("{name} does something useful"),
        params,
        validate_required: true,
    }
}

#[test]
fn server_status_display() {
    assert_eq!(McpServerStatus::Connected.to_string(), "CONNECTED");
    assert_eq!(McpServerStatus::Disconnected.to_string(), "DISCONNECTED");
    assert_eq!(McpServerStatus::Connecting.to_string(), "CONNECTING");
}

#[test]
fn server_status_roundtrip_serde() {
    for s in [
        McpServerStatus::Connected,
        McpServerStatus::Disconnected,
        McpServerStatus::Connecting,
    ] {
        let encoded = serde_json::to_string(&s).unwrap();
        let restored: McpServerStatus = serde_json::from_str(&encoded).unwrap();
        assert_eq!(restored, s);
    }
}

#[test]
fn tool_fqn_is_correct() {
    let tool = make_tool("search_code", "mcp_github_search_code");
    assert_eq!(tool.fqn, "mcp_github_search_code");
}

#[test]
fn tool_params_required_flag() {
    let tool = make_tool("search", "mcp_x_search");
    assert!(tool.params["query"].required);
    assert!(!tool.params["limit"].required);
}

#[test]
fn tool_roundtrip_serde() {
    let tool = make_tool("list_repos", "mcp_github_list_repos");
    let encoded = serde_json::to_string_pretty(&tool).unwrap();
    let restored: McpTool = serde_json::from_str(&encoded).unwrap();
    assert_eq!(restored.name, "list_repos");
    assert_eq!(restored.fqn, "mcp_github_list_repos");
    assert_eq!(restored.params.len(), 2);
}

#[test]
fn tool_with_enum_param() {
    let mut params = HashMap::new();
    params.insert(
        "order".to_string(),
        McpToolParam {
            param_type: "string".to_string(),
            description: "Sort order".to_string(),
            required: false,
            enum_values: vec![json!("asc"), json!("desc")],
        },
    );
    let tool = McpTool {
        name: "list_issues".to_string(),
        fqn: "mcp_github_list_issues".to_string(),
        description: "Lists issues".to_string(),
        params,
        validate_required: false,
    };
    assert_eq!(tool.params["order"].enum_values.len(), 2);
}

#[test]
fn tool_call_roundtrip_serde() {
    let mut args = HashMap::new();
    args.insert("query".to_string(), json!("rust async"));
    args.insert("limit".to_string(), json!(10));
    let call = McpToolCall {
        tool_fqn: "mcp_github_search_code".to_string(),
        args,
    };
    let encoded = serde_json::to_string_pretty(&call).unwrap();
    let restored: McpToolCall = serde_json::from_str(&encoded).unwrap();
    assert_eq!(restored.tool_fqn, "mcp_github_search_code");
    assert_eq!(restored.args["query"], json!("rust async"));
    assert_eq!(restored.args["limit"], json!(10));
}

#[test]
fn tool_result_success() {
    let result = McpToolResult {
        success: true,
        content: "Found 42 results".to_string(),
        data: None,
        error: None,
    };
    assert!(result.success);
    assert!(result.error.is_none());
}

#[test]
fn tool_result_failure_has_error() {
    let result = McpToolResult {
        success: false,
        content: String::new(),
        data: None,
        error: Some("Rate limit exceeded".to_string()),
    };
    assert!(!result.success);
    assert_eq!(result.error.as_deref(), Some("Rate limit exceeded"));
}

#[test]
fn tool_result_with_structured_data() {
    let result = McpToolResult {
        success: true,
        content: "ok".to_string(),
        data: Some(json!({"total": 5})),
        error: None,
    };
    assert_eq!(result.data.unwrap()["total"], json!(5));
}

#[test]
fn server_info_connected_no_error() {
    let info = McpServerInfo {
        name: "github".to_string(),
        status: McpServerStatus::Connected,
        description: "GitHub tools".to_string(),
        tools: vec![make_tool("search_code", "mcp_github_search_code")],
        error: None,
    };
    assert_eq!(info.tools.len(), 1);
    assert!(info.error.is_none());
}

#[test]
fn server_info_disconnected_with_error() {
    let info = McpServerInfo {
        name: "broken".to_string(),
        status: McpServerStatus::Disconnected,
        description: String::new(),
        tools: vec![],
        error: Some("Connection refused".to_string()),
    };
    assert!(info.tools.is_empty());
    assert_eq!(info.error.as_deref(), Some("Connection refused"));
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
