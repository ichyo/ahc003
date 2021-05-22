use std::fs::File;
use std::io::Write;

use clap::Clap;
use spq::models::*;
use spq::simulator::Simulator;
use spq::solver::run_solver;

/// Try out a single test case
#[derive(Clap, Debug)]
#[clap(name = "hello")]
struct Arguments {
    /// Seed to generate test case
    #[clap(short, long)]
    seed: u64,
    /// Seed to generate test case
    #[clap(short, long)]
    output: Option<String>,
}

struct TryoutEnvironment(Simulator, Option<File>);

impl Environment for TryoutEnvironment {
    fn next_query(&self) -> Option<Query> {
        self.0.next_query()
    }
    fn do_answer(&mut self, path: &[Dir]) -> f64 {
        if let Some(f) = &mut self.1 {
            writeln!(
                f,
                "{}",
                path.iter().map(|d| d.to_char()).collect::<String>()
            )
            .expect("write failed");
        }
        self.0.do_answer(&path)
    }
}

fn main() {
    let args = Arguments::parse();

    let file = args.output.map(|s| File::create(s).unwrap());

    let mut env = TryoutEnvironment(Simulator::from_seed(args.seed), file);
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
