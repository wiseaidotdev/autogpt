# GitGPT

<span class="badge badge-blue">Feature: git</span>

GitGPT automates version control by creating atomic Git commits whenever an agent completes a task. It monitors agent status, stages the entire workspace, and commits with a structured message that traces the commit back to the original user goal and agent role.

## What GitGPT Solves

In autonomous or quasi-autonomous workflows, tracking what changed and why is difficult without discipline. GitGPT enforces a clean, auditable Git history automatically, every agent action becomes a commit with a descriptive, machine-generated message.

## Enabling GitGPT

```sh
cargo install autogpt --features git,gem,gpt
```

`AUTOGPT_WORKSPACE` must point to a directory that is a Git repository (or within one).

## How It Works

GitGPT runs after each agent completes. It:

1. Checks if the agent status is `Completed`
2. Stages all modified files (`git add .`) in the workspace
3. Computes a structured commit message from the task description and agent role
4. Creates a signed commit using `GitGPT <gitgpt@wiseai.dev>` as the author

## Example Commit Message

```
commit cc448377fae752ba28847c873751ba1170d19fc0 (HEAD -> master)
Author: GitGPT <gitgpt@wiseai.dev>
Date: Mon Apr 7 00:25:59 2025 +0300

    User Request: Project Goal: "Develop a patient management system
    encompassing appointment scheduling, patient records, and billing features.",
    Agent Role: "frontend", programming language: "Python", framework: "FastAPI"
    Output:
    - Implementing FastAPI, construct a frontend in Python for a patient
      management system encompassing appointment scheduling, patient records,
      and billing features.
    - Step 1: Develop the user interface components for appointment scheduling.
    - Step 2: Build the UI for managing and displaying patient records.
    - Step 3: Create the billing interface for generating and displaying invoices.
```

## SDK Usage

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let backend = BackendGPT::new(
        "Backend Developer",
        "Generate a REST API for a blog in Python using FastAPI.",
        "python",
    ).await;

    let git_agent = GitGPT::new("GitGPT", "Commit backend changes").await;

    AutoGPT::default()
        .with(agents![backend, git_agent])
        .build()
        .expect("Failed to build AutoGPT")
        .run()
        .await
        .unwrap();
}
```

<div class="callout callout-tip">
<strong>💡 Tip</strong>
Initialize a Git repository in your workspace directory before running AutoGPT with GitGPT enabled:
<code>git init workspace/ && git -C workspace/ commit --allow-empty -m "init"</code>
</div>
