use shadowghost::cli::CliInterface;
use shadowghost::prelude::*;
use std::io::{self, Write};

async fn auto_initialize_and_start(
    mut core: ShadowGhostCore,
) -> Result<ShadowGhostCore, Box<dyn std::error::Error>> {
    if !core.is_initialized() {
        println!("Initializing Shadow Ghost...");

        let stored_name = core.get_stored_user_name().await;
        let user_name = if let Some(name) = stored_name {
            println!("Welcome back, {}!", name);
            name
        } else {
            print!("Enter your name (default: user): ");
            io::stdout().flush()?;

            let mut name = String::new();
            io::stdin().read_line(&mut name)?;
            let name = name.trim();

            if name.is_empty() {
                "user".to_string()
            } else {
                name.to_string()
            }
        };

        match core.initialize(Some(user_name.clone())).await {
            Ok(()) => {
                println!("Initialized successfully as '{}'!", user_name);
                if let Some(peer_info) = core.get_peer_info().await {
                    println!("👤 You are now: {}", peer_info);
                }
            }
            Err(e) => {
                println!("Failed to initialize: {}", e);
                return Err(Box::new(e));
            }
        }
    }

    if let Ok(connection_info) = core.get_connection_info().await {
        println!("\n🔗 Connection Information:");
        for line in connection_info.lines() {
            println!("  {}", line);
        }
    }

    println!("\nStarting server...");
    match core.start_server().await {
        Ok(()) => {
            println!("✓ Server started successfully!");
            println!("You can now receive messages from other peers.");
        }
        Err(e) => {
            println!("Failed to start server: {}", e);
            println!("You can start it later with the 'start' command.");
        }
    }

    if let Ok(contacts) = core.get_contacts().await {
        let mut unread_total = 0;
        for contact in &contacts {
            if let Ok(count) = core.get_unread_message_count(&contact.name).await {
                if count > 0 {
                    unread_total += count;
                    println!("💬 {} unread messages from {}", count, contact.name);
                }
            }
        }
        if unread_total > 0 {
            println!("📨 Total: {} unread messages", unread_total);
        }
    }

    Ok(core)
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Starting Shadow Ghost Messenger");

    let exe_dir = std::env::current_exe()?.parent().unwrap().to_path_buf();
    let config_path = exe_dir.join("config.toml");

    let core = ShadowGhostCore::new(&config_path)?;
    let initialized_core = auto_initialize_and_start(core).await?;

    println!("\n{}", "=".repeat(60));
    println!("Shadow Ghost Messenger is ready!");
    println!("Quick start:");
    println!("1. Generate your link: 'link'");
    println!("2. Share it with friends");
    println!("3. Add their links: 'add <sg-link>'");
    println!("4. Start chatting: 'chat <name>'");
    println!("{}", "=".repeat(60));

    let mut cli = CliInterface::new(initialized_core);
    let result = cli.run().await;

    log::info!("Shutting down application");

    result
}
