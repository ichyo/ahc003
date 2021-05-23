use clap::Clap;
use env_logger::Env;
use log::info;
use spq::simulator::Simulator;
use spq::solver::run_solver;
use std::sync::mpsc;
use threadpool::ThreadPool;

/// Evaluate solver by multiple test cases
#[derive(Clap, Debug)]
#[clap(name = "hello")]
struct Arguments {
    /// the number of test cases
    #[clap(short, long, default_value = "100")]
    num: u64,

    /// concurrency level
    #[clap(short, long, default_value = "5")]
    concurrency: usize,
}

fn mean(data: &[f64]) -> f64 {
    let sum = data.iter().sum::<f64>();
    let count = data.len();
    sum / count as f64
}

fn std_deviation(data: &[f64]) -> f64 {
    let data_mean = mean(data);
    let count = data.len();

    let variance = data
        .iter()
        .map(|value| {
            let diff = data_mean - value;
            diff * diff
        })
        .sum::<f64>()
        / count as f64;

    variance.sqrt()
}

fn main() -> Result<(), mpsc::RecvError> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Arguments::parse();
    let pool = ThreadPool::new(args.concurrency);

    let (tx, rx) = mpsc::channel();

    for seed in 0..args.num {
        let tx = tx.clone();
        pool.execute(move || {
            let mut simulator = Simulator::from_seed(seed);
            run_solver(&mut simulator);
            tx.send((seed, simulator)).expect("failed to send");
        });
    }

    let mut ratio_scores = Vec::new();

    for _ in 0..args.num {
        let (seed, simulator) = rx.recv()?;
        info!("seed={:4} score={:.4}", seed, simulator.ratio_score());
        ratio_scores.push(simulator.ratio_score());
    }

    info!("mean: {:.6}", mean(&ratio_scores));
    info!("sd:   {:.6}", std_deviation(&ratio_scores));

    Ok(())
}
