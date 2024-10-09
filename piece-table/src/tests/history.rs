use crate::history::{
    change::{Change, ChangeType},
    commit::Commit,
    entry::Entry,
    History,
};
use crate::piece_table::{
    piece::{Piece, PieceSource},
    PieceTable,
};
use crate::tests::piece_table::validate_table;

#[doc(hidden)]
fn validate_history(history: &History, changes: &Vec<Entry>, head: usize) {
    assert_eq!(history.changes, *changes);
    assert_eq!(history.head, head);
}

#[doc(hidden)]
fn create_entry(
    previous: Option<usize>,
    next: Vec<usize>,
    hot_path: Option<usize>,
    commit: Commit,
) -> Entry {
    Entry {
        previous,
        next,
        hot_path,
        commit,
    }
}

#[doc(hidden)]
mod save {
    use super::*;

    #[test]
    #[should_panic]
    fn empty_list() {
        let mut history = History {
            changes: Vec::new(),
            head: 0,
        };
        history.save(Commit::new());
    }

    #[test]
    #[should_panic]
    fn head_out_of_bounds() {
        let mut history = History::new(Commit::new());
        history.head = 5;
        history.save(Commit::new());
    }

    #[test]
    fn append() {
        let mut commit = Commit::new();
        commit.add_change(
            0,
            Piece::new(PieceSource::Addition, 0, "Hello, World!".len()),
            ChangeType::Insertion,
        );
        let changes = vec![
            create_entry(None, vec![1], None, Commit::new()),
            create_entry(Some(0), Vec::new(), None, commit),
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
            Piece::new(PieceSource::Addition, 0, "Hello, ".len()),
            ChangeType::Insertion,
        );
        let changes = vec![
            create_entry(None, vec![1], None, Commit::new()),
            create_entry(Some(0), Vec::new(), None, commit),
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
            Piece::new(PieceSource::Original, 0, "HelloWorld!".len()),
            ChangeType::Deletion,
        );
        commit.add_change(
            0,
            Piece::new(PieceSource::Original, 0, "Hello".len()),
            ChangeType::Insertion,
        );
        commit.add_change(
            1,
            Piece::new(PieceSource::Addition, 0, ", ".len()),
            ChangeType::Insertion,
        );
        commit.add_change(
            2,
            Piece::new(PieceSource::Original, "Hello".len(), "World!".len()),
            ChangeType::Insertion,
        );
        let changes = vec![
            create_entry(None, vec![1], None, Commit::new()),
            create_entry(Some(0), Vec::new(), None, commit),
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
            Piece::new(PieceSource::Addition, 0, "World!".len()),
            ChangeType::Insertion,
        );
        let mut commit2 = Commit::new();
        commit2.add_change(
            1,
            Piece::new(PieceSource::Addition, "World!".len(), ", ".len()),
            ChangeType::Insertion,
        );
        let changes = vec![
            create_entry(None, vec![1], None, Commit::new()),
            create_entry(Some(0), vec![2], None, commit1),
            create_entry(Some(1), Vec::new(), None, commit2),
        ];

        let mut table = PieceTable::from("Hello");
        table.append("World!");
        table.insert("Hello".len(), ", ");

        validate_history(&table.history, &changes, 2);
    }

    #[test]
    fn complex() {
        let mut commit1 = Commit::new();
        commit1.add_change(
            0,
            Piece::new(PieceSource::Original, 0, "Held!".len()),
            ChangeType::Deletion,
        );
        commit1.add_change(
            0,
            Piece::new(PieceSource::Original, 0, 2),
            ChangeType::Insertion,
        );
        commit1.add_change(
            1,
            Piece::new(PieceSource::Addition, 0, "llor".len()),
            ChangeType::Insertion,
        );
        commit1.add_change(
            2,
            Piece::new(PieceSource::Original, "He".len(), "ld!".len()),
            ChangeType::Insertion,
        );
        let mut commit2 = Commit::new();
        commit2.add_change(
            1,
            Piece::new(PieceSource::Addition, 0, "llor".len()),
            ChangeType::Deletion,
        );
        commit2.add_change(
            1,
            Piece::new(PieceSource::Addition, 0, "ll".len()),
            ChangeType::Insertion,
        );
        commit2.add_change(
            2,
            Piece::new(PieceSource::Addition, "llor".len(), "o, W".len()),
            ChangeType::Insertion,
        );
        commit2.add_change(
            3,
            Piece::new(PieceSource::Addition, "ll".len(), "or".len()),
            ChangeType::Insertion,
        );

        let changes = vec![
            create_entry(None, vec![1], None, Commit::new()),
            create_entry(Some(0), Vec::new(), None, commit1.clone()),
        ];

        let pieces = vec![
            Piece::new(PieceSource::Original, 0, 2),
            Piece::new(PieceSource::Addition, 0, 2),
            Piece::new(PieceSource::Addition, 4, 4),
            Piece::new(PieceSource::Addition, 2, 2),
            Piece::new(PieceSource::Original, 2, 3),
        ];

        let mut table = PieceTable::from("Held!");

        table.insert(2, "llor");
        validate_history(&table.history, &changes, 1);

        let changes = vec![
            create_entry(None, vec![1], None, Commit::new()),
            create_entry(Some(0), vec![2], None, commit1),
            create_entry(Some(1), Vec::new(), None, commit2),
        ];

        table.insert(4, "o, W");
        validate_history(&table.history, &changes, 2);
        validate_table(&table, "Held!", "lloro, W", &pieces, "Hello, World!");
    }
}

