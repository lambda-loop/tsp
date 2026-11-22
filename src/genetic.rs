

use rand::{
    *, rngs::SmallRng,
    seq::SliceRandom,
};

use fixedbitset::FixedBitSet as Set;

use crate::problem::*;

use std::cmp::Ordering::{
    Equal, Less, Greater,
};


pub fn new_tour (
    rng  : &mut SmallRng,
    t_len: usize   , 
) -> Box<[Point]> {
    let mut tour: Vec<Point> = (0..t_len)
        .map(|i| i as Point)
        .collect();
    
    tour.shuffle(rng);
    tour.into_boxed_slice()
}

// inversion based
pub fn mutation (
    tour : &mut [Point] ,
    rng  : &mut SmallRng,
    t_len: usize        ,
) {
    if t_len < 2 {
        return;
    }

    let first = rng.random_range(0..t_len);
    let second = rng.random_range(0..t_len - 1);
    let second = second + usize::from(second >= first);
    let (begin, end) = (first.min(second), first.max(second));
    tour[begin..=end].reverse()
}

pub fn crossing (
    l    : &[Point]     ,
    r    : &[Point]     ,
    r#in : &mut [Point] ,
    rng  : &mut SmallRng,
    t_len: usize        ,
    set_buffer: &mut Set, // needs to be cleared outside!
) -> Option<(usize, usize)> {
    if let Some((begin, end)) = pick_interval(rng, t_len) {
        for i in begin..=end {
            let v   = l[i];
            r#in[i] = v;
            set_buffer.insert(v as usize);

        }

        
        let mut r_i  = (end + 1) % t_len; 
        let mut in_i = (end + 1) % t_len; 
        while in_i != begin {
            let v = r[r_i]; r_i = (r_i + 1) % t_len;
            if set_buffer.contains(v as usize) {
                continue;
            }

            r#in[in_i] = v;
            in_i = (in_i + 1) % t_len;
        }
        
        Some((begin, end))
    } else {
        r#in.copy_from_slice(r);
        None
    }
}


pub fn pick_interval (
    rng  : &mut SmallRng,
    bound: usize        ,
) -> Option<(usize, usize)> {
    let i = rng.random_range(0..bound);
    let j = rng.random_range(0..bound);
    match i.cmp(&j) {
        Less    => Some((i, j)),
        Equal   => None        ,
        Greater => Some((j, i)),
    }
}
