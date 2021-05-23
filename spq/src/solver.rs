use crate::algorithms::compute_shortest_path;
use crate::models::*;
use std::ops::Index;

struct Record {
    query: Query,
    path: Vec<Dir>,
    response: u32,
}

struct GraphEstimator {
    estimation: GridLines<u32>,
    records: Vec<Record>,
}

impl Index<EdgeIndex> for GraphEstimator {
    type Output = u32;

    fn index(&self, index: EdgeIndex) -> &Self::Output {
        &self.estimation[index.line]
    }
}

impl GraphEstimator {
    fn new() -> GraphEstimator {
        GraphEstimator {
            estimation: GridLines::new(5000),
            records: Vec::new(),
        }
    }

    fn insert_new_record(&mut self, query: &Query, path: &[Dir], response: u32) {
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
        let mut count = vec![GridLines::new(0); n]; // row
        let mut length = vec![0i64; n];
        for i in 0..n {
            let mut p = self.records[i].query.src;
            for &d in &self.records[i].path {
                let edge = EdgeIndex::from_move(p, d);
                length[i] += self.estimation[edge.line] as i64;
                count[i][edge.line] += 1;
                p = p.move_to(d).unwrap();
            }
            assert!(p == self.records[i].query.dest);
        }
        let mut diff = 0i64;
        for i in 0..n {
            diff += (length[i] as i64 - self.records[i].response as i64).abs();
        }

        let mut updated = true;
        let mut loops = 0;
        trace!("Start updating estimation from diff={}", diff);
        while updated {
            loops += 1;
            updated = false;
            for sign in &[-100i64, 100i64] {
                for line in LineIndex::iter() {
                    let est = self.estimation[line];
                    let new_est = est as i64 + sign;
                    if new_est < 0 || new_est > 9000 {
                        continue;
                    }

                    let mut new_diff = 0;
                    for i in 0..n {
                        let count = count[i][line];
                        new_diff += (length[i] as i64 + sign * count
                            - self.records[i].response as i64)
                            .abs();
                    }
                    if new_diff < diff {
                        for i in 0..n {
                            let count = count[i][line];
                            length[i] += sign * count;
                        }
                        self.estimation[line] = new_est as u32;
                        diff = new_diff;
                        updated = true;
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
        let (path, estimated_length) = compute_shortest_path(&estimator, query.src, query.dest);
        trace!(
            "Sending a path: {}",
            path.iter().map(|d| d.to_char()).collect::<String>()
        );
        let response = env.do_answer(&path);
        trace!(
            "Got a response. length={:6} esimated={:6} ratio={:.2}",
            response,
            estimated_length,
            response as f64 / estimated_length as f64
        );
        estimator.insert_new_record(&query, &path, response);
        estimator.update_estimation();
    }
}
