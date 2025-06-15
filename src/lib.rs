pub mod config;
pub mod models;
pub mod oanda_client;
pub mod publisher;

#[allow(clippy::all)]
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/oanda_stream_processor.rs"));
}
