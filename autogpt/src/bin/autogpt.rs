// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use anyhow::Result;

/// The main entry point of `autogpt`.
///
/// This function parses command-line arguments using the `clap` crate, sets up agent configurations
/// based on the provided options, and executes operations according to the specified subcommand.
///
/// `autogpt` supports two modes of operation:
///
/// 1. **Networking Mode**: In this mode, `autogpt` acts as a networked agent that communicates
///    with an orchestrator over TLS-encrypted TCP. The orchestrator must be running on a machine
///    via the `orchgpt` command. `autogpt` can then connect to it either from the same machine or
///    from another machine.
///
/// 2. **Networkless (Agentic) Mode**: In this standalone mode, `autogpt` operates independently
///    without requiring a network connection to an orchestrator. It runs locally, executing tasks
///    based solely on local configurations and input.
//
/// This flexible design allows `autogpt` to be deployed in a distributed multi-agent environment
/// or as a single self-contained agents.
#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "cli")]
    {
        use anyhow::anyhow;
        #[cfg(feature = "gpt")]
        use autogpt::agents::architect::ArchitectGPT;
        #[cfg(feature = "gpt")]
        use autogpt::agents::backend::BackendGPT;
        #[cfg(feature = "gpt")]
        use autogpt::agents::designer::DesignerGPT;
        #[cfg(feature = "gpt")]
        use autogpt::agents::frontend::FrontendGPT;
        #[cfg(all(feature = "gpt", feature = "git"))]
        use autogpt::agents::git::GitGPT;
        #[cfg(all(feature = "gpt", feature = "mail"))]
        use autogpt::agents::mailer::MailerGPT;
        #[cfg(feature = "gpt")]
        use autogpt::agents::manager::ManagerGPT;
        #[cfg(feature = "gpt")]
        use autogpt::agents::optimizer::OptimizerGPT;
        use autogpt::cli::autogpt::commands::{build, new, run, test};
        use autogpt::cli::autogpt::{Cli, Commands};
        #[cfg(feature = "gpt")]
        use autogpt::common::input::read_user_input;
        #[cfg(feature = "gpt")]
        use autogpt::common::utils::Scope;
        #[cfg(feature = "gpt")]
        use autogpt::common::utils::Task;
        #[cfg(feature = "gpt")]
        use autogpt::common::utils::ask_to_run_command;
        use autogpt::common::utils::fetch_latest_version;
        use autogpt::common::utils::is_outdated;
        use autogpt::common::utils::setup_logging;
        use autogpt::prelude::CTrait;
        use autogpt::prelude::ClientType;
        #[cfg(feature = "gpt")]
        use autogpt::traits::functions::AsyncFunctions;
        #[cfg(feature = "gpt")]
        use autogpt::traits::functions::Functions;
        use clap::Parser;
        #[cfg(feature = "gpt")]
        use colored::*;
        use futures_util::StreamExt;
        use gems::messages::Content;
        use gems::messages::Message as GemMessage;
        use gems::models::Model;
        use gems::stream::StreamBuilder;
        use gems::utils::extract_text_from_partial_json;
        #[cfg(feature = "net")]
        use iac_rs::message::Message;
        #[cfg(feature = "net")]
        use iac_rs::prelude::*;
        use std::env;
        #[cfg(feature = "gpt")]
        use std::env::var;
        use std::io::Write;
        #[cfg(feature = "net")]
        use std::sync::Arc;
        use std::thread;
        use termimad::MadSkin;
        #[cfg(feature = "net")]
        use tokio::io::AsyncBufReadExt;
        #[cfg(feature = "net")]
        use tokio::signal;
        #[cfg(feature = "net")]
        use tokio::sync::Mutex;
        use tokio::time::Duration;
        #[cfg(feature = "net")]
        use tokio::time::timeout;
        use tracing::error;
        #[cfg(any(feature = "gpt", feature = "net"))]
        use tracing::{info, warn};

        setup_logging()?;

        let args: Cli = Cli::parse();

        let current_version = env!("CARGO_PKG_VERSION");

        #[allow(clippy::collapsible_if)]
        if let Some(latest_version) = fetch_latest_version().await {
            if is_outdated(current_version, &latest_version) {
                use autogpt::cli::tui::render_update_banner;
                render_update_banner(current_version, &latest_version);
            }
        }

        pub fn type_with_cursor_effect(text: &str, delay: u64, skin: &MadSkin) {
            skin.print_inline(text);
            std::io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(delay));
        }

        #[cfg(feature = "net")]
        async fn run_client() -> Result<()> {
            let signer = Signer::new(KeyPair::generate());
            let address =
                env::var("ORCHESTRATOR_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8443".to_string());
            let client = Arc::new(Mutex::<Client>::new(
                Client::connect(&address, signer.clone()).await?,
            ));
            let mut stdin = tokio::io::BufReader::new(tokio::io::stdin());
            let mut input_line = String::new();
            let public_key_bytes = signer.verifying_key().as_slice().to_vec();
            let register_key_msg = Message {
                from: "autogpt".into(),
                to: "orchestrator".into(),
                msg_type: MessageType::RegisterKey,
                extra_data: public_key_bytes,
                ..Default::default()
            };
            client.lock().await.send(register_key_msg).await?;
            let shutdown_signal = signal::ctrl_c();
            tokio::pin!(shutdown_signal);
            loop {
                print!("> ");
                std::io::stdout().flush()?;
                input_line.clear();
                tokio::select! {
                    read = stdin.read_line(&mut input_line) => {
                        if read? == 0 {
                            break;
                        }
                        let input = input_line.trim();
                        if input.is_empty() {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty command?".bright_yellow().bold());
                            continue;
                        }

                        if !input.starts_with('/') {
                            error!("{}", "[*] \"AGI\": ❌ Command must begin with a '/' followed by the agent name.".bright_red().bold());
                            continue;
                        }

                        if let Some((to, rest)) = input.split_once(' ') {
                            let mut parts = rest.trim().splitn(2, ' ');
                            if let Some(action) = parts.next() {
                                let (actual_input, lang) = if action.eq_ignore_ascii_case("create") {
                                    ("", "")
                                } else if let Some(remaining) = parts.next() {
                                    if let Some((input, lang)) = remaining.split_once('|') {
                                        (input.trim(), lang.trim())
                                    } else {
                                        (remaining.trim(), "python")
                                    }
                                } else {
                                    error!("{}", "[*] \"AGI\": ❌ Invalid command format. Use: /<agent> <action>".bright_red().bold());
                                    continue;
                                };

                                let payload = format!("input={actual_input};language={lang}");

                                let msg = Message {
                                    from: "cli".into(),
                                    to: to.trim_start_matches('/').into(),
                                    msg_type: action.into(),
                                    payload_json: payload,
                                    ..Default::default()
                                };

                                let client = client.lock().await;
                                client.send(msg).await?;
                                info!("{}", "[*] \"AGI\": ✅ Sent!".green().bold());

                                let response = timeout(Duration::from_secs(30), client.receive()).await?;

                                match response {
                                    Ok(Some(resp)) => {
                                        info!(
                                            "{} {}",
                                            "[*] \"AGI\": 📬 Got Response →".bright_green().bold(),
                                            resp.payload_json.trim()
                                        );
                                    }
                                    Ok(None) => {
                                        warn!("{}", "[*] \"AGI\": ❌ No response from server.".bright_red().bold());
                                        break;
                                    }
                                    Err(_) => {
                                        warn!("{}", "[*] \"AGI\": ⏱️ Timeout waiting for response.".bright_red().bold());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    _ = &mut shutdown_signal => {
                        info!("\n[*] \"AGI\": 👋 Graceful shutdown requested.");
                        break;
                    }
                }
            }

            Ok(())
        }
        if let Some(prompt) = args.prompt {
            let skin = MadSkin::default();
            let mut client = ClientType::from_env();
            match &mut client {
                #[cfg(feature = "gem")]
                ClientType::Gemini(gem_client) => {
                    let parameters = StreamBuilder::default()
                        .model(Model::Flash3Preview)
                        .input(GemMessage::User {
                            content: Content::Text(prompt),
                            name: None,
                        })
                        .build()?;

                    let response = gem_client.stream().generate(parameters).await?;
                    let mut stream = response.bytes_stream();

                    let delay = 1;
                    let mut message: String = Default::default();
                    while let Some(mut chunk) = stream.next().await {
                        if let Ok(parsed_json) = std::str::from_utf8(chunk.as_mut().unwrap()) {
                            if let Some(text_value) = extract_text_from_partial_json(parsed_json) {
                                let lines: Vec<&str> = text_value
                                    .split("\\n")
                                    .flat_map(|s| s.split('\n'))
                                    .collect();
                                for line in lines {
                                    message.push_str(&line.replace('\\', ""));
                                    if !line.is_empty() {
                                        type_with_cursor_effect(
                                            &line.replace('\\', ""),
                                            delay,
                                            &skin,
                                        );
                                    } else {
                                        println!("\n");
                                    }
                                }
                            }
                        } else {
                            error!("Failed to parse chunk: {:?}", chunk.as_ref().unwrap());
                        }
                    }
                }

                #[cfg(feature = "oai")]
                ClientType::OpenAI(_oai_client) => {
                    // TODO: Implement streaming for OpenAI
                    todo!("Implement me plz.");
                }

                #[cfg(feature = "cld")]
                ClientType::Anthropic(_client) => {
                    // TODO: Implement streaming for Anthropic
                    todo!("Implement me plz.");
                }

                #[cfg(feature = "xai")]
                ClientType::Xai(_xai_client) => {
                    // TODO: Implement streaming for Xai
                    todo!("Implement me plz.");
                }

                #[cfg(feature = "co")]
                ClientType::Cohere(co_client) => {
                    use cohere_rust::api::GenerateModel;
                    use cohere_rust::api::chat::ChatRequest;

                    let chat_request = ChatRequest {
                        message: &prompt,
                        model: Some(GenerateModel::Custom("command-a-03-2025".to_string())),
                        max_tokens: Some(4096),
                        ..Default::default()
                    };

                    match co_client.chat(&chat_request).await {
                        Ok(mut receiver) => {
                            while let Some(res) = receiver.recv().await {
                                if let Ok(cohere_rust::api::chat::ChatStreamResponse::ChatTextGeneration { text, .. }) = res {
                                    let lines: Vec<&str> = text.split('\n').collect();
                                    for (i, line) in lines.iter().enumerate() {
                                        if !line.is_empty() {
                                            type_with_cursor_effect(line, 1, &skin);
                                        }
                                        if i < lines.len() - 1 {
                                            println!();
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Cohere chat streaming failed: {}", e);
                        }
                    }
                }

                #[allow(unreachable_patterns)]
                _ => {
                    return Err(anyhow!(
                        "No valid AI client configured. Enable `co`, `gem`, `oai`, `cld`, or `xai` feature."
                    ));
                }
            }
            println!();
        } else if args.command.is_none() && args.net {
            #[cfg(feature = "net")]
            {
                // --net flag: connect to orchgpt orchestrator.
                info!(
                    "{}",
                    "[*] \"AGI\": 🌐 Networking mode - connecting to orchestrator..."
                        .bright_green()
                );
                loop {
                    match run_client().await {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            error!("Client error: {}", e);
                            info!("{}", "[*] \"AGI\": 🔁 Reconnecting in 3s...".bright_green());
                            tokio::time::sleep(Duration::from_secs(3)).await;
                        }
                    }
                }
            }
            #[cfg(not(feature = "net"))]
            {
                anyhow::bail!(
                    "Networking feature 'net' is not enabled. Recompile with --features net."
                );
            }
        } else if args.command.is_none() {
            use autogpt::agents::generic::run_generic_agent_loop;
            run_generic_agent_loop(args.yolo, args.session.as_deref()).await?;
        } else if let Some(command) = args.command {
            #[cfg(feature = "gpt")]
            let workspace = var("AUTOGPT_WORKSPACE").unwrap_or_else(|_| "workspace/".to_string());
            #[cfg(feature = "gpt")]
            let language = "python";

            #[cfg(feature = "gpt")]
            let mut _git_agent = GitGPT::default();
            #[cfg(feature = "gpt")]
            let mut _optimizer_gpt = OptimizerGPT::default();

            #[cfg(feature = "gpt")]
            if !matches!(
                command,
                Commands::Test
                    | Commands::Run { feature: _ }
                    | Commands::Build { out: _ }
                    | Commands::New { name: _ }
            ) {
                _git_agent = GitGPT::new("GitGPT", "Commit all changes").await;
                let behavior =
                    "Expertise lies in modularizing monolithic source code into clean components";
                let persona = "OptimizerGPT";
                _optimizer_gpt = OptimizerGPT::new(persona, behavior, language).await;
            }
            match command {
                #[cfg(feature = "gpt")]
                Commands::Man => {
                    let behavior = "Expertise at managing projects at scale";
                    let persona = "ManagerGPT";
                    #[allow(unused_assignments)]
                    let mut manager = ManagerGPT::new(persona, behavior, "", language);

                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        let input = read_user_input()?;
                        manager = ManagerGPT::new(persona, behavior, &input, language);

                        if !input.is_empty() {
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );

                            let _ = manager.execute(true, true, 3).await;
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                                .bright_yellow()
                                                .bold());
                        }
                    }
                }
                #[cfg(feature = "gpt")]
                Commands::Arch => {
                    let behavior = "Expertise at managing projects at scale";
                    let persona = "ArchitectGPT";
                    let mut git_agent = GitGPT::default();
                    let mut architect_agent = ArchitectGPT::new(persona, behavior).await;
                    let workspace = workspace.clone() + "architect";
                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        let input = read_user_input()?;

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );
                            let mut task = Task {
                                description: input.into(),
                                scope: Some(Scope {
                                    crud: true,
                                    auth: false,
                                    external: true,
                                }),
                                urls: None,
                                frontend_code: None,
                                backend_code: None,
                                api_schema: None,
                            };

                            architect_agent
                                .execute(&mut task, true, false, 3)
                                .await
                                .unwrap();
                            info!(
                                "{}",
                                "[*] \"AGI\": Committing the new code to Git..."
                                    .green()
                                    .bold()
                            );
                            let _ = git_agent.execute(&mut task, true, false, 1).await;
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());

                            if let Err(e) = ask_to_run_command(
                                architect_agent.get_agent().clone(),
                                language,
                                &workspace,
                            )
                            .await
                            {
                                error!(
                                    "{}",
                                    format!("[*] AGI Runtime Error: {e}").bright_red().bold()
                                );
                                break;
                            }
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                            .bright_yellow()
                                            .bold());

                            if let Err(e) = ask_to_run_command(
                                architect_agent.get_agent().clone(),
                                language,
                                &workspace,
                            )
                            .await
                            {
                                error!(
                                    "{}",
                                    format!("[*] AGI Runtime Error: {e}").bright_red().bold()
                                );
                                break;
                            }
                        }
                    }
                }
                #[cfg(feature = "gpt")]
                Commands::Front => {
                    let behavior = "Expertise lies in writing frontend code";
                    let persona = "FrontendGPT";
                    let mut git_agent = GitGPT::default();
                    let workspace = workspace.clone() + "frontend";
                    let mut frontend_agent = FrontendGPT::new(persona, behavior, language).await;

                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        let input = read_user_input()?;

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );

                            let mut task = Task {
                                description: input.into(),
                                scope: Some(Scope {
                                    crud: true,
                                    auth: false,
                                    external: true,
                                }),
                                urls: None,
                                frontend_code: None,
                                backend_code: None,
                                api_schema: None,
                            };

                            frontend_agent
                                .execute(&mut task, true, false, 3)
                                .await
                                .unwrap();
                            info!(
                                "{}",
                                "[*] \"AGI\": Committing the new code to Git..."
                                    .green()
                                    .bold()
                            );
                            let _ = git_agent.execute(&mut task, true, false, 1).await;
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());

                            if let Err(e) = ask_to_run_command(
                                frontend_agent.get_agent().clone(),
                                language,
                                &workspace,
                            )
                            .await
                            {
                                error!(
                                    "{}",
                                    format!("[*] AGI Runtime Error: {e}").bright_red().bold()
                                );
                                break;
                            }
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                            .bright_yellow()
                                            .bold());
                            if let Err(e) = ask_to_run_command(
                                frontend_agent.get_agent().clone(),
                                language,
                                &workspace,
                            )
                            .await
                            {
                                error!(
                                    "{}",
                                    format!("[*] AGI Runtime Error: {e}").bright_red().bold()
                                );
                                break;
                            }
                        }
                    }
                }
                #[cfg(feature = "gpt")]
                Commands::Back => {
                    let behavior =
                        "Expertise lies in writing backend code for web servers and databases";
                    let persona = "BackendGPT";
                    let mut git_agent = GitGPT::default();
                    let workspace = workspace.clone() + "backend";
                    let mut backend_gpt = BackendGPT::new(persona, behavior, language).await;

                    let mut task = Task {
                        description: Default::default(),
                        scope: Some(Scope {
                            crud: true,
                            auth: true,
                            external: true,
                        }),
                        urls: None,
                        frontend_code: None,
                        backend_code: None,
                        api_schema: None,
                    };

                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        let input = read_user_input()?;

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );
                            task.description = input.into();

                            backend_gpt
                                .execute(&mut task, true, false, 3)
                                .await
                                .unwrap();
                            info!(
                                "{}",
                                "[*] \"AGI\": Committing the new code to Git..."
                                    .green()
                                    .bold()
                            );
                            let _ = git_agent.execute(&mut task, true, false, 1).await;
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());

                            if let Err(e) = ask_to_run_command(
                                backend_gpt.get_agent().clone(),
                                language,
                                &workspace,
                            )
                            .await
                            {
                                error!(
                                    "{}",
                                    format!("[*] AGI Runtime Error: {e}").bright_red().bold()
                                );
                                break;
                            }
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                                    .bright_yellow()
                                                    .bold());
                            if let Err(e) = ask_to_run_command(
                                backend_gpt.get_agent().clone(),
                                language,
                                &workspace,
                            )
                            .await
                            {
                                error!(
                                    "{}",
                                    format!("[*] AGI Runtime Error: {e}").bright_red().bold()
                                );
                                break;
                            }
                        }
                    }
                }
                #[cfg(feature = "gpt")]
                Commands::Design => {
                    let behavior = "Crafts stunning web design layouts";
                    let persona = "Web Designer";
                    let mut git_agent = GitGPT::default();
                    let mut designer_agent = DesignerGPT::new(persona, behavior).await;
                    let mut task = Task {
                        description: "".into(),
                        scope: None,
                        urls: None,
                        backend_code: None,
                        frontend_code: None,
                        api_schema: None,
                    };

                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        let input = read_user_input()?;

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );
                            task.description = input.into();

                            designer_agent.execute(&mut task, true, false, 3).await?;

                            info!(
                                "{}",
                                "[*] \"AGI\": Committing the new files to Git..."
                                    .green()
                                    .bold()
                            );

                            git_agent.execute(&mut task, true, false, 1).await?;
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                                    .bright_yellow()
                                                    .bold());
                        }
                    }
                }
                #[cfg(feature = "gpt")]
                Commands::Mail => {
                    let behavior = "Expertise at summarizing emails";
                    let persona = "Mailer";
                    let mut mailer_agent = MailerGPT::new(persona, behavior).await;
                    let mut task = Task {
                        description: "".into(),
                        scope: Some(Scope {
                            crud: true,
                            auth: false,
                            external: true,
                        }),
                        urls: None,
                        frontend_code: None,
                        backend_code: None,
                        api_schema: None,
                    };

                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        let input = read_user_input()?;

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );
                            task.description = input.into();

                            let _ = mailer_agent.execute(&mut task, true, false, 3).await;
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                                    .bright_yellow()
                                                    .bold());
                        }
                    }
                }
                #[cfg(feature = "gpt")]
                Commands::Git => {
                    let behavior = "Expertise at managing git repositories";
                    let persona = "GitGPT";
                    let mut git_agent = GitGPT::new(persona, behavior).await;
                    let mut task = Task {
                        description: "".into(),
                        scope: Some(Scope {
                            crud: true,
                            auth: false,
                            external: true,
                        }),
                        urls: None,
                        frontend_code: None,
                        backend_code: None,
                        api_schema: None,
                    };

                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        let input = read_user_input()?;

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );
                            task.description = input.into();

                            let _result = git_agent.execute(&mut task, true, false, 1).await;
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                                    .bright_yellow()
                                                    .bold());
                        }
                    }
                }
                #[cfg(feature = "gpt")]
                Commands::Opt => {
                    let behavior = "Expertise at optimizing codebases";
                    let persona = "OptimizerGPT";
                    let mut optimizer_agent = OptimizerGPT::new(persona, behavior, language).await;

                    let mut task = Task {
                        description: "".into(),
                        scope: Some(Scope {
                            crud: true,
                            auth: false,
                            external: true,
                        }),
                        urls: None,
                        frontend_code: None,
                        backend_code: None,
                        api_schema: None,
                    };

                    info!(
                        "{}",
                        "[*] \"AGI\": 🌟 Welcome! What would you like to work on today?"
                            .bright_green()
                    );

                    loop {
                        let input = read_user_input()?;

                        if !input.is_empty() {
                            let input = input.clone();
                            info!(
                                "{}",
                                "[*] \"AGI\": 🫡 Roger! Executing your command..."
                                    .bright_yellow()
                                    .bold()
                            );
                            task.description = input.into();

                            let _result = optimizer_agent.execute(&mut task, true, false, 1).await;
                            info!("{}", "[*] \"AGI\": ✅ Done!".green().bold());
                        } else {
                            warn!("{}", "[*] \"AGI\": 🤔 You've entered an empty project description? What exactly does that entail?"
                                                    .bright_yellow()
                                                    .bold());
                        }
                    }
                }
                #[cfg(not(feature = "gpt"))]
                Commands::Man
                | Commands::Arch
                | Commands::Front
                | Commands::Back
                | Commands::Design
                | Commands::Mail
                | Commands::Git
                | Commands::Opt => {
                    anyhow::bail!(
                        "Agentic commands require the 'gpt' feature. Recompile with --features gpt."
                    );
                }
                Commands::New { name } => new::handle_new(&name)?,
                Commands::Build { out } => build::handle_build(out)?,
                Commands::Run { feature } => run::handle_run(feature.unwrap_or_default())?,
                Commands::Test => test::handle_test()?,
            };
        }
    }
    Ok(())
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
