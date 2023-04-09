//! Alpha - Beta algorithm.
use std::fmt;

use super::Strategy;
use crate::configuration::{Configuration, Movement};
use crate::shmem::AtomicMove;
use rayon::prelude::*;
use std::collections::HashMap;
// use std::time::{Duration, Instant};

/// Anytime alpha beta algorithm.
/// Any time algorithms will compute until a deadline is hit and the process is killed.
/// They are therefore run in another process and communicate through shared memory.
/// This function is intended to be called from blobwar_iterative_deepening.
pub fn alpha_beta_anytime(state: &Configuration) {
    let mut movement = AtomicMove::connect().expect("failed connecting to shmem");
    let mut memo: HashMap<String, (i8, Movement)> = HashMap::new();

    // let start_time = Instant::now();
    for depth in 1..100{
        let chosen_movement = AlphaBeta(depth).compute_next_move(state, Some(&mut memo));
        movement.store(chosen_movement);
        // let end_time = Instant::now();
        // println!("Time elapsed: {:?}", end_time.duration_since(start_time));
    }
}

/// Alpha - Beta algorithm with given maximum number of recursions.
pub struct AlphaBeta(pub u8);

impl fmt::Display for AlphaBeta {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Alpha - Beta (max level: {})", self.0)
    }
}

fn alpha_beta(depth: u8, state: &Configuration, mut alpha: i8, beta: i8) -> (i8, Option<Movement>) {
    if depth == 0 || !state.can_move() {
        return (state.value(), None);
    }
    let mut best_score = i8::MIN;
    let mut best_move: Option<Movement> = None;
    for movement in state.movements() {
        let next_conf: Configuration = state.play(&movement);
        let (score, _) = alpha_beta(depth - 1, &next_conf, -beta, -alpha);
        if score > best_score {
            best_score = score;
            best_move = Some(movement);
            if best_score > alpha {
                alpha = best_score;
            }
            if beta < alpha {
                break;
            }
        }
    }
    assert_ne!(best_score, i8::MIN);
    (-best_score, best_move)
}

// Alpha - Beta algorithm in functional programming
fn alpha_beta_func(
    depth: u8,
    state: &Configuration,
    mut alpha: i8,
    beta: i8,
) -> (i8, Option<Movement>) {
    if depth == 0 || !state.can_move() {
        return (state.value(), None);
    }
    let result = state
        .movements()
        .try_fold((i8::MIN, None), |acc, movement| {
            let (mut bscore, mut bmove) = acc;
            let (score, _) = alpha_beta_func(depth - 1, &(state.play(&movement)), -beta, -alpha);
            if score > bscore {
                bscore = score;
                bmove = Some(movement);
                if bscore > alpha {
                    alpha = bscore;
                }
                if beta < alpha {
                    return Err((bscore, bmove));
                }
            }
            Ok((bscore, bmove))
        });
    let (bscore, bmove) = match result {
        Ok((bscore, bmove)) => (bscore, bmove),
        Err((bscore, bmove)) => (bscore, bmove),
    };
    (-bscore, bmove)
}


// Alpha - Beta algorithm in functional programming
fn alpha_beta_func_memo(
    depth: u8,
    state: &Configuration,
    mut alpha: i8,
    beta: i8,
    memo: &mut HashMap<String, (i8, Movement)>
) -> (i8, Option<Movement>) {
    // check the value of the prev iteration
    if let Some((bscore, bmove)) = memo.get(&state.serialize()) {
        return (*bscore, Some(*bmove));
    }
    if depth == 0 || !state.can_move() {
        return (state.value(), None);
    }
    let result = state
        .movements()
        .try_fold((i8::MIN, None), |acc, movement| {
            let (mut bscore, mut bmove) = acc;
            let (score, _) = alpha_beta_func_memo(depth - 1, &(state.play(&movement)), -beta, -alpha, memo);
            if score > bscore {
                bscore = score;
                bmove = Some(movement);
                if bscore > alpha {
                    alpha = bscore;
                }
                if beta < alpha {
                    return Err((bscore, bmove));
                }
            }
            Ok((bscore, bmove))
        });
    let (bscore, bmove) = match result {
        Ok((bscore, bmove)) => (bscore, bmove),
        Err((bscore, bmove)) => (bscore, bmove),
    };
    memo.insert(state.serialize(), (-bscore, bmove.unwrap()));
    (-bscore, bmove)
}

// Pseudo parallel alpha beta
fn palpha_beta(
    depth: u8,
    state: &Configuration,
    mut alpha: i8,
    beta: i8,
) -> (i8, Option<Movement>) {
    if depth == 0 || !state.can_move() {
        return (state.value(), None);
    }
    let (bmove, (bscore, _)) = state
        .movements()
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|movement| (movement, alpha_beta_func(depth - 1, &(state.play(&movement)), -beta, -alpha)))
        .max_by_key(|&(_, (value, _))| value)
        .unwrap();
    (-bscore, Some(bmove))
}

