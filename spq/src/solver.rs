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
    let mut v_count = vec![0; GRID_LEN]; // y
    let mut v_sum = vec![0.0; GRID_LEN]; // y
    let mut v_count_all = 0;

    let mut h_count = vec![0; GRID_LEN]; // x
    let mut h_sum = vec![0.0; GRID_LEN]; // x
    let mut h_count_all = 0;

    let exp_rate = 0.0;

    while let Some(query) = env.next_query() {
        let src = query.src;
        let dest = query.dest;
        let mut path = Vec::new();

        if query.width() <= query.height() {
            // find the best cm (c0 <= cm <= c1)
            // (r0, c0) -> (r0, cm) -> (r1, cm) -> (r1, c1)
            let c0 = query.src.c.min(query.dest.c);
            let c1 = query.src.c.max(query.dest.c);
            let cm = (c0..=c1)
                .max_by_key(|&c| {
                    if h_count[c as usize] == 0 {
                        return NotNan(f64::MAX);
                    }
                    let count = h_count[c as usize] as f64;
                    let count_all = h_count_all as f64;
                    let mean = h_sum[c as usize] / count;
                    let value = mean + exp_rate * (2.0 * count_all.ln() / count).sqrt();
                    NotNan(value)
                })
                .unwrap();

            let mid0 = Pos::new(src.r, cm);
            let mid1 = Pos::new(dest.r, cm);
            path.extend(construct_path(src, mid0));
            path.extend(construct_path(mid0, mid1));
            path.extend(construct_path(mid1, dest));

            let result = env.do_answer(&path);
            let result_avg = result / path.len() as f64;
            h_count[cm as usize] += 1;
            h_sum[cm as usize] += 10000.0 - result_avg;
            h_count_all += 1;
        } else {
            // find the best rm (r0 <= rm <= r1)
            // (r0, c0) -> (rm, c0) -> (rm, c1) -> (r1, c1)
            let r0 = query.src.r.min(query.dest.r);
            let r1 = query.src.r.max(query.dest.r);
            let rm = (r0..=r1)
                .max_by_key(|&r| {
                    if v_count[r as usize] == 0 {
                        return NotNan(f64::MAX);
                    }
                    let count = v_count[r as usize] as f64;
                    let count_all = v_count_all as f64;
                    let mean = v_sum[r as usize] / count;
                    NotNan(mean + exp_rate * (2.0 * count_all.ln() / count).sqrt())
                })
                .unwrap();
            let mid0 = Pos::new(rm, src.c);
            let mid1 = Pos::new(rm, dest.c);
            path.extend(construct_path(src, mid0));
            path.extend(construct_path(mid0, mid1));
            path.extend(construct_path(mid1, dest));

            let result = env.do_answer(&path);
            let result_avg = result / path.len() as f64;
            v_count[rm as usize] += 1;
            v_sum[rm as usize] += 100000.0 - result_avg;
            v_count_all += 1;
        }
    }
}
