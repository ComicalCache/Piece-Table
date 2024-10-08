use crate::piece_table::Commit;

struct Entry {
    previous: Option<usize>,
    next: Vec<usize>,

    // TODO: "hot path" of where head currently is
    commit: Commit,
}

impl Entry {
    fn new(commit: Commit) -> Self {
        Entry {
            previous: None,
            next: Vec::new(),
            commit,
        }
    }
}

pub(crate) struct History {
    changes: Vec<Entry>,
    head: usize,
}

impl History {
    pub(crate) fn new(commit: Commit) -> Self {
        History {
            changes: vec![Entry::new(commit)],
            head: 0,
        }
    }

    pub(crate) fn save(&mut self, commit: Commit) {
        assert!(self.head < self.changes.len(), "Head is out of bounds");
        let prev_head = self.head;
        self.head = self.changes.len();

        self.changes[prev_head].next.push(self.head);

        let mut entry = Entry::new(commit);
        entry.previous = Some(prev_head);
        self.changes.push(entry);
    }
}
