use std::net::SocketAddr;

#[cfg(feature = "cli")]
use clap::{Parser, Subcommand};

#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(
    name = "shadowghost",
    version = env!("CARGO_PKG_VERSION"),
    about = "ShadowGhost — P2P messenger that works when Telegram doesn't"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
enum Commands {
    /// Listen for an incoming peer connection
    Listen {
        #[arg(short, long, default_value = "8888", help = "TCP port to listen on")]
        port: u16,
        #[arg(short, long, default_value = "anonymous", help = "Your display name")]
        name: String,
    },
    /// Connect to a listening peer
    Connect {
        #[arg(short, long, help = "Peer address, e.g. 192.168.1.5:8888")]
        addr: SocketAddr,
        #[arg(short, long, default_value = "anonymous", help = "Your display name")]
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "cli")]
    {
        let cli = Cli::parse();
        match cli.command {
            Some(Commands::Listen { port, name }) => {
                let node = shadowghost::p2p::P2pNode::listen(port, name).await?;
                node.run_chat_loop().await?;
                return Ok(());
            }
            Some(Commands::Connect { addr, name }) => {
                let node = shadowghost::p2p::P2pNode::connect(addr, name).await?;
                node.run_chat_loop().await?;
                return Ok(());
            }
            None => {}
        }
    }

    // Fallback: legacy interactive REPL (no subcommand given or cli feature off)
    let mut cli = shadowghost::ui::CliInterface::new();
    cli.run().await?;
    Ok(())
}
