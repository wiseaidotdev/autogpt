// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)]

/// Prompt for generating backend server code for any language and framework.
pub(crate) const WEBSERVER_CODE_PROMPT: &str = r#"<role>You are a senior backend engineer. Generate complete, production-ready server code in the requested language and framework.</role>

<rules>
- Generate all code in a single self-contained module. Do not import from sibling modules or relative paths.
- Base your implementation on the provided code template; modify it to match the project description.
- Output only raw source code. No backticks, no fences, no commentary.
</rules>

<examples>
Input - project: "REST API for managing tasks", template: "async fn create_task() -> impl IntoResponse {}"
Output (Python/FastAPI):
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import List
import uuid

app = FastAPI()
tasks = {}

class Task(BaseModel):
    title: str
    done: bool = False

@app.post("/tasks", status_code=201)
def create_task(task: Task):
    tid = str(uuid.uuid4())
    tasks[tid] = task.dict()
    return {"id": tid, **tasks[tid]}

@app.get("/tasks/{task_id}")
def get_task(task_id: str):
    if task_id not in tasks:
        raise HTTPException(status_code=404, detail="Not found")
    return {"id": task_id, **tasks[task_id]}

Output (Rust/Axum):
use axum::{Router, Json, extract::Path, http::StatusCode, routing::{get, post}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task { title: String, done: bool }

type Store = Arc<Mutex<HashMap<String, Task>>>;

#[tokio::main]
async fn main() {
    let store: Store = Arc::new(Mutex::new(HashMap::new()));
    let app = Router::new()
        .route("/tasks", post(create_task))
        .route("/tasks/:id", get(get_task))
        .with_state(store);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn create_task(axum::extract::State(store): axum::extract::State<Store>, Json(task): Json<Task>) -> (StatusCode, Json<serde_json::Value>) {
    let id = uuid::Uuid::new_v4().to_string();
    store.lock().unwrap().insert(id.clone(), task.clone());
    (StatusCode::CREATED, Json(serde_json::json!({"id": id, "title": task.title, "done": task.done})))
}

async fn get_task(axum::extract::State(store): axum::extract::State<Store>, Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let store = store.lock().unwrap();
    store.get(&id).map(|t| Json(serde_json::json!({"id": id, "title": t.title, "done": t.done}))).ok_or(StatusCode::NOT_FOUND)
}
</examples>

<context>
<project>{TASK_DESCRIPTION}</project>
<template>{CODE_TEMPLATE}</template>
</context>"#;

/// Prompt for improving existing backend code.
pub(crate) const IMPROVED_WEBSERVER_CODE_PROMPT: &str = r#"<role>You are a senior backend engineer. Improve the provided backend code.</role>

<rules>
- Fix any bugs and add any missing functionality required by the project description.
- Keep all code in one self-contained module. No relative imports.
- Output only raw source code. No backticks, no fences, no commentary.
</rules>

<context>
<project>{TASK_DESCRIPTION}</project>
<current_code>{CODE_TEMPLATE}</current_code>
</context>"#;

/// Prompt for fixing bugs in backend code.
pub(crate) const FIX_CODE_PROMPT: &str = r#"<role>You are a senior backend engineer. Fix the bugs in the provided code.</role>

<rules>
- Fix all identified bugs. Do not add unrelated changes.
- Output only the corrected source code. No backticks, no fences, no commentary.
</rules>"#;

/// Prompt for extracting REST API endpoint definitions as JSON schema.
pub(crate) const API_ENDPOINTS_PROMPT: &str = r#"<role>You are an API schema extractor. Analyze the provided backend source code and produce JSON schema representations for each REST endpoint.</role>

<schema>
Return a JSON array. Each element contains:
- "route": URL path string
- "dynamic": "true" or "false"
- "method": HTTP method (lowercase)
- "body": object describing request body fields and types (empty object if none)
- "response": string describing the response type
</schema>

<rules>
- All values must be strings (including boolean values).
- Output only the JSON array. No backticks, no commentary.
</rules>

<examples>
Input (Python/FastAPI):
@app.get("/")
async def root(): return {"message": "Hello World"}

@app.get("/weather")
async def get_weather(city: str): ...

Output:
[{"route":"/","dynamic":"false","method":"get","body":{},"response":"object"},{"route":"/weather","dynamic":"false","method":"get","body":{"city":"string"},"response":"object"}]

Input (Rust/Axum):
fn all_routes() -> Router {
    Router::new()
        .route("/gems/generate-content", post(generate_content))
        .route("/gems/count-tokens", post(count_tokens))
}
#[derive(Deserialize)]
struct GenerateContentRequest { input_text: String }

Output:
[{"route":"/gems/generate-content","dynamic":"false","method":"post","body":{"input_text":"string"},"response":"string"},{"route":"/gems/count-tokens","dynamic":"false","method":"post","body":{"input_text":"string"},"response":"string"}]
</examples>

<source_code>{BACKEND_CODE}</source_code>"#;

/// Prompt for determining environment setup commands and entry point for any requested backend language.
pub(crate) const ENV_SETUP_PROMPT: &str = r#"<role>You are a senior DevOps and backend architect. Given a programming language, output the shell commands to scaffold a new project and the relative path to the primary source entry file.</role>

<schema>
Output ONLY this raw JSON object - no markdown fences, no backticks, no commentary, no extra text before or after:
{"commands":["command1","command2"],"entry_point":"path/to/main/file"}
</schema>

<rules>
- Commands must be non-interactive and suitable for a Linux shell.
- Use standard, minimalist scaffolding only.
- For Python: create a .venv with `python3 -m venv .venv`, then install with `.venv/bin/pip install -r requirements.txt` if requirements.txt exists.
- For Rust: use `cargo init`.
- For JavaScript/TypeScript: use `npm init -y`.
- For Go: use `go mod init app`.
- For Java: use `mvn archetype:generate -DgroupId=com.app -DartifactId=app -DarchetypeArtifactId=maven-archetype-quickstart -DinteractiveMode=false`.
- For unknown languages: create a `src/` directory and return `src/main` as the entry point.
- The entry_point must be a relative file path (from the workspace root) to the main server file.
- CRITICAL: output ONLY the raw JSON. Starting character must be `{`. Ending character must be `}`. No other wrapping.
</rules>

<examples>
language: python
{"commands":["python3 -m venv .venv","touch requirements.txt"],"entry_point":"main.py"}

language: rust
{"commands":["cargo init"],"entry_point":"src/main.rs"}

language: javascript
{"commands":["npm init -y"],"entry_point":"src/index.js"}

language: typescript
{"commands":["npm init -y","npm install typescript @types/node --save-dev"],"entry_point":"src/index.ts"}

language: go
{"commands":["go mod init app"],"entry_point":"main.go"}

language: java
{"commands":["mvn archetype:generate -DgroupId=com.app -DartifactId=app -DarchetypeArtifactId=maven-archetype-quickstart -DinteractiveMode=false"],"entry_point":"app/src/main/java/com/app/App.java"}

language: ruby
{"commands":["bundle init"],"entry_point":"app.rb"}

language: php
{"commands":["composer init --no-interaction"],"entry_point":"index.php"}
</examples>

<language>{LANGUAGE}</language>"#;
