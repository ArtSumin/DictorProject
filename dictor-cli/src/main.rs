// main.rs — CLI entry point.
// Only dependency injection and startup here.
// No business logic.
//
// Order:
// 1. Parse CLI arguments (interface)
// 2. Create HTTP client (infrastructure)
// 3. Create use case (application)
// 4. Execute and output result

use clap::Parser;
use std::process;

// Import layers from lib.rs
use dictor::application::health::HealthCheckUseCase;
use dictor::application::transcribe::TranscribeUseCase;
use dictor::interface::cli::{CliArgs, Commands};
use dictor::infrastructure::stt_client::HttpSttClient;

fn main() {
    let args = CliArgs::parse();

    match args.command {
        Some(Commands::Health) => {
            let client = HttpSttClient::new(&args.server_url);
            let use_case = HealthCheckUseCase::new(Box::new(client));
            match use_case.execute() {
                Ok(status) => {
                    if status.is_ready() {
                        println!("OK (model loaded)");
                    } else {
                        println!(
                            "Server responded: status={}, model_loaded={}",
                            status.status, status.model_loaded
                        );
                    }
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    process::exit(1);
                }
            }
        }
        None => {
            let Some(audio_file) = args.audio_file else {
                eprintln!(
                    "Error: missing AUDIO_FILE. Usage: dictor-cli <AUDIO_FILE> or dictor-cli health"
                );
                process::exit(2);
            };
            let client = HttpSttClient::new(&args.server_url);
            let use_case = TranscribeUseCase::new(Box::new(client));
            match use_case.execute(&audio_file, &args.preprompt) {
                Ok(result) => println!("{}", result.text),
                Err(err) => {
                    eprintln!("Error: {}", err);
                    process::exit(1);
                }
            }
        }
    }
}
