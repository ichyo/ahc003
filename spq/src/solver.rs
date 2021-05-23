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

pub fn run_solver<E: Environment>(env: &mut E) {
    while let Some(query) = env.next_query() {
        debug!("query {:?} -> {:?}", query.src, query.dest);
        env.do_answer(&construct_path(query.src, query.dest));
    }
}
