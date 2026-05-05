# FrontendGPT

<span class="badge badge-orange">Feature: gpt</span> <span class="badge badge-gray">CLI: autogpt front</span>

FrontendGPT generates complete frontend user interface code for web applications. It produces HTML, CSS, JavaScript, and framework-specific components, responsive by default and styled appropriately for the described use case.

## What FrontendGPT Solves

Translating a feature description into a working, well-structured UI requires decisions about layout, component hierarchy, data binding, and styling. FrontendGPT makes those decisions automatically, generating a complete starting point rather than skeleton files.

## Supported Stacks

FrontendGPT accepts the programming language / framework as a string:

| Value        | Output                                            |
| ------------ | ------------------------------------------------- |
| `javascript` | Vanilla HTML/CSS/JS or React/Vue based on context |
| `react`      | React component tree with JSX                     |
| `python`     | Jinja2 HTML templates (for FastAPI/Flask)         |
| `rust`       | Yew WASM components                               |

## CLI Usage

```sh
autogpt front
```

## SDK Usage

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let persona = "UX/UI Designer";
    let behavior = "Generate UI for a weather app using React JS.";

    let agent = FrontendGPT::new(persona, behavior, "javascript").await;

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
workspace/frontend/
├── main.py         # (Python) Template renderer entry point
├── template.py     # (Python) Jinja2 templates
└── ...             # HTML/CSS/JS files for JS-based projects
```

## Pairing with BackendGPT

FrontendGPT is aware of the backend API schema when run via ManagerGPT. This means the generated frontend code calls the correct API endpoints defined by BackendGPT, producing a cohesive full-stack result.
