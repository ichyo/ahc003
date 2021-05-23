use crate::models::*;
use num_traits::{Bounded, Num};
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;

#[derive(PartialOrd, PartialEq)]
pub struct UnwrapOrd<T: PartialOrd + PartialEq>(pub T);

impl<T: PartialOrd + PartialEq> Eq for UnwrapOrd<T> {}

impl<T: PartialOrd + PartialEq> Ord for UnwrapOrd<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

pub fn compute_shortest_cost<G: GridGraph<T>, T: Bounded + Num + Copy + PartialOrd>(
    graph: &G,
    src: Pos,
    dest: Pos,
) -> T {
    let mut dist: Grid<T> = Grid::new(T::max_value());
    let mut queue = BinaryHeap::new();
    dist[src] = T::zero();
    queue.push(Reverse((UnwrapOrd(T::zero()), src)));
    while let Some(Reverse((UnwrapOrd(d), p))) = queue.pop() {
        if p == dest {
            break;
        }
        if dist[p] != d {
            continue;
        }
        for dir in Dir::iter() {
            if let Some(q) = p.move_to(dir) {
                if dist[q] > d + *graph.get(p, dir) {
                    dist[q] = d + *graph.get(p, dir);
                    queue.push(Reverse((UnwrapOrd(dist[q]), q)));
                }
            }
        }
    }
    dist[dest]
}

pub fn compute_shortest_path<G: GridGraph<T>, T: Bounded + Num + Copy + PartialOrd>(
    graph: &G,
    src: Pos,
    dest: Pos,
) -> (Vec<Dir>, T) {
    let mut dist: Grid<T> = Grid::new(T::max_value());
    let mut prev: Grid<Dir> = Grid::new(Dir::Up);
    let mut queue = BinaryHeap::new();
    dist[src] = T::zero();
    queue.push(Reverse((UnwrapOrd(T::zero()), src)));
    while let Some(Reverse((UnwrapOrd(d), p))) = queue.pop() {
        if p == dest {
            break;
        }
        if dist[p] != d {
            continue;
        }
        for dir in Dir::iter() {
            if let Some(q) = p.move_to(dir) {
                if dist[q] > d + *graph.get(p, dir) {
                    dist[q] = d + *graph.get(p, dir);
                    prev[q] = dir;
                    queue.push(Reverse((UnwrapOrd(dist[q]), q)));
                }
            }
        }
    }
    let mut path = Vec::new();
    let mut p = dest;
    while p != src {
        let d = prev[p];
        path.push(d);
        p = p.move_to(d.rev()).unwrap();
    }
    path.reverse();
    (path, dist[dest])
}
