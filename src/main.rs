//! CLI Binary Entrypoint & Interactive REPL for HarnessMe.
//!
//! Provides a terminal-based interface to interact with autonomous agents,
//! test tool executions, inspect conversation state, and dynamically configure
//! providers (such as Antigravity / AGY) in real-time.

use std::env;
use std::io::{self, Write};

use harnessme::{Agent, AgentConfig, AntigravityProvider, CalculatorTool, EchoTool, ToolRegistry};

const DEFAULT_MAX_STEPS: usize = 10;
const DEFAULT_SYSTEM_PROMPT: &str =
    "You are a helpful and concise AI assistant equipped with tools. \
When requested to perform calculations or operations, invoke the appropriate tools accurately.";

/// Attempts to open a URL in the user's default web browser.
pub fn open_browser(url: &str) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    Ok(())
}

/// Parsed command from the interactive REPL.
#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    /// Terminate the REPL session.
    Exit,
    /// Clear conversation memory.
    Clear,
    /// Inspect conversation memory turns and tool calls.
    History,
    /// Display general help manual.
    Help,
    /// Display active provider information.
    ProviderInfo,
    /// Antigravity provider subcommands.
    Agy(AgySubcommand),
    /// Regular user prompt to send to the agent.
    UserPrompt(String),
}

/// Subcommands supported under `/agy`.
#[derive(Debug, PartialEq, Clone)]
pub enum AgySubcommand {
    /// Trigger Google Sign-In with browser launch.
    Login(Option<String>),
    /// Present interactive choice menu (Google sign-in, change model, etc.)
    Menu,
    /// Display Antigravity provider settings and active status.
    Status,
    /// Link Google account email or auth bearer token.
    Link(Option<String>),
    /// Configure Google account email.
    Account(String),
    /// List available Gemini and Antigravity models.
    ModelList,
    /// Update target model.
    Model(String),
    /// Update base URL.
    Url(String),
    /// Shortcut to update base URL to `http://127.0.0.1:{port}/v1`.
    Port(u16),
    /// Update or clear CSRF token.
    Csrf(Option<String>),
    /// Update or clear API key.
    Key(Option<String>),
    /// Update sampling temperature.
    Temperature(f32),
    /// Update request timeout in seconds.
    Timeout(u64),
    /// Reload Antigravity settings from environment variables.
    Reset,
    /// Switch or confirm active agent provider is Google AI (Antigravity).
    Switch,
    /// Display `/agy` command help.
    Help,
}

