//! Dumb greedy algorithm.
use super::Strategy;
use crate::{configuration::{Configuration, Movement}, positions::Position};
use std::fmt;

/// Dumb algorithm.
/// Amongst all possible movements return the one which yields the configuration with the best
/// immediate value.
pub struct Greedy();

impl fmt::Display for Greedy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Greedy")
    }
}

impl Strategy for Greedy {
    fn compute_next_move(&mut self, state: &Configuration) -> Option<Movement> {
        let mut best_score = state.value();
        let mut best_move: Option<Movement> = None;
        for movement in state.movements() {
            let next_conf : Configuration = state.play(&movement);
            let score = next_conf.value();
            if score > best_score {
                best_score = score;
                best_move = Some(movement);
            }
        }
        best_move
    }
}
