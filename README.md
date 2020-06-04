# paxos

[![crates.io](https://meritbadge.herokuapp.com/paxos)](https://crates.io/crates/paxos)
[![documentation](https://docs.rs/paxos/badge.svg)](https://docs.rs/paxos)
[![chat](https://img.shields.io/discord/509773073294295082.svg?logo=discord)](https://discord.gg/Z6VsXds)
[![sponsors](https://img.shields.io/opencollective/backers/sled)](https://github.com/sponsors/spacejam)

Currently, this is an implementation of
[CASPaxos](https://arxiv.org/pdf/1802.07000.pdf)
built on top of the sled lightweight database kit.

It is being grown into a more featureful replication
library that is mindful of modern consensus research.

# roadmap

- [ ] membership reconfiguration
- [ ] cheap-paxos storage reduction
- [ ] gossip-based replication of state
- [ ] log reclamation to avoid infinite space usage
- [ ] read-only followers that don't slow down writes

# References

* [A Generalised Solution to Distributed Consensus](https://arxiv.org/abs/1902.06776)
* [Cheap Paxos](https://lamport.azurewebsites.net/pubs/web-dsn-submission.pdf)
* [Edelweiss: Automatic Storage Reclamation for Distributed Programming](http://www.neilconway.org/docs/vldb2014_edelweiss.pdf)

