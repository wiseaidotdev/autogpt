# ArchitectGPT

<span class="badge badge-orange">Feature: gpt</span> <span class="badge badge-gray">CLI: autogpt arch</span>

ArchitectGPT translates a high-level project description into a concrete system architecture. It determines the technologies, frameworks, and communication patterns needed, then generates an executable Python script using the [`diagrams`](https://github.com/mingrammer/diagrams) library that renders a PNG architecture diagram.

## What ArchitectGPT Solves

Software architecture decisions are typically made informally and documented inconsistently. ArchitectGPT externalizes this process: given a goal, it produces a versioned, reproducible visual diagram that serves as the authoritative architectural reference for all other agents.

## How It Works

1. ArchitectGPT receives a project goal from ManagerGPT or directly from the user
2. It calls the configured LLM to generate a Python script using the `diagrams` library
3. The script is written to `workspace/architect/diagram.py`
4. A Python virtual environment is set up with `diagrams` installed in `workspace/architect/.venv/`
5. Running the script produces a PNG in that directory

## CLI Usage

```sh
autogpt arch
```

Example session:

```
> Generate a Kubernetes architecture for a web app with Prometheus and Grafana monitoring.
```

After the agent completes:

```sh
# Render the diagram
./workspace/architect/.venv/bin/python ./workspace/architect/diagram.py
# ➜  simple_web_application_on_kubernetes.png created
```

## SDK Usage

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let persona = "Lead UX/UI Designer";
    let behavior = r#"Generate a diagram for a simple web application running on Kubernetes.
    It consists of a single Deployment with 2 replicas, a Service to expose the Deployment,
    and an Ingress to route external traffic. Also include a basic monitoring setup
    with Prometheus and Grafana."#;

    let agent = ArchitectGPT::new(persona, behavior).await;

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

```
workspace/architect/
├── .venv/               # Python venv with diagrams installed
├── diagram.py           # Generated diagrams script
└── *.png                # Rendered architecture diagram
```

<div class="callout callout-info">
<strong>ℹ️ Note</strong>
Python 3.12+ and Graphviz must be installed on the host for diagram rendering. AutoGPT creates the venv automatically but Graphviz must be available in PATH.
</div>
