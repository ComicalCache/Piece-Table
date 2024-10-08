use std::{
    fmt::Display,
    ops::{Not, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

#[cfg_attr(test, derive(PartialEq, Debug))]
#[derive(Clone, Copy)]
pub(crate) enum PieceSource {
    Original,
    Addition,
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct Piece {
    source: PieceSource,
    offset: usize,
    length: usize,
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

pub struct PieceTable {
    pub(crate) original: String,
    pub(crate) addition: String,
    pub(crate) pieces: Vec<Piece>,

    pub(crate) total_length: usize,
}

impl PieceTable {
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
        }
    }

    pub fn insert<S: AsRef<str>>(&mut self, mut pos: usize, string: S) {
        assert!(
            pos <= self.total_length,
            "Insert position must be within or at the end of the text"
        );

        let string = string.as_ref();
        assert!(string.is_empty().not(), "Inserted string must not be empty");

        let special_case = pos == 0 || pos == self.total_length;

        let piece = Piece::new(PieceSource::Addition, self.addition.len(), string.len());
        self.total_length += string.len();
        self.addition.push_str(string);

        if special_case {
            // FIXME: insertion at the very front is slow, can this be done faster?
            let idx = if pos == 0 { 0 } else { self.pieces.len() };

            self.pieces.insert(idx, piece);
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
        } else {
            // Split existing piece into two and insert new piece between
            let trailing = Piece::new(
                self.pieces[idx].source,
                self.pieces[idx].offset + pos,
                self.pieces[idx].length - pos,
            );

            self.pieces[idx].length = pos;
            self.pieces.insert(idx + 1, piece);
            self.pieces.insert(idx + 2, trailing);
        }
    }

    pub fn append<S: AsRef<str>>(&mut self, string: S) {
        self.insert(self.total_length, string);
    }

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

        self.total_length -= n;
        for remove_piece in remove.iter().rev() {
            match remove_piece {
                Remove::End(idx, len) => self.pieces[*idx].length -= *len,
                Remove::Full(idx) => _ = self.pieces.remove(*idx),
                Remove::Start(idx, len) => {
                    self.pieces[*idx].length -= *len;
                    self.pieces[*idx].offset += *len;
                }
                Remove::Slice(idx, offset) => {
                    let piece = &self.pieces[*idx];
                    let leading = Piece::new(piece.source, piece.offset, *offset);
                    let trailing_len = piece.length - *offset - n;
                    let trailing = Piece::new(piece.source, *offset + n, trailing_len);

                    self.pieces[*idx] = leading;
                    self.pieces.insert(*idx + 1, trailing);
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.total_length
    }

    fn _slice(&self, lower: usize, upper: usize) -> String {
        assert!(
            lower < upper,
            "Lower slice bound must be smaller than upper"
        );
        assert!(
            upper <= self.total_length,
            "Upper slice bound must be within the bounds of the text"
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
                PieceSource::Original => &self.original,
                PieceSource::Addition => &self.addition,
            };

            let capture_start = lower <= prev_len;
            let capture_end = upper >= len && len > lower;
            let caputre_slice = prev_len < lower && upper < len;

            eprintln!("{}, {}", piece.offset, piece.length);
            eprintln!("{lower}, {upper}, {prev_len}, {len}");
            eprintln!("{capture_start}, {capture_end}, {caputre_slice}\n---");

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
            eprintln!("{out}\n---");
        }

        out
    }
}

pub trait Slice<T> {
    fn slice(&self, index: T) -> String;
}

impl Slice<Range<usize>> for PieceTable {
    fn slice(&self, index: Range<usize>) -> String {
        self._slice(index.start, index.end)
    }
}

impl Slice<RangeFrom<usize>> for PieceTable {
    fn slice(&self, index: RangeFrom<usize>) -> String {
        self._slice(index.start, self.total_length)
    }
}

impl Slice<RangeFull> for PieceTable {
    fn slice(&self, _index: RangeFull) -> String {
        self._slice(0, self.total_length)
    }
}

impl Slice<RangeInclusive<usize>> for PieceTable {
    fn slice(&self, index: RangeInclusive<usize>) -> String {
        assert!(
            *index.end() != usize::MAX,
            "Upper incluse index must not be usize::MAX"
        );

        self.slice(*index.start()..*index.end() + 1)
    }
}

impl Slice<RangeTo<usize>> for PieceTable {
    fn slice(&self, index: RangeTo<usize>) -> String {
        self.slice(0..index.end)
    }
}

impl Slice<RangeToInclusive<usize>> for PieceTable {
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