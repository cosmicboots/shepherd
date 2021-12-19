pub mod config;
use config::{Config, Repository};
use std::env;
use std::error::Error;

#[derive(Debug)]
/// Holds the current state of the application.
///
/// This is used when processing command line arguments, and is thus loaded before the
/// configuration file is.
pub struct State {
    cmd: Option<Cmd>,
    url: Option<String>,
    name: Option<String>,
    pub config: String,
}

#[derive(Debug)]
/// Used to signal which main command is going to be run
enum Cmd {
    Add,
    Help,
    Fetch,
    List,
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
            name: None,
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
            } else if let None = state.cmd {
                // add command
                if x == "add" {
                    let name = args.next();
                    match name {
                        Some(x) => {
                            state.name = Some(x);
                            let url = args.next();
                            match url {
                                Some(x) => {
                                    state.cmd = Some(Cmd::Add);
                                    state.url = Some(x);
                                }
                                None => {
                                    eprintln!("No URL provided for command\n");
                                }
                            }
                        }
                        None => eprintln!("No Name provided for command\n"),
                    }
                }
                // fetch command
                else if x == "fetch" {
                    match state.cmd {
                        None => {
                            state.cmd = Some(Cmd::Fetch);
                        }
                        _ => {}
                    }
                }
                // list command
                else if x == "list" {
                    match state.cmd {
                        None => {
                            state.cmd = Some(Cmd::List);
                        }
                        _ => {}
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

#[allow(unreachable_patterns)]
pub fn run(state: State, mut config: Config) -> Result<(), Box<dyn Error>> {
    match state.cmd {
        Some(Cmd::Help) => {
            println!("{}", help_msg());
        }
        Some(Cmd::DumpConfig) => {
            println!("{}", config.to_string().unwrap());
        }
        Some(Cmd::Add) => {
            let repo = Repository::new(
                match state.name {
                    Some(x) => x,
                    _ => String::new(),
                },
                match state.url {
                    Some(x) => x,
                    _ => String::new(),
                },
            );
            if let None = config.repositories.iter().find(|x| **x == repo) {
                config.repositories.push(repo);
                config.save_config()?;
                println!("Repository has been added");
            } else {
                println!("Repository is already being tracked");
            }
        }
        Some(Cmd::List) => {
            // Get the largest entry by name
            let width: usize = match config
                .repositories
                .iter()
                .max_by(|x, y| x.name.len().cmp(&y.name.len()))
            {
                Some(x) => x.name.len(),
                None => 0,
            };
            println!("{:width$} URL", "Name", width = width);
            for repo in config.repositories.iter() {
                println!("{:width$} {}", repo.name, repo.url, width = width);
            }
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
    --config        Specify the location of the configuration file
    --dump-config   Dump the current configuration

COMMANDS:
General
    help    Print out this help message

Manage Repositories
    add     Add another git repo to keep track of
    fetch   Update currently tracked repos"
    )
}
