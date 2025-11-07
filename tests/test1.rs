use narwhal_tusk::consensus::{ConsensusHandle, choose_leader};
use narwhal_tusk::types::{ValidatorInfo, ValidatorSet, Transaction};

fn make_validator_set(n: u32) -> ValidatorSet {
    let vals = (1..=n)
        .map(|id| ValidatorInfo { id, stake: 1 })
        .collect();
    ValidatorSet::new(vals)
}

/*
    Simple test with 4 nodes

    go to round 2 to force commit

*/

#[tokio::test]
async fn commits_leader_after_two_rounds() {

    let vset = make_validator_set(4);
    let mut c = ConsensusHandle::new(vset);

    // Round 0, author 1 proposes, all vote
    let b0 = c.propose_block(vec![Transaction::new("r0".into())], 1).await.unwrap();
    for v in 1..=4 { c.vote_block(&b0.hash, v).await.unwrap(); }
    assert!(c.cert_is_valid(&b0.hash).await, "cert should be valid after quorum");

    // Advance to round 2 so tusk_round = 0
    c.advance_round().await;
    c.advance_round().await;

    let committed = c.commit_blocks().await;

    assert!(committed.contains(&b0.hash), "leader of r0 should commit at r2");
}