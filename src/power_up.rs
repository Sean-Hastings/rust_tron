#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum PowerUp {
    DoubleSpeed {
        duration: usize
    },
    Armor,
    Bomb
}