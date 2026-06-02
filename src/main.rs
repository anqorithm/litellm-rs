//! LiteLLM-RS - High-performance async AI gateway
//!
//! Async gateway service supporting multiple AI providers

#![allow(missing_docs)]

use clap::parser::ValueSource;
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use litellm_rs::server;
use litellm_rs::storage::database::Database;
use litellm_rs::{Config, VERSION};
use std::path::PathBuf;
use std::process::ExitCode;
#[cfg(any(feature = "tracing", test))]
use tracing::Level;

const DEFAULT_CONFIG_PATH: &str = "config/gateway.yaml";

#[derive(Debug, Parser)]
#[command(
    name = "gateway",
    version = VERSION,
    about = "Run and manage the LiteLLM-RS gateway"
)]
struct Cli {
    /// Path to the gateway configuration file.
    #[arg(
        short,
        long,
        global = true,
        default_value = DEFAULT_CONFIG_PATH,
        value_name = "FILE"
    )]
    config: PathBuf,

    /// Override the configured bind host.
    #[arg(long, global = true, value_name = "HOST")]
    host: Option<String>,

    /// Override the configured bind port.
    #[arg(long, global = true, value_name = "PORT")]
    port: Option<u16>,

    /// Set gateway log verbosity.
    #[arg(long, global = true, value_name = "LEVEL")]
    log_level: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Start the gateway server.
    Serve,
    /// Validate the gateway configuration file.
    ValidateConfig,
    /// Manage gateway database state.
    Database {
        #[command(subcommand)]
        command: DatabaseCommands,
    },
}

#[derive(Debug, Subcommand)]
enum DatabaseCommands {
    /// Run database migrations.
    Migrate,
}

#[derive(Debug)]
struct ParsedCli {
    cli: Cli,
    config_is_explicit: bool,
}

fn parse_cli_from<I, T>(args: I) -> Result<ParsedCli, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let matches = Cli::command().try_get_matches_from(args)?;
    let config_is_explicit = matches.value_source("config") == Some(ValueSource::CommandLine);
    let cli = Cli::from_arg_matches(&matches)?;

    Ok(ParsedCli {
        cli,
        config_is_explicit,
    })
}

#[cfg(any(feature = "tracing", test))]
fn parse_log_level(level: Option<&str>) -> Level {
    match level.unwrap_or("info").to_ascii_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "warn" | "warning" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    }
}

fn init_logging(log_level: Option<&str>) {
    #[cfg(feature = "tracing")]
    {
        tracing_subscriber::fmt()
            .with_max_level(parse_log_level(log_level))
            .with_target(false)
            .with_thread_ids(false)
            .init();
    }
    #[cfg(not(feature = "tracing"))]
    let _ = log_level;
}

async fn load_config(config_path: &PathBuf) -> litellm_rs::Result<Config> {
    load_config_with_overrides(config_path, None, None).await
}

async fn load_config_with_overrides(
    config_path: &PathBuf,
    host: Option<&str>,
    port: Option<u16>,
) -> litellm_rs::Result<Config> {
    let mut config = Config::from_file(config_path).await?;
    if let Some(host) = host {
        config.gateway.server.host = host.to_string();
    }
    if let Some(port) = port {
        config.gateway.server.port = port;
    }
    config.validate()?;
    Ok(config)
}

#[tokio::main]
async fn main() -> ExitCode {
    let ParsedCli {
        cli,
        config_is_explicit,
    } = parse_cli_from(std::env::args_os()).unwrap_or_else(|error| error.exit());
    init_logging(cli.log_level.as_deref());

    let command = cli.command.unwrap_or(Commands::Serve);
    let result = match command {
        Commands::Serve => {
            if config_is_explicit {
                server::builder::run_server_with_config_overrides(
                    &cli.config,
                    cli.host.as_deref(),
                    cli.port,
                )
                .await
            } else {
                server::builder::run_server_with_default_config_overrides(
                    cli.host.as_deref(),
                    cli.port,
                )
                .await
            }
        }
        Commands::ValidateConfig => {
            match load_config_with_overrides(&cli.config, cli.host.as_deref(), cli.port).await {
                Ok(_) => {
                    println!("Configuration is valid: {}", cli.config.display());
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        Commands::Database {
            command: DatabaseCommands::Migrate,
        } => match load_config(&cli.config).await {
            Ok(config) => match Database::new(&config.storage().database).await {
                Ok(database) => {
                    let result = database.migrate().await;
                    if result.is_ok() {
                        println!("Database migrations completed");
                    }
                    result
                }
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        },
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // Print error using Display (not Debug) to preserve newlines
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_trailing_global_config_for_migration() {
        let parsed = match parse_cli_from([
            "gateway",
            "database",
            "migrate",
            "--config",
            "/tmp/gateway.yaml",
        ]) {
            Ok(parsed) => parsed,
            Err(error) => panic!("expected database migrate args to parse: {error}"),
        };
        let cli = parsed.cli;

        assert_eq!(cli.config, PathBuf::from("/tmp/gateway.yaml"));
        assert!(parsed.config_is_explicit);
        match cli.command {
            Some(Commands::Database {
                command: DatabaseCommands::Migrate,
            }) => {}
            _ => panic!("expected database migrate command"),
        }
    }

    #[test]
    fn defaults_to_config_file_when_omitted() {
        let parsed = match parse_cli_from(["gateway", "validate-config"]) {
            Ok(parsed) => parsed,
            Err(error) => panic!("expected validate-config args to parse: {error}"),
        };
        let cli = parsed.cli;
        assert_eq!(cli.config, PathBuf::from("config/gateway.yaml"));
        assert!(!parsed.config_is_explicit);
        assert!(matches!(cli.command, Some(Commands::ValidateConfig)));
    }

    #[test]
    fn serve_without_config_uses_implicit_default_config() {
        let parsed = match parse_cli_from(["gateway", "serve"]) {
            Ok(parsed) => parsed,
            Err(error) => panic!("expected serve args to parse: {error}"),
        };
        let cli = parsed.cli;

        assert_eq!(cli.config, PathBuf::from(DEFAULT_CONFIG_PATH));
        assert!(!parsed.config_is_explicit);
        assert!(matches!(cli.command, Some(Commands::Serve)));
    }

    #[test]
    fn accepts_legacy_startup_overrides() {
        let parsed = match parse_cli_from([
            "gateway",
            "--config",
            "/tmp/gateway.yaml",
            "--host",
            "0.0.0.0",
            "--port",
            "8080",
            "--log-level",
            "debug",
        ]) {
            Ok(parsed) => parsed,
            Err(error) => panic!("expected legacy startup args to parse: {error}"),
        };
        let cli = parsed.cli;

        assert_eq!(cli.config, PathBuf::from("/tmp/gateway.yaml"));
        assert!(parsed.config_is_explicit);
        assert_eq!(cli.host.as_deref(), Some("0.0.0.0"));
        assert_eq!(cli.port, Some(8080));
        assert_eq!(cli.log_level.as_deref(), Some("debug"));
        assert!(cli.command.is_none());
    }

    #[test]
    fn parses_log_levels() {
        assert_eq!(parse_log_level(Some("trace")), Level::TRACE);
        assert_eq!(parse_log_level(Some("debug")), Level::DEBUG);
        assert_eq!(parse_log_level(Some("warn")), Level::WARN);
        assert_eq!(parse_log_level(Some("error")), Level::ERROR);
        assert_eq!(parse_log_level(Some("unknown")), Level::INFO);
    }
}