mod undo {
    use super::*;

    #[test]
    #[should_panic]
    fn empty_list() {
        let mut history = History {
            changes: Vec::new(),
            head: 0,
        };
        history.undo();
    }

    #[test]
    #[should_panic]
    fn head_out_of_bounds() {
        let mut history = History::new(Commit::new());
        history.head = 5;
        history.undo();
    }

    #[test]
    #[should_panic]
    fn new_head_out_of_bounds() {
        let mut history = History::new(Commit::new());
        history
            .changes
            .push(create_entry(Some(34324), Vec::new(), None, Commit::new()));
        history.head = 1;
        history.undo();
    }

    #[test]
    #[should_panic]
    fn originial_text_out_of_bounds() {
        let mut history = History::new(Commit::new());
        history.save(Commit {
            changes: vec![Change::new(
                0,
                Piece::new(PieceSource::Original, 10, 17),
                ChangeType::Insertion,
            )],
        });

        let mut table = PieceTable::from("Hello, World!");
        table.history = history;
        table.undo();
    }

    #[test]
    #[should_panic]
    fn addition_text_out_of_bounds() {
        let mut history = History::new(Commit::new());
        history.save(Commit {
            changes: vec![Change::new(
                0,
                Piece::new(PieceSource::Addition, 20, 22),
                ChangeType::Insertion,
            )],
        });

        let mut table = PieceTable::from("Hello, World!");
        table.history = history;
        table.undo();
    }

    #[test]
    #[should_panic]
    fn deletion_out_of_bounds() {
        let mut history = History::new(Commit::new());
        history.save(Commit {
            changes: vec![Change::new(
                34,
                Piece::new(PieceSource::Original, 0, 0),
                ChangeType::Deletion,
            )],
        });

        let mut table = PieceTable::from("");
        table.history = history;
        table.undo();
    }

    #[test]
    #[should_panic]
    fn insertion_out_of_bounds() {
        let mut history = History::new(Commit::new());
        history.save(Commit {
            changes: vec![Change::new(
                34,
                Piece::new(PieceSource::Addition, 0, 0),
                ChangeType::Insertion,
            )],
        });

        let mut table = PieceTable::from("");
        table.history = history;
        table.undo();
    }

    #[test]
    fn empty() {
        let history = History::new(Commit::new());
        let mut table = PieceTable::from("");
        table.undo();

        validate_history(&table.history, &history.changes, 0);
        validate_table(&table, "", String::new(), &Vec::new(), String::new());

        table.undo();

        validate_history(&table.history, &history.changes, 0);
        validate_table(&table, "", String::new(), &Vec::new(), String::new());
    }

    #[test]
    fn no_changes() {
        let history = History::new(Commit::new());
        let pieces = vec![Piece::new(PieceSource::Original, 0, "Hello, World!".len())];

        let mut table = PieceTable::from("Hello, World!");
        table.undo();

        validate_history(&table.history, &history.changes, 0);
        validate_table(
            &table,
            "Hello, World!",
            String::new(),
            &pieces,
            "Hello, World!",
        );

        table.undo();

        validate_history(&table.history, &history.changes, 0);
        validate_table(
            &table,
            "Hello, World!",
            String::new(),
            &pieces,
            "Hello, World!",
        );
    }

