use std::{fmt::Display, ops::Not};

#[cfg_attr(test, derive(PartialEq, Debug))]
#[derive(Clone, Copy)]
enum PieceSource {
    Original,
    Addition,
}

#[cfg_attr(test, derive(PartialEq, Debug))]
struct Piece {
    source: PieceSource,
    offset: usize,
    length: usize,
}

impl Piece {
    pub fn new(source: PieceSource, offset: usize, length: usize) -> Self {
        Piece {
            source,
            offset,
            length,
        }
    }
}

pub struct PieceTable {
    original: String,
    addition: String,
    pieces: Vec<Piece>,

    total_length: usize,
}

impl PieceTable {
    pub fn from<S: AsRef<str>>(string: S) -> Self {
        let string = String::from(string.as_ref());
        let string_len = string.len();
        let piece = Piece::new(PieceSource::Original, 0, string_len);

        PieceTable {
            original: string,
            addition: String::new(),
            pieces: vec![piece],
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
}

impl Display for PieceTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PieceSource::*;

        for piece in self.pieces.iter() {
            let source = match piece.source {
                Original => &self.original,
                Addition => &self.addition,
            };
            let offset = piece.offset;
            let len = piece.length;

            write!(f, "{}", &source[offset..offset + len])?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validate_table<S1: AsRef<str>, S2: AsRef<str>, S3: AsRef<str>>(
        table: &PieceTable,
        original: S1,
        addition: S2,
        pieces: &Vec<Piece>,
        expected: S3,
    ) {
        let original = original.as_ref();
        let addition = addition.as_ref();
        let expected = expected.as_ref();

        assert_eq!(table.original, original);
        assert_eq!(table.addition, addition);
        assert_eq!(table.pieces, *pieces);
        assert_eq!(table.total_length, expected.len());
        assert_eq!(table.to_string(), expected);
    }

    mod from {
        use super::*;

        #[test]
        fn str() {
            let pieces = vec![Piece::new(PieceSource::Original, 0, "Hello, World!".len())];

            let table = PieceTable::from("Hello, World!");
            validate_table(
                &table,
                "Hello, World!",
                String::new(),
                &pieces,
                "Hello, World!",
            );
        }

        #[test]
        fn string() {
            let pieces = vec![Piece::new(PieceSource::Original, 0, "Hello, World!".len())];

            let test_string: String = String::from("Hello, World!");
            let table = PieceTable::from(test_string.clone());
            validate_table(&table, &test_string, String::new(), &pieces, &test_string);
        }

        #[test]
        fn empty() {
            let pieces = vec![Piece::new(PieceSource::Original, 0, 0)];

            let table = PieceTable::from("");
            validate_table(&table, "", "", &pieces, "");
        }
    }

    mod insert {
        use super::*;

        #[test]
        fn front() {
            let pieces = vec![
                Piece::new(PieceSource::Addition, 0, "Hello, ".len()),
                Piece::new(PieceSource::Original, 0, "World!".len()),
            ];

            let mut table = PieceTable::from("World!");
            table.insert(0, "Hello, ");
            validate_table(&table, "World!", "Hello, ", &pieces, "Hello, World!");
        }

        #[test]
        fn back() {
            let pieces = vec![
                Piece::new(PieceSource::Original, 0, "Hello, ".len()),
                Piece::new(PieceSource::Addition, 0, "World!".len()),
            ];

            let mut table = PieceTable::from("Hello, ");
            table.insert("Hello, ".len(), "World!");
            validate_table(&table, "Hello, ", "World!", &pieces, "Hello, World!");
        }

        #[test]
        fn between_pieces() {
            let pieces = vec![
                Piece::new(PieceSource::Addition, 0, "Hello".len()),
                Piece::new(PieceSource::Addition, "Hello".len(), ", ".len()),
                Piece::new(PieceSource::Original, 0, "World!".len()),
            ];

            let mut table = PieceTable::from("World!");
            table.insert(0, "Hello");
            table.insert("Hello".len(), ", ");
            eprintln!("{table}");
            validate_table(&table, "World!", "Hello, ", &pieces, "Hello, World!");
        }

        #[test]
        fn middle() {
            let pieces = vec![
                Piece::new(PieceSource::Original, 0, "Hello".len()),
                Piece::new(PieceSource::Addition, 0, ", ".len()),
                Piece::new(PieceSource::Original, 5, "World!".len()),
            ];

            let mut table = PieceTable::from("HelloWorld!");
            table.insert("Hello".len(), ", ");
            validate_table(&table, "HelloWorld!", ", ", &pieces, "Hello, World!");
        }

        #[test]
        fn middle_twice() {
            let pieces = vec![
                Piece::new(PieceSource::Original, 0, 2),
                Piece::new(PieceSource::Addition, 0, 2),
                Piece::new(PieceSource::Addition, 4, 4),
                Piece::new(PieceSource::Addition, 2, 2),
                Piece::new(PieceSource::Original, 2, 3),
            ];

            let mut table = PieceTable::from("Held!");
            table.insert(2, "llor");
            table.insert(4, "o, W");
            validate_table(&table, "Held!", "lloro, W", &pieces, "Hello, World!");
        }
    }

    mod remove {
        use super::*;

        fn get_simple_table() -> PieceTable {
            let mut table = PieceTable::from("Hello,");
            table.insert("Hello,".len(), " World!");
            table
        }

        fn get_complex_table() -> PieceTable {
            let mut table = PieceTable::from("Held!");
            table.insert(2, "llor");
            table.insert(4, "o, W");
            table
        }

        #[test]
        fn front() {
            let pieces = vec![Piece::new(
                PieceSource::Original,
                "Hello, ".len(),
                "World!".len(),
            )];

            let mut table = PieceTable::from("Hello, World!");
            table.remove(0, "Hello, ".len());
            validate_table(&table, "Hello, World!", String::new(), &pieces, "World!");
        }

        #[test]
        fn front_single() {
            let pieces = vec![Piece::new(
                PieceSource::Original,
                "H".len(),
                "ello, World!".len(),
            )];

            let mut table = PieceTable::from("Hello, World!");
            table.remove(0, "H".len());
            validate_table(
                &table,
                "Hello, World!",
                String::new(),
                &pieces,
                "ello, World!",
            );
        }

        #[test]
        fn back() {
            let pieces = vec![Piece::new(PieceSource::Original, 0, "Hello, ".len())];

            let mut table = PieceTable::from("Hello, World!");
            table.remove("Hello, ".len(), "World!".len());
            validate_table(&table, "Hello, World!", String::new(), &pieces, "Hello, ");
        }

        #[test]
        fn back_single() {
            let pieces = vec![Piece::new(PieceSource::Original, 0, "Hello, World".len())];

            let mut table = PieceTable::from("Hello, World!");
            table.remove("Hello, World".len(), "!".len());
            validate_table(
                &table,
                "Hello, World!",
                String::new(),
                &pieces,
                "Hello, World",
            );
        }

        #[test]
        fn slice() {
            let pieces = vec![
                Piece::new(PieceSource::Original, 0, "Hello".len()),
                Piece::new(PieceSource::Original, "Hello, ".len(), "World!".len()),
            ];

            let mut table = PieceTable::from("Hello, World!");
            table.remove("Hello".len(), ", ".len());
            validate_table(
                &table,
                "Hello, World!",
                String::new(),
                &pieces,
                "HelloWorld!",
            );
        }

        #[test]
        fn cross_piece() {
            let pieces = vec![
                Piece::new(PieceSource::Original, 0, "Hello".len()),
                Piece::new(PieceSource::Addition, 1, "World!".len()),
            ];

            let mut table = get_simple_table();
            table.remove("Hello".len(), 2);
            validate_table(&table, "Hello,", " World!", &pieces, "HelloWorld!");
        }

        #[test]
        fn cross_back() {
            let pieces = vec![Piece::new(PieceSource::Original, 0, "Hello".len())];

            let mut table = get_simple_table();
            table.remove("Hello".len(), ", World!".len());
            validate_table(&table, "Hello,", " World!", &pieces, "Hello");
        }

        #[test]
        fn front_cross() {
            let pieces = vec![Piece::new(PieceSource::Addition, 1, "World!".len())];

            let mut table = get_simple_table();
            table.remove(0, "Hello, ".len());
            validate_table(&table, "Hello,", " World!", &pieces, "World!");
        }

        #[test]
        fn full() {
            let pieces = vec![
                Piece::new(PieceSource::Original, 0, 2),
                Piece::new(PieceSource::Addition, 0, 2),
                Piece::new(PieceSource::Addition, 2, 2),
                Piece::new(PieceSource::Original, 2, 3),
            ];

            let mut table = get_complex_table();
            table.remove("Hell".len(), "o, W".len());
            validate_table(&table, "Held!", "lloro, W", &pieces, "Hellorld!");
        }

        #[test]
        fn full_cross() {
            let pieces = vec![
                Piece::new(PieceSource::Original, 0, 2),
                Piece::new(PieceSource::Addition, 0, 2),
                Piece::new(PieceSource::Addition, 3, 1),
                Piece::new(PieceSource::Original, 2, 3),
            ];

            let mut table = get_complex_table();
            table.remove("Hell".len(), "o, Wo".len());
            validate_table(&table, "Held!", "lloro, W", &pieces, "Hellrld!");
        }

        #[test]
        fn cross_full() {
            let pieces = vec![
                Piece::new(PieceSource::Original, 0, 2),
                Piece::new(PieceSource::Addition, 0, 1),
                Piece::new(PieceSource::Addition, 2, 2),
                Piece::new(PieceSource::Original, 2, 3),
            ];

            let mut table = get_complex_table();
            table.remove("Hel".len(), "lo, W".len());
            validate_table(&table, "Held!", "lloro, W", &pieces, "Helorld!");
        }

        #[test]
        fn cross_full_cross() {
            let pieces = vec![
                Piece::new(PieceSource::Original, 0, 2),
                Piece::new(PieceSource::Addition, 0, 1),
                Piece::new(PieceSource::Addition, 3, 1),
                Piece::new(PieceSource::Original, 2, 3),
            ];

            let mut table = get_complex_table();
            table.remove("Hel".len(), "lo, Wo".len());
            validate_table(&table, "Held!", "lloro, W", &pieces, "Helrld!");
        }
    }
}
