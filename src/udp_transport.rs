use std::{
    convert::TryInto,
    io,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
};

use bincode::{deserialize, serialize};
use crc32fast::Hasher;
use serde::{de::DeserializeOwned, Serialize};

const MAX_SZ: usize = 64 * 1024;

/// A transport that uses UDP and bincode for sending messages
pub struct UdpTransport {
    socket: UdpSocket,
}

impl UdpTransport {
    /// Create a new UdpTransport
    pub fn new<A: ToSocketAddrs>(
        addr: A,
    ) -> io::Result<UdpTransport> {
        let socket = UdpSocket::bind(addr)?;
        Ok(UdpTransport { socket })
    }
}

impl UdpTransport {
    /// Blocks until the next message is received.
    fn next_message<M: DeserializeOwned>(
        &mut self,
    ) -> (SocketAddr, M) {
        let mut buf = [0; MAX_SZ];
        let (n, from) =
            self.socket.recv_from(&mut buf).unwrap();

        let crc_sz = std::mem::size_of::<u32>();
        let data_buf = &buf[..n - crc_sz];
        let crc_buf = &buf[n - crc_sz..];

        let mut hasher = Hasher::new();
        hasher.update(&data_buf);
        let hash = hasher.finalize();

        let crc_array: [u8; 4] =
            crc_buf.try_into().unwrap();
        assert_eq!(u32::from_le_bytes(crc_array), hash);

        let msg: M = deserialize(&buf[..n]).unwrap();
        (from, msg)
    }

    /// Enqueues the message to be sent. May be sent 0-N times with no ordering guarantees.
    fn send_message<M: Serialize>(
        &mut self,
        to: SocketAddr,
        msg: M,
    ) {
        let mut serialized = serialize(&msg).unwrap();
        let mut hasher = Hasher::new();
        hasher.update(&serialized);
        let hash = hasher.finalize();
        serialized.copy_from_slice(&hash.to_le_bytes());
        assert!(serialized.len() <= MAX_SZ);

        let n =
            self.socket.send_to(&serialized, to).unwrap();
        assert_eq!(n, serialized.len());
    }
}
