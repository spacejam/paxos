use std::{
    io,
    net::{ToSocketAddrs, UdpSocket},
};

use bincode::{deserialize, serialize};

use crate::{Reactor, Transport};

/// A transport that uses UDP and bincode for sending messages
pub struct UdpTransport {
    socket: UdpSocket,
}

impl UdpTransport {
    /// Create a new UdpTransport
    pub fn new<A: ToSocketAddrs>(addr: A) -> io::Result<UdpTransport> {
        let socket = UdpSocket::bind(addr)?;
        Ok(UdpTransport { socket })
    }
}

impl<R: Reactor<Peer = String>> Transport<R> for UdpTransport {
    /// Blocks until the next message is received.
    fn next_message(&mut self) -> (R::Peer, R::Message) {
        let mut buf = [0; 64 * 1024];
        loop {
            let (n, from) = self.socket.recv_from(&mut buf).unwrap();
            let msg: R::Message = deserialize(&buf[..n]).unwrap();
            return (from.to_string(), msg);
        }
    }

    /// Enqueues the message to be sent. May be sent 0-N times with no ordering guarantees.
    fn send_message(&mut self, to: R::Peer, msg: R::Message) {
        let serialized = serialize(&msg).unwrap();
        let n = self.socket.send_to(&serialized, to).unwrap();
        assert_eq!(n, serialized.len());
    }
}
