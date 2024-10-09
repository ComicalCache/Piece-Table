use crate::{
    piece_table::{
        piece::{Piece, PieceSource},
        PieceTable,
    },
    PieceTableSlice,
};

pub(crate) fn validate_table<S1: AsRef<str>, S2: AsRef<str>, S3: AsRef<str>>(
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
        let pieces = vec![];

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

    #[test]
    #[should_panic]
    fn out_of_bounds_pos() {
        let mut table = PieceTable::from("");
        table.insert(1, "Hello, World!");
    }

    #[test]
    #[should_panic]
    fn empty_string() {
        let mut table = PieceTable::from("Hello, World!");
        table.insert(0, "");
    }
}

mod append {
    use super::*;

    #[test]
    fn empty() {
        let pieces = vec![Piece::new(PieceSource::Addition, 0, "Hello, World!".len())];

        let mut table = PieceTable::from("");
        table.append("Hello, World!");
        validate_table(
            &table,
            String::new(),
            "Hello, World!",
            &pieces,
            "Hello, World!",
        );
    }

    #[test]
    fn multiple() {
        let pieces = vec![
            Piece::new(PieceSource::Original, 0, "Hello".len()),
            Piece::new(PieceSource::Addition, 0, ", ".len()),
            Piece::new(PieceSource::Addition, ", ".len(), "World".len()),
            Piece::new(PieceSource::Addition, ", World".len(), "!".len()),
        ];

        let mut table = PieceTable::from("Hello");
        table.append(", ");
        table.append("World");
        table.append("!");
        validate_table(&table, "Hello", ", World!", &pieces, "Hello, World!");
    }

    #[test]
    #[should_panic]
    fn empty_string() {
        let mut table = PieceTable::from("");
        table.append("");
    }
}

mod remove {
    use super::*;

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

    #[test]
    #[should_panic]
    fn empty() {
        let mut table = PieceTable::from("");
        table.remove(0, 5);
    }

    #[test]
    #[should_panic]
    fn out_of_bounds() {
        let mut table = PieceTable::from("Hello, World!");
        table.remove(15, 5);
    }

    #[test]
    #[should_panic]
    fn zero_characters() {
        let mut table = PieceTable::from("Hello, World!");
        table.remove(5, 0);
    }
}

mod slice {
    use super::*;

    #[test]
    fn range() {
        let table = get_complex_table();

        for i in 0.."Hello, World!".len() {
            for j in (i + 1).."Hello, World!".len() {
                assert_eq!(table.slice(i..j), "Hello, World"[i..j]);
            }
        }
    }

    #[test]
    fn range_from() {
        let table = get_complex_table();

        for i in 0.."Hello, World!".len() {
            assert_eq!(table.slice(i..), "Hello, World!"[i..]);
        }
    }

    #[test]
    fn range_full() {
        let table = get_complex_table();
        assert_eq!(table.slice(..), "Hello, World!"[..]);
    }

    #[test]
    fn range_inclusive() {
        let table = get_complex_table();

        for i in 0.."Hello, World!".len() {
            for j in (i + 1).."Hello, World!".len() - 1 {
                assert_eq!(table.slice(i..=j), "Hello, World"[i..=j]);
            }
        }
    }

    #[test]
    fn range_to() {
        let table = get_complex_table();

        for i in 1.."Hello, World!".len() {
            assert_eq!(table.slice(..i), "Hello, World!"[..i]);
        }
    }

    #[test]
    fn range_to_inclusive() {
        let table = get_complex_table();

        for i in 1.."Hello, World!".len() - 1 {
            assert_eq!(table.slice(..=i), "Hello, World!"[..=i]);
        }
    }

    #[test]
    #[should_panic]
    fn upper_smaller_lower() {
        let table = PieceTable::from("Hello, World!");
        table.slice(3..1);
    }

    #[test]
    #[should_panic]
    fn upper_equal_lower() {
        let table = PieceTable::from("Hello, World!");
        table.slice(3..3);
    }

    #[test]
    #[should_panic]
    fn out_of_lower_bounds() {
        let table = PieceTable::from("Hello, World!");
        table.slice(20..22);
    }

    #[test]
    #[should_panic]
    fn out_of_upper_bounds() {
        let table = PieceTable::from("Hello, World!");
        table.slice(0..22);
    }
}
