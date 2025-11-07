
use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;

use crate::{Block, Hash, ValidatorId, Certificate};
use rand::Rng;

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

struct NetInner {
    config: SimulationConfig,
    // TODO:routes will change over time
    routes: RwLock<HashMap<ValidatorId, mpsc::UnboundedSender<NetworkMsg>>>
}

impl NetInner {
    fn new(config: SimulationConfig) -> Self {
        Self {
            config,
            routes: RwLock::new(HashMap::new()),
        }
    }
}

pub struct Simulator {
    inner: Arc<NetInner>,
}

pub struct NetworkHandle {
    inner:Arc<NetInner>,
}

impl Simulator {
    pub fn new(config: SimulationConfig) -> Self {
        Self { inner: Arc::new(NetInner::new(config)) }
    }

    // create a handle at the start
    pub fn handle(&self) -> NetworkHandle {
        NetworkHandle {inner: Arc::clone(&self.inner)}
    }

    pub async fn register_node(&self, id: ValidatorId) -> mpsc::UnboundedReceiver<NetworkMsg> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.inner.routes.write().await.insert(id,tx);
        rx
    }
}

impl NetworkHandle {
    pub async fn send(&self, mut msg: NetworkMsg) {
        // packet loss
        let loss = self.inner.config.packet_loss_rate.clamp(0.0, 1.0);
        if rand::random::<f64>() < loss {
            return
        }

        let (low, high) = self.inner.config.latency_ms;
        let latency = rand::rng().random_range(low..high);

        // Option<Sender>
        let tx_opt = {
            let routes = self.inner.routes.read().await;
            routes.get(&msg.to).cloned()
        };

        // delay with latency
        if let Some(tx) = tx_opt {
            tokio::spawn(async move {
                sleep(Duration::from_millis(latency)).await;
                let _ = tx.send(msg);
            });
        }
    }

    pub async fn broadcast(&self, from: ValidatorId, payload: MessagePayload) {
        let targets: Vec<ValidatorId> = {
            let routes = self.inner.routes.read().await;
            routes.keys().copied().filter(|&id| id != from).collect()
        };

        // send messages to the targets in my routes
        for to in targets {
            let msg = NetworkMsg { from, to, payload: payload.clone()};
            self.send(msg).await;
        }
    }
}