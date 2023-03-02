extern crate blobwar;
//use blobwar::board::Board;
use blobwar::configuration::Configuration;
use blobwar::strategy::{Greedy, Human, MinMax};

fn main() {
    let board = Default::default();
    let mut game = Configuration::new(&board);
    game.battle(Greedy(), MinMax(5));
}
