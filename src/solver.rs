use crate::models::*;

pub fn run_solver<E: Environment>(env: &mut E) {
    while let Some(_query) = env.next_query() {
        env.do_answer(Vec::new());
    }
}
