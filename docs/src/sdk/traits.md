# Traits Reference

AutoGPT defines a layered set of traits. Understanding them is essential for building custom agents and implementing advanced integrations.

## `Agent`

Defined in `autogpt::traits::agent`. The base trait every agent must implement. It describes the core state of an agent: its persona, behavior, memory, status, tools, and scheduling.

```rust
pub trait Agent: Debug {
    fn new(persona: Cow<'static, str>, behavior: Cow<'static, str>) -> Self where Self: Sized;
    fn update(&mut self, status: Status);
    fn behavior(&self) -> &Cow<'static, str>;
    fn persona(&self) -> &Cow<'static, str>;
    fn status(&self) -> &Status;
    fn memory(&self) -> &Vec<Message>;
    fn tools(&self) -> &Vec<Tool>;
    fn knowledge(&self) -> &Knowledge;
    fn planner(&self) -> Option<&Planner>;
    fn profile(&self) -> &Persona;
    fn reflection(&self) -> Option<&Reflection>;
    fn scheduler(&self) -> Option<&TaskScheduler>;
    fn capabilities(&self) -> &HashSet<Capability>;
    fn context(&self) -> &ContextManager;
    fn tasks(&self) -> &Vec<Task>;
    fn memory_mut(&mut self) -> &mut Vec<Message>;
    fn planner_mut(&mut self) -> Option<&mut Planner>;
    fn context_mut(&mut self) -> &mut ContextManager;
}
```

The `Auto` derive macro implements `Agent` for structs containing an `AgentGPT` field by forwarding all calls to that field.

## `Functions`

Defined in `autogpt::traits::functions`. A synchronous accessor trait that provides access to the underlying `AgentGPT`:

```rust
pub trait Functions {
    fn get_agent(&self) -> &AgentGPT;
}
```

## `AsyncFunctions`

Defined in `autogpt::traits::functions`. The async capability trait. It provides LLM interaction methods:

```rust
#[async_trait]
pub trait AsyncFunctions: Send + Sync {
    async fn execute<'a>(&'a mut self, task: &'a mut Task, execute: bool, browse: bool, max_tries: u64) -> Result<()>;

    // Only available with `mem` feature:
    async fn save_ltm(&mut self, message: Message) -> Result<()>;
    async fn get_ltm(&self) -> Result<Vec<Message>>;
    async fn ltm_context(&self) -> String;

    // Only available with an LLM feature (gem, oai, cld, xai, co):
    async fn generate(&mut self, request: &str) -> Result<String>;
    async fn imagen(&mut self, request: &str) -> Result<Vec<u8>>;
    async fn stream(&mut self, request: &str) -> Result<ReqResponse>;
}
```

## `Executor`

Defined in `autogpt::traits::functions`. This is the trait you implement in your custom agents. It has one required method:

```rust
#[async_trait]
pub trait Executor {
    async fn execute<'a>(
        &'a mut self,
        task: &'a mut Task,
        execute: bool,
        browse: bool,
        max_tries: u64,
    ) -> Result<()>;
}
```

`AsyncFunctions::execute` delegates to your `Executor::execute` implementation. The key distinction: `AsyncFunctions` is the trait-object-safe interface used by `AutoGPT`; `Executor` is the concrete implementation target for your custom types.

## `AgentFunctions` (Composite)

Defined in `autogpt::traits::composite`. This supertrait is what `agents!` and `AutoGPT` operate on:

```rust
pub trait AgentFunctions: Agent + AsyncFunctions + Send + Sync {}
```

Any type that implements both `Agent` and `AsyncFunctions` automatically satisfies `AgentFunctions`. The `Auto` derive macro generates this blanket implementation.

## `Collaborate`

Defined in `autogpt::traits::functions`. Used for networked multi-agent communication (requires `net` feature):

```rust
#[async_trait]
pub trait Collaborate: Send + Sync {
    async fn handle_task(&mut self, task: Task) -> Result<()>;
    async fn receive_message(&mut self, message: AgentMessage) -> Result<()>;
    fn get_id(&self) -> &str;
}
```

## `Status` Enum

```rust
pub enum Status {
    Idle,
    Active,
    Completed,
    Error,
}
```

Agents begin as `Idle`, transition to `Active` during execution, and end as either `Completed` or `Error`.
