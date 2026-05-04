// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use autogpt::common::utils::{Message, Status};
use autogpt::prelude::*;
use autogpt::traits::agent::Agent;
use std::borrow::Cow;

#[derive(Debug, Default)]
pub struct MockAgent {
    behavior: Cow<'static, str>,
    persona: Cow<'static, str>,
    status: Status,
    memory: Vec<Message>,
    tools: Vec<Tool>,
    knowledge: Knowledge,
    planner: Option<Planner>,
    profile: Persona,
    collaborators: Vec<Collaborator>,
    reflection: Option<Reflection>,
    scheduler: Option<TaskScheduler>,
    capabilities: HashSet<Capability>,
    context: ContextManager,
    tasks: Vec<Task>,
}

impl Agent for MockAgent {
    fn new(
        persona: std::borrow::Cow<'static, str>,
        behavior: std::borrow::Cow<'static, str>,
    ) -> Self {
        MockAgent {
            behavior,
            persona,
            ..Default::default()
        }
    }

    fn update(&mut self, status: Status) {
        self.status = status;
    }

    fn behavior(&self) -> &std::borrow::Cow<'static, str> {
        &self.behavior
    }

    fn persona(&self) -> &std::borrow::Cow<'static, str> {
        &self.persona
    }

    fn status(&self) -> &Status {
        &self.status
    }

    fn memory(&self) -> &Vec<Message> {
        &self.memory
    }

    fn tools(&self) -> &Vec<Tool> {
        &self.tools
    }

    fn knowledge(&self) -> &Knowledge {
        &self.knowledge
    }

    fn planner(&self) -> Option<&Planner> {
        self.planner.as_ref()
    }

    fn profile(&self) -> &Persona {
        &self.profile
    }

    fn collaborators(&self) -> Vec<Collaborator> {
        self.collaborators.clone()
    }

    fn reflection(&self) -> Option<&Reflection> {
        self.reflection.as_ref()
    }

    fn scheduler(&self) -> Option<&TaskScheduler> {
        self.scheduler.as_ref()
    }

    fn capabilities(&self) -> &std::collections::HashSet<Capability> {
        &self.capabilities
    }

    fn context(&self) -> &ContextManager {
        &self.context
    }

    fn tasks(&self) -> &Vec<Task> {
        &self.tasks
    }

    fn memory_mut(&mut self) -> &mut Vec<Message> {
        &mut self.memory
    }

    fn planner_mut(&mut self) -> Option<&mut Planner> {
        self.planner.as_mut()
    }

    fn context_mut(&mut self) -> &mut ContextManager {
        &mut self.context
    }
}

#[test]
fn test_agent_creation() {
    let persona = Cow::Borrowed("Persona");
    let behavior = Cow::Borrowed("Behavior");
    let agent = MockAgent::new(persona.clone(), behavior.clone());

    assert_eq!(*agent.behavior(), *behavior);
    assert_eq!(*agent.persona(), *persona);
    assert_eq!(*agent.status(), Status::Idle);
    assert!(agent.memory().is_empty());
}

#[test]
fn test_agent_update() {
    let mut agent = MockAgent::new(Cow::Borrowed("Persona"), Cow::Borrowed("Behavior"));

    agent.update(Status::Active);
    assert_eq!(*agent.status(), Status::Active);

    agent.update(Status::InUnitTesting);
    assert_eq!(*agent.status(), Status::InUnitTesting);
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
