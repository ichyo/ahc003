use log::info;
use std::fs::File;
use std::io::Write;

use clap::Clap;
use spq::models::*;
use spq::simulator::Simulator;
use spq::solver::run_solver;

use env_logger::Env;

/// Try out a single test case
#[derive(Clap, Debug)]
#[clap(name = "hello")]
struct Arguments {
    /// Seed to generate test case
    #[clap(short, long)]
    seed: u64,
    /// Output file for visualizer
    #[clap(short, long)]
    output: Option<String>,
}

struct TryoutEnvironment(Simulator, Option<File>);

impl Environment for TryoutEnvironment {
    fn next_query(&self) -> Option<Query> {
        self.0.next_query()
    }
    fn do_answer(&mut self, path: &[Dir]) -> u32 {
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
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let args = Arguments::parse();

    let file = args.output.map(|s| File::create(s).unwrap());

    let mut env = TryoutEnvironment(Simulator::from_seed(args.seed), file);
    run_solver(&mut env);

    let simulator = &env.0;

    info!("raw_score  : {:.4}", simulator.raw_score());
    info!("ratio_score: {:.6}", simulator.ratio_score());
}
