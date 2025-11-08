use narwhal_tusk::{network::*, node::*, types::*};

#[tokio::main]
async fn main() {
    println!("run test");

    let n: u32 = 10;
    // simulation time in seconds
    let time = 5;
    let config = SimulationConfig{
        ..Default::default()
        /*
            or define
         */
    };

    let vals= (1..=n)
        .map(|id| ValidatorInfo {id, stake: 1})
        .collect();
    let vset = ValidatorSet::new(vals);

    let mut sim = Simulator::new(config);
    let net = sim.handle();
    let start = std::time::Instant::now();

    let mut tasks = Vec::with_capacity(n as usize);

    for id in 1..=n {
        let rx = sim.register_node(id).await;
        let node = Node::new(id, rx, vset.clone());
        let net_clone = net.clone();
        tasks.push(tokio::spawn( async move {
            node.run_node(net_clone).await;
        }));
    }

    println!("Starting {} node simulation for {} seconds...", n, time);
    tokio::time::sleep(std::time::Duration::from_secs(time)).await;
    println!("Shut down");

    // kill leftovers
    for t in tasks {
        t.abort();
    }
}
