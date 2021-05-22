use spq::simulator::Simulator;
use spq::solver::run_solver;

fn main() {
    let seed = match std::env::args().nth(1) {
        Some(seed) => seed.parse::<u64>().unwrap(),
        None => panic!("Usage: cargo run --bin tryout <seed>"),
    };
    let mut simulator = Simulator::from_seed(seed);
    run_solver(&mut simulator);
    println!("score: {}", simulator.raw_score());
    println!("atcoder: {}", simulator.atcoder_score() * 100);
}
