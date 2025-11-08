use narwhal_tusk::consensus::{ConsensusHandle, choose_leader};
use narwhal_tusk::types::{ValidatorInfo, ValidatorSet, Transaction};

fn make_validator_set(n: u32) -> ValidatorSet {
    let vals = (1..=n)
        .map(|id| ValidatorInfo { id, stake: 1 })
        .collect();
    ValidatorSet::new(vals)
}

/*
    Simple test with 4 nodes, multiple rounds

    go to round 2 to force commit

*/

#[tokio::test]
async fn commits_leader_after_two_rounds() {

    let vset = make_validator_set(4);
    let mut c = ConsensusHandle::new(vset);

    // since we're at round 0 round%voters should be 1 (1 + round%voter)
    assert_eq!(choose_leader(0, 4), 1);

    // Round 0, author 1 proposes, all vote
    let b0 = c.propose_block(vec![Transaction::new("r0".into())], 1).await.unwrap();
    
    for v in 1..=4 { 
        c.vote_block(&b0.hash, v).await.unwrap(); 
    }
    
    assert!(c.cert_is_valid(&b0.hash).await, "cert should be valid after quorum");

    // Round 0
    let committed0 = c.commit_blocks().await;
    assert!(committed0.is_empty(), "no commit at round 0");

    // Round 1
    c.advance_round().await;
    let committed1 = c.commit_blocks().await;
    assert!(committed1.is_empty(), "no commit at round 1");

    // Round 2 - Commit
    c.advance_round().await;
    let committed2 = c.commit_blocks().await;
    assert!(committed2.contains(&b0.hash), "leader of r0 should commit at r2");

    // Round 3 - no new:
    let committed3 = c.commit_blocks().await;
    assert!(committed3.is_empty(), "round 3 should have nothing");
}

#[tokio::test]
async fn reject_invalid_voter() {
    let vset = make_validator_set(4);

    let mut c = ConsensusHandle::new(vset);

    let b0 = c.propose_block(vec![Transaction::new("valid voter".into())], 4).await.unwrap();
    
    // let voter 5 vote on block 0
    let invalid_vote = c.vote_block(&b0.hash, 5).await.unwrap_err();

    // shoudl return error
    assert!(invalid_vote.contains("Not a valid voter - not in validator set"));
}