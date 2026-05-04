// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use autogpt::agents::agent::AgentGPT;
use autogpt::common::utils::{Message, Status};
use autogpt::traits::agent::Agent;
use std::borrow::Cow;

#[test]
fn test_create_agent() {
    let persona = "Lead UX/UI Designer";
    let behavior = "Creates innovative website designs and user experiences";

    let agent = AgentGPT::new_borrowed(persona, behavior);

    assert_eq!(*agent.behavior(), *behavior);
    assert_eq!(*agent.persona(), *persona);
    assert_eq!(*agent.status(), Status::Idle);
    assert!(agent.memory().is_empty());
}

#[test]
fn test_update_status() {
    let persona = "Lead Web Developer";
    let behavior = "Develops cutting-edge web applications with advanced features";

    let mut agent = AgentGPT::new_borrowed(persona, behavior);

    agent.update(Status::Active);
    assert_eq!(*agent.status(), Status::Active);

    agent.update(Status::InUnitTesting);
    assert_eq!(*agent.status(), Status::InUnitTesting);
}

#[test]
fn test_access_properties() {
    let persona = "Lead UX/UI Designer";
    let behavior = "Creates innovative website designs and user experiences";

    let agent = AgentGPT::new_borrowed(persona, behavior);

    assert_eq!(*agent.behavior(), behavior);
    assert_eq!(*agent.persona(), persona);
}

#[test]
fn test_memory() {
    let persona = "Lead Web Developer";
    let behavior = "Develops cutting-edge web applications with advanced features";

    let mut agent = AgentGPT::new_borrowed(persona, behavior);

    assert!(agent.memory().clone().is_empty());

    let message = Message {
        role: Cow::Borrowed("Role"),
        content: Cow::Borrowed("Content"),
    };

    agent.add_message(message);

    assert_eq!(agent.memory().len(), 1);
    assert_eq!(agent.memory()[0].role, "Role");
    assert_eq!(agent.memory()[0].content, "Content");
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
