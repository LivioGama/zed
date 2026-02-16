use crate::ProjectPath;
use collections::HashMap;
use std::time::SystemTime;

const DEFAULT_MAX_ENTRIES_PER_PATH: usize = 50;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalHistoryEntry {
    pub captured_at: SystemTime,
    pub text: String,
}

#[derive(Debug)]
pub struct LocalHistory {
    max_entries_per_path: usize,
    entries_by_path: HashMap<ProjectPath, Vec<LocalHistoryEntry>>,
}

impl Default for LocalHistory {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_ENTRIES_PER_PATH)
    }
}

impl LocalHistory {
    pub fn new(max_entries_per_path: usize) -> Self {
        Self {
            max_entries_per_path,
            entries_by_path: HashMap::default(),
        }
    }

    pub fn add_entry(&mut self, path: ProjectPath, captured_at: SystemTime, text: String) {
        let entries = self.entries_by_path.entry(path).or_default();
        entries.insert(0, LocalHistoryEntry { captured_at, text });

        if entries.len() > self.max_entries_per_path {
            entries.truncate(self.max_entries_per_path);
        }
    }

    pub fn entries_for_path(&self, path: &ProjectPath) -> &[LocalHistoryEntry] {
        self.entries_by_path
            .get(path)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn entries_for_prefix<'a>(
        &'a self,
        prefix: &'a ProjectPath,
    ) -> impl Iterator<Item = (&'a ProjectPath, &'a [LocalHistoryEntry])> + 'a {
        self.entries_by_path
            .iter()
            .filter(move |(path, _)| path.starts_with(prefix))
            .map(|(path, entries)| (path, entries.as_slice()))
    }

    pub fn clear_path(&mut self, path: &ProjectPath) {
        self.entries_by_path.remove(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WorktreeId;
    use util::rel_path::rel_path;

    fn build_path(name: &str) -> ProjectPath {
        ProjectPath {
            worktree_id: WorktreeId::from_usize(1),
            path: rel_path(name).into(),
        }
    }

    #[test]
    fn local_history_retains_only_the_most_recent_entries() {
        let mut history = LocalHistory::new(2);
        let path = build_path("test.rs");

        history.add_entry(path.clone(), SystemTime::UNIX_EPOCH, "first".to_string());
        history.add_entry(path.clone(), SystemTime::UNIX_EPOCH, "second".to_string());
        history.add_entry(path.clone(), SystemTime::UNIX_EPOCH, "third".to_string());

        let entries = history.entries_for_path(&path);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "third");
        assert_eq!(entries[1].text, "second");
    }

    #[test]
    fn local_history_clear_path_removes_all_entries() {
        let mut history = LocalHistory::new(5);
        let path = build_path("test.rs");

        history.add_entry(path.clone(), SystemTime::UNIX_EPOCH, "entry".to_string());
        history.clear_path(&path);

        assert!(history.entries_for_path(&path).is_empty());
    }
}
