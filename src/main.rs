//! CLI Binary Entrypoint & Interactive REPL for HarnessMe.
//!
//! Provides a terminal-based interface to interact with autonomous agents,
//! test tool executions, and inspect conversation state in real-time.

use std::env;
use std::io::{self, Write};

use harnessme::{
    Agent, AgentConfig, CalculatorTool, EchoTool, OpenAiCompatibleProvider, ToolRegistry,
};

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_MODEL: &str = "gpt-4o-mini";
const DEFAULT_MAX_STEPS: usize = 10;
const DEFAULT_TEMPERATURE: f32 = 0.7;
const DEFAULT_SYSTEM_PROMPT: &str =
    "You are a helpful and concise AI assistant equipped with tools. \
When requested to perform calculations or operations, invoke the appropriate tools accurately.";

fn main() {
    let api_key = env::var("OPENAI_API_KEY").ok();
    let base_url = env::var("OPENAI_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
    let model = env::var("HARNESS_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());
    let system_prompt =
        env::var("HARNESS_SYSTEM_PROMPT").unwrap_or_else(|_| DEFAULT_SYSTEM_PROMPT.to_string());
    let max_iterations = env::var("HARNESS_MAX_STEPS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_MAX_STEPS);
    let temperature = env::var("HARNESS_TEMPERATURE")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(DEFAULT_TEMPERATURE);

    println!("====================================================");
    println!("             HarnessMe Agent Interactive CLI         ");
    println!("====================================================");
    println!(" Model        : {model}");
    println!(" Base URL     : {base_url}");
    println!(
        " API Key      : {}",
        if api_key.is_some() {
            "Configured (hidden)"
        } else {
            "None (local endpoint mode)"
        }
    );
    println!(" Max Steps    : {max_iterations}");
    println!(" Temperature  : {temperature}");
    println!("----------------------------------------------------");

    if api_key.is_none() && base_url.starts_with("https://api.openai.com") {
        eprintln!(
            "Notice: OPENAI_API_KEY is not set. If connecting to OpenAI, please export OPENAI_API_KEY."
        );
        eprintln!("For local models (Ollama/vLLM), configure OPENAI_BASE_URL (e.g. http://localhost:11434/v1).");
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
    println!(" Commands: 'exit'/'quit' to exit, 'clear' to reset history, 'history' to inspect.");
    println!("====================================================\n");

    // Initialize provider
    let provider = OpenAiCompatibleProvider::new(&model, api_key)
        .with_base_url(&base_url)
        .with_temperature(temperature);

    // Initialize agent
    let config = AgentConfig::new()
        .with_system_prompt(system_prompt)
        .with_max_iterations(max_iterations);

    let mut agent = Agent::with_config(provider, registry, config);

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
                // EOF reached
                println!("\nGoodbye!");
                break;
            }
            Ok(_) => {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    continue;
                }

                match trimmed.to_lowercase().as_str() {
                    "exit" | "quit" => {
                        println!("Goodbye!");
                        break;
                    }
                    "clear" | "reset" => {
                        agent.clear_history();
                        println!("system > Conversation history cleared.");
                        continue;
                    }
                    "history" => {
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
                        continue;
                    }
                    _ => {}
                }

                match agent.run(trimmed) {
                    Ok(response) => {
                        println!("agent > {response}\n");
                    }
                    Err(err) => {
                        eprintln!("error > {err}\n");
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
