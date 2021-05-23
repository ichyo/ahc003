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

impl GridGraph<u32> for GraphEstimator {
    fn get(&self, p: Pos, d: Dir) -> &u32 {
        match d {
            Dir::Up => &self.estimation.horizontal[p.r as usize - 1],
            Dir::Down => &self.estimation.horizontal[p.r as usize],
            Dir::Left => &self.estimation.verticle[p.c as usize - 1],
            Dir::Right => &self.estimation.verticle[p.c as usize],
        }
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
}

pub fn run_solver<E: Environment>(env: &mut E) {
    let mut estimator = GraphEstimator::new();
    while let Some(query) = env.next_query() {
        debug!(
            "Start processing a query ({:2}, {:2}) -> ({:2}, {:2}) width={:2} height={:2}",
            query.src.r,
            query.src.c,
            query.dest.r,
            query.dest.c,
            query.width(),
            query.height()
        );
        let (path, estimated_length) = compute_shortest_path(&estimator, query.src, query.dest);
        debug!(
            "Sending a path: {}",
            path.iter().map(|d| d.to_char()).collect::<String>()
        );
        let response = env.do_answer(&path);
        debug!(
            "Got a response. length={:.2} esimated={:6} ratio={:.2}",
            response,
            estimated_length,
            response / estimated_length as f64
        );
        estimator.insert_new_record(&query, &path, response);
    }
}
