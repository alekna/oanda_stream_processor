use serde::{Deserialize, Deserializer};
use std::str::FromStr;

fn deserialize_string_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    u64::from_str(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PriceLevel {
    pub price: String,
    #[serde(deserialize_with = "deserialize_string_to_u64")]
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

impl StreamMessage {
    pub fn get_type(&self) -> &str {
        match self {
            StreamMessage::PriceTick(_) => "PRICE_TICK",
            StreamMessage::Heartbeat(_) => "HEARTBEAT",
            StreamMessage::Unknown(_) => "UNKNOWN",
        }
    }
}
