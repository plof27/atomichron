use std::{collections::HashMap, time::SystemTime};

use uuid::Uuid;

/// A single time entry
#[derive(Debug)]
struct Entry {
    id: Uuid,

    project: Option<String>,
    description: Option<String>,
    tags: Vec<String>,

    start_time: SystemTime,
    end_time: Option<SystemTime>,
}

impl Entry {
    fn new(project: Option<String>, description: Option<String>, tags: Vec<String>) -> Self {
        Entry {
            id: Uuid::new_v4(),
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

/// A set of time entries
#[derive(Debug, Default)]
pub struct EntryList {
    /// All entries
    entries: HashMap<Uuid, Entry>,

    /// The currently running entry, if any.
    ///
    /// This field is set when a new entry is started, and cleared when it is stopped (or reset)
    current_entry: Option<Uuid>,
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

        self.current_entry = Some(entry.id);
        self.entries.insert(entry.id, entry);
    }

    /// Stops the current entry, if any
    ///
    /// Returns the [`Uuid`] of the entry stopped, if anything was actually stopped
    pub fn stop_current_entry(&mut self) -> Option<Uuid> {
        if let Some(id) = self.current_entry {
            self.entries
                .get_mut(&id)
                .expect("Failed to fetch current entry") // TODO: 2022-10-15 replace with actual error logging
                .stop();

            self.current_entry = None;

            Some(id)
        } else {
            None
        }
    }

    /// Stops and discards the current entry, if any
    ///
    /// This effectively lets you "cancel" a entry that was started incorrectly
    /// Returns the [`Uuid`] of the entry stopped, if anything was actually stopped
    pub fn clear_current_entry(&mut self) -> Option<Uuid> {
        if let Some(id) = self.current_entry {
            self.entries.remove(&id);
            self.current_entry = None;

            Some(id)
        } else {
            None
        }
    }
}
