use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    fs::{self, File},
    io::ErrorKind,
    path::Path,
    time::SystemTime,
};
use uuid::{Bytes, Uuid};

use crate::{errors::Result, Error};

/// A single time entry
#[derive(Debug, Eq, Serialize, Deserialize)]
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

    /// Get the raw id of this entry
    ///
    /// Note: for most uses, `uuid` is preferred
    pub fn id(&self) -> Bytes {
        self.id
    }

    /// Get the [`Uuid`] of this entry
    pub fn uuid(&self) -> Uuid {
        Uuid::from_bytes(self.id)
    }

    /// Get the project string of this entry, if set
    pub fn project(&self) -> &Option<String> {
        &self.project
    }

    /// Get the description string of this entry, if set
    pub fn description(&self) -> &Option<String> {
        &self.description
    }

    /// Get the tags of this entry
    ///
    /// Note: for entries without tags, this will be an empty [`Vec`]
    pub fn tags(&self) -> &Vec<String> {
        &self.tags
    }

    /// Get the start time of this entry
    pub fn start_time(&self) -> SystemTime {
        self.start_time
    }

    /// Get the end time of this entry, if it's finished
    pub fn end_time(&self) -> Option<SystemTime> {
        self.end_time
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

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.start_time.partial_cmp(&other.start_time)
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start_time.cmp(&other.start_time)
    }
}

/// Error message to use for `.expect(...)` when attempting to retrieve the current entry from the entry list
///
/// This case comes up a lot, so it's useful to standardize the message.
const NO_CURRENT_ENTRY_MESSAGE: &str = "Failure retrieving current entry from entry list";

/// A set of time entries
///
/// We have to use raw [`Bytes`] here because [`Uuid`] doesn't implement [`Serialize`] or [`Deserialize`].
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct EntryList {
    /// All entries
    pub(crate) entries: HashMap<Bytes, Entry>,

    /// The currently running entry, if any.
    ///
    /// This field is set when a new entry is started, and cleared when it is stopped (or reset)
    pub(crate) current_entry: Option<Bytes>,
}

impl EntryList {
    /// Creates a new, empty list
    pub fn new() -> Self {
        EntryList {
            entries: HashMap::new(),
            current_entry: None,
        }
    }

    /// Deserializes an entry list from the file path provided
    ///
    /// # Errors
    /// - Returns an error if anything goes wrong reading the file
    pub fn load<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        match fs::read(path) {
            Ok(bytes) => ron::de::from_bytes(&bytes).map_err(Error::from),
            Err(e) => Err(e.into()),
        }
    }

    /// Deserializes an entry list from the file path provided, or creates a new
    /// one if the file does not exist.
    ///
    /// # Errors
    /// - Returns and error if anything goes wrong reading the file (other than not finding it)
    pub fn load_or_create<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        match fs::read(path) {
            Ok(bytes) => ron::de::from_bytes(&bytes).map_err(Error::from),
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    Ok(EntryList::new())
                } else {
                    Err(e.into())
                }
            }
        }
    }

    /// Serializes and saves this entry list to the path provided
    pub fn save<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let out_file = File::create(path)?;
        ron::ser::to_writer(out_file, self)?;
        Ok(())
    }

    /// Starts a new entry
    ///
    /// Returns the newly created [`Entry`]
    pub fn start_entry(
        &mut self,
        project: Option<String>,
        description: Option<String>,
        tags: Vec<String>,
    ) -> &Entry {
        let entry = Entry::new(project, description, tags);
        let id = entry.id;

        self.current_entry = Some(id);
        self.entries.insert(id, entry);

        self.entries.get(&id).expect(NO_CURRENT_ENTRY_MESSAGE)
    }

    /// Stops the current entry, if any
    ///
    /// Returns the [`Entry`] of the entry stopped, if anything was actually stopped
    pub fn stop_current_entry(
        &mut self,
        project: Option<String>,
        description: Option<String>,
        tags: Vec<String>,
    ) -> Option<&Entry> {
        if let Some(id) = self.current_entry {
            let entry = self.entries.get_mut(&id).expect(NO_CURRENT_ENTRY_MESSAGE);

            // Stop the timer
            entry.stop();
            self.current_entry = None;

            // Update information based on what was provided
            if project.is_some() {
                entry.project = project;
            }
            if description.is_some() {
                entry.description = description;
            }
            if tags.len() > 0 {
                entry.tags = tags;
            }

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
            let entry = self.entries.remove(&id).expect(NO_CURRENT_ENTRY_MESSAGE);
            self.current_entry = None;

            Some(entry)
        } else {
            None
        }
    }

    /// Gets the current entry, if any
    pub fn current_entry(&self) -> Option<&Entry> {
        self.current_entry
            .map(|id| self.entries.get(&id).expect(NO_CURRENT_ENTRY_MESSAGE))
    }
}
