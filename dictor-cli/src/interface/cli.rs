// interface/cli.rs — CLI adapter
//
// Thin layer: parses CLI args, outputs result to stdout. No business logic.

use clap::{Parser, Subcommand};

/// Command-line arguments for Dictor CLI.
/// `#[derive(Parser)]` — clap macro that auto-generates argument parsing.
#[derive(Parser, Debug)]
#[command(
    name = "dictor-cli",
    about = "CLI client for Dictor STT server",
    version,
    // With a subcommand, do not require parent positional args (`AUDIO_FILE`).
    subcommand_negates_reqs = true
)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Path to the audio file (not used with `health`)
    #[arg(value_name = "AUDIO_FILE")]
    pub audio_file: Option<String>,

    /// STT server URL
    #[arg(
        long = "server",
        global = true,
        default_value = "http://localhost:8000"
    )]
    pub server_url: String,

    /// Preprompt for STT (model hint about language/context)
    #[arg(long = "preprompt", global = true, default_value = "")]
    pub preprompt: String,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
pub enum Commands {
    /// Check STT server health (GET /health)
    Health,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_parse_audio_file_only() {
        let args = CliArgs::parse_from(["dictor-cli", "audio.wav"]);
        assert_eq!(args.audio_file.as_deref(), Some("audio.wav"));
        assert_eq!(args.command, None);
        assert_eq!(args.server_url, "http://localhost:8000");
        assert_eq!(args.preprompt, "");
    }

    #[test]
    fn test_parse_with_custom_server() {
        let args = CliArgs::parse_from([
            "dictor-cli",
            "audio.wav",
            "--server",
            "http://example.com:9000",
        ]);
        assert_eq!(args.audio_file.as_deref(), Some("audio.wav"));
        assert_eq!(args.server_url, "http://example.com:9000");
        assert_eq!(args.preprompt, "");
    }

    #[test]
    fn test_parse_with_preprompt() {
        let args = CliArgs::parse_from([
            "dictor-cli",
            "audio.wav",
            "--preprompt",
            "Always return the text in the same language as the input.",
        ]);
        assert_eq!(args.audio_file.as_deref(), Some("audio.wav"));
        assert_eq!(args.preprompt, "Always return the text in the same language as the input.");
    }

    #[test]
    fn test_parse_bare_invocation_no_positional() {
        let args = CliArgs::try_parse_from(["dictor-cli"]).expect("parse");
        assert_eq!(args.command, None);
        assert_eq!(args.audio_file, None);
    }

    #[test]
    fn test_parse_health_subcommand() {
        let args = CliArgs::parse_from(["dictor-cli", "health"]);
        assert_eq!(args.command, Some(Commands::Health));
        assert_eq!(args.audio_file, None);
        assert_eq!(args.server_url, "http://localhost:8000");
    }

    #[test]
    fn test_parse_health_with_custom_server() {
        let args = CliArgs::parse_from([
            "dictor-cli",
            "health",
            "--server",
            "http://example.com:9000",
        ]);
        assert_eq!(args.command, Some(Commands::Health));
        assert_eq!(args.server_url, "http://example.com:9000");
    }
}
