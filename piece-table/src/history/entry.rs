use super::commit::Commit;

#[cfg_attr(test, derive(PartialEq, Debug))]
/// History Entry with position information inside the History
pub(crate) struct Entry {
    /// Index of the previous/following (if any) entries in the history
    pub(crate) previous: Option<usize>,
    pub(crate) next: Vec<usize>,

    /// Path of last head location
    /// Enables hot redo that always redos to the last head location, even at a fork
    pub(crate) hot_path: Option<usize>,

    pub(crate) commit: Commit,
}

impl Entry {
    pub(crate) fn new(commit: Commit) -> Self {
        Entry {
            previous: None,
            next: Vec::new(),
            hot_path: None,
            commit,
        }
    }
}