fn alpha_beta_sorted(
    depth: u8,
    state: &Configuration,
    mut alpha: i8,
    beta: i8,
) -> (i8, Option<Movement>) {
    if depth == 0 || !state.can_move() {
        return (state.value(), None);
    }
    let mut best_score = i8::MIN;
    let mut best_move: Option<Movement> = None;
    let mut sorted_moves: Vec<Movement> = state.movements().collect::<Vec<Movement>>();
    sorted_moves.sort_by_key(|m| -state.play(m).value());
    for movement in sorted_moves.into_iter() {
        let next_conf: Configuration = state.play(&movement);
        let (score, _) = alpha_beta_sorted(depth - 1, &next_conf, -beta, -alpha);
        if score > best_score {
            best_score = score;
            best_move = Some(movement);
            if best_score > alpha {
                alpha = best_score;
            }
            if beta < alpha {
                break;
            }
        }
    }
    assert_ne!(best_score, i8::MIN);
    (-best_score, best_move)
}

fn negascout(depth: u8, state: &Configuration, mut alpha: i8, beta: i8,
            memo: Option<&mut HashMap<String, (i8, Movement)>>
) -> (i8, Option<Movement>) {
    if depth == 0 || !state.can_move() {
        return (state.value(), None);
    }

    // check the value of the prev iteration
    let memo = memo.unwrap();
    if let Some((bscore, bmove)) = memo.get(&state.serialize()) {
        return (*bscore, Some(*bmove));
    }
    let result = state
        .movements()
        .try_fold((i8::MIN, None), |acc, movement| {
            let (mut bscore, mut bmove) = acc;
            let (score, _) = alpha_beta_func(depth - 1, &(state.play(&movement)), -beta, -alpha);
            if score > bscore {
                bscore = score;
                bmove = Some(movement);
                if bscore > alpha {
                    alpha = bscore;
                }
                if beta < alpha {
                    return Err((bscore, bmove));
                }
            }
            Ok((bscore, bmove))
        });
    let (bscore, bmove) = match result {
        Ok((bscore, bmove)) => (bscore, bmove),
        Err((bscore, bmove)) => (bscore, bmove),
    };
    memo.insert(state.serialize(), (-bscore, bmove.unwrap()));
    (-bscore, bmove)
}

// static mut ets: Duration = Duration::from_secs(0);
// static mut etn: Duration = Duration::from_secs(0);
// static mut etnf: Duration = Duration::from_secs(0);
// static mut etp: Duration = Duration::from_secs(0);
// static mut etm: Duration = Duration::from_secs(0);
// static mut count: u32 = 0;
impl Strategy for AlphaBeta {
    fn compute_next_move(&mut self, state: &Configuration,
                    memo: Option<&mut HashMap<String, (i8, Movement)>>,
    ) -> Option<Movement> {

        // let start_time = Instant::now();
        // let (s0, mv0) = palpha_beta(self.0, state, i8::MIN + 1, i8::MAX);
        // let end_time = Instant::now();
        // unsafe {
        //     count += 1;
        //     etp += end_time.duration_since(start_time);
        //     println!("avg p {:?}", etp/count);
        // }

        // let start_time = Instant::now();
        // let (s1, mv1) = alpha_beta_sorted(self.0, state, i8::MIN + 1, i8::MAX);
        // let end_time = Instant::now();
        // unsafe {
        //     ets += end_time.duration_since(start_time);
        //     println!("avg sorted {:?}", ets/count);
        // }

        // let start_time = Instant::now();
        // let (_, mv2) = alpha_beta(self.0, state, i8::MIN + 1, i8::MAX);
        // let end_time = Instant::now();
        // let duration = end_time.duration_since(start_time);
        // if duration.as_secs_f32() > 1.0 {
        //     unreachable!("Duration was greater than 1 second");
        // }
        // unsafe {
        //     etn += end_time.duration_since(start_time);
        //     println!("avg normal {:?}", etn/count);
        // }

        // let start_time = Instant::now();
        // let (s2, mv2) = alpha_beta_func(self.0, state, i8::MIN + 1, i8::MAX);
        // let end_time = Instant::now();
        // unsafe {
        //     etnf += end_time.duration_since(start_time);
        //     println!("avg normal func {:?}", etnf/count);
        // }

        // let mut memo: HashMap<String, (i8, Movement)> = HashMap::new();
        // let start_time = Instant::now();
        // let (s3, mv3) = alpha_beta_func_memo(self.0, state, i8::MIN + 1, i8::MAX, &mut memo);
        // let end_time = Instant::now();
        // unsafe {
        //     etm += end_time.duration_since(start_time);
        //     println!("avg memo {:?}", etm/count);
        // }

        // assert!(s0 == s1 && s1 == s2);
        // let mv = if state.current_player {
        //     // blue_player
        // let (_, mv) = negascout(self.0, state, i8::MIN + 1, i8::MAX, memo);
        //     mv
        // } else {
        //     // red player
        //     let (_, mv) = alpha_beta_sorted(self.0, state, i8::MIN + 1, i8::MAX);
        //     mv
        // };
        let (_, mv2) = alpha_beta(self.0, state, i8::MIN + 1, i8::MAX);
        mv2
    }
}
