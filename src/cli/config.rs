extern crate toml;

use std::fs::File;
use std::io::{self, BufRead, Read, Write};

use error::CLIError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub live: Option<EndpointConfig>,
    pub staging: Option<EndpointConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EndpointConfig {
    pub key: String,
    pub secret: String,
}

impl Config {
    pub fn parse_config(path: &str) -> Result<Config, CLIError> {
        let mut config_toml = String::new();

        File::open(path)
            .map_err(CLIError::ConfigStorageFailure)
            .and_then(|mut file| {
                file.read_to_string(&mut config_toml).unwrap_or_else(
                    |err| {
                        panic!("Error while reading config: [{}]", err)
                    },
                );

                toml::from_str(&config_toml).or(Err(CLIError::InvalidConfig))
            })
    }

    pub fn create(path: &str) -> Result<Config, CLIError> {
        let new_config = Config::generate();
        new_config.store(path).and_then(|_| Ok(new_config))
    }

    fn generate() -> Config {
        println!("\x1B[1mconfig.toml Generator\x1B[0m");
        Config {
            live: Some(EndpointConfig {
                key: Config::prompt_for_input("Production API Key: "),
                secret: Config::prompt_for_input("Production API Secret: "),
            }),
            staging: Some(EndpointConfig {
                key: Config::prompt_for_input("Staging API Key: "),
                secret: Config::prompt_for_input("Staging API Secret: "),
            }),
        }
    }

    fn store(&self, path: &str) -> Result<(), CLIError> {
        let mut file = File::create(path)?;
        file.write_all(&toml::to_string(&self)
            .expect("Failed to parse config")
            .into_bytes())
            .map_err(CLIError::ConfigStorageFailure)
    }

    fn prompt_for_input(prompt: &str) -> String {
        print!("{}", prompt);
        io::stdout().flush();
        let stdin = io::stdin();
        let input = stdin
            .lock()
            .lines()
            .next()
            .expect("Input could not be found")
            .expect("Input could not be read");

        input
    }
}
