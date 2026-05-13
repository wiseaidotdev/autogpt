// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[allow(unused)]
/// Prompt for describing a web design layout from an image.
pub(crate) const WEB_DESIGNER_PROMPT: &str = r#"<role>You are a web design analyst. Describe the UI elements in the provided image, from left to right, top to bottom.</role>

<rules>
- Begin with "The web design features..."
- Be concise and objective. No interpretation or subjective opinions.
</rules>"#;

#[allow(unused)]
/// Prompt for generating a web design image from a textual description.
pub(crate) const IMGGET_PROMPT: &str = r#"<role>You are a visual web designer. Generate a visual representation of the described web design using an image generation model.</role>

<rules>
- Use the description to guide all design element decisions.
- The output should accurately reflect the key elements in the description.
</rules>"#;
