mod audit;
mod authorize;
mod generate;
mod init;
mod link;
mod list;
mod remove;
mod set;
mod show;

use crate::context::Context;
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
    /// Ensure ~/.ssh, ~/.ssh/conf.d, and ~/.ssh/config are ready for ssv-managed hosts
    #[command(visible_alias = "i")]
    Init,
    /// Generate a key pair and host configuration file
    #[command(visible_alias = "g")]
    Generate {
        /// Host identifier to manage
        #[arg(value_name = "HOST_ID")]
        host: String,
        /// HostName override for SSH config
        #[arg(short = 'n', long = "hostname", value_name = "HOSTNAME")]
        hostname: Option<String>,
        /// Key type to generate
        #[arg(short = 't', long = "type", default_value = "ed25519", value_name = "TYPE")]
        key_type: String,
        /// Optional user override for SSH config
        #[arg(short = 'u', long, value_name = "USER")]
        user: Option<String>,
        /// Optional port override for SSH config
        #[arg(short = 'p', long, value_name = "PORT")]
        port: Option<u16>,
    },
    /// Update HostName, user, or port for an existing managed host
    #[command(visible_alias = "s")]
    Set {
        /// Host identifier to update
        #[arg(value_name = "HOST")]
        host: String,
        /// New HostName for the SSH config
        #[arg(short = 'n', long = "hostname", value_name = "HOSTNAME")]
        hostname: Option<String>,
        /// New user for the SSH config
        #[arg(short = 'u', long, value_name = "USER")]
        user: Option<String>,
        /// New port for the SSH config
        #[arg(short = 'p', long, value_name = "PORT")]
        port: Option<u16>,
    },
    /// Install a managed host's public key on the remote server
    #[command(visible_alias = "az")]
    Authorize {
        /// Host identifier to authorize
        #[arg(value_name = "HOST")]
        host: String,
    },
    /// List managed hosts
    #[command(visible_alias = "ls")]
    List,
    /// Remove key pairs and configuration for a host
    #[command(visible_alias = "rm")]
    Remove {
        /// Host identifier to remove
        #[arg(value_name = "HOST")]
        host: String,
    },
    /// Print the public key for a managed host
    #[command(visible_alias = "sw")]
    Show { host: String },
    /// Link a repository to a managed host
    #[command(visible_alias = "ln")]
    Link {
        /// Host identifier to link
        #[arg(value_name = "HOST")]
        host: String,
    },
    /// Inspect managed SSH assets without modifying them
    #[command(visible_alias = "au")]
    Audit,
}

pub fn run() {
    let command = Cli::parse().command;
    let result = Context::from_env().and_then(|ctx| dispatch(&ctx, command));

    match result {
        Ok(Exit::Success) => {}
        Ok(Exit::Failure) => std::process::exit(1),
        Err(error) => {
            eprintln!("Error: {error}");
            std::process::exit(1);
        }
    }
}

fn dispatch(ctx: &Context, command: Commands) -> Result {
    match command {
        Commands::Init => init::run(ctx),
        Commands::Generate { host, hostname, key_type, user, port } => {
            generate::run(ctx, &host, hostname.as_deref(), &key_type, user.as_deref(), port)
        }
        Commands::Set { host, hostname, user, port } => {
            set::run(ctx, &host, hostname.as_deref(), user.as_deref(), port)
        }
        Commands::Authorize { host } => authorize::run(ctx, &host),
        Commands::List => list::run(ctx),
        Commands::Remove { host } => remove::run(ctx, &host),
        Commands::Show { host } => show::run(ctx, &host),
        Commands::Link { host } => link::run(ctx, &host),
        Commands::Audit => audit::run(ctx),
    }
}

pub(crate) enum Exit {
    Success,
    Failure,
}

pub(crate) type Result = std::result::Result<Exit, AppError>;
