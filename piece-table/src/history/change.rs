use crate::piece_table::piece::Piece;

#[cfg_attr(test, derive(PartialEq, Debug, Clone, Copy))]
pub(crate) enum ChangeType {
    Deletion,
    Insertion,
}

#[cfg_attr(test, derive(PartialEq, Debug, Clone, Copy))]
/// The change to the Piece Table
pub(crate) struct Change {
    /// Position of the Piece in the Piece Table (Vec)
    /// at the time of changing
    pub(crate) pos: usize,
    pub(crate) piece: Piece,
    pub(crate) typ: ChangeType,
}

impl Change {
    pub(crate) fn new(pos: usize, piece: Piece, typ: ChangeType) -> Self {
        Change { pos, piece, typ }
    }
}
