use std::{
    fmt::Display,
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

use crate::piece_table::PieceTable;

pub trait PieceTableSlice<T> {
    /// Returns a slice of the text stored in the Piece Table
    fn slice(&self, index: T) -> String;
}

impl PieceTableSlice<Range<usize>> for PieceTable {
    /// Returns the text stored in the Piece Table from pos `start..end`
    ///
    /// # Panic
    /// Panics if
    /// - Range is out of bounds
    /// - `end <= start`
    fn slice(&self, index: Range<usize>) -> String {
        self._slice(index.start, index.end)
    }
}

impl PieceTableSlice<RangeFrom<usize>> for PieceTable {
    /// Returns the text stored in the Piece Table from pos `start..`
    ///
    /// # Panic
    /// Panics if range is out of bounds
    fn slice(&self, index: RangeFrom<usize>) -> String {
        self._slice(index.start, self.total_length)
    }
}

impl PieceTableSlice<RangeFull> for PieceTable {
    /// Returns the entire text stored in the Piece Table (prefer `PieceTable::to_string`)
    fn slice(&self, _index: RangeFull) -> String {
        self._slice(0, self.total_length)
    }
}

impl PieceTableSlice<RangeInclusive<usize>> for PieceTable {
    /// Returns the text stored in the Piece Table from pos `start..=end`
    ///
    /// # Panic
    /// Panics if
    /// - Range is out of bounds
    /// - `end + 1 <= start`
    /// - `end == usize::MAX`
    fn slice(&self, index: RangeInclusive<usize>) -> String {
        assert!(
            *index.end() != usize::MAX,
            "Upper incluse index must not be usize::MAX"
        );

        self.slice(*index.start()..*index.end() + 1)
    }
}

impl PieceTableSlice<RangeTo<usize>> for PieceTable {
    /// Returns the text stored in the Piece Table from pos `..end`
    ///
    ///
    /// # Panic
    /// Panics if range is out of bounds
    fn slice(&self, index: RangeTo<usize>) -> String {
        self.slice(0..index.end)
    }
}

impl PieceTableSlice<RangeToInclusive<usize>> for PieceTable {
    /// Returns the text stored in the Piece Table from pos `..=end`
    ///
    /// # Panic
    /// Panics if
    /// - Range is out of bounds
    /// - `end == usize::MAX`
    fn slice(&self, index: RangeToInclusive<usize>) -> String {
        assert!(
            index.end != usize::MAX,
            "Upper incluse index must not be usize::MAX"
        );

        self.slice(0..index.end + 1)
    }
}

impl Display for PieceTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.total_length == 0 {
            write!(f, "")
        } else {
            write!(f, "{}", self._slice(0, self.total_length))
        }
    }
}
