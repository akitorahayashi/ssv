use clap::{Parser, Subcommand};
use ssv::commands;
use ssv::error::AppError;

#[derive(Parser)]
#[command(name = "ssv")]
#[command(about = "Lifecycle manager for SSH keys and configuration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a key pair and host configuration file
    #[clap(visible_alias = "gen")]
    Generate {
        /// Hostname to manage (e.g., github.com)
        #[arg(long, value_name = "HOST")]
        host: String,
        /// Key type to generate (default: ed25519)
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
    #[clap(visible_alias = "ls")]
    List,
    /// Remove key pairs and configuration for a host
    #[clap(visible_alias = "rm")]
    Remove {
        /// Hostname to remove
        #[arg(long, value_name = "HOST")]
        host: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result: Result<(), AppError> = match cli.command {
        Commands::Generate { host, key_type, user, port } => {
            commands::generate(&host, &key_type, user.as_deref(), port)
        }
        Commands::List => commands::list().map(|_| ()),
        Commands::Remove { host } => commands::remove(&host),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
