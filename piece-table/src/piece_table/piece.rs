#[cfg_attr(test, derive(PartialEq, Debug))]
#[derive(Clone, Copy)]
pub(crate) enum PieceSource {
    Original,
    Addition,
}

#[cfg_attr(test, derive(PartialEq, Debug))]
#[derive(Clone, Copy)]
/// Piece of the Piece Table
pub(crate) struct Piece {
    pub(crate) source: PieceSource,
    /// Offset in the data source
    pub(crate) offset: usize,
    /// Length of pointed at content
    pub(crate) length: usize,
}

impl Piece {
    pub(crate) fn new(source: PieceSource, offset: usize, length: usize) -> Self {
        Piece {
            source,
            offset,
            length,
        }
    }
}
