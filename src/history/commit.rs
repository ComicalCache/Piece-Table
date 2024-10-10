use crate::piece_table::piece::Piece;

use super::change::{Change, ChangeType};

#[cfg_attr(test, derive(PartialEq, Debug, Clone))]
/// Collection of Changes
pub(crate) struct Commit {
    pub(crate) changes: Vec<Change>,
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
