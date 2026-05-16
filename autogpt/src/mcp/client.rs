// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # MCP client.
//!
//! Provides a lightweight, transport-agnostic MCP client that can communicate with
//! stdio-based, SSE, and streamable-HTTP MCP servers.  The client establishes a
//! connection, discovers available tools, and executes individual tool calls.

use {
    crate::mcp::settings::{McpServerConfig, McpTransport},
    crate::mcp::types::{McpServerInfo, McpServerStatus, McpTool, McpToolParam, McpToolResult},
    anyhow::{Context, Result, anyhow},
    serde::{Deserialize, Serialize},
    serde_json::{Value, from_str, json, to_string},
    std::collections::HashMap,
    std::env::{var, vars},
    std::io::{BufRead, BufReader, Write},
    std::process::{Child, ChildStdin, Command, Stdio},
    std::sync::mpsc::{self, Receiver},
    std::thread::spawn,
    std::time::Duration,
};

/// JSON-RPC 2.0 request envelope.
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    params: Value,
}

/// JSON-RPC 2.0 response envelope.
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    id: Option<Value>,
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

/// JSON-RPC error object.
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

/// State carried by an active stdio transport connection.
struct StdioTransport {
    child: Child,
    stdin: ChildStdin,
    rx: Receiver<String>,
    timeout: Duration,
    next_id: u64,
}

impl StdioTransport {
    /// Spawns the server process and performs the MCP `initialize` handshake.
    fn connect(config: &McpServerConfig) -> Result<Self> {
        let cmd = config
            .command
            .as_deref()
            .ok_or_else(|| anyhow!("stdio transport requires a 'command'"))?;

        let mut builder = Command::new(cmd);
        builder
            .args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        builder.env_clear();
        for (k, v) in vars() {
            if !should_redact(&k) {
                builder.env(k, v);
            }
        }

        if let Some(ref cwd) = config.cwd {
            builder.current_dir(cwd);
        }

        for (k, v) in &config.env {
            builder.env(k, expand_env_vars(v));
        }

        let mut child = builder
            .spawn()
            .with_context(|| format!("Spawning MCP server process: {cmd}"))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("No stdin on child process"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("No stdout on child process"))?;
        let timeout = Duration::from_millis(config.timeout_ms);
        let (tx, rx) = mpsc::channel();

        spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        if tx.send(line).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let mut transport = Self {
            child,
            stdin,
            rx,
            timeout,
            next_id: 0,
        };
        transport.initialize()?;
        Ok(transport)
    }

    fn next_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    /// Sends a JSON-RPC request and reads the next response via the background channel.
    fn call(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id();
        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id,
            method: method.to_string(),
            params,
        };
        let mut line = to_string(&req).context("Serializing JSON-RPC request")?;
        line.push('\n');
        self.stdin
            .write_all(line.as_bytes())
            .context("Writing to MCP server stdin")?;
        self.stdin.flush().context("Flushing MCP server stdin")?;

        loop {
            let response_line = self
                .rx
                .recv_timeout(self.timeout)
                .map_err(|_| anyhow!("MCP server timed out after {:?}", self.timeout))?;

            if response_line.trim().is_empty() {
                continue;
            }

            let resp: JsonRpcResponse = match from_str(response_line.trim()) {
                Ok(r) => r,
                Err(_) => continue,
            };

            if let Some(resp_id) = resp.id.as_ref()
                && resp_id.as_u64() == Some(id)
            {
                if let Some(err) = resp.error {
                    return Err(anyhow!("MCP error {}: {}", err.code, err.message));
                }
                return resp
                    .result
                    .ok_or_else(|| anyhow!("JSON-RPC response missing 'result'"));
            }
        }
    }

    /// Sends the `initialize` notification required by the MCP protocol.
    fn initialize(&mut self) -> Result<()> {
        let _ = self.call(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "autogpt", "version": env!("CARGO_PKG_VERSION") }
            }),
        )?;
        let notif =
            json!({ "jsonrpc": "2.0", "method": "notifications/initialized", "params": {} });
        let mut line = to_string(&notif).context("Serializing initialized notification")?;
        line.push('\n');
        self.stdin
            .write_all(line.as_bytes())
            .context("Writing initialized notification")?;
        self.stdin
            .flush()
            .context("Flushing initialized notification")?;
        Ok(())
    }

    fn list_tools(&mut self) -> Result<Vec<McpTool>> {
        let result = self.call("tools/list", json!({}))?;
        parse_tool_list(&result)
    }

    fn call_tool(
        &mut self,
        tool_name: &str,
        args: &HashMap<String, Value>,
    ) -> Result<McpToolResult> {
        let result = self.call(
            "tools/call",
            json!({ "name": tool_name, "arguments": args }),
        )?;
        parse_tool_result(result)
    }
}

