use std::fmt::Display;

use super::PieceTable;

impl Display for PieceTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.total_length == 0 {
            write!(f, "")
        } else {
            write!(f, "{}", self._slice(0, self.total_length))
        }
    }
}
