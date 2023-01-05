use crate::{Entry, EntryList};

impl EntryList {
    /// Returns all the entries in this list, sorted by their `start_time`
    pub fn get_entries_in_order(&self, ascending: bool) -> Vec<&Entry> {
        let mut entries: Vec<_> = self.entries.values().collect();
        entries.sort_unstable();

        if !ascending {
            entries.reverse();
        }

        entries
    }
}
