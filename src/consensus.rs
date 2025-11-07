#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)] 

use std::{collections::{HashMap, HashSet}, path::Ancestors, vec};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{Block, Certificate, Hash, Transaction, ValidatorId, ValidatorSet, dag, types};
use crate::{dag::DAG};

#[derive(Default)]
pub struct ConsensusState {
    pub dag: DAG,
    pub current_round: u32,
    pub committed_blocks: HashSet<Hash>,
    pub certificates: HashMap<Hash, Certificate>,
}

#[derive(Clone)]
pub struct ConsensusHandle {
    state: Arc<RwLock<ConsensusState>>,
    validator_set: Arc<ValidatorSet>,
}

impl ConsensusHandle {
    pub fn new(validator_set: ValidatorSet) -> Self {
        Self {
            state: Arc::new(RwLock::new(ConsensusState::default())),
            validator_set: Arc::new(validator_set),
        }
    }
    /* Narwhal Methods:
        1. Block proposal
        2. Voting
        3. Certificate Creation (we combine this into voting)
     */

    // Author is broadcasting block to other nodes
    pub async fn propose_block(&mut self, txs: Vec<Transaction>, author: ValidatorId) -> Result<Block, String> {
        // annotate parents so that .collect() will give me a Vec
        
        let env = self.state.read().await;
        let parents: Vec<Hash> = env.dag.get_frontier().iter().copied().collect();

        // do I want to maintain my block

        // new block
        let block = Block::new(txs, parents, author, env.current_round);
        drop(env);
        
        // ?.. I guess I haven't voted yet
        {
            let mut env = self.state.write().await;
            env.dag.insert_block(block.clone())?;
        }

        Ok(block)        
    }

    // Vote on if block is fine
    pub async fn vote_block(&mut self, block_hash: &Hash, voter: ValidatorId) -> Result<(), String> {
        // check if valid block
        {
            let env = self.state.read().await;
            if !env.dag.contains_block(block_hash) {
                return Err("Not a valid block - not in dag".to_string());
            }

            // check if valid voter
            if !self.validator_set.validators.contains_key(&voter){
                return Err("Not a valid voter - not in validator set".to_string());
            }
        }
        // drop lock

        // create cert if does not exist so we can vote on it
        {
            let mut env = self.state.write().await;

            let round = env.current_round;
            let cert = env
                .certificates
                .entry(*block_hash)
                .or_insert_with(|| Certificate::new(*block_hash, round));

            let sig = [voter as u8; 64];
            cert.add_signature(voter, sig);

        }
        // drop lock

        Ok(())
    }

    pub async fn accept_block(&mut self, block: Block) -> Result<(), String> {
        let mut env = self.state.write().await;
        if !self.validator_set.validators.contains_key(&block.author) {
            return Err("Block author not in set".to_string());
        }

        env.dag.insert_block(block)?;
        Ok(())
    }

    // inefficient wih RwLock - just directly read it
    /*
    pub async fn get_round_blocks(&self, round:u32) -> Vec<Hash> {
        let mut result = Vec::new();

        for (hash,cert) in self.certificates.iter() {
            if cert.round == round {
                // hash should be copy - otherwise I need to come back to this
                result.push(*hash);
            }
        }

        result
    }
    */

    /*  Tusk Methods:
        choose a leader = pick validator
        leader proposes = block gets made (in this case already made)
        consensus = which blocks are committed    
     */

    #[allow(clippy::collapsible_if)]
    pub async fn get_leader(&self, round:u32) -> Option<Hash> {
        println!("run get leader");
        let leader = choose_leader(round, self.validator_set.validators.len() as u32);

        // find the block that the leader proposed
        let env = self.state.read().await;
        if let Some(leader_hash) = env.dag.get_author_round_block(leader, round) {
            
            // if the leader has a cert
            if let Some(leader_cert) = env.certificates.get(&leader_hash) {

                // if the leader cert is valid
                if leader_cert.is_valid_cert(&self.validator_set) {
                    println!("I have a leader");
                    return Some(leader_hash)
                }
            }
        }

        None

    }

    // if the blocks have enough votes then we can commit them
        // where are these blocks located?
        // they don't all have to be frontier
    #[allow(clippy::collapsible_if)]
    pub async fn commit_blocks(&mut self) ->Vec<Hash> {
        let mut committed = Vec::new();

        let tusk_round = {
            let env = self.state.read().await;
            if env.current_round < 2 {
                return committed;
            }

            // we reduce round to keep it around 3 
            env.current_round - 2
        };

        if let Some(leader_block) = self.get_leader(tusk_round).await {
            let mut env = self.state.write().await;

            if env.committed_blocks.insert(leader_block) {
                committed.push(leader_block);

                // commit ancestors that have certs
                for anc in env.dag.get_ancestors(&leader_block) {
                    if env.certificates.contains_key(&anc) && env.committed_blocks.insert(anc) {
                        committed.push(anc);
                    }
                }
            }
        }

        /*
        // if round greater than 2
        if self.current_round >= 2 {
            let tusk_round = self.current_round - 2;

            // if exists a leader
            if let Some(leader_block) = self.get_leader(tusk_round) {
                println!("have leader, go to commit blocks");

                // if leader is not already committed (unlikely)
                if !self.committed_blocks.contains(&leader_block) {
                    println!("ah I found it");
                    committed.push(leader_block);
                    self.committed_blocks.insert(leader_block);

                    // we commit all the causal ancestors as well
                    let ancestors = self.dag.get_ancestors(&leader_block);
                    println!("there are {} descendants", ancestors.len());
                    for ancestor in ancestors {
                        // if they have a cert
                        if let Some(cert) = self.certificates.get(&ancestor) {

                            // if the descendant isn't already in committed_blocks Hashset
                            if self.committed_blocks.insert(ancestor) {
                                committed.push(ancestor);
                            }
                        }
                    }
                    println!("commited blocks are {}", self.committed_blocks.len());
                }
            }
        }
        */

        committed
    }
    // since each handle has its own state - can't keep the rounds in the simulation
    pub async fn advance_round(&self) {
        let mut env = self.state.write().await;
        env.current_round += 1;
    }

    // separate check valid
    pub async fn cert_is_valid(&self, hash: &Hash) -> bool {
        let env = self.state.read().await;
        if let Some(cert) = env.certificates.get(hash) {
            return cert.is_valid_cert(&self.validator_set);
        }

        return false
    }
}

// what do I need here
    // round, DAG
//pub fn committed_blocks()



pub fn choose_leader(round:u32, validator_count: u32) -> u32 {
    // skip using shared coin
        // go round-robin
    1 + (round % validator_count)
}