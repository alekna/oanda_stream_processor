use serde::{Deserialize, Deserializer};
use std::str::FromStr;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PriceLevel {
    pub price: String,
    pub liquidity: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PriceTick {
    pub asks: Vec<PriceLevel>,
    pub bids: Vec<PriceLevel>,
    pub closeout_ask: String,
    pub closeout_bid: String,
    pub instrument: String,
    pub status: String,
    pub time: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Heartbeat {
    pub time: String,
    #[serde(rename = "type")]
    pub message_type: String,
}

#[derive(Debug, Clone)]
pub enum StreamMessage {
    PriceTick(PriceTick),
    Heartbeat(Heartbeat),
    Unknown(serde_json::Value),
}
