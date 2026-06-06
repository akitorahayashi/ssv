mod audit;
mod generate;
mod list;
mod remove;
mod show;

use crate::error::AppError;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ssv")]
#[command(version)]
#[command(about = "Lifecycle manager for SSH keys and configuration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a key pair and host configuration file
    #[command(visible_alias = "gen")]
    Generate {
        /// Hostname to manage
        #[arg(long, value_name = "HOST")]
        host: String,
        /// Key type to generate
        #[arg(long = "type", default_value = "ed25519", value_name = "TYPE")]
        key_type: String,
        /// Optional user override for SSH config
        #[arg(long, value_name = "USER")]
        user: Option<String>,
        /// Optional port override for SSH config
        #[arg(long, value_name = "PORT")]
        port: Option<u16>,
    },
    /// List managed hosts
    #[command(visible_alias = "ls")]
    List,
    /// Remove key pairs and configuration for a host
    #[command(visible_alias = "rm")]
    Remove {
        /// Hostname to remove
        #[arg(long, value_name = "HOST")]
        host: String,
    },
    /// Print the public key for a managed host
    Show { host: String },
    /// Inspect managed SSH assets without modifying them
    Audit,
}

pub fn run() {
    let result = match Cli::parse().command {
        Commands::Generate { host, key_type, user, port } => {
            generate::run(&host, &key_type, user.as_deref(), port)
        }
        Commands::List => list::run(),
        Commands::Remove { host } => remove::run(&host),
        Commands::Show { host } => show::run(&host),
        Commands::Audit => audit::run(),
    };

    match result {
        Ok(Exit::Success) => {}
        Ok(Exit::Failure) => std::process::exit(1),
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}

pub(crate) enum Exit {
    Success,
    Failure,
}

pub(crate) type Result = std::result::Result<Exit, AppError>;
