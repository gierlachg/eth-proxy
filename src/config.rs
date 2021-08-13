use std::{env, fs};

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};

const HOST_VARIABLE_NAME: &str = "HOST";
const PORT_VARIABLE_NAME: &str = "PORT";
const ETHERSCAN_DOMAIN_VARIABLE_NAME: &str = "ETHERSCAN_DOMAIN";
const ETHERSCAN_API_KEY_VARIABLE_NAME: &str = "ETHERSCAN_API_KEY";

const LOGGING_CONFIGURATION_FILE_NAME: &str = "log4rs.yml";

#[derive(Clone)]
pub(crate) struct Config {
    version: String,
    host: String,
    port: String,
    etherscan_domain: String,
    etherscan_api_key: String,
}

impl Config {
    pub(crate) fn init() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            host: env::var(HOST_VARIABLE_NAME).expect(&format!("Missing '{}' variable", HOST_VARIABLE_NAME)),
            port: env::var(PORT_VARIABLE_NAME).expect(&format!("Missing '{}' variable", PORT_VARIABLE_NAME)),
            etherscan_domain: env::var(ETHERSCAN_DOMAIN_VARIABLE_NAME)
                .expect(&format!("Missing '{}' variable", ETHERSCAN_DOMAIN_VARIABLE_NAME)),
            etherscan_api_key: env::var(ETHERSCAN_API_KEY_VARIABLE_NAME)
                .expect(&format!("Missing '{}' variable", ETHERSCAN_API_KEY_VARIABLE_NAME)),
        }
    }

    pub(crate) fn version(&self) -> &str {
        &self.version
    }

    pub(crate) fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub(crate) fn etherscan_domain(&self) -> &str {
        &self.etherscan_domain
    }

    pub(crate) fn etherscan_api_key(&self) -> &str {
        &self.etherscan_api_key
    }
}

pub(crate) struct Logging {}

impl Logging {
    pub(crate) fn init() -> Self {
        match fs::metadata(LOGGING_CONFIGURATION_FILE_NAME) {
            Ok(_) => log4rs::init_file(LOGGING_CONFIGURATION_FILE_NAME, Default::default()).unwrap(),
            Err(_) => {
                let _ = log4rs::init_config(
                    log4rs::Config::builder()
                        .appender(Appender::builder().build("stdout", Box::new(ConsoleAppender::builder().build())))
                        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
                        .unwrap(),
                );
            }
        }
        Self {}
    }
}
