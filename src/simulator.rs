use crate::models::*;
use rand::prelude::*;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub struct Simulator {
    turn: usize,
    graph: GridGraph,
    queries: Vec<QueryParam>,
    visited: Grid<usize>,
    score: f64,
}

impl Simulator {
    pub fn raw_score(&self) -> f64 {
        self.score
    }
    pub fn atcoder_score(&self) -> i64 {
        (self.score * 2312311.0).round() as i64
    }
}

impl Simulator {
    pub fn from_seed(seed: u64) -> Simulator {
        let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
        let d: i32 = rng.gen_range(100, 2001);
        let m = rng.gen_range(1, 3u32) as usize;
        let hb = (0..GRID_LEN)
            .map(|_| {
                (0..m)
                    .map(|_| rng.gen_range(1000 + d, 9001 - d))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut x = vec![vec![]; GRID_LEN];
        for i in 0..GRID_LEN {
            x[i].push(0);
            if m == 2 {
                x[i].push(rng.gen_range(1, GRID_LEN as u32 - 1) as usize);
            }
            x[i].push(GRID_LEN - 1);
        }
        let mut h = [[0; GRID_LEN - 1]; GRID_LEN];
        for i in 0..GRID_LEN {
            for p in 0..m {
                for j in x[i][p]..x[i][p + 1] {
                    h[i][j] = (hb[i][p] + rng.gen_range(-d, d + 1)) as u32;
                }
            }
        }

        let vb = (0..GRID_LEN)
            .map(|_| {
                (0..m)
                    .map(|_| rng.gen_range(1000 + d, 9001 - d))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let mut y = vec![vec![]; GRID_LEN];
        for j in 0..GRID_LEN {
            y[j].push(0);
            if m == 2 {
                y[j].push(rng.gen_range(1, GRID_LEN as u32 - 1) as usize);
            }
            y[j].push(GRID_LEN - 1);
        }
        let mut v = [[0; GRID_LEN]; GRID_LEN - 1];
        for j in 0..GRID_LEN {
            for p in 0..m {
                for i in y[j][p]..y[j][p + 1] {
                    v[i][j] = (vb[j][p] + rng.gen_range(-d, d + 1)) as u32;
                }
            }
        }
        let mut s = vec![];
        let mut t = vec![];
        let mut e = vec![];

        fn dist(p: (usize, usize), q: (usize, usize)) -> usize {
            let di = if p.0 < q.0 { q.0 - p.0 } else { p.0 - q.0 };
            let dj = if p.1 < q.1 { q.1 - p.1 } else { p.1 - q.1 };
            di + dj
        }

        for _ in 0..NUM_TURN {
            let mut sk = (0, 0);
            let mut tk = (0, 0);
            while dist(sk, tk) < 10 {
                sk = (
                    rng.gen_range(0, GRID_LEN as u32) as usize,
                    rng.gen_range(0, GRID_LEN as u32) as usize,
                );
                tk = (
                    rng.gen_range(0, GRID_LEN as u32) as usize,
                    rng.gen_range(0, GRID_LEN as u32) as usize,
                );
            }
            s.push(sk);
            t.push(tk);
            e.push(rng.gen_range(0.9, 1.1));
        }
        Simulator {
            turn: 0,
            graph: GridGraph {
                h_cost: h,
                v_cost: v,
            },
            visited: Grid::new(usize::max_value()),
            queries: (0..NUM_TURN)
                .map(|i| QueryParam {
                    query: Query {
                        src: Pos::new(s[i].0 as u8, s[i].1 as u8),
                        dest: Pos::new(t[i].0 as u8, t[i].1 as u8),
                    },
                    res_factor: e[i],
                })
                .collect(),
            score: 0.0,
        }
    }
}

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
