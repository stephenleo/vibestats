mod checkpoint;
mod commands;
mod config;
mod github_api;
mod harnesses;
mod hooks;
mod logger;
mod sync;

use clap::{Parser, Subcommand};

#[derive(Clone, Debug, clap::ValueEnum)]
enum HarnessArg {
    All,
    Claude,
    Codex,
}

impl From<HarnessArg> for commands::sync::HarnessSelection {
    fn from(value: HarnessArg) -> Self {
        match value {
            HarnessArg::All => Self::All,
            HarnessArg::Claude => Self::Claude,
            HarnessArg::Codex => Self::Codex,
        }
    }
}

#[derive(Parser)]
#[command(
    name = "vibestats",
    version,
    about = "Track your Claude Code and Codex session activity"
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
        /// Which harness to sync. Defaults to all supported harnesses.
        #[arg(long, value_enum, default_value_t = HarnessArg::All)]
        harness: HarnessArg,
        /// Suppress human-readable output for hook execution
        #[arg(long, hide = true)]
        quiet: bool,
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
    /// Run the SessionStart hook
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
        /// Permanently delete all historical Hive partition files
        #[arg(long)]
        purge_history: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Sync {
            backfill,
            harness,
            quiet,
        } => commands::sync::run(backfill, harness.into(), quiet),
        Commands::Status => commands::status::run(),
        Commands::Machines { subcommand } => match subcommand {
            MachinesSubcommand::List => commands::machines::list(),
            MachinesSubcommand::Remove {
                machine_id,
                purge_history,
            } => commands::machines::remove(&machine_id, purge_history),
        },
        Commands::Auth => commands::auth::run(),
        Commands::Uninstall => commands::uninstall::run(),
        Commands::SessionStart => {
            hooks::session_start::run();
            std::process::exit(0);
        }
    }
}
