pub mod config;
use config::Config;
use std::error::Error;
use std::env;

#[derive(Debug)]
/// Holds the current state of the application.
///
/// This is used when processing command line arguments, and is thus loaded before the
/// configuration file is.
pub struct State {
    cmd: Option<Cmd>,
    url: Option<String>,
    pub config: String,
}

#[derive(Debug)]
/// Used to signal which main command is going to be run
enum Cmd {
    Fetch,
    Help,
    DumpConfig,
}

impl State {
    /// Parses command line flags/arguments and creates the application state.
    ///
    /// Argument parsing goes left to right. The first not flag argument it comes across is set as
    /// the cmd.
    ///
    /// Parsing priority is as follows:
    /// - `--long-flag` type arguments
    /// - `-mcsf` multi character short flag type arguments
    /// - `command` type arguments
    pub fn new(mut args: std::env::Args) -> State {
        let mut state = State {
            cmd: None,
            url: None,
            config: format!("{}/.config/shepherd/config.toml", env::var("HOME").unwrap()),
        };

        let mut arg = args.next();
        // Iterate through the command line arguments
        while let Some(x) = &arg {
            // Long arguments
            if x.starts_with("--") {
                let option = x.strip_prefix("--").unwrap();
                match option {
                    "help" => state.cmd = Some(Cmd::Help),
                    "dump-config" => state.cmd = Some(Cmd::DumpConfig),
                    "config" => {
                        let file = args.next();
                        match file {
                            Some(x) => state.config = x,
                            None => eprintln!("Expected config argument"),
                        }
                    }
                    _ => {}
                }
            }
            // short arguments
            else if x.starts_with("-") {
                let options = x.strip_prefix("-").unwrap().chars();
                for opt in options {
                    match opt {
                        'h' => state.cmd = Some(Cmd::Help),
                        _ => {}
                    }
                }
            }
            // Fetch command
            else if x == "fetch" {
                let url = args.next();
                match url {
                    Some(x) => match state.cmd {
                        None => {
                            state.cmd = Some(Cmd::Fetch);
                            state.url = Some(x);
                        }
                        _ => {}
                    },
                    None => {
                        eprintln!("No URL provided for \'number\' command\n");
                    }
                }
            }
            arg = args.next();
        }

        // Fall back to Help command if no other command was given
        state.cmd = match state.cmd {
            Some(x) => Some(x),
            None => Some(Cmd::Help),
        };
        state
    }
}

pub fn run(state: State, config: Config) -> Result<(), Box<dyn Error>> {
    match state.cmd {
        Some(Cmd::Help) => {
            println!("{}", help_msg());
        }
        Some(Cmd::DumpConfig) => {
            println!("{}", config.to_string().unwrap());
        }
        Some(x) => {
            println!("{:?} hasn't been implemented yet!", x)
        }
        _ => {}
    }
    Ok(())
}

fn help_msg() -> String {
    format!(
        "Git repository manager

USAGE:
    shepherd [--help] <command> [<args>]

OPTIONS:
    -h, --help      Print out this help message
    --dump-config   dump the current configuration

COMMANDS:
General
    help    Print out this help message

Manage Repositories
    clone   Add another git repo to keep track of
    fetch   Update currently tracked repos"
    )
}
