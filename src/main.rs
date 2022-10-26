use libp2p::gossipsub::{
    Gossipsub, GossipsubEvent, IdentTopic as Topic, MessageAuthenticity, ValidationMode,
};
use libp2p::mdns::{Mdns, MdnsConfig, MdnsEvent};
use libp2p::swarm::SwarmBuilder;
use libp2p::tcp::{GenTcpConfig, TokioTcpTransport};
use libp2p::{
    core::upgrade, futures::StreamExt, gossipsub, identity, mplex, noise, swarm::SwarmEvent,
    NetworkBehaviour, PeerId,
};
use libp2p::{Multiaddr, Transport};
use std::error::Error;
use std::time::Duration;
use tokio::io::{self, AsyncBufReadExt};
use tokio::{self, select};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {}", local_peer_id);

    let transport = TokioTcpTransport::new(GenTcpConfig::default().nodelay(true))
        .upgrade(upgrade::Version::V1)
        .authenticate(
            noise::NoiseAuthenticated::xx(&local_key)
                .expect("Signing libp2p-noise static DH keypair failed"),
        )
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    #[derive(NetworkBehaviour)]
    #[behaviour(out_event = "MyBehaviourEvent")]
    struct MyBehaviour {
        gossipsub: Gossipsub,
        mdns: Mdns,
    }

    #[allow(clippy::large_enum_variant)]
    enum MyBehaviourEvent {
        Gossipsub(GossipsubEvent),
        Mdns(MdnsEvent),
    }

    impl From<GossipsubEvent> for MyBehaviourEvent {
        fn from(v: GossipsubEvent) -> Self {
            Self::Gossipsub(v)
        }
    }

    impl From<MdnsEvent> for MyBehaviourEvent {
        fn from(v: MdnsEvent) -> Self {
            Self::Mdns(v)
        }
    }

    let topic = Topic::new("chat-trehund");

    let mut swarm = {
        let mdns = Mdns::new(MdnsConfig::default())?;

        let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(ValidationMode::Strict)
            .build()
            .expect("Valid config");

        let mut gossipsub: gossipsub::Gossipsub =
            gossipsub::Gossipsub::new(MessageAuthenticity::Signed(local_key), gossipsub_config)
                .expect("Correct configuration");

        gossipsub.subscribe(&topic).unwrap();

        let behaviour = MyBehaviour { gossipsub, mdns };

        SwarmBuilder::new(transport, behaviour, local_peer_id)
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build()
    };

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    let mut stdin = io::BufReader::new(io::stdin()).lines();
    println!("Please enter you name: ");
    let name = stdin
        .next_line()
        .await
        .expect("Valid name")
        .unwrap_or(String::from("anonymous"))
        .trim()
        .to_owned();

    let mut valid_addr = false;
    while !valid_addr {
        println!("Enter an address (blank to get a new one)");
        let address = stdin
            .next_line()
            .await
            .expect("Valid addr")
            .unwrap()
            .to_owned();

        if address == String::new() {
            break;
        }

        if let Ok(addr) = address.parse::<Multiaddr>() {
            match swarm.dial(addr) {
                Ok(_) => {
                    valid_addr = true;
                    println!("Dialed {:?}", address)
                }
                Err(err) => println!("Dialed error {}", err),
            }
        };
    }

    // Clear screen
    print!("\x1B[2J\x1B[1;1H");

    loop {
        select! {
            line = stdin.next_line() => {
                let line = format!("{}: {}", name, line?.expect("stdin closed"));
                if let Err(e) = swarm
                .behaviour_mut()
                .gossipsub.publish(topic.clone(), line.as_bytes()) {
                    println!("Publish error: {}", e);
                }
            },
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, ..} => {
                    if !valid_addr {
                        println!("{}", address)
                    }
                },
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(MdnsEvent::Discovered(list))) => {
                    for (peer_id, _) in list {
                        println!("mDNS discovered a new peer: {}", peer_id);
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(MdnsEvent::Expired(list))) => {
                    for (peer_id, _) in list {
                        println!("mDNS discover peer has expired: {}", peer_id);
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(GossipsubEvent::Message {
                    propagation_source: _,
                    message_id: _,
                    message
                })) => println!("{}", String::from_utf8_lossy(&message.data)),
                _ => {}
            }
        }
    }
}
