//! Alpha - Beta algorithm.
use std::fmt;

use super::Strategy;
use crate::configuration::{Configuration, Movement};
use crate::shmem::AtomicMove;

/// Anytime alpha beta algorithm.
/// Any time algorithms will compute until a deadline is hit and the process is killed.
/// They are therefore run in another process and communicate through shared memory.
/// This function is intended to be called from blobwar_iterative_deepening.
pub fn alpha_beta_anytime(state: &Configuration) {
    let mut movement = AtomicMove::connect().expect("failed connecting to shmem");
    for depth in 1..100 {
        let chosen_movement = AlphaBeta(depth).compute_next_move(state);
        movement.store(chosen_movement);
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
    if depth == 0 || state.game_over() {
        return (state.value(), None);
    }
    let mut best_score = i8::MIN;
    let mut best_move: Option<Movement> = None;
    for movement in state.movements() {
        let next_conf : Configuration = state.play(&movement);
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
    if best_score == -128 {
        best_score = state.value();
        best_move = None;
    }
    (-best_score, best_move)
}

impl Strategy for AlphaBeta {
    fn compute_next_move(&mut self, state: &Configuration) -> Option<Movement> {
        let (_, mv) = alpha_beta(self.0, state, i8::MIN + 1, i8::MAX);
        mv
    }
}
