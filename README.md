## README
---
This is a Rust Narwhal and Tusk consensus implementation for my Blockchain ecosystems assignment #3 as part of understanding the Aptos Blockchain. The original paper can be found here: https://arxiv.org/pdf/2105.11827

Narwhal and Tusk is a Byzantine fault tolerant consensus mechanism.
- Narwhal is a mempool protocol for collecting transaction batches and builds a DAG of certificates
- Tusk is a consensus layer that allows the nodes in the network to order the DAG blocks

## Background
The original blockchain suffers from throughput and scalability due to tradeoffs for security. In short, the following happens:
1. Transactions are collected
2. THey are compiled and put into blocks
3. Validators confirm they make sense and are correct with other nodes
4. The block is then written to the chain

## Goal
In Narwhal and Tusk we want to separate the process of collection and validation from workers. That way we can enable parallel instead of sequential processing to reduce bottlenecks and increase throughput

For Narwhal:
1. Nodes now do consistent broadcast blocks instead of transactions
2. Propose certificates for multiple blocks (blocks include past certificates from all validators)
3. Restrict block creation rate (we have rounds now)
4. Scale out (workers can focus on validation and creating blocks)

Tusk:
1. We will read the DAG (Created by Narwhal) and establish an order
2. Each validator has their own local DAG and combine them
4. Every 3 rounds (1 wave) we can look back and elect a leader
5. Votes are the 'edges' of the DAG (we need f+1 votes)
6. Waves build upon other waves

## Running simulation
We can trigger with cargo run for a straightforward 4 node comparison