    #[test]
    fn middle_once() {
        let mut commit = Commit::new();
        commit.add_change(
            0,
            Piece::new(PieceSource::Original, 0, "HelloWorld!".len()),
            ChangeType::Deletion,
        );
        commit.add_change(
            0,
            Piece::new(PieceSource::Original, 0, "Hello".len()),
            ChangeType::Insertion,
        );
        commit.add_change(
            1,
            Piece::new(PieceSource::Addition, 0, ", ".len()),
            ChangeType::Insertion,
        );
        commit.add_change(
            2,
            Piece::new(PieceSource::Original, "Hello".len(), "World!".len()),
            ChangeType::Insertion,
        );
        let changes = vec![
            create_entry(None, vec![1], Some(1), Commit::new()),
            create_entry(Some(0), Vec::new(), None, commit),
        ];

        let pieces = vec![Piece::new(PieceSource::Original, 0, "HelloWorld!".len())];

        let mut table = PieceTable::from("HelloWorld!");
        table.insert("Hello".len(), ", ");
        table.undo();

        validate_history(&table.history, &changes, 0);
        validate_table(&table, "HelloWorld!", ", ", &pieces, "HelloWorld!");
    }

    #[test]
    fn between_pieces() {
        let mut commit1 = Commit::new();
        commit1.add_change(
            1,
            Piece::new(PieceSource::Addition, 0, "World!".len()),
            ChangeType::Insertion,
        );
        let mut commit2 = Commit::new();
        commit2.add_change(
            1,
            Piece::new(PieceSource::Addition, "World!".len(), ", ".len()),
            ChangeType::Insertion,
        );
        let changes = vec![
            create_entry(None, vec![1], None, Commit::new()),
            create_entry(Some(0), vec![2], Some(2), commit1.clone()),
            create_entry(Some(1), Vec::new(), None, commit2.clone()),
        ];

        let pieces = vec![
            Piece::new(PieceSource::Original, 0, "Hello".len()),
            Piece::new(PieceSource::Addition, 0, "World!".len()),
        ];

        let mut table = PieceTable::from("Hello");
        table.append("World!");
        table.insert("Hello".len(), ", ");
        table.undo();

        validate_history(&table.history, &changes, 1);
        validate_table(&table, "Hello", "World!, ", &pieces, "HelloWorld!");

        // Second undo
        let changes = vec![
            create_entry(None, vec![1], Some(1), Commit::new()),
            create_entry(Some(0), vec![2], Some(2), commit1),
            create_entry(Some(1), Vec::new(), None, commit2),
        ];

        let pieces = vec![Piece::new(PieceSource::Original, 0, "Hello".len())];

        table.undo();

        validate_history(&table.history, &changes, 0);
        validate_table(&table, "Hello", "World!, ", &pieces, "Hello");
    }
}

mod hot_redo {
    use super::*;

    #[test]
    #[should_panic]
    fn empty_list() {
        let mut history = History {
            changes: Vec::new(),
            head: 0,
        };
        history.hot_redo();
    }

    #[test]
    #[should_panic]
    fn head_out_of_bounds() {
        let mut history = History::new(Commit::new());
        history.head = 5;
        history.hot_redo();
    }

    #[test]
    #[should_panic]
    fn hot_head_out_of_bounds() {
        let mut history = History::new(Commit::new());
        history.changes[0].hot_path = Some(234);
        history.changes[0].next.push(234);
        history.hot_redo();
    }

    #[test]
    #[should_panic]
    fn hot_head_not_in_path() {
        let mut history = History::new(Commit::new());
        history.save(Commit::new());
        history.changes[0].hot_path = Some(1);
        history.changes[0].next.clear();
        history.head = 0;
        history.hot_redo();
    }

    #[test]
    #[should_panic]
    fn originial_text_out_of_bounds() {
        let mut history = History::new(Commit::new());
        history.changes[0].hot_path = Some(1);
        history.changes[0].next.push(1);
        history.save(Commit {
            changes: vec![Change::new(
                0,
                Piece::new(PieceSource::Original, 10, 17),
                ChangeType::Insertion,
            )],
        });
        history.head = 0;

        let mut table = PieceTable::from("Hello, World!");
        table.history = history;
        table.hot_redo();
    }

    #[test]
    #[should_panic]
    fn addition_text_out_of_bounds() {
        let mut history = History::new(Commit::new());
        history.changes[0].hot_path = Some(1);
        history.changes[0].next.push(1);
        history.save(Commit {
            changes: vec![Change::new(
                0,
                Piece::new(PieceSource::Addition, 20, 22),
                ChangeType::Insertion,
            )],
        });
        history.head = 0;

        let mut table = PieceTable::from("Hello, World!");
        table.history = history;
        table.hot_redo();
    }

    #[test]
    #[should_panic]
    fn deletion_out_of_bounds() {
        let mut history = History::new(Commit::new());
        history.changes[0].hot_path = Some(1);
        history.changes[0].next.push(1);
        history.save(Commit {
            changes: vec![Change::new(
                34,
                Piece::new(PieceSource::Original, 0, 0),
                ChangeType::Deletion,
            )],
        });
        history.head = 0;

        let mut table = PieceTable::from("");
        table.history = history;
        table.hot_redo();
    }

