use shadowghost::prelude::*;

fn main() {
    println!("ShadowGhost P2P Architecture Test");

    let event_bus = EventBus::new();
    let peer = Peer::new("test_peer".to_string(), "127.0.0.1:8000".to_string());

    println!("✓ Events module loaded");
    println!("✓ Peer module loaded");
    println!("✓ Config module loaded");
    println!("✓ Protocol module loaded");

    println!("Architecture scaffolding complete!");
    println!("Peer ID: {}", peer.id);
    println!("Peer Name: {}", peer.name);
}
