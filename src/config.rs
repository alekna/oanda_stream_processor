use std::env;
use std::error::Error;

#[derive(Debug, Clone)] // Added Clone derive here
pub struct Config {
    pub auth_token: String,
    pub account_id: String,
    pub environment: String,
    pub instruments: String,
    pub zmq_address: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let auth_token = env::var("OANDA_AUTH_TOKEN")
            .map_err(|_| "OANDA_AUTH_TOKEN environment variable not set")?;
        let account_id = env::var("OANDA_ACCOUNT_ID")
            .map_err(|_| "OANDA_ACCOUNT_ID environment variable not set")?;
        let environment = env::var("OANDA_ENVIRONMENT")
            .map_err(|_| "OANDA_ENVIRONMENT environment variable not set. Use 'fxtrade' or 'fxpractice'.")?;
        let instruments = env::var("OANDA_INSTRUMENTS")
            .map_err(|_| "OANDA_INSTRUMENTS environment variable not set (e.g., EUR_USD,USD_CAD)")?;

        let zmq_address = env::var("ZMQ_PUBLISHER_ADDRESS")
            .unwrap_or_else(|_| "tcp://*:9500".to_string());

        Ok(Config {
            auth_token,
            account_id,
            environment,
            instruments,
            zmq_address,
        })
    }

    pub fn base_url(&self) -> String {
        format!("https://stream-{}.oanda.com", self.environment)
    }
}
