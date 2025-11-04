use narwhal_tusk::{consensus::{self, ConsensusEnv}, types::*};

fn main() {
    println!("run test");

    let validators = vec![
        ValidatorInfo { id: 1, stake: 100},
        ValidatorInfo { id: 2, stake: 200},
        ValidatorInfo { id: 3, stake: 300},
        ValidatorInfo { id: 4, stake: 400},
    ];

    let validator_set = ValidatorSet::new(validators);
    let mut consensus_env = ConsensusEnv::new(validator_set);

    for round in 0..5 {
        println!("\nthis is Round {}", round);

        let mut round_blocks = Vec::new();
        // for rounds in between we will propose
        for validator_id in 1..=4 {
            
            // each validator proposes a block
            let txs = vec![Transaction::new(format!("tx on round {}, from id: {}", round, validator_id))];
            
            // run propose block
            match consensus_env.propose_block(txs, validator_id) {

                // if there is a block returned from propose then we can push
                // this is not committed/cert yet
                Ok(block) => {
                    println!("block proposed by {}", validator_id);
                    round_blocks.push(block.hash);
                }
                // catch any error
                Err(e) => println!("Error, {}", e),
            }
        }

        for block_hash in &round_blocks {
            
            for voter in 1..=4 {
                // let them vote
                // if error then print out
                match consensus_env.vote_block(block_hash, voter) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error {}",e);
                    }
                }
            }
        }

        //println!("where am I crashing __ 1");
        let committed = consensus_env.commit_blocks();
        //println!("where am I crashing __ 2");
        if !committed.is_empty() {
            println!("{} blocks were committed", committed.len());
        }
        //println!("where am I crashing __ 3");
        // end of the round - advance
        consensus_env.current_round += 1;
    }

    println!("end");
    println!("Total blocks committed is {}", consensus_env.committed_blocks.len());
}
