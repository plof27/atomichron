use std::{collections::HashMap, fmt::Display, time::SystemTime};

use log::info;
use serde::{Deserialize, Serialize};
use uuid::{Bytes, Uuid};

/// Log target for messages that are intended to be output directly
///
/// These log events map to meaningful, user-facing events and information.
/// Used primarily for the CLI.
pub const EVENT_OUTPUT_TARGET: &str = "atomichron::events_output";

/// A single time entry
#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    id: Bytes,

    project: Option<String>,
    description: Option<String>,
    tags: Vec<String>,

    start_time: SystemTime,
    end_time: Option<SystemTime>,
}

impl Entry {
    fn new(project: Option<String>, description: Option<String>, tags: Vec<String>) -> Self {
        Entry {
            id: Uuid::new_v4().into_bytes(),
            project,
            description,
            tags,
            start_time: SystemTime::now(),
            end_time: None,
        }
    }

    fn stop(&mut self) {
        if self.end_time.is_none() {
            self.end_time = Some(SystemTime::now());
        } // TODO: 2022-10-15 emit a warning if end_time is Some
    }
}

impl Display for Entry {
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

/// A set of time entries
///
/// We have to use raw [`Bytes`] here because [`Uuid`] doesn't implement [`Serialize`] or [`Deserialize`].
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct EntryList {
    /// All entries
    entries: HashMap<Bytes, Entry>,

    /// The currently running entry, if any.
    ///
    /// This field is set when a new entry is started, and cleared when it is stopped (or reset)
    current_entry: Option<Bytes>,
}

impl EntryList {
    /// Creates a new, empty list
    pub fn new() -> Self {
        EntryList {
            entries: HashMap::new(),
            current_entry: None,
        }
    }

    /// Starts a new entry
    pub fn start_entry(
        &mut self,
        project: Option<String>,
        description: Option<String>,
        tags: Vec<String>,
    ) {
        // Stop current entry, if any
        self.stop_current_entry();

        let entry = Entry::new(project, description, tags);
        info!(target: EVENT_OUTPUT_TARGET, "Starting entry: {}", entry);

        self.current_entry = Some(entry.id);
        self.entries.insert(entry.id, entry);
    }

    /// Stops the current entry, if any
    ///
    /// Returns the [`Entry`] of the entry stopped, if anything was actually stopped
    pub fn stop_current_entry(&mut self) -> Option<&Entry> {
        if let Some(id) = self.current_entry {
            let entry = self
                .entries
                .get_mut(&id)
                .expect("Failed to fetch current entry");
            info!(target: EVENT_OUTPUT_TARGET, "Stopping entry: {}", entry);

            entry.stop();
            self.current_entry = None;

            Some(entry)
        } else {
            None
        }
    }

    /// Stops and discards the current entry, if any
    ///
    /// This effectively lets you "cancel" a entry that was started incorrectly
    /// Returns the [`Entry`] of the entry stopped, if anything was actually stopped
    pub fn clear_current_entry(&mut self) -> Option<Entry> {
        if let Some(id) = self.current_entry {
            let entry = self
                .entries
                .remove(&id)
                .expect("Failed to fetch current entry");
            self.current_entry = None;

            info!(target: EVENT_OUTPUT_TARGET, "Clearing entry: {}", entry);

            Some(entry)
        } else {
            None
        }
    }

    /// Gets the current entry, if any
    pub fn current_entry(&self) -> Option<&Entry> {
        self.current_entry.map(|id| {
            self.entries
                .get(&id)
                .expect("Failed to fetch current entry")
        })
    }
}
