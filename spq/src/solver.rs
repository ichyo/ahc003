use crate::models::*;

pub fn run_solver<E: Environment>(env: &mut E) {
    while let Some(query) = env.next_query() {
        let mut path = Vec::new();
        let mut p = query.src;
        while p.r > query.dest.r {
            path.push(Dir::Up);
            p.r -= 1;
        }
        while p.r < query.dest.r {
            path.push(Dir::Down);
            p.r += 1;
        }
        while p.c > query.dest.c {
            path.push(Dir::Left);
            p.c -= 1;
        }
        while p.c < query.dest.c {
            path.push(Dir::Right);
            p.c += 1;
        }
        env.do_answer(path);
    }
}
