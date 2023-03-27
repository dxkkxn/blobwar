//! Implementation of the min max algorithm.
use super::Strategy;
use crate::configuration::{Configuration, Movement};
use crate::shmem::AtomicMove;
use std::fmt;

/// Min-Max algorithm with a given recursion depth.
pub struct MinMax(pub u8);

// Given the evaluation function state.value this algo is more a negMax
// than min_max, the eval fuction negates its self at every recursive call
// so we calculate the max and we return the negation of the result to the
// parent node
fn min_max(depth: u8, state: &Configuration) -> (i8, Option<Movement>) {
    if depth == 0 || !state.can_move() {
        return (state.value(), None);
    }
    let mut best_score = i8::MIN;
    let mut best_move: Option<Movement> = None;
    for movement in state.movements() {
        let next_conf: Configuration = state.play(&movement);
        let (score, _) = min_max(depth - 1, &next_conf);
        if score > best_score {
            best_score = score;
            best_move = Some(movement);
        }
    }
    assert_ne!(best_score, i8::MIN); // maybe get rid of this for performance
    (-best_score, best_move)
}


impl Strategy for MinMax {
    fn compute_next_move(&mut self, state: &Configuration) -> Option<Movement> {
        let (_, mv) = min_max(self.0, state);
        mv
    }
}

impl fmt::Display for MinMax {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Min - Max (max level: {})", self.0)
    }
}

/// Anytime min max algorithm.
/// Any time algorithms will compute until a deadline is hit and the process is killed.
/// They are therefore run in another process and communicate through shared memory.
/// This function is intended to be called from blobwar_iterative_deepening.
pub fn min_max_anytime(state: &Configuration) {
    let mut movement = AtomicMove::connect().expect("failed connecting to shmem");
    for depth in 1..100 {
        movement.store(MinMax(depth).compute_next_move(state));
    }
}
