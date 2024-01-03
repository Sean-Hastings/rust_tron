use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Position {
    pub row: usize,
    pub column: usize
}

impl Position {
    pub fn new(row: usize, column: usize) -> Self {
        Self { row, column }
    }

    pub fn offset(&self, row_offset: isize, col_offset: isize) -> Result<Position, ()>{
        let new_row = (self.row as isize) + row_offset;
        let new_col = (self.column as isize) + col_offset;
        if new_row < 0 || new_col < 0 {
            Err(())
        } else {
            Ok(Position { row: new_row as usize, column: new_col as usize })
        }
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.row, self.column)
    }
}