use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
/// The internal representation of the configuration file.
pub struct Config {
    #[serde(skip)] // Skip serializing the configration file location
    config_location: String,
    pub source_dir: String,
    pub repositories: Vec<Repository>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
/// The internal representation of a single repository
pub struct Repository {
    pub name: String,
    pub url: String,
}

impl Repository {
    pub fn new(url: String) -> Repository {
        Repository {
            name: url.clone(),
            url,
        }
    }
}

impl Config {
    /// Create new Config struct with default values
    ///
    /// The `source_dir` field defaults to `$HOME/sources`
    pub fn new() -> Config {
        Config {
            source_dir: format!("{}/sources", env::var("HOME").unwrap()),
            repositories: vec![],
            config_location: String::new(),
        }
    }

    /// Read TOML file and load values into a Config struct
    ///
    /// If the filename doesn't exist, `read_config()` will write the current struct to the given
    /// config file.
    pub fn read_config(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
        // Load raw yaml file. If the file doesn't exist, create it
        if std::path::Path::new(filename).exists() {
            let raw = fs::read_to_string(filename)?;
            // Convert string to our Config struct
            let toml: Config = toml::from_str(&raw)?;
            *self = toml;
        } else {
            let config: String = toml::to_string(&self)?;
            // Get the path to the config file
            let path = match Path::new(filename).parent() {
                Some(x) => x,
                _ => Path::new(filename),
            };
            // Make sure the path exists
            fs::create_dir_all(path).unwrap();
            // Write default Config struct to file
            fs::write(filename, config).expect("Couldn't write file");
        }
        self.config_location = filename.to_string();
        Ok(())
    }

    /// Save the current configuration to a file.
    ///
    /// This has to be called after `read_config()`.
    pub fn save_config(&self) -> Result<(), Box<dyn Error>> {
        fs::write(&self.config_location, toml::to_string(&self)?)?;
        Ok(())
    }

    /// Create a string of the current loaded config.
    ///
    /// Useful for dumping the config
    pub fn to_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}
