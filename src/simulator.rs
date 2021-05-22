use crate::models::*;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub struct Simulator {
    turn: usize,
    graph: GridGraph,
    queries: Vec<QueryParam>,
    visited: Grid<usize>,
    score: f64,
}

impl Simulator {}

#[derive(Debug, Clone)]
struct QueryParam {
    query: Query,
    res_factor: f64,
}
struct GridGraph {
    h_cost: [[u32; GRID_LEN - 1]; GRID_LEN],
    v_cost: [[u32; GRID_LEN]; GRID_LEN - 1],
}

impl GridGraph {
    fn get_cost(&self, p: Pos, d: Dir) -> u32 {
        assert!(p.move_to(d).is_some());
        match d {
            Dir::Up => self.v_cost[p.r as usize - 1][p.c as usize],
            Dir::Down => self.v_cost[p.r as usize][p.c as usize],
            Dir::Left => self.h_cost[p.r as usize][p.c as usize - 1],
            Dir::Right => self.h_cost[p.r as usize][p.c as usize],
        }
    }
}

impl Simulator {
    fn compute_shortest_path(&self, query: &Query) -> u32 {
        let src = query.src;
        let dest = query.dest;
        let mut dist: Grid<u32> = Grid::new(0);
        let mut prev: Grid<Dir> = Grid::new(Dir::Up);
        let mut queue = BinaryHeap::new();
        dist[src] = 0;
        queue.push(Reverse((0, src)));
        while let Some(Reverse((d, p))) = queue.pop() {
            if p == dest {
                break;
            }
            if dist[p] != d {
                continue;
            }
            for dir in Dir::iter() {
                if let Some(q) = p.move_to(dir) {
                    if dist[q] > d + self.graph.get_cost(p, dir) {
                        dist[q] = d + self.graph.get_cost(p, dir);
                        prev[q] = dir;
                        queue.push(Reverse((dist[q], q)));
                    }
                }
            }
        }
        dist[dest]
    }

    fn compute_path_length(&mut self, path: &[Dir]) -> Result<u32, String> {
        let src = self.queries[self.turn].query.src;
        let dest = self.queries[self.turn].query.dest;
        let mut p = src;
        let mut sum = 0;
        for &d in path {
            if self.visited[p] == self.turn {
                return Err(format!(
                    "visiting ({},{}) twice (query {})",
                    p.r,
                    p.c,
                    self.turn + 1
                ));
            }
            self.visited[p] = self.turn;

            let np = match p.move_to(d) {
                Some(np) => np,
                None => return Err(format!("going outside the map (query {})", self.turn + 1)),
            };

            sum += self.graph.get_cost(np, d);
            p = np;
        }
        if p != dest {
            return Err(format!("not an s-t path (query {})", self.turn + 1));
        }
        Ok(sum)
    }
}

impl Environment for Simulator {
    fn next_query(&self) -> Option<Query> {
        if self.turn < NUM_TURN {
            Some(self.queries[self.turn].query.clone())
        } else {
            None
        }
    }

    fn do_answer(&mut self, path: Vec<Dir>) -> f64 {
        let query = self.queries[self.turn].clone();
        let length = self.compute_path_length(&path).expect("invalid path");
        let best = self.compute_shortest_path(&query.query);
        let ratio = length as f64 / best as f64;
        assert!(length <= best);
        self.score = self.score * 0.998 + ratio;
        self.turn += 1;
        ratio * query.res_factor
    }
}