impl Drop for StdioTransport {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

/// Expands `$VAR`, `${VAR}`, and `%VAR%` references inside a string using the current environment.
fn expand_env_vars(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            if let Some(&'{') = chars.peek() {
                chars.next();
                let mut var_name = String::new();
                for nc in chars.by_ref() {
                    if nc == '}' {
                        break;
                    }
                    var_name.push(nc);
                }
                result.push_str(&var(&var_name).unwrap_or_default());
            } else {
                let mut var_name = String::new();
                while let Some(&nc) = chars.peek() {
                    if nc.is_alphanumeric() || nc == '_' {
                        var_name.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if !var_name.is_empty() {
                    result.push_str(&var(&var_name).unwrap_or_default());
                } else {
                    result.push('$');
                }
            }
        } else if c == '%' {
            let mut var_name = String::new();
            let mut closed = false;
            for nc in chars.by_ref() {
                if nc == '%' {
                    closed = true;
                    break;
                }
                var_name.push(nc);
            }
            if closed && !var_name.is_empty() {
                result.push_str(&var(&var_name).unwrap_or_default());
            } else {
                result.push('%');
                result.push_str(&var_name);
                if closed {
                    result.push('%');
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Checks if an environment variable key is sensitive and should be redacted by default.
fn should_redact(key: &str) -> bool {
    let key = key.to_uppercase();
    key.contains("API_KEY")
        || key.contains("TOKEN")
        || key.contains("SECRET")
        || key.contains("PASSWORD")
        || key.contains("AUTH")
        || key.contains("CREDENTIAL")
        || key == "GOOGLE_APPLICATION_CREDENTIALS"
}

/// Parses the `tools/list` result payload into a vector of `McpTool` descriptors.
fn parse_tool_list(value: &Value) -> Result<Vec<McpTool>> {
    let tools_arr = value
        .get("tools")
        .and_then(|t| t.as_array())
        .ok_or_else(|| anyhow!("tools/list response missing 'tools' array"))?;

    let mut tools = Vec::with_capacity(tools_arr.len());
    for t in tools_arr {
        let name = t
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        let description = t
            .get("description")
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .to_string();

        let mut params: HashMap<String, McpToolParam> = HashMap::new();
        if let Some(schema) = t.get("inputSchema")
            && let Some(props) = schema.get("properties").and_then(|p| p.as_object())
        {
            let required_fields: Vec<&str> = schema
                .get("required")
                .and_then(|r| r.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                .unwrap_or_default();

            for (pname, pschema) in props {
                params.insert(
                    pname.clone(),
                    McpToolParam {
                        param_type: pschema
                            .get("type")
                            .and_then(|t| t.as_str())
                            .unwrap_or("string")
                            .to_string(),
                        description: pschema
                            .get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("")
                            .to_string(),
                        required: required_fields.contains(&pname.as_str()),
                        enum_values: pschema
                            .get("enum")
                            .and_then(|e| e.as_array())
                            .cloned()
                            .unwrap_or_default(),
                    },
                );
            }
        }

        tools.push(McpTool {
            fqn: name.clone(),
            name,
            description,
            params,
            validate_required: false,
        });
    }
    Ok(tools)
}

/// Parses a `tools/call` result payload into a `McpToolResult`.
fn parse_tool_result(value: Value) -> Result<McpToolResult> {
    let is_error = value
        .get("isError")
        .and_then(|e| e.as_bool())
        .unwrap_or(false);
    let content_arr = value
        .get("content")
        .and_then(|c| c.as_array())
        .cloned()
        .unwrap_or_default();

    let mut text_parts: Vec<String> = Vec::new();
    for block in &content_arr {
        if block.get("type").and_then(|t| t.as_str()) == Some("text")
            && let Some(t) = block.get("text").and_then(|t| t.as_str())
        {
            text_parts.push(t.to_string());
        }
    }
    let content = text_parts.join("\n");

    Ok(McpToolResult {
        success: !is_error,
        content: content.clone(),
        data: if content_arr.len() > 1 {
            Some(Value::Array(content_arr))
        } else {
            None
        },
        error: if is_error { Some(content) } else { None },
    })
}

/// Performs an HTTP or SSE tool call via the MCP streamable-HTTP transport.
fn http_tool_call(
    base_url: &str,
    headers: &HashMap<String, String>,
    tool_name: &str,
    args: &HashMap<String, Value>,
    timeout: Duration,
) -> Result<McpToolResult> {
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
        .context("Building reqwest blocking client")?;

    let body = json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": tool_name, "arguments": args }
    });

    let mut req = client.post(base_url).json(&body);
    for (k, v) in headers {
        req = req.header(k.as_str(), v.as_str());
    }

    let response = req.send().context("Sending HTTP request to MCP server")?;
    let status = response.status();
    let text = response
        .text()
        .context("Reading MCP server HTTP response body")?;

    if !status.is_success() {
        return Err(anyhow!("MCP server returned HTTP {status}: {text}"));
    }

    let resp: JsonRpcResponse = from_str(&text).context("Parsing MCP HTTP JSON-RPC response")?;
    if let Some(err) = resp.error {
        return Err(anyhow!("MCP error {}: {}", err.code, err.message));
    }
    parse_tool_result(resp.result.unwrap_or(Value::Null))
}

/// Performs an HTTP `tools/list` discovery request.
fn http_list_tools(
    base_url: &str,
    headers: &HashMap<String, String>,
    timeout: Duration,
) -> Result<Vec<McpTool>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
        .context("Building reqwest blocking client for tool discovery")?;

    let body = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {} });
    let mut req = client.post(base_url).json(&body);
    for (k, v) in headers {
        req = req.header(k.as_str(), v.as_str());
    }

    let response = req.send().context("Sending tools/list HTTP request")?;
    let text = response
        .text()
        .context("Reading tools/list HTTP response body")?;
    let resp: JsonRpcResponse =
        from_str(&text).context("Parsing tools/list HTTP JSON-RPC response")?;
    if let Some(err) = resp.error {
        return Err(anyhow!("MCP error {}: {}", err.code, err.message));
    }
    parse_tool_list(&resp.result.unwrap_or(Value::Null))
}

/// The live handle kept for a successfully connected MCP server.
enum Transport {
    Stdio(StdioTransport),
    Http {
        base_url: String,
        headers: HashMap<String, String>,
        timeout: Duration,
    },
}

impl Transport {
    fn list_tools(&mut self) -> Result<Vec<McpTool>> {
        match self {
            Self::Stdio(t) => t.list_tools(),
            Self::Http {
                base_url,
                headers,
                timeout,
            } => http_list_tools(base_url, headers, *timeout),
        }
    }

    fn call_tool(
        &mut self,
        tool_name: &str,
        args: &HashMap<String, Value>,
    ) -> Result<McpToolResult> {
        match self {
            Self::Stdio(t) => t.call_tool(tool_name, args),
            Self::Http {
                base_url,
                headers,
                timeout,
            } => http_tool_call(base_url, headers, tool_name, args, *timeout),
        }
    }
}

/// A connected client for a single MCP server.
///
/// One `McpClient` is created per configured server during the discovery phase.
/// After `connect()` succeeds, discovered tools are stored in `tools` and the
/// transport stays alive for subsequent `call_tool` invocations.
pub struct McpClient {
    /// Server name matching the key in `settings.mcp`.
    pub name: String,
    /// Current connection status.
    pub status: McpServerStatus,
    /// Tools discovered from this server. Empty until `connect()` succeeds.
    pub tools: Vec<McpTool>,
    /// Live error message from the last failed connection attempt.
    pub error: Option<String>,
    transport: Option<Transport>,
}

impl McpClient {
    /// Creates a new client shell; call [`connect`](Self::connect) to establish the session.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: McpServerStatus::Disconnected,
            tools: Vec::new(),
            error: None,
            transport: None,
        }
    }

