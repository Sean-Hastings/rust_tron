use std::fmt::{Display, Formatter};
use crate::position::Position;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Action {
    Up,
    Down,
    Left,
    Right
}

pub const ACTIONS: [Action; 4] = [
    Action::Up,
    Action::Right,
    Action::Down,
    Action::Left
];

impl Action {
    pub fn offset_position(&self, position: &Position) -> Result<Position, ()>{
        match self {
            Action::Up => {
                position.offset(-1, 0)
            }
            Action::Down => {
                position.offset(1, 0)
            }
            Action::Left => {
                position.offset(0, -1)
            }
            Action::Right => {
                position.offset(0, 1)
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub enum PlayerState {
    Alive {
        position: Position,
        boost: usize,
        armor: usize
    },
    Dead
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Player {
    id: usize,
    state: PlayerState
}

impl Player {
    pub fn new(id: usize, position: Position) -> Self {
        Player {
            id,
            state: PlayerState::Alive {
                position,
                boost: 0,
                armor: 0
            }
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_state(&self) -> PlayerState {
        self.state
    }

    pub fn set_state(&mut self, state: PlayerState) {
        self.state = state;
    }

    pub fn speed_boost(&mut self, duration: usize) -> Result<(), ()> {
        match self.get_state() {
            PlayerState::Alive { position, boost, armor } => {
                self.set_state(PlayerState::Alive {position, boost: boost + duration, armor});
                Ok(())
            }
            PlayerState::Dead => { Err(()) }
        }
    }

    pub fn armor_up(&mut self) -> Result<(), ()> {
        match self.get_state() {
            PlayerState::Alive { position, boost, armor } => {
                self.set_state(PlayerState::Alive {position, boost, armor: armor + 1});
                Ok(())
            }
            PlayerState::Dead => { Err(()) }
        }
    }

    pub fn take_damage(&mut self) -> Result<(), ()> {
        match self.get_state() {
            PlayerState::Alive { position, boost, armor } => {
                let new_state = if armor > 0 {
                    PlayerState::Alive {position, boost, armor: armor - 1}
                } else {
                    PlayerState::Dead
                };
                self.set_state(new_state);

                Ok(())
            }
            PlayerState::Dead => { Err(()) }
        }
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.state {
            PlayerState::Alive { position, boost, armor } => {
                write!(f, "Player({}: {}-{} @{})", self.id, boost, armor, position)
            }
            PlayerState::Dead => {
                write!(f, "Player({}: DEAD)", self.id)
            }
        }
    }
}