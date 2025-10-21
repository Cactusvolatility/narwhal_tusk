use sha2::{Sha256, Digest};
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;

//use crate::validator;

pub type Hash = [u8; 32];
pub type ValidatorId = u32;
pub type Signature = [u8; 64];

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: u64,
    pub data: String,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub hash: Hash,
    pub txs: Vec<Transaction>,
    pub parents: Vec<Hash>,
    pub author: ValidatorId,
    pub round: u32,
}

// what should my certificate have?
    // how many people?
    // from who?
    // hash of the certificate
#[derive(Debug, Clone)]
pub struct Certificate {
    pub round: u32,
    pub block_hash: Hash,
    pub signatures: Vec<(ValidatorId, Signature)>
}

// what should vote have
    // who voted
    // does the vote need to be hashed?
    // vote on what block
    // which round
#[derive(Debug, Clone)]
pub struct Vote {
    pub block_hash: Hash,
    pub round: u32,
    pub voter: ValidatorId,
    pub signature: Signature,
}

// which validators
    // need the round
    // 
#[derive(Debug, Clone)]
pub struct ValidatorSet {
    pub validators: HashMap<ValidatorId,ValidatorInfo>,
    pub threshold: usize,
}

// ValidatorInfo
#[derive(Debug, Clone)]
pub struct ValidatorInfo {
    pub id: ValidatorId,
    pub stake: u64,
}

// hash function
/*
    we'll use SHA256 to hash things
    In this demo we'll be hashing things such as the  node ID

*/
impl Block {
    pub fn new(txs: Vec<Transaction>, parents: Vec<Hash>, author: ValidatorId, round: u32) -> Self {
        let mut block = Self {
            hash: [0; 32],
            txs,
            parents,
            author,
            round,
        };

        block.hash = block.hash_fn();
        block

    }
    pub fn hash_fn(&self) -> Hash {
        let mut hasher = Sha256::new();


        

        //TODO:
            // hasher for transactions
            // hasher for parents
        
        // how do I separate the transactions from the metdata
            // how do I hash a Vec?
        for tx in &self.txs {
            hasher.update(&tx.id.to_le_bytes());
            hasher.update(tx.data.as_bytes());
        }

        // parent is a Vec<Hash> where Hash is [u8; 32]
            // do I need to own these Hashes? no
        for parent in &self.parents{
            // these are already hashes - do I need to rehash them? no
            hasher.update(parent);
        }
                
        // add the round and the author to the hasher
            // this is metadata hashing
        hasher.update(&self.round.to_le_bytes());
        hasher.update(&self.author.to_le_bytes());

        // we finalize the hash and use into() to convert to Hash type
            // hashed everything
        hasher.finalize().into()
            
    }
    //TODO:
        // is_genesis (parents is empty?)
    pub fn is_genesis(&self) -> bool {
        if self.parents.is_empty() {
            return true
        }

        return false
    }
        // parent_count (parents.len())
    
    pub fn parent_count(&self) -> usize {
        self.parents.len()
    }
        // tx_count (txs.len())
    
    pub fn tx_count(&self) -> usize {
        self.txs.len()
    }
        // verify
    
    pub fn verify(&self) -> bool {
        self.hash == self.hash_fn()
    }
        // size_bytes (network limits?)
    pub fn size_bytes(&self) -> usize {
        32 + // hash is 32 bytes
        // for each transation it has some bytes
        self.txs.iter()
            .map(|tx| 8 + tx.data.len())
            .sum::<usize>()
            +
        self.parents.len() * 32 + // each parent is a Hash of 32 bytes
        4 + // author 
        4 // round
    }
}

static TX_COUNTER: AtomicU64 = AtomicU64::new(0);

impl Transaction {
    pub fn new(data: String) -> Self {
        // just use an atomic counter
        Self {
            id: TX_COUNTER.fetch_add(1, Ordering::Relaxed),
            data,
        }
    }
    
}

impl ValidatorSet {
    // given a list of validators we add them to the hashmap
    pub fn new(validators: Vec<ValidatorInfo>) -> Self {
        // need to use into_iter since hashmap needs to own the data
        let validator_map = validators.into_iter()
            .map(|nodes| (nodes.id, nodes)) // convert to tuples
            .collect::<HashMap<ValidatorId, ValidatorInfo>>(); // convert to hashmap
        let total = validator_map.len();

        // 2f + 1
        let threshold = (2 * total/3) + 1;

        // return a Self
        Self {
            validators: validator_map,
            threshold,
        }
    }
}

impl Certificate {
    pub fn new(block_hash: Hash, round: u32) -> Self{
        Self {
            block_hash,
            round,
            signatures: Vec::new(),
        }
    }

    pub fn add_signature(&mut self, validator_id: ValidatorId, signature: Signature) {
        self.signatures.push((validator_id,signature));
    }

    // check if we have enough valid signatures
        // use a ref since dont want to take ownership from hashmap
    pub fn is_valid_cert(&self, validator_set: &ValidatorSet) -> bool{
        // check if cert has enough signatures from valid validators from the set
        let mut count = 0;
        for (id,signature) in &self.signatures {
            if validator_set.validators.contains_key(id) {
                count += 1;
            }
        }

        if count >= validator_set.threshold {
            return true
        }

        return false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_transaction() -> Transaction {
        Transaction::new("test transation".to_string())
    }

    fn create_validators() -> ValidatorSet {
        let validators = vec! [
            ValidatorInfo { id: 1, stake: 100},
            ValidatorInfo { id: 2, stake: 200},
            ValidatorInfo { id: 3, stake: 300},
            ValidatorInfo { id: 4, stake: 400},
        ];
        ValidatorSet::new(validators)
    }

    #[test]
    fn test_blocks() {
        // test blocks
        let txs = vec![
            create_transaction(),
            create_transaction(),
            create_transaction(),
            create_transaction()
        ];
        let parents = vec![[1u8;32],[2u8;32],[3u8;32]];
        let dummy_block = Block::new(txs, parents, 1, 22);

        assert!(dummy_block.verify());
        let size = dummy_block.size_bytes();
        println!("Size is {}", size);
    }

    #[test]
    fn test_cert() {
        // we want to test certs
        let validators_set = create_validators();
        let mut cert = Certificate::new([1u8; 32], 1);

        cert.add_signature(1, [1u8; 64]);
        cert.add_signature(2, [2u8; 64]);
        cert.add_signature(3, [3u8; 64]);

        assert!(cert.is_valid_cert(&validators_set));
    }

}