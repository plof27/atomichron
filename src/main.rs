use clap::{ArgAction, Args, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Starts a new time entry. If the timer is currently running, the current entry will be stopped and the new one started.
    Start(EntryInfo),
    /// Stops the current time entry. If project or description are provided, they will overwrite any project or description set when the timer was started.
    Stop(EntryInfo),
    /// Stops the current time entry, then discards it.
    Reset,
    /// Displays the current status.
    Status,
    /// Logs all entries, grouped by day.
    Log,
}

#[derive(Args)]
struct EntryInfo {
    /// Optional project for this entry
    project: Option<String>,
    /// Optional description for this entry
    description: Option<String>,

    /// Optional list of tags for this entry
    #[arg(short, long, action=ArgAction::Append)]
    tags: Vec<String>,
}

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Start(info) => {
            println!(
                "Starting {:?}: {:?} +{:?}",
                info.project, info.description, info.tags
            );
        }
        Commands::Stop(info) => todo!(),
        Commands::Reset => todo!(),
        Commands::Status => todo!(),
        Commands::Log => todo!(),
    }
}
