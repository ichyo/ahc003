use spq::remote::RemoteEnvironment;
use spq::solver::run_solver;

fn main() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let stdin = stdin.lock();
    let stdout = stdout.lock();
    let mut env = RemoteEnvironment::new(stdin, stdout);
    run_solver(&mut env);
}