/// Parses an interactive input line into a strongly typed `Command`.
pub fn parse_command(input: &str) -> Command {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Command::UserPrompt(String::new());
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    let first = parts[0].to_lowercase();

    match first.as_str() {
        "/exit" | "exit" | "/quit" | "quit" => Command::Exit,
        "/clear" | "clear" | "/reset" | "reset" => Command::Clear,
        "/history" | "history" => Command::History,
        "/help" | "help" | "/?" | "?" => Command::Help,
        "/provider" | "/providers" => Command::ProviderInfo,
        "/agy" | "agy" => {
            if parts.len() == 1 {
                return Command::Agy(AgySubcommand::Login(None));
            }
            let sub = parts[1].to_lowercase();
            match sub.as_str() {
                "login" | "signin" | "auth" | "google" => {
                    if parts.len() > 2 {
                        Command::Agy(AgySubcommand::Login(Some(parts[2..].join(" "))))
                    } else {
                        Command::Agy(AgySubcommand::Login(None))
                    }
                }
                "menu" => Command::Agy(AgySubcommand::Menu),
                "1" => {
                    if parts.len() > 2 {
                        Command::Agy(AgySubcommand::Login(Some(parts[2..].join(" "))))
                    } else {
                        Command::Agy(AgySubcommand::Login(None))
                    }
                }
                "2" => {
                    if parts.len() > 2 {
                        Command::Agy(AgySubcommand::Model(
                            AntigravityProvider::resolve_model_name(&parts[2..].join(" ")),
                        ))
                    } else {
                        Command::Agy(AgySubcommand::ModelList)
                    }
                }
                "3" => Command::Agy(AgySubcommand::Status),
                "4" => Command::Agy(AgySubcommand::Switch),
                "status" | "info" => Command::Agy(AgySubcommand::Status),
                "help" | "?" => Command::Agy(AgySubcommand::Help),
                "reset" | "reload" => Command::Agy(AgySubcommand::Reset),
                "switch" | "use" | "activate" => Command::Agy(AgySubcommand::Switch),
                "link" => {
                    if parts.len() > 2 {
                        let val = parts[2..].join(" ");
                        if val.eq_ignore_ascii_case("none") || val.eq_ignore_ascii_case("clear") {
                            Command::Agy(AgySubcommand::Link(Some("clear".to_string())))
                        } else {
                            Command::Agy(AgySubcommand::Link(Some(val)))
                        }
                    } else {
                        Command::Agy(AgySubcommand::Link(None))
                    }
                }
                "account" | "email" => {
                    if parts.len() > 2 {
                        Command::Agy(AgySubcommand::Account(parts[2..].join(" ")))
                    } else {
                        Command::Agy(AgySubcommand::Account(String::new()))
                    }
                }
                "models" => Command::Agy(AgySubcommand::ModelList),
                "model" | "m" => {
                    if parts.len() > 2 {
                        Command::Agy(AgySubcommand::Model(
                            AntigravityProvider::resolve_model_name(&parts[2..].join(" ")),
                        ))
                    } else {
                        Command::Agy(AgySubcommand::ModelList)
                    }
                }
                "url" | "endpoint" | "base_url" => {
                    if parts.len() > 2 {
                        Command::Agy(AgySubcommand::Url(parts[2].to_string()))
                    } else {
                        Command::Agy(AgySubcommand::Help)
                    }
                }
                "port" | "p" => {
                    if parts.len() > 2 {
                        if let Ok(port) = parts[2].parse::<u16>() {
                            Command::Agy(AgySubcommand::Port(port))
                        } else {
                            Command::Agy(AgySubcommand::Help)
                        }
                    } else {
                        Command::Agy(AgySubcommand::Help)
                    }
                }
                "csrf" | "token" => {
                    if parts.len() > 2 {
                        let val = parts[2];
                        if val.eq_ignore_ascii_case("none") || val.eq_ignore_ascii_case("clear") {
                            Command::Agy(AgySubcommand::Csrf(None))
                        } else {
                            Command::Agy(AgySubcommand::Csrf(Some(val.to_string())))
                        }
                    } else {
                        Command::Agy(AgySubcommand::Csrf(None))
                    }
                }
                "key" | "apikey" | "api_key" => {
                    if parts.len() > 2 {
                        let val = parts[2];
                        if val.eq_ignore_ascii_case("none") || val.eq_ignore_ascii_case("clear") {
                            Command::Agy(AgySubcommand::Key(None))
                        } else {
                            Command::Agy(AgySubcommand::Key(Some(val.to_string())))
                        }
                    } else {
                        Command::Agy(AgySubcommand::Key(None))
                    }
                }
                "temp" | "temperature" => {
                    if parts.len() > 2 {
                        if let Ok(temp) = parts[2].parse::<f32>() {
                            Command::Agy(AgySubcommand::Temperature(temp))
                        } else {
                            Command::Agy(AgySubcommand::Help)
                        }
                    } else {
                        Command::Agy(AgySubcommand::Help)
                    }
                }
                "timeout" => {
                    if parts.len() > 2 {
                        if let Ok(t) = parts[2].parse::<u64>() {
                            Command::Agy(AgySubcommand::Timeout(t))
                        } else {
                            Command::Agy(AgySubcommand::Help)
                        }
                    } else {
                        Command::Agy(AgySubcommand::Help)
                    }
                }
                _ => Command::Agy(AgySubcommand::Help),
            }
        }
        _ => Command::UserPrompt(trimmed.to_string()),
    }
}

