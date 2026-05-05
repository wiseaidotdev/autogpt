# BackendGPT

<span class="badge badge-orange">Feature: gpt</span> <span class="badge badge-gray">CLI: autogpt back</span>

BackendGPT specializes in generating production-ready server-side code. Given a project goal and a target language, it generates complete backend implementations including API endpoints, data models, database integrations, authentication logic, and dependency configurations.

## What BackendGPT Solves

Writing boilerplate backend code, e.g. route handlers, middleware, schema definitions, Docker configurations is time-consuming and predictable. BackendGPT generates this scaffolding instantly, correctly structured for the chosen language and framework, so you can focus on business logic.

## Supported Languages

BackendGPT accepts any programming language string. The LLM adapts its output accordingly. Common examples:

| Language     | Typical Output                              |
| ------------ | ------------------------------------------- |
| `rust`       | Axum/Actix-web server with Cargo.toml       |
| `python`     | FastAPI or Flask app with requirements.txt  |
| `javascript` | Express.js or Fastify app with package.json |

## CLI Usage

```sh
autogpt back
```

You will be prompted for a language and project goal.

## SDK Usage

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let persona = "Backend Developer";
    let behavior = "Develop a weather backend API in Rust using axum.";

    let agent = BackendGPT::new(persona, behavior, "rust").await;

    AutoGPT::default()
        .with(agents![agent])
        .build()
        .expect("Failed to build AutoGPT")
        .run()
        .await
        .unwrap();
}
```

The third argument to `BackendGPT::new` is the target programming language.

## Output

```
workspace/backend/
├── main.py        # (Python) Application entry point
├── template.py    # (Python) Template/model definitions
└── ...            # Additional files based on the project
```

For Rust projects the generated files are placed as a complete Cargo project structure.

## Retry Behavior

BackendGPT attempts to compile and validate the generated code when `execute` is enabled. If compilation fails it automatically retries up to `max_tries` times, passing the compiler error back to the LLM for correction.

```rust
AutoGPT::default()
    .with(agents![agent])
    .max_tries(3)   // retry up to 3 times on compile failure
    .build()
    .unwrap()
    .run()
    .await
    .unwrap();
```
