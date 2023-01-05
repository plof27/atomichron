use clap::{Args, Parser, Subcommand};
use std::fmt::Display;

use atomichron::EntryList;

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
    Clear,
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
    #[arg(short, long)]
    description: Option<String>,

    /// Optional list of tags for this entry, separated by commas
    #[arg(short, long, value_delimiter = ',')]
    tags: Vec<String>,
}

impl Display for EntryInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} {:?}",
            self.project.as_ref().unwrap_or(&"".to_string()),
            self.description.as_ref().unwrap_or(&"".to_string()),
            self.tags
        )
    }
}

fn main() -> atomichron::Result<()> {
    // Load entries
    let mut entries = EntryList::load_or_create("./entries.ron")?;

    // Read and process args
    let args = Cli::parse();

    match &args.command {
        Commands::Start(info) => {
            if let Some(entry) = entries.stop_current_entry(None, None, Vec::new()) {
                println!("Stopping entry {}", entry);
            }

            let new_entry = entries.start_entry(
                info.project.clone(),
                info.description.clone(),
                info.tags.clone(),
            );
            println!("Starting entry {}", new_entry);
        }
        Commands::Stop(info) => match entries.stop_current_entry(
            info.project.clone(),
            info.description.clone(),
            info.tags.clone(),
        ) {
            Some(entry) => println!("Stopping entry {}", entry),
            None => println!("No entry started"),
        },
        Commands::Clear => match entries.clear_current_entry() {
            Some(entry) => println!("Clearing entry {}", entry),
            None => println!("No entry started"),
        },
        Commands::Status => match entries.current_entry() {
            Some(entry) => println!("Running timer for {}", entry),
            None => println!("No entry started"),
        },
        Commands::Log => {
            let list = entries.get_entries_in_order(false);
            for entry in list {
                println!("{}", entry);
            }
        }
    }

    // Save updated entries
    entries.save("./entries.ron")?;

    Ok(())
}
