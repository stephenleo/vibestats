mod checkpoint;
mod config;
mod github_api;
mod hooks;
mod jsonl_parser;
mod logger;
mod sync;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "vibestats",
    version,
    about = "Track your Claude Code session activity"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync session data to vibestats-data
    Sync {
        /// Run a full historical backfill
        #[arg(long)]
        backfill: bool,
    },
    /// Show current sync status and last sync time
    Status,
    /// Manage registered machines
    Machines {
        #[command(subcommand)]
        subcommand: MachinesSubcommand,
    },
    /// Authenticate with GitHub
    Auth,
    /// Uninstall vibestats
    Uninstall,
    /// Run the SessionStart hook (called by Claude Code at session start)
    SessionStart,
}

#[derive(Subcommand)]
enum MachinesSubcommand {
    /// List all registered machines
    List,
    /// Remove a machine by ID
    Remove {
        /// Machine ID to remove
        machine_id: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Sync { backfill: _ } => println!("not yet implemented"),
        Commands::Status => println!("not yet implemented"),
        Commands::Machines { subcommand } => match subcommand {
            MachinesSubcommand::List => println!("not yet implemented"),
            MachinesSubcommand::Remove { machine_id: _ } => println!("not yet implemented"),
        },
        Commands::Auth => println!("not yet implemented"),
        Commands::Uninstall => println!("not yet implemented"),
        Commands::SessionStart => {
            hooks::session_start::run();
            std::process::exit(0);
        }
    }
}