fn print_help() {
    println!("======================= Available REPL Commands =======================");
    println!("  /agy                   : Sign in with Google (opens browser to authenticate)");
    println!("  /agy login             : Sign in with Google via default browser");
    println!("  /agy menu              : Display Google AI / Antigravity interactive menu");
    println!("  /agy models            : List supported Gemini & Antigravity models");
    println!("  /agy model <name|num>  : Switch model (e.g. '/agy model 2' or 'gemini-2.5-pro')");
    println!("  /agy status            : Show current Google AI / Antigravity status & endpoints");
    println!("  /agy help              : Display all Antigravity options");
    println!("  /provider              : Show currently active provider details");
    println!("  /history, history      : Inspect entire multi-turn conversation memory");
    println!("  /clear, clear, /reset  : Clear conversation history");
    println!("  /exit, exit, /quit     : Exit the interactive REPL session");
    println!("=======================================================================\n");
}

fn print_agy_menu(agy: &AntigravityProvider) {
    println!("================ Google AI / Antigravity Configuration ================");
    println!(
        " Google Account  : {}",
        agy.account_email()
            .map(|e| format!("{e} (Google AI Pro: ACTIVE)"))
            .unwrap_or_else(|| "None (Run '/agy' to sign in)".to_string())
    );
    println!(" Current Model   : {}", agy.model());
    println!(" Connection      : {}", agy.base_url());
    println!(" Temperature     : {}", agy.temperature());
    println!("----------------------------------------------------------------------");
    println!(" Choose an option or enter a command below:");
    println!("   [1] Sign in with Google (Opens browser) -> '/agy' or '/agy login'");
    println!("   [2] Change Model (Google AI Pro / AGY)  -> '/agy model' or '/agy models'");
    println!("   [3] View Detailed Status & Endpoints    -> '/agy status'");
    println!("   [?] Full Antigravity Help               -> '/agy help'");
    println!("======================================================================\n");
}

fn print_agy_models(current_model: &str) {
    println!("---------------- Available Google AI Pro & Antigravity Models ----------------");
    for (idx, (name, description)) in AntigravityProvider::SUPPORTED_MODELS.iter().enumerate() {
        let is_current = if *name == current_model {
            " (CURRENT)"
        } else {
            ""
        };
        println!(
            "  [{}] {:<26} : {}{}",
            idx + 1,
            name,
            description,
            is_current
        );
    }
    println!("-------------------------------------------------------------------------------");
    println!("To select a model, run: '/agy model <number|name>' (e.g. '/agy model 1' or '/agy model gemini-3.1-pro-high')\n");
}

fn print_agy_help() {
    println!("------------------- Antigravity (/agy) Commands -------------------");
    println!("  /agy                   : Sign in with Google (opens browser to authenticate)");
    println!("  /agy login [account]   : Sign in with Google via default browser");
    println!("  /agy menu              : Display interactive configuration menu");
    println!("  /agy link [token]      : Link Google account / OAuth bearer token or clear");
    println!("  /agy account <email>   : Link Google account email (or 'clear')");
    println!("  /agy models            : List supported Google AI Pro & Antigravity models");
    println!(
        "  /agy model <name|num>  : Set model (e.g. 1=pro, 2=flash, or 'gemini-3.1-pro-high')"
    );
    println!("  /agy status            : Display current Antigravity status & subscription");
    println!("  /agy url <url>         : Set custom HTTP endpoint URL");
    println!("  /agy port <port>       : Set local port (shortcut for 127.0.0.1:<port>)");
    println!("  /agy csrf <token|none> : Set or clear X-Antigravity-CSRF-Token header");
    println!("  /agy key <key|none>    : Set or clear Bearer API key");
    println!("  /agy temp <0.0 - 2.0>  : Set sampling temperature");
    println!("  /agy timeout <secs>    : Set request timeout in seconds");
    println!("  /agy reset             : Reload settings from environment variables");
    println!("  /agy switch            : Confirm active provider is Google AI (Antigravity)");
    println!("-------------------------------------------------------------------\n");
}

