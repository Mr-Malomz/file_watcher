mod command;
mod provider;
mod watcher;
use command::cli::run_cli;

fn main() {
    let _ = run_cli();
}
