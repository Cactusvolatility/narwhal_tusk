
use std::{collections::HashMap, thread::sleep, time::Duration};

use futures::channel::mpsc;

use crate::{Block, Hash, ValidatorId, Certificate};

#[derive(Clone, Debug)]
pub struct NetworkMsg {
    pub from: ValidatorId,
    pub to: ValidatorId,
    pub payload: MessagePayload,
}

#[derive(Clone, Debug)]
pub enum MessagePayload {
    Block(Block),
    Vote { block_hash: Hash, voter: ValidatorId},
    // only need to send the cert
    Certificate { block_hash: Hash}
}

#[derive(Clone)]
pub struct SimulationConfig {
    pub latency_ms: (u64,u64),
    pub packet_loss_rate: f64,
    pub node_churn: f64,
    pub byzantine_nodes: Vec<ValidatorId>,
}

impl  Default for SimulationConfig {
    fn default() -> Self {
        Self {
            latency_ms: (30,200),
            packet_loss_rate: 0.02,
            node_churn: 0.001,
            byzantine_nodes: vec![],
        }
    }
}

pub struct Simulator {
    config: SimulationConfig,
    node_channels: HashMap<ValidatorId, mpsc::UnboundedSender<NetworkMsg>>,
    message_counts: HashMap<ValidatorId, usize>,
}

impl Simulator {
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            config,
            node_channels: HashMap::new(),
            message_counts: HashMap::new(),
        }
    }

    // set up channel
    pub fn register_node(&mut self, node_id: ValidatorId) -> mpsc::UnboundedReceiver<NetworkMsg> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.node_channels.insert(node_id, tx);
        self.message_counts.insert(node_id, 0);
        return rx
    }

    // broadcast
    pub async fn broadcast_msg(&mut self, message: NetworkMsg) {
        // make sure it's not from ourselves
        let target_nodes: Vec<ValidatorId> = self.node_channels.keys().copied()
            .filter(|&id| id != message.from)
            .collect();

        for target in target_nodes {
            let mut msg = message.clone();
            msg.to = target;
            self.send_message(msg).await;
        }

    }

    pub async fn send_message(&mut self, message: NetworkMsg) {
        if rand::thread_rng().gen_bool(self.config.packet_loss_rate.clamp(0.0, 1.0)) 
        {
            return;
        }
        
        let (min_latency, max_latency) = self.config.latency_ms;
        let latency = rand::thread_rng().gen_range(min_latency,max_latency);

        if let Some(tx) = self.node_channels.get(&message.to) {
            let tx = tx.clone();
            tokio::spawn(async move {
                sleep(Duration::from_millis(latency)).await;
                let _ = tx.send(message);
            });
        }
    
    }

}