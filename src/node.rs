use std::sync::Arc;
use tokio::sync::{RwLock,mpsc};
use tokio::time::{interval, Duration};

use crate::{ConsensusHandle, Transaction, ValidatorId, ValidatorSet, network::{MessagePayload, NetworkHandle, NetworkMsg}};

pub struct Node {
    pub id: ValidatorId,
    pub rx: mpsc::UnboundedReceiver<NetworkMsg>,
    pub consensus: ConsensusHandle,
}

impl Node {
    pub fn new(id:ValidatorId, rx: mpsc::UnboundedReceiver<NetworkMsg>, val_set: ValidatorSet) -> Self {
        Self { 
            id, 
            rx, 
            consensus: ConsensusHandle::new(val_set)
        }
    }

    pub async fn local_propose(&mut self, txs: Vec<Transaction>, net: &NetworkHandle) -> Result<(), String> {
        let block = self.consensus.propose_block(txs, self.id).await?;
        net.broadcast(self.id, MessagePayload::Block(block)).await;
        Ok(())
    }

    pub async fn run_node(mut self, net: NetworkHandle) {

        let mut propose_tick = interval(Duration::from_millis(100));
        let mut commit_tick = interval(Duration::from_millis(200));
        // let round be equal to 350 so that propose and commit are inside
        let mut round_tick = interval(Duration::from_millis(350));

        loop {
            tokio::select! {
                // if I receive a message then run the following
                Some(msg) = self.rx.recv() => {
                    match msg.payload {
                        // received block
                        MessagePayload::Block(block) => {
                            let hash = block.hash;
                            let _ = self.consensus.accept_block(block).await;

                            let _ = self.consensus.vote_block(&hash, self.id).await;
                            net.broadcast(self.id, MessagePayload::Vote { block_hash: hash, voter: self.id }).await;
                        }
                        // received vote
                        MessagePayload::Vote { block_hash, voter } => {
                            let _ = self.consensus.vote_block(&block_hash, voter).await;

                            if self.consensus.cert_is_valid(&block_hash).await {
                                net.broadcast(self.id, MessagePayload::Certificate { block_hash }).await;
                            }
                        }
                        // received cert: commit
                        MessagePayload::Certificate { block_hash: _ } => {
                            let _ = self.consensus.commit_blocks().await;
                        }
                    }
                }

                // otherwise tick and then propose
                _ = propose_tick.tick() => {
                    let txs = vec![Transaction::new(format!("node {} tx", self.id))];
                    if let Ok(block) = self.consensus.propose_block(txs, self.id).await {
                        net.broadcast(self.id, MessagePayload::Block(block)).await;
                    }
                }

                _ = commit_tick.tick() => {
                    let _ = self.consensus.commit_blocks().await;
                }

                _ = round_tick.tick() => {
                    self.consensus.advance_round().await;
                }


            }
        }
    }
}