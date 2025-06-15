use std::error::Error;
use zmq::{Context, Socket, SocketType};
use prost::Message;
use crate::proto::StreamMessageProto;

pub struct ZmqPublisher {
    socket: Socket,
}

impl ZmqPublisher {
    pub fn new(address: &str) -> Result<Self, Box<dyn Error>> {
        let context = Context::new();
        let socket = context.socket(SocketType::PUB)?;
        socket.bind(address)?;
        println!("ZMQ Publisher bound to {}", address);
        Ok(ZmqPublisher { socket })
    }

    pub fn publish(&self, message_proto: &StreamMessageProto) -> Result<(), Box<dyn Error>> {
        let mut buf = Vec::new();
        message_proto.encode(&mut buf)?;
        self.socket.send(&buf, 0)?;
        Ok(())
    }
}
