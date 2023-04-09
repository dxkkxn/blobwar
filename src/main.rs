extern crate blobwar;
use blobwar::board::Board;
use blobwar::configuration::Configuration;
use blobwar::strategy::{Greedy, Human, MinMax, AlphaBeta};
use blobwar::strategy::{alpha_beta_anytime, min_max_anytime};

fn main() {
    // let board = Board::load("constrained").expect("failed loading board");
    let board = Default::default();
    let mut game = Configuration::new(&board);
    // game.battle(MinMax(2), Greedy()); // red, blue
    // game.battle(Greedy(), MinMax(2)); // red, blue
    // game.battle(MinMax(3), MinMax(2)); // red, blue
    game.battle(MinMax(3), AlphaBeta(4)); // red, blue
    // game.battle(AlphaBeta(4), MinMax(3)); // red, blue
    // alpha_beta_anytime(&game);
    // game.battle(alpha_beta_any_time(), MinMax(3)); // red, blue
    // game.battle(MinMax(3), MinMax(4)); // red, blue
    // game.battle(MinMax(4), MinMax(3)); // red, blue
}
