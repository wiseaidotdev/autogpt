# Contributing

Contributions to AutoGPT are welcome! Whether you are fixing a bug, adding a new agent, improving documentation, or proposing a protocol change, the process is straightforward.

## Setting Up the Development Environment

```sh
git clone https://github.com/wiseaidotdev/autogpt.git
cd autogpt/autogpt

# Build with all features to catch compilation errors across all agents
cargo build --all-features
```

## Running the Test Suite

```sh
cargo test --all-features
```

## Code Style

AutoGPT targets **Rust 1.89+** and enforces `clippy` warnings as errors in CI:

```sh
cargo clippy --all-features -- -D warnings
cargo fmt --check
```

Format your code before opening a PR:

```sh
cargo fmt
```

## Commit Message Format

AutoGPT uses conventional commit prefixes:

| Prefix      | Use for                                   |
| ----------- | ----------------------------------------- |
| `feat:`     | New features or agents                    |
| `fix:`      | Bug fixes                                 |
| `docs:`     | Documentation changes                     |
| `refactor:` | Code improvements without behavior change |
| `test:`     | Test additions or fixes                   |
| `chore:`    | Build system, CI, dependency updates      |

Example: `feat: add CoderGPT agent for code review tasks`

## Adding a New Built-in Agent

1. Create `autogpt/src/agents/<your_agent>.rs` implementing `Executor`
2. Register it in `autogpt/src/agents.rs` by adding a `pub mod` declaration
3. Add a feature flag in `Cargo.toml` if the agent has optional dependencies
4. Re-export from `autogpt/src/prelude.rs` under the appropriate `cfg` block
5. Write documentation in `docs/src/agents/<your-agent>.md`
6. Add an entry to `docs/src/agents/overview.md` and `docs/src/SUMMARY.md`

## Adding Documentation

The docs site is built with [mdBook](https://rust-lang.github.io/mdBook/). To preview locally:

```sh
cargo install mdbook
cd docs
mdbook serve --port 3001
# Open http://localhost:3001
```

Build for production:

```sh
mdbook build
# Output at docs/book/
```

## Opening a Pull Request

1. Fork the repository on GitHub
2. Create a feature branch: `git checkout -b feat/my-new-feature`
3. Make your changes and add tests where applicable
4. Ensure `cargo test --all-features`, `cargo clippy`, and `cargo fmt --check` all pass
5. Push and open a pull request against `main`

## Reporting Issues

Open a [GitHub Issue](https://github.com/wiseaidotdev/autogpt/issues) with:

- A clear description of the bug or feature request
- Steps to reproduce (for bugs)
- Your OS, Rust version (`rustc --version`), and AutoGPT version

## License

By submitting a contribution you agree that your code will be distributed under the [MIT License](https://github.com/wiseaidotdev/autogpt/blob/main/LICENSE).
