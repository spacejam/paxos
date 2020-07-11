use std::{collections::HashMap, io, net::SocketAddr};

use rand::{seq::SliceRandom, thread_rng, Rng};
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

const N_SERVERS: usize = 5;
const N_CLIENTS: usize = 15;
const N_REQUESTS: usize = 40;

type From = usize;
type To = usize;
type Id = u64;
type Success = bool;

#[derive(Debug)]
struct Request {
    from: SocketAddr,
    message: Message,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
struct Ballot {
    ts: u64,
    uuid: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
enum Message {
    // request ID and proposed ID
    Request {
        uuid: Uuid,
        id: Id,
    },

    // proposal accepted?, request ID, server's highest known ID
    Response {
        success: Success,
        uuid: Uuid,
        id: Id,
    },
}

#[derive(Debug)]
enum Node {
    Server(Server),
    Client(Client),
}

impl Node {
    fn receive(
        &mut self,
        from: From,
        message: Message,
    ) -> Vec<(To, Message)> {
        match (self, message) {
            (
                Node::Server(server),
                Message::Request { uuid, id },
            ) => server.propose(from, uuid, id),
            (
                Node::Client(client),
                Message::Response { success, uuid, id },
            ) => client.receive(from, success, uuid, id),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Default)]
struct Server {
    max_id: u64,
}

impl Server {
    fn propose(
        &mut self,
        from: From,
        uuid: Uuid,
        id: Id,
    ) -> Vec<(To, Message)> {
        if id > self.max_id {
            self.max_id = id;
            return vec![(
                from,
                Message::Response {
                    success: true,
                    uuid,
                    id,
                },
            )];
        }
        vec![(
            from,
            Message::Response {
                success: false,
                uuid,
                id: self.max_id,
            },
        )]
    }
}

#[derive(Debug, Default)]
struct Client {
    last_id: Id,
    total_successes: Vec<Id>,
    total_retries: usize,

    // in-flight request ID
    current_uuid: Uuid,
    received_successes: usize,
    received_failures: Vec<Id>,
}

impl Client {
    fn generate_requests(&mut self) -> Vec<(To, Message)> {
        let mut ret = vec![];

        let new_uuid = Uuid::new_v4();
        self.current_uuid = new_uuid;
        self.received_successes = 0;
        self.received_failures.clear();

        for id in 0..N_SERVERS {
            ret.push((
                id,
                Message::Request {
                    uuid: new_uuid,
                    id: self.last_id + 1,
                },
            ))
        }

        ret
    }

    fn receive(
        &mut self,
        _from: From,
        success: Success,
        uuid: Uuid,
        id: Id,
    ) -> Vec<(To, Message)> {
        if uuid != self.current_uuid {
            return vec![];
        }

        if success {
            assert_eq!(id, self.last_id + 1);
            self.received_successes += 1;

            if self.received_successes > N_SERVERS / 2 {
                assert!(self.last_id < id);
                self.last_id = id;
                // println!("SUCCESS; ID = {}", id);

                self.total_successes.push(id);

                if self.total_successes.len() < N_REQUESTS {
                    return self.generate_requests();
                } else {
                    println!(
                        "finished, ids: {:?} total retries: {}",
                        self.total_successes, self.total_retries
                    );

                    for xs in
                        self.total_successes.windows(2)
                    {
                        let (x1, x2) = (xs[0], xs[1]);
                        assert!(x1 < x2);
                    }
                    // break receiving uuid
                    self.current_uuid = Uuid::new_v4();
                }
            }
        } else {
            self.received_failures.push(id);

            if self.received_failures.len() > N_SERVERS / 2
            {
                self.last_id = id;
                self.total_retries += 1;
                // println!("FAILURE; ID = {}", id);
                return self.generate_requests();
            }
        }

        vec![]
    }
}

fn main() {
    // fake cluster
    let mut in_flight: Vec<(From, To, Message)> = vec![];
    let mut computers = vec![];

    for _ in 0..N_SERVERS {
        computers.push(Node::Server(Server::default()));
    }
    for _ in 0..N_CLIENTS {
        computers.push(Node::Client(Client::default()));
    }

    // seed initial requests
    for sender in N_SERVERS..N_SERVERS + N_CLIENTS {
        let client = if let Node::Client(client) =
            &mut computers[sender]
        {
            client
        } else {
            unreachable!()
        };

        let outbound = client.generate_requests();

        for (to, message) in outbound {
            in_flight.push((sender, to, message));
        }
    }

    loop {
        if in_flight.is_empty() {
            return;
        }

        let (from, to, message) = in_flight.pop().unwrap();

        // println!("from={} to={} message={:?}", from, to, message);
        let outbound = computers[to].receive(from, message);

        let mut rng = thread_rng();
        for (destination, message) in outbound {
            if rng.gen_ratio(1, 10) {
                // just drop the outbound message
                // simulates loss
                // continue;
            }
            in_flight.push((to, destination, message));
        }

        // chaos
        in_flight.shuffle(&mut rng);
    }
}
