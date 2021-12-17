use std::env;
use shepherd::State;
use shepherd::config::Config;

fn main() {
    let args = env::args();
    let state = State::new(args);
    let mut config = Config::new();
    if let Err(e) = config.read_config(&state.config) {
        eprintln!("Configuration Error: {}", e);
        std::process::exit(1);
    }
    if let Err(e) = shepherd::run(state, config) {
        eprintln!("{}", e);
    }
}
