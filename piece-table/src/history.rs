use crate::piece_table::Piece;

#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) enum ChangeType {
    Deletion,
    Insertion,
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct Change {
    /// Position of the Piece in the Piece Table (Vec)
    /// at the time of changing
    pos: usize,
    piece: Piece,
    typ: ChangeType,
}

impl Change {
    pub(crate) fn new(pos: usize, piece: Piece, typ: ChangeType) -> Self {
        Change { pos, piece, typ }
    }
}

#[cfg_attr(test, derive(PartialEq, Debug))]
/// Changes made by manipulating the Piece Table
pub(crate) struct Commit {
    changes: Vec<Change>,
}

impl Commit {
    pub(crate) fn new() -> Commit {
        Commit {
            changes: Vec::new(),
        }
    }

    pub(crate) fn add_change(&mut self, pos: usize, piece: Piece, typ: ChangeType) {
        self.changes.push(Change::new(pos, piece, typ));
    }
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct Entry {
    /// Index of the previous/following (if any) entries in the history
    pub(crate) previous: Option<usize>,
    pub(crate) next: Vec<usize>,

    // TODO: "hot path" of where head currently is
    pub(crate) commit: Commit,
}

impl Entry {
    pub(crate) fn new(commit: Commit) -> Self {
        Entry {
            previous: None,
            next: Vec::new(),
            commit,
        }
    }
}

pub(crate) struct History {
    pub(crate) changes: Vec<Entry>,
    pub(crate) head: usize,
}

/// Keeps the entire edit history of the Piece Table
impl History {
    pub(crate) fn new(commit: Commit) -> Self {
        History {
            changes: vec![Entry::new(commit)],
            head: 0,
        }
    }

    /// Updates the history and adds the latest commit
    pub(crate) fn save(&mut self, commit: Commit) {
        assert!(self.head < self.changes.len(), "Head is out of bounds");

        let prev_head = self.head;
        self.head = self.changes.len();

        self.changes[prev_head].next.push(self.head);

        let mut entry = Entry::new(commit);
        entry.previous = Some(prev_head);
        self.changes.push(entry);
    }
}
