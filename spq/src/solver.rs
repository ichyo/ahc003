use crate::algorithms::compute_shortest_path;
use crate::models::*;
use std::cmp::Ordering;

#[derive(Debug, PartialEq, PartialOrd)]
struct NotNan(f64);

impl Eq for NotNan {}

impl Ord for NotNan {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn construct_path(src: Pos, dest: Pos) -> Vec<Dir> {
    let mut path = Vec::new();
    let mut p = src;
    while p.r > dest.r {
        path.push(Dir::Up);
        p.r -= 1;
    }
    while p.r < dest.r {
        path.push(Dir::Down);
        p.r += 1;
    }
    while p.c > dest.c {
        path.push(Dir::Left);
        p.c -= 1;
    }
    while p.c < dest.c {
        path.push(Dir::Right);
        p.c += 1;
    }
    path
}

struct Estimation {
    verticle: [u32; GRID_LEN],
    horizontal: [u32; GRID_LEN],
}

impl Estimation {
    fn new() -> Estimation {
        Estimation {
            verticle: [5000; GRID_LEN],
            horizontal: [5000; GRID_LEN],
        }
    }
}

struct Record {
    query: Query,
    path: Vec<Dir>,
    response: f64,
}

struct GraphEstimator {
    estimation: Estimation,
    records: Vec<Record>,
}

impl GridGraph<u32> for Estimation {
    fn get(&self, p: Pos, d: Dir) -> &u32 {
        match compuate_line(p, d) {
            (Axis::Vertical, c) => &self.verticle[c as usize],
            (Axis::Horizontal, r) => &self.horizontal[r as usize],
        }
    }
}

enum Axis {
    Horizontal, // left-right. row
    Vertical,   // up-down. col
}

fn compuate_line(p: Pos, d: Dir) -> (Axis, u8) {
    match d {
        Dir::Up => (Axis::Vertical, p.c),
        Dir::Down => (Axis::Vertical, p.c),
        Dir::Left => (Axis::Horizontal, p.r),
        Dir::Right => (Axis::Horizontal, p.r),
    }
}

impl GraphEstimator {
    fn new() -> GraphEstimator {
        GraphEstimator {
            estimation: Estimation::new(),
            records: Vec::new(),
        }
    }

    fn insert_new_record(&mut self, query: &Query, path: &[Dir], response: f64) {
        self.records.push(Record {
            query: query.clone(),
            path: path.iter().cloned().collect(),
            response,
        });
    }

    fn update_estimation(&mut self) {
        let n = self.records.len();
        if n == 0 {
            return;
        }
        let mut h_count = vec![[0i64; GRID_LEN]; n]; // row
        let mut v_count = vec![[0i64; GRID_LEN]; n]; // col
        let mut length = vec![0i64; n];
        for i in 0..n {
            let mut p = self.records[i].query.src;
            for &d in &self.records[i].path {
                length[i] += *self.estimation.get(p, d) as i64;
                match compuate_line(p, d) {
                    (Axis::Horizontal, r) => {
                        h_count[i][r as usize] += 1;
                    }
                    (Axis::Vertical, c) => {
                        v_count[i][c as usize] += 1;
                    }
                }
                p = p.move_to(d).unwrap();
            }
            assert!(p == self.records[i].query.dest);
        }
        let mut diff = 0i64;
        for i in 0..n {
            diff += (length[i] as i64 - self.records[i].response.round() as i64).abs();
        }

        let mut updated = true;
        let mut loops = 0;
        trace!("Start updating estimation from diff={}", diff);
        while updated {
            loops += 1;
            updated = false;
            for sign in &[-1i64, 1i64] {
                for axis in &[Axis::Vertical, Axis::Horizontal] {
                    for x in 0..GRID_LEN {
                        let est = match *axis {
                            Axis::Horizontal => &mut self.estimation.horizontal[x as usize],
                            Axis::Vertical => &mut self.estimation.verticle[x as usize],
                        };
                        let new_est = *est as i64 + sign;
                        if new_est < 0 || new_est > 9000 {
                            continue;
                        }

                        let mut new_diff = 0;
                        for i in 0..n {
                            let count = match axis {
                                Axis::Horizontal => h_count[i][x as usize],
                                Axis::Vertical => v_count[i][x as usize],
                            };
                            new_diff += (length[i] as i64 + sign * count
                                - self.records[i].response.round() as i64)
                                .abs();
                        }
                        if new_diff < diff {
                            for i in 0..n {
                                let count = match axis {
                                    Axis::Horizontal => h_count[i][x as usize],
                                    Axis::Vertical => v_count[i][x as usize],
                                };
                                length[i] += sign * count;
                            }
                            *est = new_est as u32;
                            diff = new_diff;
                            updated = true;
                        }
                    }
                }
            }
        }
        trace!("Finish updating estimation. diff={} loops={}", diff, loops);
    }
}

pub fn run_solver<E: Environment>(env: &mut E) {
    let mut estimator = GraphEstimator::new();
    while let Some(query) = env.next_query() {
        trace!(
            "Start processing a query ({:2}, {:2}) -> ({:2}, {:2}) width={:2} height={:2}",
            query.src.r,
            query.src.c,
            query.dest.r,
            query.dest.c,
            query.width(),
            query.height()
        );
        let (path, estimated_length) =
            compute_shortest_path(&estimator.estimation, query.src, query.dest);
        trace!(
            "Sending a path: {}",
            path.iter().map(|d| d.to_char()).collect::<String>()
        );
        let response = env.do_answer(&path);
        trace!(
            "Got a response. length={:.2} esimated={:6} ratio={:.2}",
            response,
            estimated_length,
            response / estimated_length as f64
        );
        estimator.insert_new_record(&query, &path, response);
        estimator.update_estimation();
    }
}
