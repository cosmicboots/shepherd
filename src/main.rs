use std::env;
use shepherd::State;
use shepherd::config::Config;

fn main() {
    let args = env::args();
    let state = State::new(args);
    let mut config = Config::new();
    config.read_config(&state.config).unwrap();
    if let Err(e) = shepherd::run(state, config) {
        eprintln!("{}", e);
    }
}
