use std::{
    fmt::Display,
    ops::{Not, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

use crate::History;

#[cfg_attr(test, derive(PartialEq, Debug))]
#[derive(Clone, Copy)]
pub(crate) enum Source {
    Original,
    Addition,
}

#[cfg_attr(test, derive(PartialEq, Debug))]
#[derive(Clone, Copy)]
pub(crate) struct Piece {
    source: Source,
    /// Offset in the data source
    offset: usize,
    /// Length of pointed at content
    length: usize,
}

impl Piece {
    pub(crate) fn new(source: Source, offset: usize, length: usize) -> Self {
        Piece {
            source,
            offset,
            length,
        }
    }
}

pub(crate) enum ChangeType {
    Deletion,
    Insertion,
}

pub(crate) struct Change {
    /// Position of the Piece in the Piece Table (Vec)
    pos: usize,
    piece: Piece,
    typ: ChangeType,
}

impl Change {
    pub(crate) fn new(pos: usize, piece: Piece, typ: ChangeType) -> Self {
        Change { pos, piece, typ }
    }
}

/// Changes made by manipulating the Piece Table
///
/// ### Important
/// The order of operations is important, to keep the indices consistent
/// all deletions happen *before* any additions
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

/// Simple Piece Table
pub struct PieceTable {
    /// Read only input data
    pub(crate) original: String,
    /// Data added while editing
    pub(crate) addition: String,
    /// List of pieces that point to data contained in text
    pub(crate) pieces: Vec<Piece>,

    /// Length of the text contained in the Piece Table
    pub(crate) total_length: usize,

    /// Edit history
    pub(crate) history: History,
}

impl PieceTable {
    /// Creates a Piece Table from a string
    ///
    /// ### Example
    /// ```
    /// pub use piece_table::PieceTable;
    ///
    /// let table = PieceTable::from("Hello, World!");
    /// assert_eq!(table.to_string(), "Hello, World!");
    /// ```
    pub fn from<S: AsRef<str>>(string: S) -> Self {
        let string = String::from(string.as_ref());
        let string_len = string.len();
        let pieces = if string.is_empty().not() {
            vec![Piece::new(Source::Original, 0, string_len)]
        } else {
            vec![]
        };

        PieceTable {
            original: string,
            addition: String::new(),
            pieces,
            total_length: string_len,
            history: History::new(Commit::new()),
        }
    }

    /// Inserts `string` at `pos`.
    ///
    /// If `pos == PieceTable::len`, it appends the str (see `PieceTable::append`),
    ///
    /// ### Panic
    /// Panics when either:
    /// - `pos > PieceTable::len`
    /// - `string` is empty
    ///
    /// ### Example
    /// ```
    /// use piece_table::PieceTable;
    ///
    /// let mut table = PieceTable::from("HelloWorld!");
    /// table.insert(5, ", ");
    /// assert_eq!(table.to_string(), "Hello, World!");
    /// ```
    pub fn insert<S: AsRef<str>>(&mut self, mut pos: usize, string: S) {
        assert!(
            pos <= self.total_length,
            "Insert position must be within or at the end of the text"
        );

        let string = string.as_ref();
        assert!(string.is_empty().not(), "Inserted string must not be empty");

        let special_case = pos == 0 || pos == self.total_length;

        let piece = Piece::new(Source::Addition, self.addition.len(), string.len());
        let mut commit = Commit::new();

        self.total_length += string.len();
        self.addition.push_str(string);

        if special_case {
            // FIXME: insertion at the very front is slow, can this be done faster?
            let idx = if pos == 0 { 0 } else { self.pieces.len() };

            self.pieces.insert(idx, piece);
            commit.add_change(idx, piece, ChangeType::Insertion);

            self.history.save(commit);
            return;
        }

        let mut len = 0;
        let mut idx = 0;
        for (i, piece) in self.pieces.iter().enumerate() {
            if len + piece.length <= pos {
                len += piece.length;
                continue;
            }

            idx = i;
            pos = pos - len;
            break;
        }

        if pos == 0 {
            self.pieces.insert(idx, piece);
            commit.add_change(idx, piece, ChangeType::Insertion);
        } else {
            // Split existing piece into two and insert new piece between
            let trailing = Piece::new(
                self.pieces[idx].source,
                self.pieces[idx].offset + pos,
                self.pieces[idx].length - pos,
            );

            let old_piece = self.pieces[idx];
            self.pieces[idx].length = pos;
            commit.add_change(pos, old_piece, ChangeType::Deletion);
            commit.add_change(idx, self.pieces[idx], ChangeType::Insertion);

            self.pieces.insert(idx + 1, piece);
            commit.add_change(idx + 1, piece, ChangeType::Insertion);

            self.pieces.insert(idx + 2, trailing);
            commit.add_change(idx + 2, trailing, ChangeType::Insertion);
        }

        self.history.save(commit);
    }

    /// Appends a string at the end.
    ///
    /// ### Panic
    /// Panics if the string is empty
    ///
    /// ### Example
    /// ```
    /// use piece_table::PieceTable;
    ///
    /// let mut table = PieceTable::from("Hello, ");
    /// table.append("World!");
    /// assert_eq!(table.to_string(), "Hello, World!");
    /// ```
    pub fn append<S: AsRef<str>>(&mut self, string: S) {
        self.insert(self.total_length, string);
    }

    /// Removes a string from `pos` of length `n`
    ///
    /// ### Panic
    /// Panics if:
    /// - `pos + n > PieceTable::len`
    /// - `n == 0`
    ///
    /// ### Example
    /// ```
    /// use piece_table::PieceTable;
    ///
    /// let mut table = PieceTable::from("Hello, World!");
    /// table.remove(5, 8);
    /// assert_eq!(table.to_string(), "Hello");
    /// ```
    pub fn remove(&mut self, pos: usize, n: usize) {
        let end = pos + n;
        assert!(
            end <= self.total_length,
            "Deleted string must be within the text"
        );
        assert!(n != 0, "Must remove at least 1 character");

        type Index = usize;
        type Length = usize;
        type Offset = usize;
        enum Remove {
            End(Index, Length),
            Full(Index),
            Start(Index, Length),
            Slice(Index, Offset),
        }

        let mut remove = Vec::new();
        let mut len = 0;
        for (idx, piece) in self.pieces.iter().enumerate() {
            if len >= end {
                break;
            }
            let prev_len = len;
            len += piece.length;

            let remove_start = pos <= prev_len;
            let remove_end = end >= len && len > pos;
            let remove_slice = prev_len < pos && end < len;

            match (remove_start, remove_end) {
                (false, true) => remove.push(Remove::End(idx, len - pos)),
                (true, true) => remove.push(Remove::Full(idx)),
                (true, false) => remove.push(Remove::Start(idx, end - prev_len)),
                _ if remove_slice => remove.push(Remove::Slice(idx, pos - prev_len)),
                _ => continue,
            }
        }

        let mut commit = Commit::new();
        self.total_length -= n;
        for remove_piece in remove.iter().rev() {
            match remove_piece {
                Remove::End(idx, len) => {
                    let old_piece = self.pieces[*idx];
                    self.pieces[*idx].length -= *len;
                    commit.add_change(*idx, old_piece, ChangeType::Deletion);
                    commit.add_change(*idx, self.pieces[*idx], ChangeType::Insertion);
                }
                Remove::Full(idx) => {
                    commit.add_change(*idx, self.pieces.remove(*idx), ChangeType::Deletion);
                }
                Remove::Start(idx, len) => {
                    let old_piece = self.pieces[*idx];
                    self.pieces[*idx].length -= *len;
                    self.pieces[*idx].offset += *len;
                    commit.add_change(*idx, old_piece, ChangeType::Deletion);
                    commit.add_change(*idx, self.pieces[*idx], ChangeType::Insertion);
                }
                Remove::Slice(idx, offset) => {
                    let old_piece = self.pieces[*idx];
                    let leading = Piece::new(old_piece.source, old_piece.offset, *offset);
                    let trailing_len = old_piece.length - *offset - n;
                    let trailing = Piece::new(old_piece.source, *offset + n, trailing_len);

                    self.pieces[*idx] = leading;
                    commit.add_change(*idx, old_piece, ChangeType::Deletion);
                    commit.add_change(*idx, leading, ChangeType::Insertion);

                    self.pieces.insert(*idx + 1, trailing);
                    commit.add_change(*idx + 1, trailing, ChangeType::Insertion);
                }
            }
        }

        self.history.save(commit);
    }

    /// Returns the length of the text stored in the Piece Table
    pub fn len(&self) -> usize {
        self.total_length
    }

    /// Returns text stored in the Piece Table (`upper` is exclusive)
    fn _slice(&self, lower: usize, upper: usize) -> String {
        assert!(
            lower < upper,
            "Lower slice bound must be smaller than upper"
        );
        assert!(
            upper <= self.total_length,
            "Slice bounds must be within the bounds of the text"
        );

        let mut out = String::new();

        let mut len = 0;
        for piece in self.pieces.iter() {
            if len >= upper {
                break;
            }

            let prev_len = len;
            len += piece.length;

            let source = match piece.source {
                Source::Original => &self.original,
                Source::Addition => &self.addition,
            };

            let capture_start = lower <= prev_len;
            let capture_end = upper >= len && len > lower;
            let caputre_slice = prev_len < lower && upper < len;

            match (capture_start, capture_end) {
                (false, true) => {
                    let start = piece.offset + (lower - prev_len);
                    let end = piece.offset + piece.length;
                    out.push_str(&source[start..end])
                }
                (true, true) => out.push_str(&source[piece.offset..piece.offset + piece.length]),
                (true, false) => {
                    let start = piece.offset;
                    let end = piece.offset + (upper - prev_len);
                    out.push_str(&source[start..end]);
                }
                _ if caputre_slice => {
                    let start = piece.offset + (lower - prev_len);
                    let end = piece.offset + (upper - prev_len);
                    out.push_str(&source[start..end]);
                }
                _ => continue,
            }
        }

        out
    }
}

pub trait Slice<T> {
    fn slice(&self, index: T) -> String;
}

impl Slice<Range<usize>> for PieceTable {
    /// Returns the text stored in the Piece Table from pos `start..end`
    ///
    /// ### Panic
    /// Panics if
    /// - Range is out of bounds
    /// - `end <= start`
    fn slice(&self, index: Range<usize>) -> String {
        self._slice(index.start, index.end)
    }
}

impl Slice<RangeFrom<usize>> for PieceTable {
    /// Returns the text stored in the Piece Table from pos `start..`
    ///
    /// ### Panic
    /// Panics if range is out of bounds
    fn slice(&self, index: RangeFrom<usize>) -> String {
        self._slice(index.start, self.total_length)
    }
}

impl Slice<RangeFull> for PieceTable {
    /// Returns the entire text stored in the Piece Table (prefer `PieceTable::to_string`)
    fn slice(&self, _index: RangeFull) -> String {
        self._slice(0, self.total_length)
    }
}

impl Slice<RangeInclusive<usize>> for PieceTable {
    /// Returns the text stored in the Piece Table from pos `start..=end`
    ///
    /// ### Panic
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

impl Slice<RangeTo<usize>> for PieceTable {
    /// Returns the text stored in the Piece Table from pos `..end`
    ///
    ///
    /// ### Panic
    /// Panics if range is out of bounds
    fn slice(&self, index: RangeTo<usize>) -> String {
        self.slice(0..index.end)
    }
}

impl Slice<RangeToInclusive<usize>> for PieceTable {
    /// Returns the text stored in the Piece Table from pos `..=end`
    ///
    /// ### Panic
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
