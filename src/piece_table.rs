pub mod display_trait;
pub mod slice_trait;

pub(crate) mod piece;

use std::ops::Not;

use piece::{Piece, PieceSource};

use crate::{
    history::{change::ChangeType, commit::Commit},
    History,
};

/// Simple Piece Table with a history to enable undo/redo operations
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
    /// # Example
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
            vec![Piece::new(PieceSource::Original, 0, string_len)]
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
    /// # Panic
    /// Panics when either:
    /// - `pos > PieceTable::len`
    /// - `string` is empty
    ///
    /// # Example
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

        let piece = Piece::new(PieceSource::Addition, self.addition.len(), string.len());
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

        use ChangeType::*;
        if pos == 0 {
            self.pieces.insert(idx, piece);
            commit.add_change(idx, piece, Insertion);
        } else {
            // Split existing piece into two and insert new piece between
            let trailing = Piece::new(
                self.pieces[idx].source,
                self.pieces[idx].offset + pos,
                self.pieces[idx].length - pos,
            );

            let old_piece = self.pieces[idx];
            self.pieces[idx].length = pos;
            commit.add_change(idx, old_piece, Deletion);
            commit.add_change(idx, self.pieces[idx], Insertion);

            self.pieces.insert(idx + 1, piece);
            commit.add_change(idx + 1, piece, Insertion);

            self.pieces.insert(idx + 2, trailing);
            commit.add_change(idx + 2, trailing, Insertion);
        }

        self.history.save(commit);
    }

    /// Appends a string at the end
    ///
    /// # Panic
    /// Panics if the string is empty
    ///
    /// # Example
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
    /// # Panic
    /// Panics if:
    /// - `pos + n > PieceTable::len`
    /// - `n == 0`
    ///
    /// # Example
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
            "Removed string must be within the text"
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
        use Remove::*;

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
                (false, true) => remove.push(End(idx, len - pos)),
                (true, true) => remove.push(Full(idx)),
                (true, false) => remove.push(Start(idx, end - prev_len)),
                _ if remove_slice => remove.push(Slice(idx, pos - prev_len)),
                _ => continue,
            }
        }

        use ChangeType::*;
        let mut commit = Commit::new();
        self.total_length -= n;
        for remove_piece in remove.iter().rev() {
            match remove_piece {
                End(idx, len) => {
                    let old_piece = self.pieces[*idx];
                    self.pieces[*idx].length -= *len;
                    commit.add_change(*idx, old_piece, Deletion);
                    commit.add_change(*idx, self.pieces[*idx], Insertion);
                }
                Full(idx) => {
                    commit.add_change(*idx, self.pieces.remove(*idx), Deletion);
                }
                Start(idx, len) => {
                    let old_piece = self.pieces[*idx];
                    self.pieces[*idx].length -= *len;
                    self.pieces[*idx].offset += *len;
                    commit.add_change(*idx, old_piece, Deletion);
                    commit.add_change(*idx, self.pieces[*idx], Insertion);
                }
                Slice(idx, offset) => {
                    let old_piece = self.pieces[*idx];
                    let leading = Piece::new(old_piece.source, old_piece.offset, *offset);
                    let trailing_len = old_piece.length - *offset - n;
                    let trailing = Piece::new(old_piece.source, *offset + n, trailing_len);

                    self.pieces[*idx] = leading;
                    commit.add_change(*idx, old_piece, Deletion);
                    commit.add_change(*idx, leading, Insertion);

                    self.pieces.insert(*idx + 1, trailing);
                    commit.add_change(*idx + 1, trailing, Insertion);
                }
            }
        }

        self.history.save(commit);
    }

    /// Returns the length of the text stored in the Piece Table
    pub fn len(&self) -> usize {
        self.total_length
    }

    /// Reverts the Piece Table to the state *before* the last changes
    ///
    /// # Example
    /// ```
    /// use piece_table::PieceTable;
    ///
    /// let mut table = PieceTable::from("Hello, World!");
    /// table.append("World!");
    /// table.append("World!");
    /// assert_eq!(table.to_string(), "Hello, World!World!World!");
    /// table.undo();
    /// assert_eq!(table.to_string(), "Hello, World!World!");
    /// table.undo();
    /// assert_eq!(table.to_string(), "Hello, World!");
    /// ```
    pub fn undo(&mut self) {
        let commit = match self.history.undo() {
            Some(commit) => commit,
            None => return,
        };

        // FIXME: lots of duplication with hot_redo
        for change in commit.changes.iter().rev() {
            match change.piece.source {
                PieceSource::Original => assert!(
                    change.piece.offset + change.piece.length <= self.original.len(),
                    "Piece change is out of original text bounds"
                ),
                PieceSource::Addition => assert!(
                    change.piece.offset + change.piece.length <= self.addition.len(),
                    "Piece change is out of addition text bounds"
                ),
            }

            match change.typ {
                ChangeType::Deletion => {
                    assert!(
                        change.pos <= self.pieces.len(),
                        "Undo delete position is out of bounds"
                    );

                    self.pieces.insert(change.pos, change.piece);
                    self.total_length += change.piece.length;
                }
                ChangeType::Insertion => {
                    assert!(
                        change.pos < self.pieces.len(),
                        "Undo insert position is out of bounds"
                    );

                    let removed_piece = self.pieces.remove(change.pos);
                    self.total_length -= removed_piece.length;
                }
            }
        }
    }

    /// Restores the Piece Table to the "hot" state *after* the last undo.
    ///
    /// Hot state means the state the head was last at (e.g. at a fork in the history
    ///     it can quickly be redone to the last head position without having to select it).
    ///
    /// # Example
    /// ```
    /// use piece_table::PieceTable;
    ///
    /// let mut table = PieceTable::from("Hello, World!");
    /// table.append("World!");
    /// table.append("World!");
    /// table.undo();
    /// table.undo();
    /// assert_eq!(table.to_string(), "Hello, World!");
    /// table.hot_redo();
    /// assert_eq!(table.to_string(), "Hello, World!World!");
    /// table.hot_redo();
    /// assert_eq!(table.to_string(), "Hello, World!World!World!");
    /// ```
    pub fn hot_redo(&mut self) {
        let commit = match self.history.hot_redo() {
            Some(commit) => commit,
            None => return,
        };

        // FIXME: lots of duplication with undo
        for change in &commit.changes {
            match change.piece.source {
                PieceSource::Original => assert!(
                    change.piece.offset + change.piece.length <= self.original.len(),
                    "Piece change is out of original text bounds"
                ),
                PieceSource::Addition => assert!(
                    change.piece.offset + change.piece.length <= self.addition.len(),
                    "Piece change is out of addition text bounds"
                ),
            }

            match change.typ {
                ChangeType::Deletion => {
                    assert!(
                        change.pos < self.pieces.len(),
                        "Redo delete position is out of bounds"
                    );

                    let removed_piece = self.pieces.remove(change.pos);
                    self.total_length -= removed_piece.length;
                }
                ChangeType::Insertion => {
                    assert!(
                        change.pos <= self.pieces.len(),
                        "Redo insert position is out of bounds"
                    );

                    self.pieces.insert(change.pos, change.piece);
                    self.total_length += change.piece.length;
                }
            }
        }
    }

    /// Returns text stored in the Piece Table (`upper` is exclusive)
    pub(crate) fn _slice(&self, lower: usize, upper: usize) -> String {
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
        for piece in &self.pieces {
            if len >= upper {
                break;
            }

            let prev_len = len;
            len += piece.length;

            let source = match piece.source {
                PieceSource::Original => &self.original,
                PieceSource::Addition => &self.addition,
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