    /// Establishes a connection according to the server's transport type, then discovers tools.
    pub fn connect(&mut self, config: &McpServerConfig) -> Result<()> {
        self.status = McpServerStatus::Connecting;
        self.error = None;

        let mut transport = match config.transport {
            McpTransport::Stdio => {
                Transport::Stdio(StdioTransport::connect(config).inspect_err(|e| {
                    self.status = McpServerStatus::Disconnected;
                    self.error = Some(e.to_string());
                })?)
            }
            McpTransport::Http | McpTransport::Sse => {
                let url = if config.transport == McpTransport::Http {
                    config.http_url.clone().or_else(|| config.url.clone())
                } else {
                    config.url.clone()
                }
                .ok_or_else(|| anyhow!("HTTP/SSE transport requires 'http_url' or 'url'"))?;

                let headers = config
                    .headers
                    .iter()
                    .map(|(k, v)| (k.clone(), expand_env_vars(v)))
                    .collect();
                Transport::Http {
                    base_url: url,
                    headers,
                    timeout: Duration::from_millis(config.timeout_ms),
                }
            }
        };

        match transport.list_tools() {
            Ok(mut tools) => {
                let server_prefix = sanitize_name(&self.name);
                for tool in &mut tools {
                    tool.fqn = format!("mcp_{server_prefix}_{}", sanitize_name(&tool.name));
                    if config.trust {
                        tool.validate_required = false;
                    }
                }
                tools.retain(|t| {
                    let included =
                        config.include_tools.is_empty() || config.include_tools.contains(&t.name);
                    let excluded = config.exclude_tools.contains(&t.name);
                    included && !excluded
                });
                self.tools = tools;
                self.status = McpServerStatus::Connected;
                self.transport = Some(transport);
                Ok(())
            }
            Err(e) => {
                self.status = McpServerStatus::Disconnected;
                self.error = Some(e.to_string());
                Err(e)
            }
        }
    }

    /// Calls a tool by its original server-side name (not its FQN).
    pub fn call_tool(
        &mut self,
        tool_name: &str,
        args: HashMap<String, Value>,
    ) -> Result<McpToolResult> {
        let transport = self
            .transport
            .as_mut()
            .ok_or_else(|| anyhow!("MCP server '{}' is not connected", self.name))?;
        transport.call_tool(tool_name, &args)
    }

    /// Converts this client to a summary `McpServerInfo` struct.
    pub fn to_server_info(&self, description: &str) -> McpServerInfo {
        McpServerInfo {
            name: self.name.clone(),
            status: self.status,
            description: description.to_string(),
            tools: self.tools.clone(),
            error: self.error.clone(),
        }
    }
}

/// Replaces characters not in `[a-zA-Z0-9_\-]` with underscores.
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
