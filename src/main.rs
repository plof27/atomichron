use clap::{Args, Parser, Subcommand};
use std::{
    fmt::Display,
    fs::{self, File},
};

use atomichron::{EntryList, EVENT_OUTPUT_TARGET};

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

fn main() {
    // Init logger
    fern::Dispatch::new()
        .format(|out, message, record| out.finish(format_args!("[{}] {}", record.level(), message)))
        .filter(|metadata| metadata.target() == EVENT_OUTPUT_TARGET)
        .chain(std::io::stdout())
        .apply()
        .expect("Failed to initialize logger");

    // Load entries
    let mut entries = match fs::read("./entries.ron") {
        Ok(bytes) => ron::de::from_bytes(&bytes).expect("Failure deserializing time entries"),
        Err(_) => EntryList::new(),
    };

    // Read and process args
    let args = Cli::parse();

    match &args.command {
        Commands::Start(info) => {
            entries.start_entry(
                info.project.clone(),
                info.description.clone(),
                info.tags.clone(),
            );
        }
        Commands::Stop(info) => {
            if entries.stop_current_entry().is_none() {
                println!("No entry started");
            }
        }
        Commands::Clear => {
            if entries.clear_current_entry().is_none() {
                println!("No entry started");
            }
        }
        Commands::Status => match entries.current_entry() {
            Some(entry) => println!("Started project {:?}", entry),
            None => println!("No entry started"),
        },
        Commands::Log => todo!(),
    }

    // Save updated entries
    let out_file = File::create("./entries.ron").expect("Failure opening entries file for writing");
    ron::ser::to_writer(out_file, &entries).expect("Failure writing entries file");
}