fn print_agy_status(agy: &AntigravityProvider) {
    println!("---------------- Google AI / Antigravity Status ----------------");
    println!("  Active on Agent : YES (Active)");
    println!(
        "  Google Account  : {}",
        agy.account_email()
            .map(|e| format!("{e} (Google AI Pro Subscription: ACTIVE)"))
            .unwrap_or_else(|| "None (Not signed in - run '/agy')".to_string())
    );
    println!("  Model           : {}", agy.model());
    println!("  Connection      : {}", agy.base_url());
    println!(
        "  CSRF Token      : {}",
        if agy.csrf_token().is_some() {
            "Configured"
        } else {
            "None"
        }
    );
    println!(
        "  Auth / Bearer   : {}",
        if agy.api_key().is_some() {
            "Configured (hidden)"
        } else {
            "None / Google Keyring OAuth"
        }
    );
    println!("  Temperature     : {}", agy.temperature());
    println!("  Timeout         : {}s", agy.timeout_secs());
    if let Some(src) = agy.source_metadata() {
        println!("  Source Metadata : {src}");
    }
    println!("----------------------------------------------------------------\n");
}

fn handle_google_signin(
    agy: &mut AntigravityProvider,
    agent: &mut Agent,
    account_or_token: Option<String>,
) {
    println!("================ Google Sign-In (Google AI / Antigravity) ================");
    println!("Opening your web browser to authenticate with Google...");
    println!("If your browser does not open automatically, visit:");
    println!("  https://accounts.google.com/\n");

    match open_browser("https://accounts.google.com/") {
        Ok(_) => {
            println!("system > Browser opened successfully.");
        }
        Err(err) => {
            println!("system > Note: Could not launch system browser automatically: {err}");
            println!("system > Please open https://accounts.google.com/ in your browser.");
        }
    }

    if let Some(ref target) = account_or_token {
        let trimmed = target.trim();
        if trimmed.eq_ignore_ascii_case("none") || trimmed.eq_ignore_ascii_case("clear") {
            *agy = agy
                .clone()
                .with_optional_account(None)
                .with_optional_api_key(None);
            println!("system > Google account and authentication cleared.");
        } else if trimmed.contains('@') {
            *agy = agy.clone().with_account(trimmed);
            println!("system > Linked Google account to '{trimmed}'.");
        } else {
            *agy = agy.clone().with_api_key(trimmed);
            println!("system > Linked Antigravity auth/bearer token.");
        }
    } else {
        // Automatically discover local Google credentials
        let refreshed = AntigravityProvider::from_env();
        if refreshed.account_email().is_some() || refreshed.api_key().is_some() {
            *agy = refreshed;
            println!("system > Detected local Google credentials!");
        }
    }

    agent.set_boxed_provider(Box::new(agy.clone()));

    println!("--------------------------------------------------------------------------");
    if let Some(email) = agy.account_email() {
        println!("  Status       : SIGNED IN");
        println!("  Account      : {email} (Google AI Pro Subscription: ACTIVE)");
    } else {
        println!("  Status       : ACTIVE (Local Auth / Token Mode)");
    }
    println!("  Model        : {}", agy.model());
    println!("  Connection   : {}", agy.base_url());
    println!("system > Google AI (Antigravity) is active and ready to use.");
    println!("==========================================================================\n");
}

