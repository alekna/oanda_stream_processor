use tokio::io::{AsyncBufReadExt, BufReader};
use reqwest::Client;
use crate::config::Config;
use crate::models::{StreamMessage, PriceTick, Heartbeat};
use tokio::sync::mpsc;
use std::error::Error;
use futures::TryStreamExt;
use tracing::{info, error, warn}; // Import tracing macros for this file too

pub async fn connect_to_stream(
    config: &Config,
    tx: mpsc::Sender<StreamMessage>,
) -> Result<(), Box<dyn Error>> {
    let url = format!(
        "{}/v3/accounts/{}/pricing/stream?instruments={}",
        config.base_url(),
        config.account_id,
        urlencoding::encode(&config.instruments)
    );

    info!("Connecting to OANDA stream at: {}", url); // Changed from println! to info!

    let client = Client::new();

    let response = client
        .get(&url)
        .bearer_auth(&config.auth_token)
        .send()
        .await?
        .error_for_status()?;

    info!("Successfully connected to OANDA stream. Reading data..."); // Changed from println! to info!

    let stream_reader = tokio_util::io::StreamReader::new(
        response.bytes_stream()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    );

    let mut reader = BufReader::new(stream_reader);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                info!("Stream closed by server."); // Changed from println! to info!
                break;
            },
            Ok(_) => {
                let trimmed_line = line.trim();
                if trimmed_line.is_empty() {
                    continue;
                }

                let raw_json: serde_json::Value = match serde_json::from_str(trimmed_line) {
                    Ok(val) => val,
                    Err(e) => {
                        error!("Error parsing JSON line: '{}' -> {}", trimmed_line, e); // Changed from eprintln! to error!
                        continue;
                    }
                };

                let message = if raw_json.get("type").and_then(|t| t.as_str()) == Some("HEARTBEAT") {
                    match serde_json::from_value::<Heartbeat>(raw_json.clone()) {
                        Ok(hb) => StreamMessage::Heartbeat(hb),
                        Err(e) => {
                            error!("Error deserializing Heartbeat: {} from {}", e, trimmed_line); // Changed from eprintln! to error!
                            StreamMessage::Unknown(raw_json)
                        }
                    }
                } else if raw_json.get("instrument").is_some() {
                    match serde_json::from_value::<PriceTick>(raw_json.clone()) {
                        Ok(pt) => StreamMessage::PriceTick(pt),
                        Err(e) => {
                            error!("Error deserializing PriceTick: {} from {}", e, trimmed_line); // Changed from eprintln! to error!
                            StreamMessage::Unknown(raw_json)
                        }
                    }
                } else {
                    warn!("Unknown message type or missing discriminator: {}", trimmed_line); // Changed from eprintln! to warn!
                    StreamMessage::Unknown(raw_json)
                };

                if let Err(e) = tx.send(message).await {
                    error!("Error sending message to channel: {}", e); // Changed from eprintln! to error!
                    break;
                }
            }
            Err(e) => {
                error!("Error reading from stream: {}", e); // Changed from eprintln! to error!
                break;
            }
        }
    }
    Ok(())
}
