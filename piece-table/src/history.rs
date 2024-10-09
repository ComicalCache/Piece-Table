pub(crate) mod change;
pub(crate) mod commit;
pub(crate) mod entry;

use std::ops::Not;

use commit::Commit;
use entry::Entry;

/// Manipulation history of a Piece Tree.
///
/// Creates a tree of changes that can be traversed forward and backward.
pub(crate) struct History {
    pub(crate) changes: Vec<Entry>,
    pub(crate) head: usize,
}

impl History {
    pub(crate) fn new(commit: Commit) -> Self {
        History {
            changes: vec![Entry::new(commit)],
            head: 0,
        }
    }

    /// Updates the history and adds the latest commit
    pub(crate) fn save(&mut self, commit: Commit) {
        assert!(
            self.changes.is_empty().not(),
            "History is empty when saving"
        );
        assert!(self.head < self.changes.len(), "Head is out of bounds");

        let prev_head = self.head;
        self.head = self.changes.len();

        self.changes[prev_head].next.push(self.head);

        let mut entry = Entry::new(commit);
        entry.previous = Some(prev_head);
        self.changes.push(entry);
    }

    /// Undos a commit by updating the history to the new head and returning the commit
    pub(crate) fn undo(&mut self) -> Option<&Commit> {
        assert!(
            self.changes.is_empty().not(),
            "History is empty when undoing"
        );
        assert!(self.head < self.changes.len(), "Head is out of bounds");

        let prev_head = self.head;
        self.head = self.changes[prev_head].previous?;

        assert!(self.head < self.changes.len(), "New head is out of bounds");

        self.changes[self.head].hot_path = Some(prev_head);
        Some(&self.changes[prev_head].commit)
    }

    /// Greedily redos on the "hot path", the path of the last head locations
    pub(crate) fn hot_redo(&mut self) -> Option<&Commit> {
        assert!(
            self.changes.is_empty().not(),
            "History is empty when undoing"
        );
        assert!(self.head < self.changes.len(), "Head is out of bounds");

        let hot_head = self.changes[self.head].hot_path?;

        assert!(hot_head < self.changes.len(), "Hot head is out of bounds");
        assert!(
            self.changes[self.head].next.contains(&hot_head),
            "Hot path is not in path of change"
        );

        self.head = hot_head;

        Some(&self.changes[self.head].commit)
    }
}
