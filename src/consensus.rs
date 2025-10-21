#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)] 

use std::{collections::{HashMap, HashSet}, path::Ancestors, vec};

use crate::{types, Block, Certificate, Hash, ValidatorId, ValidatorSet, Transaction};
use crate::{dag::DAG};

pub struct ConsensusEnv {
    pub dag: DAG,
    pub validator_set: ValidatorSet,
    pub current_round: u32,
    pub committed_blocks: HashSet<Hash>,
    pub certificates: HashMap<Hash, Certificate>,
}

impl ConsensusEnv {
    pub fn new(validator_set: ValidatorSet) -> Self {
        Self {
            dag: DAG::default(),
            validator_set,
            current_round: 0,
            committed_blocks: HashSet::new(),
            certificates:HashMap::new(),
        }
    }
    /* Narwhal Methods:
        1. Block proposal
        2. Voting
        3. Certificate Creation (we combine this into voting)
     */

    // Author is broadcasting block to other nodes
    pub fn propose_block(&mut self, txs: Vec<Transaction>, author: ValidatorId) -> Result<Block, String> {
        // annotate parents so that .collect() will give me a Vec
        let parents: Vec<Hash> = self.dag.get_frontier().iter().copied().collect();

        // new block
        let block = Block::new(txs, parents, author, self.current_round);
        // ?.. I guess I haven't voted yet
        self.dag.insert_block(block.clone())?;

        Ok(block)        
    }

    // Vote on if block is fine
    pub fn vote_block(&mut self, block_hash: &Hash, voter: ValidatorId) -> Result<(), String> {
        // check if valid block
        if !self.dag.contains_block(block_hash) {
            return Err("Not a valid block - not in dag".to_string());
        }

        // check if valid voter
        if !self.validator_set.validators.contains_key(&voter){
            return Err("Not a valid voter - not in validator set".to_string());
        }

        // create cert if does not exist so we can vote on it
        if !self.certificates.contains_key(block_hash) {
            let cert = Certificate::new(*block_hash, self.current_round);
            self.certificates.insert(*block_hash, cert);
        }

        // add the signature to the cert
        let signature = [voter as u8; 64];
        if let Some(cert) = self.certificates.get_mut(block_hash) {
            cert.add_signature(voter, signature);
        }

        Ok(())
    }

    pub fn get_round_blocks(&self, round:u32) -> Vec<Hash> {
        let mut result = Vec::new();

        for (hash,cert) in self.certificates.iter() {
            if cert.round == round {
                // hash should be copy - otherwise I need to come back to this
                result.push(*hash);
            }
        }

        result
    }

    /*  Tusk Methods:
        choose a leader = pick validator
        leader proposes = block gets made (in this case already made)
        consensus = which blocks are committed    
     */

    #[allow(clippy::collapsible_if)]
    pub fn get_leader(&self, round:u32) -> Option<Hash> {
        println!("run get leader");
        let leader = choose_leader(round, self.validator_set.validators.len() as u32);

        // find the block that the leader proposed
        if let Some(leader_hash) = self.dag.get_author_round_block(leader, round) {
            
            // if the leader has a cert
            if let Some(leader_cert) = self.certificates.get(&leader_hash) {

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
    pub fn commit_blocks(&mut self) ->Vec<Hash> {
        let mut committed = Vec::new();

        if self.current_round < 2 {
            // shouldn't this be an error
            return committed
        }

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

        committed
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