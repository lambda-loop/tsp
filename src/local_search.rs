use crate::problem::{Point, TSP};

#[derive(Clone, Copy, Default)]
pub struct LocalSearch {
    pub two_opt: bool,
    pub relocate: bool,
    pub lin_kernighan: bool,
}

impl LocalSearch {
    pub fn improve(self, tsp: &TSP, tour: &mut [Point]) {
        if self.two_opt {
            two_opt(tsp, tour);
        }
        if self.relocate {
            relocate(tsp, tour);
        }
        if self.lin_kernighan {
            lin_kernighan(tsp, tour);
        }
    }
}

fn two_opt(tsp: &TSP, tour: &mut [Point]) {
    let len = tour.len();
    let distance = |a, b| u64::from(tsp.distance(tour[a] as usize, tour[b] as usize));
    let mut best = None;
    let mut best_saving = 0;

    // Compare pairs of nonadjacent edges in the closed, symmetric tour.
    for i in 0..len {
        for j in i + 2..len {
            let next = (j + 1) % len;
            if next == i {
                continue;
            }
            let removed = distance(i, i + 1) + distance(j, next);
            let added = distance(i, j) + distance(i + 1, next);
            let saving = removed.saturating_sub(added);
            if saving > best_saving {
                best_saving = saving;
                best = Some((i + 1, j));
            }
        }
    }

    if let Some((start, end)) = best {
        tour[start..=end].reverse();
    }
}

fn relocate(tsp: &TSP, tour: &mut [Point]) {
    let len = tour.len();
    let distance = |a, b| u64::from(tsp.distance(tour[a] as usize, tour[b] as usize));
    let mut best = None;
    let mut best_saving = 0;

    for from in 0..len {
        let prev = (from + len - 1) % len;
        let next = (from + 1) % len;
        let removed_at_source = distance(prev, from) + distance(from, next);
        let added_at_source = distance(prev, next);
        for after in 0..len {
            if after == from || after == prev {
                continue;
            }
            let successor = (after + 1) % len;
            let removed = removed_at_source + distance(after, successor);
            let added = added_at_source + distance(after, from) + distance(from, successor);
            let saving = removed.saturating_sub(added);
            if saving > best_saving {
                best_saving = saving;
                best = Some((from, after));
            }
        }
    }

    if let Some((from, after)) = best {
        if from < after {
            tour[from..=after].rotate_left(1);
        } else {
            tour[after + 1..=from].rotate_right(1);
        }
    }
}

// Bounded sequential Lin–Kernighan search for symmetric distances.
fn lin_kernighan(tsp: &TSP, tour: &mut [Point]) {
    if tour.len() < 4 {
        return;
    }

    let initial_cost = tsp.tour_cost(tour);
    let mut order = tour.to_vec();
    let mut search = LinKernighan {
        tsp,
        used: vec![false; tsp.len],
        best: tour,
        best_gain: 0,
    };
    for _ in 0..order.len() {
        for _ in 0..2 {
            let first = order[0];
            let last = order[order.len() - 1];
            search.used[first as usize] = true;
            search.used[last as usize] = true;
            search.extend(&mut order, search.distance(first, last), 1);
            search.used[first as usize] = false;
            search.used[last as usize] = false;
            order[1..].reverse();
        }
        order.rotate_left(1);
    }
    debug_assert_eq!(
        tsp.tour_cost(search.best) + search.best_gain as u64,
        initial_cost
    );
}

struct LinKernighan<'a> {
    tsp: &'a TSP,
    used: Vec<bool>,
    best: &'a mut [Point],
    best_gain: i64,
}

impl LinKernighan<'_> {
    fn distance(&self, from: Point, to: Point) -> i64 {
        i64::from(self.tsp.distance(from as usize, to as usize))
    }

    fn extend(&mut self, order: &mut [Point], gain: i64, removed: usize) {
        // The tour is open from first to last. Add (last, join), remove
        // (join, new_last), then reverse the suffix to keep one open path.
        const MAX_EXCHANGE: usize = 5;
        const BRANCH_WIDTH: usize = 3;

        let first = order[0];
        let last = order[order.len() - 1];
        let mut continuations = [(0, 0); BRANCH_WIDTH];
        let branch_width = if removed == 1 { BRANCH_WIDTH } else { 1 };
        for i in 1..order.len() - 2 {
            let join = order[i];
            let new_last = order[i + 1];
            if self.used[join as usize] || self.used[new_last as usize] {
                continue;
            }

            let partial_gain = gain - self.distance(last, join);
            if partial_gain <= 0 {
                continue;
            }
            let next_gain = partial_gain + self.distance(join, new_last);
            let closed_gain = next_gain - self.distance(new_last, first);
            if closed_gain > self.best_gain {
                self.best_gain = closed_gain;
                self.best.copy_from_slice(order);
                self.best[i + 1..].reverse();
            }

            if removed + 1 < MAX_EXCHANGE {
                let candidates = &mut continuations[..branch_width];
                if let Some(rank) = candidates.iter().position(|&(gain, _)| next_gain > gain) {
                    candidates[rank..].rotate_right(1);
                    candidates[rank] = (next_gain, i);
                }
            }
        }

        for (next_gain, i) in continuations.into_iter().take_while(|&(gain, _)| gain > 0) {
            let join = order[i] as usize;
            let new_last = order[i + 1] as usize;
            self.used[join] = true;
            self.used[new_last] = true;
            order[i + 1..].reverse();
            self.extend(order, next_gain, removed + 1);
            order[i + 1..].reverse();
            self.used[join] = false;
            self.used[new_last] = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genetic::new_tour;
    use ::rand::{SeedableRng, rngs::SmallRng};

    #[test]
    fn operators_improve_a_crossing_and_preserve_cities() {
        let tsp = TSP::parser(
            "NAME: square\nTYPE: TSP\nDIMENSION: 4\nEDGE_WEIGHT_TYPE: EUC_2D\n\
             NODE_COORD_SECTION\n1 0 0\n2 10 0\n3 10 10\n4 0 10\nEOF\n",
        )
        .unwrap();

        for improve in [two_opt, relocate, lin_kernighan] {
            let mut tour = [0, 2, 1, 3];
            improve(&tsp, &mut tour);
            assert_eq!(tsp.tour_cost(&tour), 40);
            improve(&tsp, &mut tour);
            assert_eq!(tsp.tour_cost(&tour), 40);
            tour.sort_unstable();
            assert_eq!(tour, [0, 1, 2, 3]);
        }
    }

    #[test]
    fn lin_kernighan_can_improve_a_two_opt_local_minimum() {
        let tsp = TSP::parser(include_str!("../data/eil51.tsp")).unwrap();
        let mut rng = SmallRng::seed_from_u64(71);
        let mut escaped = false;

        for _ in 0..12 {
            let mut tour = new_tour(&mut rng, tsp.len);
            let mut cost = tsp.tour_cost(&tour);
            loop {
                two_opt(&tsp, &mut tour);
                let next = tsp.tour_cost(&tour);
                if next == cost {
                    break;
                }
                assert!(next < cost);
                cost = next;
            }

            lin_kernighan(&tsp, &mut tour);
            let improved = tsp.tour_cost(&tour);
            assert!(improved <= cost);
            escaped |= improved < cost;
            tour.sort_unstable();
            assert!(tour.iter().copied().eq(0..tsp.len as Point));
        }

        assert!(escaped);
    }
}
