//! CLI Binary Entrypoint & Interactive REPL for HarnessMe.
//!
//! Provides a terminal-based interface to interact with autonomous agents,
//! test tool executions, inspect conversation state, and dynamically configure
//! providers (such as Antigravity / AGY) in real-time.

use std::env;
use std::io::{self, Write};

use harnessme::{
    Agent, AgentConfig, AntigravityProvider, CalculatorTool, EchoTool, OpenAiCompatibleProvider,
    Provider, ToolRegistry,
};

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_MODEL: &str = "gpt-4o-mini";
const DEFAULT_MAX_STEPS: usize = 10;
const DEFAULT_TEMPERATURE: f32 = 0.7;
const DEFAULT_SYSTEM_PROMPT: &str =
    "You are a helpful and concise AI assistant equipped with tools. \
When requested to perform calculations or operations, invoke the appropriate tools accurately.";

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
    /// Display Antigravity provider settings and active status.
    Status,
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
    /// Switch active agent provider to Antigravity.
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
                return Command::Agy(AgySubcommand::Status);
            }
            let sub = parts[1].to_lowercase();
            match sub.as_str() {
                "status" | "info" => Command::Agy(AgySubcommand::Status),
                "help" | "?" => Command::Agy(AgySubcommand::Help),
                "reset" | "reload" => Command::Agy(AgySubcommand::Reset),
                "switch" | "use" | "activate" => Command::Agy(AgySubcommand::Switch),
                "model" | "m" => {
                    if parts.len() > 2 {
                        Command::Agy(AgySubcommand::Model(parts[2..].join(" ")))
                    } else {
                        Command::Agy(AgySubcommand::Help)
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
    println!("  /help, help, /?        : Show this command summary");
    println!("  /agy                   : Display active Antigravity (AGY) configuration");
    println!("  /agy [subcommand]      : Inspect or dynamically configure Antigravity");
    println!("  /provider              : Show currently active provider details");
    println!("  /history, history      : Inspect entire multi-turn conversation memory");
    println!("  /clear, clear, /reset  : Clear conversation history");
    println!("  /exit, exit, /quit     : Exit the interactive REPL session");
    println!("=======================================================================\n");
}

fn print_agy_help() {
    println!("------------------- Antigravity (/agy) Commands -------------------");
    println!("  /agy                   : Display current Antigravity status & settings");
    println!("  /agy status            : Display current Antigravity status & settings");
    println!("  /agy model <name>      : Set model (e.g. gemini-2.5-pro, gemini-2.5-flash)");
    println!("  /agy url <url>         : Set base URL (e.g. http://127.0.0.1:38035/v1)");
    println!("  /agy port <port>       : Set local port (shortcut for 127.0.0.1:<port>)");
    println!("  /agy csrf <token|none> : Set or clear X-Antigravity-CSRF-Token header");
    println!("  /agy key <key|none>    : Set or clear Bearer API key");
    println!("  /agy temp <0.0 - 2.0>  : Set sampling temperature");
    println!("  /agy timeout <secs>    : Set HTTP request timeout in seconds");
    println!("  /agy reset             : Reload settings from environment variables");
    println!("  /agy switch            : Switch agent active provider to Antigravity");
    println!("-------------------------------------------------------------------\n");
}

fn print_agy_status(agy: &AntigravityProvider, is_active: bool) {
    println!("---------------- Antigravity (AGY) Status ----------------");
    println!(
        "  Active on Agent : {}",
        if is_active {
            "YES (Active)"
        } else {
            "NO (Standby)"
        }
    );
    println!("  Model           : {}", agy.model());
    println!("  Base URL        : {}", agy.base_url());
    println!(
        "  CSRF Token      : {}",
        if agy.csrf_token().is_some() {
            "Configured"
        } else {
            "None"
        }
    );
    println!(
        "  API Key         : {}",
        if agy.api_key().is_some() {
            "Configured (hidden)"
        } else {
            "None"
        }
    );
    println!("  Temperature     : {}", agy.temperature());
    println!("  Timeout         : {}s", agy.timeout_secs());
    if let Some(src) = agy.source_metadata() {
        println!("  Source Metadata : {src}");
    }
    println!("----------------------------------------------------------\n");
}

fn main() {
    let provider_name = env::var("HARNESS_PROVIDER").unwrap_or_else(|_| {
        if env::var("ANTIGRAVITY_LS_ADDRESS").is_ok()
            || env::var("ANTIGRAVITY_BASE_URL").is_ok()
            || env::var("ANTIGRAVITY_API_KEY").is_ok()
            || env::var("AGY_API_KEY").is_ok()
        {
            "antigravity".to_string()
        } else {
            "openai".to_string()
        }
    });

    let mut is_antigravity =
        provider_name.to_lowercase() == "antigravity" || provider_name.to_lowercase() == "agy";

    let mut agy_provider = AntigravityProvider::from_env();

    let openai_api_key = env::var("OPENAI_API_KEY").ok();
    let openai_base_url =
        env::var("OPENAI_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
    let openai_model = env::var("HARNESS_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
    let openai_temperature = env::var("HARNESS_TEMPERATURE")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(DEFAULT_TEMPERATURE);

    let openai_provider = OpenAiCompatibleProvider::new(&openai_model, openai_api_key.clone())
        .with_base_url(&openai_base_url)
        .with_temperature(openai_temperature);

    let system_prompt =
        env::var("HARNESS_SYSTEM_PROMPT").unwrap_or_else(|_| DEFAULT_SYSTEM_PROMPT.to_string());
    let max_iterations = env::var("HARNESS_MAX_STEPS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_MAX_STEPS);

    println!("====================================================");
    println!("             HarnessMe Agent Interactive CLI         ");
    println!("====================================================");
    println!(
        " Provider     : {}",
        if is_antigravity {
            "Antigravity (AGY)"
        } else {
            "OpenAI-Compatible"
        }
    );
    println!(
        " Model        : {}",
        if is_antigravity {
            agy_provider.model()
        } else {
            openai_provider.model()
        }
    );
    println!(
        " Base URL     : {}",
        if is_antigravity {
            agy_provider.base_url()
        } else {
            openai_provider.base_url()
        }
    );
    println!(
        " Auth Key     : {}",
        if is_antigravity {
            if agy_provider.api_key().is_some() {
                "Configured (hidden)"
            } else {
                "None / Local Auth"
            }
        } else if openai_provider.api_key().is_some() {
            "Configured (hidden)"
        } else {
            "None (local endpoint mode)"
        }
    );
    println!(" Max Steps    : {max_iterations}");
    println!(
        " Temperature  : {}",
        if is_antigravity {
            agy_provider.temperature()
        } else {
            openai_provider.temperature()
        }
    );
    println!("----------------------------------------------------");

    if !is_antigravity
        && openai_provider.api_key().is_none()
        && openai_provider
            .base_url()
            .starts_with("https://api.openai.com")
    {
        eprintln!(
            "Notice: OPENAI_API_KEY is not set. If connecting to OpenAI, please export OPENAI_API_KEY."
        );
        eprintln!("For local models (Ollama/vLLM), configure OPENAI_BASE_URL (e.g. http://localhost:11434/v1).");
        eprintln!(
            "To switch to Antigravity, run '/agy switch' or export HARNESS_PROVIDER=antigravity."
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
    println!(" Commands: '/help' for manual, '/agy' for Antigravity config, '/exit' to quit.");
    println!("====================================================\n");

    // Initialize agent
    let config = AgentConfig::new()
        .with_system_prompt(system_prompt)
        .with_max_iterations(max_iterations);

    let active_provider: Box<dyn Provider> = if is_antigravity {
        Box::new(agy_provider.clone())
    } else {
        Box::new(openai_provider.clone())
    };

    let mut agent = Agent::with_config(active_provider, registry, config);

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
                        println!(
                            "  Provider Name : {}",
                            if is_antigravity {
                                "Antigravity (AGY)"
                            } else {
                                "OpenAI-Compatible"
                            }
                        );
                        if is_antigravity {
                            println!("  Model         : {}", agy_provider.model());
                            println!("  Base URL      : {}", agy_provider.base_url());
                            println!("  Temperature   : {}", agy_provider.temperature());
                        } else {
                            println!("  Model         : {}", openai_provider.model());
                            println!("  Base URL      : {}", openai_provider.base_url());
                            println!("  Temperature   : {}", openai_provider.temperature());
                        }
                        println!("  Use '/agy' to inspect or configure Antigravity settings.");
                        println!("------------------------------------------------------\n");
                    }
                    Command::Agy(subcommand) => match subcommand {
                        AgySubcommand::Status => {
                            print_agy_status(&agy_provider, is_antigravity);
                        }
                        AgySubcommand::Help => {
                            print_agy_help();
                        }
                        AgySubcommand::Model(model_name) => {
                            agy_provider = agy_provider.with_model(&model_name);
                            if is_antigravity {
                                agent.set_boxed_provider(Box::new(agy_provider.clone()));
                            }
                            println!("system > Antigravity model updated to '{model_name}'.");
                            if !is_antigravity {
                                println!("system > Note: Antigravity is not currently the active provider. Run '/agy switch' to activate.");
                            }
                        }
                        AgySubcommand::Url(url) => {
                            agy_provider = agy_provider.with_base_url(&url);
                            if is_antigravity {
                                agent.set_boxed_provider(Box::new(agy_provider.clone()));
                            }
                            println!("system > Antigravity base URL updated to '{url}'.");
                            if !is_antigravity {
                                println!("system > Note: Antigravity is not currently the active provider. Run '/agy switch' to activate.");
                            }
                        }
                        AgySubcommand::Port(port) => {
                            let url = format!("http://127.0.0.1:{port}/v1");
                            agy_provider = agy_provider.with_base_url(&url);
                            if is_antigravity {
                                agent.set_boxed_provider(Box::new(agy_provider.clone()));
                            }
                            println!("system > Antigravity base URL set to '{url}'.");
                            if !is_antigravity {
                                println!("system > Note: Antigravity is not currently the active provider. Run '/agy switch' to activate.");
                            }
                        }
                        AgySubcommand::Csrf(token) => {
                            if let Some(ref t) = token {
                                agy_provider = agy_provider.with_csrf_token(t);
                                println!("system > Antigravity CSRF token updated.");
                            } else {
                                agy_provider = agy_provider.with_optional_csrf_token(None);
                                println!("system > Antigravity CSRF token cleared.");
                            }
                            if is_antigravity {
                                agent.set_boxed_provider(Box::new(agy_provider.clone()));
                            }
                        }
                        AgySubcommand::Key(key) => {
                            if let Some(ref k) = key {
                                agy_provider = agy_provider.with_api_key(k);
                                println!("system > Antigravity API key updated.");
                            } else {
                                agy_provider = agy_provider.with_optional_api_key(None);
                                println!("system > Antigravity API key cleared.");
                            }
                            if is_antigravity {
                                agent.set_boxed_provider(Box::new(agy_provider.clone()));
                            }
                        }
                        AgySubcommand::Temperature(temp) => {
                            agy_provider = agy_provider.with_temperature(temp);
                            if is_antigravity {
                                agent.set_boxed_provider(Box::new(agy_provider.clone()));
                            }
                            println!("system > Antigravity temperature updated to {temp}.");
                        }
                        AgySubcommand::Timeout(timeout) => {
                            agy_provider = agy_provider.with_timeout(timeout);
                            if is_antigravity {
                                agent.set_boxed_provider(Box::new(agy_provider.clone()));
                            }
                            println!("system > Antigravity timeout updated to {timeout}s.");
                        }
                        AgySubcommand::Reset => {
                            agy_provider = AntigravityProvider::from_env();
                            if is_antigravity {
                                agent.set_boxed_provider(Box::new(agy_provider.clone()));
                            }
                            println!("system > Antigravity settings reloaded from environment.");
                            print_agy_status(&agy_provider, is_antigravity);
                        }
                        AgySubcommand::Switch => {
                            is_antigravity = true;
                            agent.set_boxed_provider(Box::new(agy_provider.clone()));
                            println!(
                                "system > Active provider switched to Antigravity (AGY) [{}: {}].",
                                agy_provider.model(),
                                agy_provider.base_url()
                            );
                        }
                    },
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
        assert_eq!(parse_command("/agy"), Command::Agy(AgySubcommand::Status));
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
            parse_command("/agy model gemini-2.5-pro"),
            Command::Agy(AgySubcommand::Model("gemini-2.5-pro".to_string()))
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
