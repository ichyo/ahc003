use crate::algorithms::compute_shortest_cost;
use crate::models::*;
use rand::prelude::*;

#[derive(Debug)]
pub struct ScoreDetail {
    pub best: u32,
    pub length: u32,
}

impl ScoreDetail {
    pub fn ratio(&self) -> f64 {
        self.best as f64 / self.length as f64
    }
}

#[derive(Debug)]
pub enum GraphParams {
    Single {
        d: u16,
        h: Vec<u16>,
        v: Vec<u16>,
    },
    Double {
        d: u16,
        h: Vec<(u16, u16)>,
        cm: Vec<u8>,
        v: Vec<(u16, u16)>,
        rm: Vec<u8>,
    },
}

pub struct Simulator {
    turn: usize,
    graph_params: GraphParams,
    graph: ArrayGridGraph<u32>,
    queries: Vec<QueryParam>,
    visited: Grid<usize>,
    score: f64,
    best_score: f64,
    score_details: Vec<ScoreDetail>,
}

impl Simulator {
    pub fn queries(&self) -> &Vec<QueryParam> {
        &self.queries
    }
    pub fn score_details(&self) -> &Vec<ScoreDetail> {
        &self.score_details
    }
    pub fn raw_score(&self) -> f64 {
        self.score
    }

    /// 0.0 ~ 1.0
    pub fn ratio_score(&self) -> f64 {
        //self.score / 432.4677387766579
        self.score / self.best_score
    }

    pub fn atcoder_score(&self) -> i64 {
        (self.score * 2312311.0).round() as i64
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

    fn do_answer(&mut self, path: &[Dir]) -> f64 {
        let query = self.queries[self.turn].clone();
        let length = self.compute_path_length(&path).expect("invalid path");
        let best = compute_shortest_cost(&self.graph, query.query.src, query.query.dest);
        let ratio = best as f64 / length as f64;
        assert!(
            length >= best,
            "score {} is better than best {}",
            length,
            best
        );
        self.score_details.push(ScoreDetail { length, best });
        self.score = self.score * 0.998 + ratio;
        self.best_score = self.best_score * 0.998 + 1.0;
        self.turn += 1;

        debug!(
            "Got a path: best={:6} output={:6} ratio={:.2}. Ratio score is {:.4}",
            best,
            length,
            ratio,
            self.ratio_score(),
        );
        debug!(
            "Returning a response: {:6} * {:.2} = {:.2}",
            length,
            query.res_factor,
            length as f64 * query.res_factor
        );

        length as f64 * query.res_factor
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

        let graph_params = if m == 1 {
            GraphParams::Single {
                d: d as u16,
                h: hb.iter().map(|v| v[0] as u16).collect(),
                v: vb.iter().map(|v| v[0] as u16).collect(),
            }
        } else {
            GraphParams::Double {
                d: d as u16,
                h: hb.iter().map(|v| (v[0] as u16, v[1] as u16)).collect(),
                rm: x.iter().map(|v| v[1] as u8).collect(),
                v: vb.iter().map(|v| (v[0] as u16, v[1] as u16)).collect(),
                cm: y.iter().map(|v| v[1] as u8).collect(),
            }
        };

        match &graph_params {
            GraphParams::Single { d, h, v } => {
                debug!("Generate single graph d={:4}", d);
                debug!("h={:?}", h);
                debug!("v={:?}", v);
            }
            GraphParams::Double { d, h, rm, v, cm } => {
                debug!("Generate double graph d={:4}", d);
                debug!("h={:?}", h);
                debug!("v={:?}", v);
                debug!("rm={:?}", rm);
                debug!("cm={:?}", cm);
            }
        };

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
            graph_params,
            graph: ArrayGridGraph::from_arrays(h, v),
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
            best_score: 0.0,
            score_details: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryParam {
    pub query: Query,
    pub res_factor: f64,
}

impl Simulator {
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
            sum += self.graph.get(p, d);

            p = np;
        }
        if p != dest {
            return Err(format!("not an s-t path (query {})", self.turn + 1));
        }
        Ok(sum)
    }
}
