use tokio::sync::mpsc;
use std::error::Error;
use prost_types::Timestamp as ProstTimestamp;
use chrono::DateTime;

mod config;
mod models;
mod oanda_client;
mod publisher;
mod proto {
    include!(concat!(env!("OUT_DIR"), "/oanda_stream_processor.rs"));
}

use proto::{PriceTickProto, HeartbeatProto, PriceLevelProto, StreamMessageProto};
use models::{StreamMessage, PriceTick, Heartbeat};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = match config::Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Configuration Error: {}", e);
            eprintln!("\nPlease ensure the following environment variables are set:");
            eprintln!("  OANDA_AUTH_TOKEN=<YOUR_TOKEN>");
            eprintln!("  OANDA_ACCOUNT_ID=<YOUR_ACCOUNT_ID>");
            eprintln!("  OANDA_ENVIRONMENT=fxtrade | fxpractice (e.g., 'fxpractice')");
            eprintln!("  OANDA_INSTRUMENTS=EUR_USD,USD_CAD (comma-separated list of instruments)");
            eprintln!("\nOptional:");
            eprintln!("  ZMQ_PUBLISHER_ADDRESS=tcp://*:9500 (default bind address for ZMQ)");
            std::process::exit(1);
        }
    };

    let publisher = publisher::ZmqPublisher::new(&config.zmq_address)?;

    let (tx, mut rx) = mpsc::channel::<StreamMessage>(100);

    let oanda_config = config.clone();
    tokio::spawn(async move {
        if let Err(e) = oanda_client::connect_to_stream(&oanda_config, tx).await {
            eprintln!("OANDA Stream Error: {}", e);
        }
    });

    while let Some(msg) = rx.recv().await {
        match msg {
            StreamMessage::PriceTick(pt) => {
                println!("[PRICE_TICK] {}: Ask {} / Bid {}", pt.instrument, pt.closeout_ask, pt.closeout_bid);

                let proto_msg = convert_price_tick_to_proto(pt)?;

                if let Err(e) = publisher.publish(&StreamMessageProto {
                    message_type: Some(proto::stream_message_proto::MessageType::PriceTick(proto_msg))
                }) {
                    eprintln!("Error publishing PriceTick via ZMQ: {}", e);
                }
            },
            StreamMessage::Heartbeat(hb) => {
                println!("[HEARTBEAT] Time: {}", hb.time);

                let proto_msg = convert_heartbeat_to_proto(hb)?;

                if let Err(e) = publisher.publish(&StreamMessageProto {
                    message_type: Some(proto::stream_message_proto::MessageType::Heartbeat(proto_msg))
                }) {
                    eprintln!("Error publishing Heartbeat via ZMQ: {}", e);
                }
            },
            StreamMessage::Unknown(val) => {
                eprintln!("[UNKNOWN_MESSAGE] Received unexpected message: {:?}", val);
            }
        }
    }

    Ok(())
}

fn parse_timestamp(time_str: &str) -> Result<ProstTimestamp, Box<dyn Error>> {
    let dt = chrono::DateTime::parse_from_rfc3339(time_str)
        .or_else(|_first_err| {
            chrono::DateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M:%S%.fZ")
        })
        .map_err(|e| format!("Failed to parse timestamp '{}': {}", time_str, e))?;

    Ok(ProstTimestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    })
}

fn convert_price_tick_to_proto(price_tick: PriceTick) -> Result<PriceTickProto, Box<dyn Error>> {
    let asks_proto: Vec<PriceLevelProto> = price_tick.asks.into_iter().map(|pl| PriceLevelProto {
        price: pl.price,
        liquidity: pl.liquidity,
    }).collect();

    let bids_proto: Vec<PriceLevelProto> = price_tick.bids.into_iter().map(|pl| PriceLevelProto {
        price: pl.price,
        liquidity: pl.liquidity,
    }).collect();

    Ok(PriceTickProto {
        asks: asks_proto,
        bids: bids_proto,
        closeout_ask: price_tick.closeout_ask,
        closeout_bid: price_tick.closeout_bid,
        instrument: price_tick.instrument,
        status: price_tick.status,
        time: Some(parse_timestamp(&price_tick.time)?),
    })
}

fn convert_heartbeat_to_proto(heartbeat: Heartbeat) -> Result<HeartbeatProto, Box<dyn Error>> {
    Ok(HeartbeatProto {
        time: Some(parse_timestamp(&heartbeat.time)?),
        r#type: heartbeat.message_type,
    })
}
