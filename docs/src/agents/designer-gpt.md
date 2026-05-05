# DesignerGPT

<span class="badge badge-blue">Feature: img</span> <span class="badge badge-gray">CLI: autogpt design</span>

DesignerGPT generates visual UI mockups and design assets using an AI image generation API ([GetImg](https://getimg.ai/)). It transforms a textual interface description into a rendered image that can be used as a design reference for frontend development.

## What DesignerGPT Solves

Getting sign-off on UI layout before any code is written saves significant rework. DesignerGPT generates visual mockups from natural language descriptions in seconds, enabling rapid design iteration without a designer in the loop.

## Enabling DesignerGPT

DesignerGPT requires the `img` feature flag and the `GETIMG_API_KEY` environment variable:

```sh
cargo install autogpt --features img,cli

export GETIMG_API_KEY=<your_getimg_api_key>
```

Obtain your API key from the [GetImg Dashboard](https://dashboard.getimg.ai/api-keys).

## CLI Usage

```sh
autogpt design
```

Describe the UI you want to generate when prompted:

```
> A dark-themed dashboard with a sidebar navigation, metrics cards, and a chart area
```

## SDK Usage

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let persona = "Senior UI/UX Designer";
    let behavior = "Design a modern dark-themed analytics dashboard with sidebar navigation.";

    let agent = DesignerGPT::new(persona, behavior).await;

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

Generated image files land in:

```
workspace/designer/
└── design_<timestamp>.png
```

<div class="callout callout-info">
<strong>ℹ️ Note</strong>
DesignerGPT is optional. When ManagerGPT runs without the <code>img</code> feature, it simply skips the design step and proceeds with architecture, backend, and frontend.
</div>
