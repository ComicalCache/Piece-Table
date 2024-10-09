mod piece_table;
pub use crate::piece_table::slice_trait::PieceTableSlice;
pub use piece_table::PieceTable;

mod history;
pub(crate) use history::History;

#[cfg(test)]
mod tests;
