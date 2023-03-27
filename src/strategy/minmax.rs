//! Implementation of the min max algorithm.
use super::Strategy;
use crate::configuration::{Configuration, Movement};
use crate::shmem::AtomicMove;
use std::fmt;
use std::time::Instant;
use rayon::prelude::*;

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


// neg_max but in functional programming
fn neg_max(depth: u8, state: &Configuration) -> (i8, Option<Movement>) {
    if depth == 0 || !state.can_move() {
        return (state.value(), None);
    }
    let (bmove, (score, _)) = state
        .movements()
        .map(|movement| (movement, neg_max(depth - 1, &(state.play(&movement)))))
        .max_by_key(|&(m, (value, _))| value)
        .unwrap();
    (-score, Some(bmove))
}


// parallel neg_max
fn pneg_max(depth: u8, state: &Configuration) -> (i8, Option<Movement>) {
    if depth == 0 || !state.can_move() {
        return (state.value(), None);
    }
    let (bmove, (score, _)) = state
        .movements()
        .par_bridge()
        .map(|movement| (movement, pneg_max(depth - 1, &(state.play(&movement)))))
        .max_by_key(|&(m, (value, _))| value)
        .unwrap();
    (-score, Some(bmove))
}

impl Strategy for MinMax {
    fn compute_next_move(&mut self, state: &Configuration) -> Option<Movement> {
        let start_time = Instant::now();
        let (_, mv) = pneg_max(self.0, state);
        let end_time = Instant::now();
        println!("Time elapsed: {:?}", end_time.duration_since(start_time));
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
