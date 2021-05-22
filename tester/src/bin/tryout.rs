use clap::Clap;
use spq::models::*;
use spq::simulator::Simulator;
use spq::solver::run_solver;

/// Try out a single test case
#[derive(Clap, Debug)]
#[clap(name = "hello")]
struct Tryout {
    /// Seed to generate test case
    #[clap(short, long)]
    seed: u64,
}

struct TryoutEnvironment(Simulator);

impl Environment for TryoutEnvironment {
    fn next_query(&self) -> Option<Query> {
        self.0.next_query()
    }
    fn do_answer(&mut self, path: Vec<Dir>) -> f64 {
        eprintln!("{}", path.iter().map(|d| d.to_char()).collect::<String>());
        self.0.do_answer(path)
    }
}

fn main() {
    let tryout = Tryout::parse();

    let mut env = TryoutEnvironment(Simulator::from_seed(tryout.seed));
    run_solver(&mut env);

    let simulator = &env.0;

    for i in 0..NUM_TURN {
        println!(
            "width: {:2}, height: {:2} -> best: {:6}, length: {:6}, ratio: {:.3}",
            simulator.queries()[i].query.width(),
            simulator.queries()[i].query.height(),
            simulator.score_details()[i].best,
            simulator.score_details()[i].length,
            simulator.score_details()[i].ratio(),
        );
    }

    println!("score: {}", simulator.raw_score());
    println!("atcoder: {}", simulator.atcoder_score() * 100);
}
