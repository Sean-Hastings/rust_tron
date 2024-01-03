use std::cmp::max;
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::fmt::{Display, Formatter, Pointer, write};
use std::time::Instant;
use crate::board::{Board, BoardCell, CellState};
use crate::player::{Action, ACTIONS, PlayerState};
use crate::position::Position;
use crate::power_up::PowerUp;

pub trait PlayerController {
    fn get_action(&mut self, board: &Board, player_id: usize) -> Action;
}

pub struct ClockwiseController {}

impl PlayerController for ClockwiseController {
    fn get_action(&mut self, board: &Board, player_id: usize) -> Action {
        for action in ACTIONS {
            match board.apply_action(player_id, action) {
                Ok(new_board) => {
                    match new_board.players()[player_id].get_state() {
                        PlayerState::Alive { .. } => { return action }
                        PlayerState::Dead => {}
                    }
                }
                Err(_) => {}
            }
        }

        Action::Up
    }
}

const MIN_SCORE: i32 = i32::MIN;
const MAX_SCORE: i32 = i32::MAX;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
struct SearchNode {
    scores: Vec<i32>,
    actions: Vec<Action>,
    state: Board,
}

impl Display for SearchNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SearchNode (scores: {:?}, actions: {:?})", self.scores, self.actions)
    }
}

pub struct BFSController {
    pub(crate) turn_time_milliseconds: u128,
    pub(crate) score_tracker: HashMap<String, i32>,
}

impl PlayerController for BFSController {
    fn get_action(&mut self, board: &Board, player_id: usize) -> Action {
        println!("searching {}", player_id);
        let mut queue = BinaryHeap::new();

        let initial_node = SearchNode {
            scores: Vec::from([score(board, player_id, &mut self.score_tracker)]),
            actions: Vec::new(),
            state: board.clone(),
        };
        queue.push(initial_node);

        let mut best_so_far = Action::Up;
        let mut score_so_far = MIN_SCORE;

        let start_time = Instant::now();
        'full: while queue.len() > 0 && start_time.elapsed().as_millis() < self.turn_time_milliseconds {
            let SearchNode { scores: _scores, actions: _actions, state } = queue.pop().unwrap();
            for us_action in ACTIONS {
                let mut actions = _actions.clone();
                actions.push(us_action);

                let them_state = match state.apply_action(player_id, us_action) {
                    Ok(board) => {
                        match board.players()[player_id].get_state() {
                            PlayerState::Alive { .. } => { board }
                            PlayerState::Dead => { continue }
                        }
                    }
                    Err(_) => {
                        //println!("invalid us action: {:?}\n{}", us_action, state);
                        continue;
                    }
                };
                let mut worst_node = SearchNode { scores: Vec::new(), actions: Vec::new(), state: them_state.clone() };
                let mut worst_score = MAX_SCORE;
                let them_id = 1-player_id;
                for them_action in ACTIONS {
                    let us_state = match them_state.apply_action(them_id, them_action) {
                        Ok(board) => {
                            board
                        }
                        Err(_) => {
                            them_state.clone()
                        }
                    };
                    let new_score = score(&us_state, player_id, &mut self.score_tracker);
                    let mut scores = _scores.clone();
                    scores.push(new_score);
                    let new_node = SearchNode { scores, actions: actions.clone(), state: us_state };

                    if new_score < worst_score {
                        worst_score = new_score;
                        worst_node = new_node;
                    }
                }
                if worst_score > score_so_far {
                    println!("{}", worst_node);
                    best_so_far = actions[0];
                    score_so_far = worst_score;

                    if worst_score == MAX_SCORE {
                        break 'full;
                    }
                }

                queue.push(worst_node);
            }
        }

        println!("best score: {}", score_so_far);
        best_so_far
    }
}

fn score(board: &Board, player_id: usize, tracker: &mut HashMap<String, i32>) -> i32 {
    let key = &format!("{}{}", player_id, board);
    if !tracker.contains_key(key) {
        let _score = match board.players()[player_id].get_state() {
            PlayerState::Alive { .. } => {
                match board.players()[1-player_id].get_state() {
                    PlayerState::Alive { .. } => {
                        calc_zone_relative(board, player_id)
                    }
                    PlayerState::Dead => { MAX_SCORE }
                }
            }
            PlayerState::Dead => { MIN_SCORE }
        };
        tracker.insert(key.clone(), _score);
    }
    let _score = *tracker.get(key).unwrap();
    //println!("{}\n{} score: {}", board, player_id, _score);
    _score
}