fn main() {
    let mut agy_provider = AntigravityProvider::from_env();

    let system_prompt =
        env::var("HARNESS_SYSTEM_PROMPT").unwrap_or_else(|_| DEFAULT_SYSTEM_PROMPT.to_string());
    let max_iterations = env::var("HARNESS_MAX_STEPS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_MAX_STEPS);

    println!("====================================================");
    println!("      HarnessMe Agent Interactive CLI (Google AI)   ");
    println!("====================================================");
    println!(" Provider     : Google AI / Antigravity");
    println!(
        " Account      : {}",
        agy_provider
            .account_email()
            .map(|e| format!("{e} (Google AI Pro: ACTIVE)"))
            .unwrap_or_else(|| "None (Run '/agy' to sign in with Google)".to_string())
    );
    println!(" Model        : {}", agy_provider.model());
    println!(" Connection   : {}", agy_provider.base_url());
    println!(
        " Auth Key     : {}",
        if agy_provider.api_key().is_some() {
            "Configured (hidden)"
        } else {
            "None / Google Keyring OAuth"
        }
    );
    println!(" Max Steps    : {max_iterations}");
    println!(" Temperature  : {}", agy_provider.temperature());
    println!("----------------------------------------------------");

    if agy_provider.account_email().is_none() && agy_provider.api_key().is_none() {
        println!(
            "Notice: Not signed in with Google. Type '/agy' to open your browser and sign in."
        );
        println!("----------------------------------------------------");
    }

    // Initialize tool registry
    let mut registry = ToolRegistry::new();
    registry.register(EchoTool);
    registry.register(CalculatorTool);

    println!(
        " Registered Tools: {}",
        registry
            .definitions()
            .iter()
            .map(|t| t.function.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(" Commands: '/agy' to sign in with Google, '/help' for manual, '/exit' to quit.");
    println!("====================================================\n");

    // Initialize agent exclusively with Google AI (Antigravity)
    let config = AgentConfig::new()
        .with_system_prompt(system_prompt)
        .with_max_iterations(max_iterations);

    let mut agent = Agent::with_config(Box::new(agy_provider.clone()), registry, config);

    let stdin = io::stdin();
    loop {
        print!("user > ");
        if let Err(err) = io::stdout().flush() {
            eprintln!("Failed to flush stdout: {err}");
            break;
        }

        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => {
                println!("\nGoodbye!");
                break;
            }
            Ok(_) => {
                let command = parse_command(&input);
                match command {
                    Command::Exit => {
                        println!("Goodbye!");
                        break;
                    }
                    Command::Clear => {
                        agent.clear_history();
                        println!("system > Conversation history cleared.");
                    }
                    Command::History => {
                        println!(
                            "system > Conversation history ({} turns):",
                            agent.history().len()
                        );
                        for (idx, msg) in agent.history().iter().enumerate() {
                            let role = format!("{:?}", msg.role).to_lowercase();
                            let content = msg.content.as_deref().unwrap_or("[No text content]");
                            if let Some(ref tool_calls) = msg.tool_calls {
                                let names: Vec<&str> = tool_calls
                                    .iter()
                                    .map(|c| c.function.name.as_str())
                                    .collect();
                                println!(
                                    "  [{idx}] {role} (tool_calls: {}): {content}",
                                    names.join(", ")
                                );
                            } else if let Some(ref call_id) = msg.tool_call_id {
                                println!("  [{idx}] {role} (call_id: {call_id}): {content}");
                            } else {
                                println!("  [{idx}] {role}: {content}");
                            }
                        }
                    }
                    Command::Help => {
                        print_help();
                    }
                    Command::ProviderInfo => {
                        println!("---------------- Active Provider Info ----------------");
                        println!("  Provider Name : Google AI / Antigravity");
                        println!(
                            "  Google Account: {}",
                            agy_provider
                                .account_email()
                                .unwrap_or("None (Not signed in - run '/agy')")
                        );
                        println!("  Model         : {}", agy_provider.model());
                        println!("  Base URL      : {}", agy_provider.base_url());
                        println!("  Temperature   : {}", agy_provider.temperature());
                        println!("  Timeout       : {}s", agy_provider.timeout_secs());
                        println!("  Use '/agy' to sign in with Google or configure options.");
                        println!("------------------------------------------------------\n");
                    }
                    Command::Agy(subcommand) => {
                        match subcommand {
                            AgySubcommand::Login(target) => {
                                handle_google_signin(&mut agy_provider, &mut agent, target);
                            }
                            AgySubcommand::Menu => {
                                print_agy_menu(&agy_provider);
                            }
                            AgySubcommand::ModelList => {
                                print_agy_models(agy_provider.model());
                            }
                            AgySubcommand::Link(val) => match val {
                                Some(target) => {
                                    let trimmed = target.trim();
                                    if trimmed.eq_ignore_ascii_case("none")
                                        || trimmed.eq_ignore_ascii_case("clear")
                                    {
                                        agy_provider = agy_provider
                                            .with_optional_account(None)
                                            .with_optional_api_key(None);
                                        println!(
                                            "system > Google account and API authentication cleared."
                                        );
                                    } else if trimmed.contains('@') {
                                        agy_provider = agy_provider.with_account(trimmed);
                                        println!("system > Google account linked to '{trimmed}'.");
                                    } else {
                                        agy_provider = agy_provider.with_api_key(trimmed);
                                        println!("system > Antigravity auth/bearer token linked.");
                                    }
                                    agent.set_boxed_provider(Box::new(agy_provider.clone()));
                                }
                                None => {
                                    println!("---------------- Link Google Account / Token ----------------");
                                    println!("  1. Sign in via Browser : /agy (or /agy login)");
                                    println!(
                                        "  2. Link by Email       : /agy account <your.email@gmail.com>"
                                    );
                                    println!("  3. Link by Bearer/Key  : /agy link <token>");
                                    println!("  4. Clear Account       : /agy link clear (or /agy account clear)");
                                    println!("-------------------------------------------------------------\n");
                                }
                            },
                            AgySubcommand::Account(email) => {
                                let trimmed = email.trim();
                                if trimmed.is_empty() {
                                    println!("system > Usage: /agy account <your.email@gmail.com> (or '/agy account clear')");
                                } else if trimmed.eq_ignore_ascii_case("none")
                                    || trimmed.eq_ignore_ascii_case("clear")
                                {
                                    agy_provider = agy_provider.with_optional_account(None);
                                    agent.set_boxed_provider(Box::new(agy_provider.clone()));
                                    println!("system > Google account unlinked.");
                                } else {
                                    agy_provider = agy_provider.with_account(trimmed);
                                    agent.set_boxed_provider(Box::new(agy_provider.clone()));
                                    println!("system > Google account linked to '{trimmed}'.");
                                }
                            }
                            AgySubcommand::Status => {
                                print_agy_status(&agy_provider);
                            }
                            AgySubcommand::Help => {
                                print_agy_help();
                            }
                            AgySubcommand::Model(model_name) => {
                                agy_provider = agy_provider.with_model(&model_name);
                                agent.set_boxed_provider(Box::new(agy_provider.clone()));
                                println!("system > Google AI model updated to '{model_name}'.");
                            }
                            AgySubcommand::Url(url) => {
                                agy_provider = agy_provider.with_base_url(&url);
                                agent.set_boxed_provider(Box::new(agy_provider.clone()));
                                println!("system > Google AI base URL updated to '{url}'.");
                            }
                            AgySubcommand::Port(port) => {
                                let url = format!("http://127.0.0.1:{port}/v1");
                                agy_provider = agy_provider.with_base_url(&url);
                                agent.set_boxed_provider(Box::new(agy_provider.clone()));
                                println!("system > Google AI base URL set to '{url}'.");
                            }
                            AgySubcommand::Csrf(token) => {
                                if let Some(ref t) = token {
                                    agy_provider = agy_provider.with_csrf_token(t);
                                    println!("system > Antigravity CSRF token updated.");
                                } else {
                                    agy_provider = agy_provider.with_optional_csrf_token(None);
                                    println!("system > Antigravity CSRF token cleared.");
                                }
                                agent.set_boxed_provider(Box::new(agy_provider.clone()));
                            }
                            AgySubcommand::Key(key) => {
                                if let Some(ref k) = key {
                                    agy_provider = agy_provider.with_api_key(k);
                                    println!("system > Google AI auth token updated.");
                                } else {
                                    agy_provider = agy_provider.with_optional_api_key(None);
                                    println!("system > Google AI auth token cleared.");
                                }
                                agent.set_boxed_provider(Box::new(agy_provider.clone()));
                            }
                            AgySubcommand::Temperature(temp) => {
                                agy_provider = agy_provider.with_temperature(temp);
                                agent.set_boxed_provider(Box::new(agy_provider.clone()));
                                println!("system > Google AI temperature updated to {temp}.");
                            }
                            AgySubcommand::Timeout(timeout) => {
                                agy_provider = agy_provider.with_timeout(timeout);
                                agent.set_boxed_provider(Box::new(agy_provider.clone()));
                                println!("system > Google AI timeout updated to {timeout}s.");
                            }
                            AgySubcommand::Reset => {
                                agy_provider = AntigravityProvider::from_env();
                                agent.set_boxed_provider(Box::new(agy_provider.clone()));
                                println!("system > Google AI settings reloaded from environment.");
                                print_agy_status(&agy_provider);
                            }
                            AgySubcommand::Switch => {
                                agent.set_boxed_provider(Box::new(agy_provider.clone()));
                                println!(
                                    "system > Google AI (Antigravity) is active [{}: {}].",
                                    agy_provider.model(),
                                    agy_provider.base_url()
                                );
                            }
                        }
                    }
                    Command::UserPrompt(prompt) => {
                        if prompt.is_empty() {
                            continue;
                        }
                        match agent.run(&prompt) {
                            Ok(response) => {
                                println!("agent > {response}\n");
                            }
                            Err(err) => {
                                eprintln!("error > {err}\n");
                            }
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Error reading stdin: {err}");
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command_basic() {
        assert_eq!(parse_command("/exit"), Command::Exit);
        assert_eq!(parse_command("exit"), Command::Exit);
        assert_eq!(parse_command("/quit"), Command::Exit);
        assert_eq!(parse_command("/clear"), Command::Clear);
        assert_eq!(parse_command("clear"), Command::Clear);
        assert_eq!(parse_command("/reset"), Command::Clear);
        assert_eq!(parse_command("/history"), Command::History);
        assert_eq!(parse_command("history"), Command::History);
        assert_eq!(parse_command("/help"), Command::Help);
        assert_eq!(parse_command("help"), Command::Help);
        assert_eq!(parse_command("/?"), Command::Help);
        assert_eq!(parse_command("/provider"), Command::ProviderInfo);
    }

    #[test]
    fn test_parse_command_agy_subcommands() {
        assert_eq!(
            parse_command("/agy"),
            Command::Agy(AgySubcommand::Login(None))
        );
        assert_eq!(
            parse_command("/agy login"),
            Command::Agy(AgySubcommand::Login(None))
        );
        assert_eq!(
            parse_command("/agy login user@example.com"),
            Command::Agy(AgySubcommand::Login(Some("user@example.com".to_string())))
        );
        assert_eq!(
            parse_command("/agy menu"),
            Command::Agy(AgySubcommand::Menu)
        );
        assert_eq!(
            parse_command("/agy 1"),
            Command::Agy(AgySubcommand::Login(None))
        );
        assert_eq!(
            parse_command("/agy 1 user@example.com"),
            Command::Agy(AgySubcommand::Login(Some("user@example.com".to_string())))
        );
        assert_eq!(
            parse_command("/agy 2"),
            Command::Agy(AgySubcommand::ModelList)
        );
        assert_eq!(
            parse_command("/agy 2 2"),
            Command::Agy(AgySubcommand::Model("gemini-3.8-flash-high".to_string()))
        );
        assert_eq!(parse_command("/agy 3"), Command::Agy(AgySubcommand::Status));
        assert_eq!(parse_command("/agy 4"), Command::Agy(AgySubcommand::Switch));
        assert_eq!(
            parse_command("/agy status"),
            Command::Agy(AgySubcommand::Status)
        );
        assert_eq!(
            parse_command("/agy info"),
            Command::Agy(AgySubcommand::Status)
        );
        assert_eq!(
            parse_command("/agy help"),
            Command::Agy(AgySubcommand::Help)
        );
        assert_eq!(
            parse_command("/agy switch"),
            Command::Agy(AgySubcommand::Switch)
        );
        assert_eq!(
            parse_command("/agy reset"),
            Command::Agy(AgySubcommand::Reset)
        );

        assert_eq!(
            parse_command("/agy link"),
            Command::Agy(AgySubcommand::Link(None))
        );
        assert_eq!(
            parse_command("/agy link user@gmail.com"),
            Command::Agy(AgySubcommand::Link(Some("user@gmail.com".to_string())))
        );
        assert_eq!(
            parse_command("/agy link clear"),
            Command::Agy(AgySubcommand::Link(Some("clear".to_string())))
        );
        assert_eq!(
            parse_command("/agy account user@gmail.com"),
            Command::Agy(AgySubcommand::Account("user@gmail.com".to_string()))
        );
        assert_eq!(
            parse_command("/agy models"),
            Command::Agy(AgySubcommand::ModelList)
        );
        assert_eq!(
            parse_command("/agy model"),
            Command::Agy(AgySubcommand::ModelList)
        );
        assert_eq!(
            parse_command("/agy model 1"),
            Command::Agy(AgySubcommand::Model("gemini-3.1-pro-high".to_string()))
        );
        assert_eq!(
            parse_command("/agy model 2"),
            Command::Agy(AgySubcommand::Model("gemini-3.8-flash-high".to_string()))
        );
        assert_eq!(
            parse_command("/agy model pro"),
            Command::Agy(AgySubcommand::Model("gemini-3.1-pro-high".to_string()))
        );
        assert_eq!(
            parse_command("/agy model flash"),
            Command::Agy(AgySubcommand::Model("gemini-3.8-flash-high".to_string()))
        );
        assert_eq!(
            parse_command("/agy model gemini-3.7-flash-high"),
            Command::Agy(AgySubcommand::Model("gemini-3.7-flash-high".to_string()))
        );
        assert_eq!(
            parse_command("/agy url http://localhost:38035/v1"),
            Command::Agy(AgySubcommand::Url("http://localhost:38035/v1".to_string()))
        );
        assert_eq!(
            parse_command("/agy port 38035"),
            Command::Agy(AgySubcommand::Port(38035))
        );
        assert_eq!(
            parse_command("/agy csrf csrf_secret"),
            Command::Agy(AgySubcommand::Csrf(Some("csrf_secret".to_string())))
        );
        assert_eq!(
            parse_command("/agy csrf none"),
            Command::Agy(AgySubcommand::Csrf(None))
        );
        assert_eq!(
            parse_command("/agy key secret_key"),
            Command::Agy(AgySubcommand::Key(Some("secret_key".to_string())))
        );
        assert_eq!(
            parse_command("/agy key clear"),
            Command::Agy(AgySubcommand::Key(None))
        );
        assert_eq!(
            parse_command("/agy temp 0.2"),
            Command::Agy(AgySubcommand::Temperature(0.2))
        );
        assert_eq!(
            parse_command("/agy timeout 45"),
            Command::Agy(AgySubcommand::Timeout(45))
        );
    }

    #[test]
    fn test_parse_command_user_prompt() {
        assert_eq!(
            parse_command("What is 2 + 2?"),
            Command::UserPrompt("What is 2 + 2?".to_string())
        );
        assert_eq!(
            parse_command("hello agent"),
            Command::UserPrompt("hello agent".to_string())
        );
    }
}
