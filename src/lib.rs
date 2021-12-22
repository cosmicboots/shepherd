pub mod config;
use config::{Config, Repository};
use std::env;
use std::error::Error;
use std::fs;
use std::process::Command;

#[derive(Debug)]
/// Holds the current state of the application.
///
/// This is used when processing command line arguments, and is thus loaded before the
/// configuration file is.
pub struct State {
    cmd: Option<Cmd>,
    url: Option<String>,
    name: Option<String>,
    category: Option<String>,
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
    ///
    /// The command argument is deterministic in the sense that it doesn't change once it's set. If
    /// no command has been given by the end of the parsing loop, it defaults to the help command
    pub fn new(mut args: std::env::Args) -> State {
        let mut state = State {
            cmd: None,
            url: None,
            name: None,
            category: None,
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
                    let next = args.next();
                    let name: Option<String>;
                    match next {
                        Some(ref x) => match &x[..] {
                            "--category" | "-c" => {
                                state.category = args.next();
                                name = args.next();
                            }
                            _ => name = next,
                        },
                        None => name = None,
                    }
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
/// This is the main run function for the program.
///
/// It starts by interpreting the state of the program as set by the argument parsing.
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
                match state.category {
                    Some(x) => Some(x),
                    _ => None,
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
            let name_width: usize = match config
                .repositories
                .iter()
                .max_by(|x, y| x.name.len().cmp(&y.name.len()))
            {
                Some(x) => x.name.len(),
                None => 0,
            };
            let url_width: usize = match config
                .repositories
                .iter()
                .max_by(|x, y| x.url.len().cmp(&y.url.len()))
            {
                Some(x) => x.url.len(),
                None => 0,
            };
            println!(
                "{:name$} {:url$} Category",
                "Name",
                "URL",
                name = name_width,
                url = url_width
            );
            for repo in config.repositories.iter() {
                println!(
                    "{name:name_w$} {url:url_w$} {cat}",
                    name = repo.name,
                    name_w = name_width,
                    url = repo.url,
                    url_w = url_width,
                    cat = match &repo.category {
                        Some(x) => x,
                        None => "",
                    },
                );
            }
        }
        Some(Cmd::Fetch) => {
            fetch_repos(config);
        }
        Some(x) => {
            println!("{:?} hasn't been implemented yet!", x)
        }
        _ => {}
    }
    Ok(())
}

fn fetch_repos(config: Config) {
    // Make sure sources directory exists
    match fs::create_dir_all(&config.source_dir) {
        Err(e) => eprintln!("{}", e),
        _ => {}
    }

    // Loop through the repositories
    for repo in config.repositories.iter() {
        // Get the full path to the repository
        let path: String = match &repo.category {
            Some(x) => format!("{}/{}/", config.source_dir, x),
            None => config.source_dir.clone(),
        };
        let fullpath: String = format!("{}/{}", path, repo.name);
        // Check to see if the folder is already cloned, if so, just fetch
        if std::path::Path::new(&fullpath).is_dir() {
            println!("=== FETCHING {} ===", repo.name);
            Command::new("git")
                .args(["-C", &fullpath, "fetch", "--all"])
                .status()
                .unwrap();
        } else {
            println!("=== {} doesn't exist locally ===", repo.name);
            println!("=== CLONING {} ===", repo.name);
            match fs::create_dir_all(&path) {
                Err(e) => eprintln!("{}", e),
                _ => {}
            }
            println!("{}", path);
            Command::new("git")
                .args(["-C", &path, "clone", &repo.url, &repo.name])
                .status()
                .unwrap();
        }
    }
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
    -c, --category  Specify the category when adding a repository

COMMANDS:
General
    help    Print out this help message

Manage Repositories
    add     Add another git repo to keep track of
    fetch   Update currently tracked repos
    list    list out the currently tracked repos"
    )
}
