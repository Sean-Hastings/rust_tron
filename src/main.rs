mod board;
mod player;
mod position;
mod power_up;
mod player_controller;
mod game;

use std::collections::HashMap;
use crate::player_controller::{PlayerController, ClockwiseController, BFSController};
use crate::game::{Game, GameState};


fn main() {
    let mut controller_clockwise = ClockwiseController{};
    let mut controller_smart = BFSController{ turn_time_milliseconds: 1000, score_tracker: HashMap::new() };
    let mut controller_smart_2 = BFSController{ turn_time_milliseconds: 1000, score_tracker: HashMap::new() };
    let mut game = Game::new_default([Box::new(controller_smart), Box::new(controller_smart_2)]);
    loop {
        match game.state() {
            GameState::Active { turn, alive_ids: _ } => {
                println!("{}", turn);
            }
            GameState::Over { winner_id: _ } => {
                break;
            }
        };
        match game.run_turn() {
            Ok((player_id, actions)) => {
                println!("{:?}: {:?}", player_id, actions);
                println!("{}", game.board());
            }
            Err(_) => {
                break
            }
        }
    }
    // run turns until err or state is over
}