fn calc_zone_relative(board: &Board, player_id: usize) -> i32 {
    let mut seen: Vec<Vec<(Position, u16)>> = Vec::new();
    let mut frontiers: Vec<VecDeque<(Position, u16)>> = Vec::new();
    let mut scores = Vec::new();
    for player in board.players() {
        let mut frontier = VecDeque::new();
        match player.get_state() {
            PlayerState::Alive { position, .. } => {
                frontier.push_back((position, 0));
            }
            PlayerState::Dead => {}
        }
        frontiers.push(frontier);
        seen.push(Vec::new());
        scores.push(0);
    }
    while !frontiers_empty(&frontiers) {
        for id in 0..frontiers.len() {
            let mut frontier = &mut frontiers[id];
            for _ in 0..frontier.len() {
                let (position, n_steps) = match frontier.pop_front() {
                    None => { continue }
                    Some(pos) => { pos }
                };
                //println!("position: {}", position);

                let score = match board.get_cell(position) {
                    Ok(cell) => {
                        match cell.get_state() {
                            CellState::Empty => { 1 }
                            CellState::PowerUp { power_up } => {
                                match power_up {
                                    PowerUp::DoubleSpeed { duration } => { 3 }
                                    PowerUp::Armor => { 4 }
                                    PowerUp::Bomb => { 5 }
                                }
                            }
                            CellState::Wall => { continue }
                            CellState::Owned { .. } => { continue }
                            CellState::Occupied { .. } => { 0 }
                        }
                    }
                    Err(_) => { continue }
                };

                match is_seen(position, &seen) {
                    Some((other_id, other_steps)) => {
                        if other_steps < n_steps || other_id == id {
                            continue
                        } else if other_steps > n_steps {
                            scores[other_id] -= score;
                        }
                    }
                    None => {
                        seen[id].push((position, n_steps));
                    }
                }

                scores[id] += score;


                'outer: for (row_off, col_off) in [(-1, 0), (0, 1), (1, 0), (0, -1)] {
                    let new_position = match position.offset(row_off, col_off) {
                        Ok(pos) => {
                            pos
                        }
                        Err(_) => { continue }
                    };
                    //println!("{}", new_position);

                    match is_seen(new_position, &seen) {
                        Some(_) => { continue }
                        None => {
                            for (pos, _) in frontier.iter() {
                                //println!("{} == {} : {}", *pos, new_position, *pos == new_position);
                                if *pos == new_position {
                                    continue 'outer;
                                }
                            }
                        }
                    }

                    //println!("{}:  {}, {}", id, new_position, n_steps + 1);
                    frontier.push_back((new_position, n_steps + 1));
                }
            }
        }
    }
    let player_score = scores[player_id];
    scores[player_id] = 0;
    let mut max_score = i32::MIN;
    for score in scores {
        max_score = max_score.max(score);
    }
    //println!("{}: player / max scores: {} / {}", player_id, player_score, max_score);
    player_score - max_score
}

/*




score_relative(board, player_id)





*/


fn is_seen(position: Position, seen: &Vec<Vec<(Position, u16)>>) -> Option<(usize, u16)> {
    for (id, seen_vec) in seen.iter().enumerate() {
        for (_position, n_steps) in seen_vec {
            if position == *_position {
                return Some((id, *n_steps))
            }
        }
    }
    None
}

fn frontiers_empty(frontiers: &Vec<VecDeque<(Position, u16)>>) -> bool {
    for frontier in frontiers {
        if frontier.len() > 0 {
            return false;
        }
    }
    true
}

/*
queue = [<(initial_state, 0, [])>]
while time() < end_time:
    queue.sort(largest score first)
    foreach state, score, actions in queue.pop()
        foreach us_action
            to_queue = <> (score = inf)
            them_state = state.apply(us_action)
            foreach them_action
                us_state = them_state.apply(them_action)
                new_score = score(us_state)
                to_queue.add( (us_state, new_score, actions + [us_action, them_action]) )
                to_queue.score = min(to_queue.score, new_score)
            queue.add(to_queue)




          5
         / \
        4   6
    /   |   |   \
  5     8   7     8
/ | \ / | / | \ / |
3 3 5 7 3 1 8 4 7 8
|\|/|\|\|\|\|\|\|\|
3444567632118748799

[<(5,)>] []
[] [(5,6), (5,4)]
[<(5,6,8,), (5,6,7)>] [(5,4)]
[<(5,6,8,), (5,6,7)>, <(5,4,8), (5,4,5)>] []
[<(5,4,8), (5,4,5)>] [(5,6,8,8), (5,6,7,8), (5,6,8,7), (5,6,7,4), (5,6,7,1)]
[<(5,6,8,8,9)>, <(5,4,8), (5,4,5)>] [(5,6,7,8), (5,6,8,7), (5,6,7,4), (5,6,7,1)]
[<(5,6,8,8,9)>, <(5,6,7,8,8), (5,6,7,8,7)>, <(5,4,8), (5,4,5)>] [(5,6,8,7), (5,6,7,4), (5,6,7,1)]
[<(5,6,8,8,9)>, <(5,6,7,8,8), (5,6,7,8,7)>, <(5,6,8,7,9), (5,6,8,7,7)>, <(5,4,8), (5,4,5)>] [(5,6,7,4), (5,6,7,1)]
...
[ *much stuff* ] []


 */