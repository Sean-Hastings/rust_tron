use crate::board::{Board};
use crate::player::{Action, PlayerState};
use crate::player_controller::PlayerController;

#[derive(Debug, Clone)]
pub enum GameState {
    Active {
        turn: usize,
        alive_ids: Vec<usize>
    },
    Over {
        winner_id: usize
    }
}

pub struct Game {
    board: Board,
    player_controllers: Vec<Box<dyn PlayerController>>,
    state: GameState,
}

impl Game {
    pub fn new_default(player_controllers: [Box<dyn PlayerController>; 2]) -> Self {
        let board = Board::new_default(10, 10).unwrap();

        Game::new(board, Vec::from(player_controllers)).unwrap()
    }

    pub fn new(board: Board, player_controllers: Vec<Box<dyn PlayerController>>) -> Result<Self, ()> {
        if board.players().len() != player_controllers.len() {
            return Err(())
        }
        let mut alive_ids = Vec::new();
        for (player_id, player) in board.players().iter().enumerate() {
            match player.get_state() {
                PlayerState::Dead => {}
                _ => {
                    alive_ids.push(player_id);
                }
            }
        }
        Ok(Game {
            board,
            player_controllers,
            state: GameState::Active {
                turn: 0,
                alive_ids
            }
        })
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn run_turn(&mut self) -> Result<(usize, Vec<Action>), ()> {
        match &self.state {
            GameState::Active { turn, alive_ids } => {
                let turn = *turn;
                let active_id = turn % self.player_controllers.len();
                if !alive_ids.contains(&active_id) {
                    self.state = GameState::Active {
                        turn: turn + 1,
                        alive_ids: alive_ids.clone()
                    };
                    return Ok((active_id, Vec::new()))
                }
                let n_actions: u8 = match self.board.players()[active_id].get_state() {
                    PlayerState::Alive { position: _, boost, armor: _ } => {
                        if boost > 0 {
                            2
                        } else {
                            1
                        }
                    }
                    PlayerState::Dead => { return Err(()) }
                };
                let mut actions = Vec::new();
                for _ in 0..n_actions {
                    let mut controller = self.player_controllers.get_mut(active_id).unwrap();
                    let action = controller.get_action(&self.board, active_id);
                    match self.apply_action(action, active_id) {
                        Ok(alive_ids) => {
                            actions.push(action);
                            if alive_ids.len() == 1 {
                                self.state = GameState::Over {
                                    winner_id: alive_ids[0]
                                };
                                break;
                            } else {
                                self.state = GameState::Active {
                                    turn: turn + 1,
                                    alive_ids
                                }
                            }
                        }
                        Err(err) => { return Err(err) }
                    }
                }
                Ok((active_id, actions))
            }
            GameState::Over { .. } => { Err(()) }
        }
    }

    fn apply_action(&mut self, action: Action, active_id: usize) -> Result<Vec<usize>,()> {
        match &self.state {
            GameState::Active { turn: _, alive_ids } => {
                if !alive_ids.contains(&active_id) {
                    return Err(())
                }
                match self.board.apply_action(active_id, action) {
                    Ok(board) => {
                        self.board = board;
                        Ok(match self.board.players()[active_id].get_state() {
                            PlayerState::Alive { .. } => { alive_ids.clone() }
                            PlayerState::Dead => {
                                let mut new_ids = alive_ids.clone();
                                new_ids.remove(active_id);
                                new_ids
                            }
                        })
                    }
                    Err(err) => { Err(err) }
                }
            }
            GameState::Over { .. } => { Err(()) }
        }
    }
}