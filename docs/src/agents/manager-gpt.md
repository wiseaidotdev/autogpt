# ManagerGPT

<span class="badge badge-orange">Feature: gpt</span> <span class="badge badge-gray">CLI: autogpt manage</span>

ManagerGPT is the top-level orchestrator in the AutoGPT agent mesh. It receives a high-level project goal, decomposes it into discrete subtasks, assigns each subtask to the appropriate specialist agent, and consolidates their outputs into a coherent final result.

## What ManagerGPT Solves

Complex software projects involve multiple domains, e.g. architecture, backend, frontend, design, that must be developed in a coordinated way. ManagerGPT eliminates the need to manually decompose tasks and coordinate specialists. You give it one goal; it handles the rest.

## How It Works

```
User Goal
    │
    ▼
ManagerGPT (decomposes)
    │
    ├──▶ ArchitectGPT  →  Architecture diagram
    ├──▶ BackendGPT    →  Server-side code
    ├──▶ FrontendGPT   →  UI code
    └──▶ DesignerGPT   →  UI mockups (optional)
    │
    ▼
ManagerGPT (consolidates)
    │
    ▼
Final Output to User
```

ManagerGPT communicates its decomposed subtasks to each agent by constructing specialized `Task` descriptions enriched with the original goal context. Each agent receives not just its slice of the work but enough context to make coherent decisions.

## CLI Usage

```sh
autogpt manage
```

AutoGPT prompts for your project goal interactively. For example:

```
> Develop a full stack app that fetches today's weather in Python using FastAPI.
```

ManagerGPT dispatches to ArchitectGPT, DesignerGPT, BackendGPT, and FrontendGPT. Terminal output:

```
[*] "ManagerGPT": Executing task: "Develop a full stack app that fetches today's weather in python using FastAPI."
[*] "ArchitectGPT": Executing tasks: Task { description: "- Design the user interface for the weather app..." }
[*] "BackendGPT": Executing tasks: Task { description: "- Using FastAPI in Python, create a backend..." }
[*] "FrontendGPT": Executing tasks: Task { description: "- Using FastAPI in Python, create a user interface..." }
[*] "ManagerGPT": Completed Task: Task { ... }
```

## SDK Usage

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let persona = "Project Manager";
    let behavior = "Develop a full-stack weather app in Python using FastAPI.";

    let agent = ManagerGPT::new(persona, behavior).await;

    AutoGPT::default()
        .with(agents![agent])
        .build()
        .expect("Failed to build AutoGPT")
        .run()
        .await
        .unwrap();
}
```

## Output

Generated code lands in the workspace directory, organized by agent:

```
workspace/
├── architect/     # Python diagrams code
├── backend/       # FastAPI backend
├── frontend/      # HTML/CSS/JS frontend
└── designer/      # UI mockup images (if img feature enabled)
```
