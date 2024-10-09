use crate::history::*;
use crate::piece_table::*;

#[doc(hidden)]
fn validate_history(history: &History, changes: &Vec<Entry>, head: usize) {
    assert_eq!(history.changes, *changes);
    assert_eq!(history.head, head);
}

#[doc(hidden)]
fn create_entry(previous: Option<usize>, next: Vec<usize>, commit: Commit) -> Entry {
    Entry {
        previous,
        next,
        commit,
    }
}

#[doc(hidden)]
mod save {
    use super::*;

    #[test]
    fn append() {
        let mut commit = Commit::new();
        commit.add_change(
            0,
            Piece::new(Source::Addition, 0, "Hello, World!".len()),
            ChangeType::Insertion,
        );
        let changes = vec![
            create_entry(None, vec![1], Commit::new()),
            create_entry(Some(0), Vec::new(), commit),
        ];

        let mut table = PieceTable::from("");
        table.append("Hello, World!");

        validate_history(&table.history, &changes, 1);
    }

    #[test]
    fn front() {
        let mut commit = Commit::new();
        commit.add_change(
            0,
            Piece::new(Source::Addition, 0, "Hello, ".len()),
            ChangeType::Insertion,
        );
        let changes = vec![
            create_entry(None, vec![1], Commit::new()),
            create_entry(Some(0), Vec::new(), commit),
        ];

        let mut table = PieceTable::from("World!");
        table.insert(0, "Hello, ");

        validate_history(&table.history, &changes, 1);
    }

    #[test]
    fn middle() {
        let mut commit = Commit::new();
        commit.add_change(
            0,
            Piece::new(Source::Original, 0, "HelloWorld!".len()),
            ChangeType::Deletion,
        );
        commit.add_change(
            0,
            Piece::new(Source::Original, 0, "Hello".len()),
            ChangeType::Insertion,
        );
        commit.add_change(
            1,
            Piece::new(Source::Addition, 0, ", ".len()),
            ChangeType::Insertion,
        );
        commit.add_change(
            2,
            Piece::new(Source::Original, "Hello".len(), "World!".len()),
            ChangeType::Insertion,
        );
        let changes = vec![
            create_entry(None, vec![1], Commit::new()),
            create_entry(Some(0), Vec::new(), commit),
        ];

        let mut table = PieceTable::from("HelloWorld!");
        table.insert("Hello".len(), ", ");

        validate_history(&table.history, &changes, 1);
    }

    #[test]
    fn between_pieces() {
        let mut commit1 = Commit::new();
        commit1.add_change(
            1,
            Piece::new(Source::Addition, 0, "World!".len()),
            ChangeType::Insertion,
        );
        let mut commit2 = Commit::new();
        commit2.add_change(
            1,
            Piece::new(Source::Addition, "World!".len(), ", ".len()),
            ChangeType::Insertion,
        );
        let changes = vec![
            create_entry(None, vec![1], Commit::new()),
            create_entry(Some(0), vec![2], commit1),
            create_entry(Some(1), Vec::new(), commit2),
        ];

        let mut table = PieceTable::from("Hello");
        table.append("World!");
        table.insert("Hello".len(), ", ");

        validate_history(&table.history, &changes, 2);
    }
}
