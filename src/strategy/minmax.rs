//! Implementation of the min max algorithm.
use super::Strategy;
use crate::configuration::{Configuration, Movement};
use crate::shmem::AtomicMove;
use std::fmt;
use rayon::prelude::*;
use std::collections::HashMap;
// use std::time::{Duration, Instant};
// use lazy_static::lazy_static;

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


fn min_max_with_avg(depth: u8, state: &Configuration, with_avg: bool) -> (i8, f32, Option<Movement>) {
    if depth == 0 || !state.can_move() {
        return (state.value(), state.value().into(), None);
    }
    let mut best_score = i8::MIN;
    let mut best_avg = f32::MIN;
    let mut best_move: Option<Movement> = None;
    let mut count: i16 = 0;
    let mut sum: i16 = 0;
    for movement in state.movements() {
        let next_conf: Configuration = state.play(&movement);
        let (score, avg, _) = min_max_with_avg(depth - 1, &next_conf, !with_avg);
        if score > best_score || (with_avg && score == best_score && avg > best_avg) {
            best_score = score;
            best_avg = avg;
            best_move = Some(movement);
        }
        count += 1;
        sum += score as i16;
    }
    assert_ne!(best_score, i8::MIN); // maybe get rid of this for performance
    (-best_score, (-sum/count).into(), best_move)
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
        // .par_bridge()
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|movement| (movement, pneg_max(depth - 1, &(state.play(&movement)))))
        .max_by_key(|&(_, (value, _))| value)
        .unwrap();
    (-score, Some(bmove))
}

//expectimax in functional programming, hard make parallel
fn pexpectimax(depth: u8, state: &Configuration, with_avg: bool) -> (i8, f32, Option<Movement>) {
    if depth == 0 || !state.can_move() {
        return (state.value(), state.value().into(), None);
    }

    let (bmove, score, _bavg, sum, count) = state
        .movements()
        .map(|movement| (movement, pexpectimax(depth - 1, &(state.play(&movement)), !with_avg)))
        .fold((None, i8::MIN, f32::MIN, 0i16, 0i16), |acc, (mov,(score, avg, _))| {
            let (mut bmove, mut max, mut bavg, sum, count) = acc;
            if score > max || (with_avg && score == max && avg > bavg) {
                max = score;
                bmove = Some(mov);
                bavg = avg;
            }
            (bmove, max, bavg, sum+score as i16, count+1)
        });
    (-score, (-sum/count).into(), bmove)
}

// static mut min_maxs: Duration = Duration::from_secs(0);
// static mut min_max_with_avgs: Duration = Duration::from_secs(0);
// static mut neg_maxs: Duration = Duration::from_secs(0);
// static mut pneg_maxs: Duration = Duration::from_secs(0);
// static mut count1: u32 = 0;

// static mut gvec: Vec<f32> = Vec::new();

impl Strategy for MinMax {
    fn compute_next_move(
        &mut self,
        state: &Configuration,
        _memo: Option<&mut HashMap<String, (i8, Movement)>>,
    ) -> Option<Movement> {
        // let start_time = Instant::now();
        // let (s0, mv0) = min_max(self.0, state);
        // let end_time = Instant::now();
        // unsafe {
        //     count1 += 1;
        //     min_maxs += end_time.duration_since(start_time);
        //     println!("avg min_max {:?}", min_maxs/count1);
        // }

        // let start_time = Instant::now();
        // let (s1, _, mv1) = pexpectimax(self.0, state, true);
        // let end_time = Instant::now();
        // let duration = end_time.duration_since(start_time);
        // // if duration.as_secs_f32() > 1.0 {
        // //     unreachable!("Duration was greater than 1 second");
        // // }
        // println!("time: {duration:?}");
        // mv1

        // let start_time = Instant::now();
        // let (s2, mv2) = neg_max(self.0, state);
        // let end_time = Instant::now();
        // unsafe {
        //     neg_maxs += end_time.duration_since(start_time);
        //     println!("avg neg_max {:?}", neg_maxs/count1);
        // }

        // let start_time = Instant::now();
        // let (s2, mv3) = pneg_max(self.0, state);
        // let end_time = Instant::now();
        // unsafe {
        //     pneg_maxs += end_time.duration_since(start_time);
        //     println!("avg normal func {:?}", pneg_maxs/count1);
        // }
        // println!("equality {}", mv0 == mv1);
        // println!("equality {}", mv1 == mv2);
        // println!("equality {}", mv2 == mv3);
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
        // let start_time = Instant::now();
        // let (_s1, mv1) = pneg_max(self.0, state);
        // let end_time = Instant::now();
        // let duration = end_time.duration_since(start_time);
        // if duration.as_secs_f32() > 1.0 {
        //     unreachable!("Duration was greater than 1 second");
        // }
        // unsafe {
        //     gvec.push(duration.as_secs_f32());
        //     println!(">{:?}", gvec);
        // }
        // mv1
        let (s1, _, mv1) = pexpectimax(self.0, state, true);
        mv1
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
        movement.store(MinMax(depth).compute_next_move(state, None));
    }
}
