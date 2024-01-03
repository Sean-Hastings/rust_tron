use std::fmt::{Display, Formatter};
use crate::game::GameState;
use crate::player::{Action, Player, PlayerState};
use crate::position::Position;
use crate::power_up::PowerUp;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum CellState {
    Empty,
    PowerUp {
        power_up: PowerUp
    },
    Wall,
    Owned {
        player_id: usize
    },
    Occupied {
        player_id: usize
    },
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct BoardCell {
    position: Position,
    state: CellState,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Board {
    width: usize,
    height: usize,
    cells: Vec<Vec<BoardCell>>,
    players: Vec<Player>
}

impl BoardCell {
    fn get_position(&self) -> Position {
        self.position
    }

    pub fn get_state(&self) -> CellState {
        self.state
    }

    fn set_state(&mut self, state: CellState) {
        self.state = state;
    }
}

impl Board {
    pub fn new_default(height: usize, width: usize) -> Result<Self, ()> {
        if height == 0 || width == 0 {
            return Err(())
        }
        let mut cells = Vec::new();
        for i_row in 0..height {
            let mut row = Vec::new();
            for i_col in 0..height {
                let position = Position {
                    row: i_row,
                    column: i_col
                };
                let state = if i_row == 0 && i_col == 0 {
                    CellState::Occupied { player_id: 0 }
                } else if i_row == (height - 1) && i_col == (width - 1) {
                    CellState::Occupied { player_id: 1 }
                } else {
                    CellState::Empty
                };
                let cell = BoardCell {
                    position,
                    state
                };
                row.push(cell);
            }
            cells.push(row);
        }

        let players = vec![
            Player::new(0, Position::new(0, 0)),
            Player::new(1, Position::new(height - 1, width - 1))
        ];

        Ok(Board {
            width,
            height,
            cells,
            players
        })
    }

    pub fn height(&self) -> usize{
        self.height
    }

    pub fn width(&self) -> usize{
        self.width
    }

    pub fn players(&self) -> &Vec<Player> {
        &self.players
    }

    pub fn get_cell(&self, position: Position) -> Result<&BoardCell, ()> {
        match self.cells.get(position.row) {
            None => { Err(()) }
            Some(row) => {
                match row.get(position.column) {
                    None => { Err(()) }
                    Some(cell) => { Ok(cell) }
                }
            }
        }
    }

    fn update_cell_state(&mut self, position: Position, state: CellState) -> Result<(), ()> {
        match self.get_cell(position) {
            Ok(_) => { self.cells[position.row][position.column].set_state(state) }
            Err(_) => { return Err(()) }
        };
        Ok(())
    }

    pub fn apply_action(&self, player_id: usize, action: Action) -> Result<Board, ()> {
        let player_position = match self.players[player_id].get_state() {
            PlayerState::Alive { position, boost: _, armor: _ } => {
                position
            }
            PlayerState::Dead => { return Err(()) }
        };
        let position_result = action.offset_position(&player_position);

        let new_position = match position_result {
            Ok(position) => {
                position
            }
            Err(err) => { return Err(err) }
        };

        self.move_player(player_id, new_position)
    }

    pub fn move_player(&self, player_id: usize, destination: Position) -> Result<Self, ()> {
        let mut new_board = self.clone();
        let mut player = new_board.players[player_id];
        match player.get_state() {
            PlayerState::Alive { position, boost, armor } => {
                match new_board.update_cell_state(position, CellState::Owned { player_id }) {
                    Ok(_) => {}
                    Err(_) => { return Err(()) }
                }
                player.set_state(PlayerState::Alive { position: destination, boost, armor });
            }
            PlayerState::Dead => { return Err(()) }
        }

        let to_cell = match new_board.get_cell(destination) {
            Ok(cell) => { cell }
            Err(_) => { return Err(()) }
        };
        match to_cell.get_state() {
            CellState::Empty => {}
            CellState::PowerUp { power_up } => {
                match power_up {
                    PowerUp::DoubleSpeed { duration } => {
                        player.speed_boost(duration).unwrap()
                    }
                    PowerUp::Armor => {
                        player.armor_up().unwrap()
                    }
                    PowerUp::Bomb => {
                        new_board.explode_around(destination)
                    }
                };
            }
            CellState::Wall => {
                player.take_damage().unwrap();
            }
            CellState::Owned { .. } => {
                player.take_damage().unwrap();
            }
            CellState::Occupied { .. } => {
                player.set_state(PlayerState::Dead)
            }
        }

        match player.get_state() {
            PlayerState::Alive { .. } => {
                match new_board.update_cell_state(destination, CellState::Occupied { player_id }) {
                    Ok(_) => {}
                    Err(_) => { return Err(()) }
                }
            }
            PlayerState::Dead => {}
        }

        new_board.players[player_id] = player;

        //println!("{}", new_board);

        Ok(new_board)
    }

    fn explode_around(&mut self, position: Position) {
        let positions = [
            position.offset(-1, -1),
            position.offset(-1, 0),
            position.offset(-1, 1),
            position.offset(0, -1),
            //position.offset(0, 0),
            position.offset(0, 1),
            position.offset(1, -1),
            position.offset(1, 0),
            position.offset(1, 1),
        ];

        for position in positions {
            match position {
                Ok(position) => {
                    let _ = self.explode_cell(position);
                }
                Err(_) => {}
            }
        }
    }

    fn explode_cell(&mut self, position: Position) -> Result<(), ()> {
        let cell = match self.get_cell(position) {
            Ok(cell) => { cell }
            Err(_) => { return Err(()) }
        };
        match cell.get_state() {
            CellState::PowerUp { .. } => {
                self.update_cell_state(position, CellState::Empty)
            }
            CellState::Wall => {
                self.update_cell_state(position, CellState::Empty)
            }
            CellState::Owned { .. } => {
                self.update_cell_state(position, CellState::Empty)
            }
            _ => { Ok(()) }
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut print_string = String::new();
        for row in &self.cells {
            for cell in row {
                let cell_symbol = match &cell.state {
                    CellState::Empty => { '*' }
                    CellState::PowerUp { power_up } => {
                        match &power_up {
                            PowerUp::DoubleSpeed { .. } => { 'S' }
                            PowerUp::Armor => { 'A' }
                            PowerUp::Bomb => { 'B' }
                        }
                    }
                    CellState::Wall => { '#' }
                    CellState::Owned { .. } => { '#' }
                    CellState::Occupied { player_id } => { player_id.to_string().pop().unwrap() }
                };
                print_string.push(cell_symbol);
            }
            print_string.push('\n');
        }
        write!(f, "{}", print_string)
    }
}