    #[test]
    #[should_panic]
    fn insertion_out_of_bounds() {
        let mut history = History::new(Commit::new());
        history.changes[0].hot_path = Some(1);
        history.changes[0].next.push(1);
        history.save(Commit {
            changes: vec![Change::new(
                34,
                Piece::new(PieceSource::Addition, 0, 0),
                ChangeType::Insertion,
            )],
        });
        history.head = 0;

        let mut table = PieceTable::from("");
        table.history = history;
        table.hot_redo();
    }

    #[test]
    fn empty() {
        let history = History::new(Commit::new());
        let mut table = PieceTable::from("");
        table.hot_redo();

        validate_history(&table.history, &history.changes, 0);
        validate_table(&table, "", String::new(), &Vec::new(), String::new());

        table.undo();
        table.hot_redo();

        validate_history(&table.history, &history.changes, 0);
        validate_table(&table, "", String::new(), &Vec::new(), String::new());
    }

    #[test]
    fn no_changes() {
        let history = History::new(Commit::new());
        let pieces = vec![Piece::new(PieceSource::Original, 0, "Hello, World!".len())];

        let mut table = PieceTable::from("Hello, World!");
        table.hot_redo();

        validate_history(&table.history, &history.changes, 0);
        validate_table(
            &table,
            "Hello, World!",
            String::new(),
            &pieces,
            "Hello, World!",
        );

        table.undo();
        table.hot_redo();

        validate_history(&table.history, &history.changes, 0);
        validate_table(
            &table,
            "Hello, World!",
            String::new(),
            &pieces,
            "Hello, World!",
        );
    }

    #[test]
    fn middle_once() {
        let mut commit = Commit::new();
        commit.add_change(
            0,
            Piece::new(PieceSource::Original, 0, "HelloWorld!".len()),
            ChangeType::Deletion,
        );
        commit.add_change(
            0,
            Piece::new(PieceSource::Original, 0, "Hello".len()),
            ChangeType::Insertion,
        );
        commit.add_change(
            1,
            Piece::new(PieceSource::Addition, 0, ", ".len()),
            ChangeType::Insertion,
        );
        commit.add_change(
            2,
            Piece::new(PieceSource::Original, "Hello".len(), "World!".len()),
            ChangeType::Insertion,
        );
        let changes = vec![
            create_entry(None, vec![1], Some(1), Commit::new()),
            create_entry(Some(0), Vec::new(), None, commit),
        ];

        let pieces = vec![
            Piece::new(PieceSource::Original, 0, "Hello".len()),
            Piece::new(PieceSource::Addition, 0, ", ".len()),
            Piece::new(PieceSource::Original, "Hello".len(), "World!".len()),
        ];

        let mut table = PieceTable::from("HelloWorld!");
        table.insert("Hello".len(), ", ");
        table.undo();
        table.hot_redo();

        validate_history(&table.history, &changes, 1);
        validate_table(&table, "HelloWorld!", ", ", &pieces, "Hello, World!");
    }

    #[test]
    fn between_pieces() {
        let mut commit1 = Commit::new();
        commit1.add_change(
            1,
            Piece::new(PieceSource::Addition, 0, "World!".len()),
            ChangeType::Insertion,
        );
        let mut commit2 = Commit::new();
        commit2.add_change(
            1,
            Piece::new(PieceSource::Addition, "World!".len(), ", ".len()),
            ChangeType::Insertion,
        );

        let mut table = PieceTable::from("Hello");
        table.append("World!");
        table.insert("Hello".len(), ", ");
        table.undo();

        // Second undo
        let changes = vec![
            create_entry(None, vec![1], Some(1), Commit::new()),
            create_entry(Some(0), vec![2], Some(2), commit1),
            create_entry(Some(1), Vec::new(), None, commit2),
        ];

        let pieces = vec![
            Piece::new(PieceSource::Original, 0, "Hello".len()),
            Piece::new(PieceSource::Addition, 0, "World!".len()),
        ];

        table.undo();
        table.hot_redo();

        validate_history(&table.history, &changes, 1);
        validate_table(&table, "Hello", "World!, ", &pieces, "HelloWorld!");

        table.hot_redo();

        let pieces = vec![
            Piece::new(PieceSource::Original, 0, "Hello".len()),
            Piece::new(PieceSource::Addition, "World!".len(), ", ".len()),
            Piece::new(PieceSource::Addition, 0, "World!".len()),
        ];

        validate_history(&table.history, &changes, 2);
        validate_table(&table, "Hello", "World!, ", &pieces, "Hello, World!");
    }
}
