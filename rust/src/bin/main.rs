use shadowghost::cli::CliInterface;
use shadowghost::prelude::*;
use std::io::{self, Write};

async fn auto_initialize_and_start(
    mut core: ShadowGhostCore,
) -> Result<ShadowGhostCore, Box<dyn std::error::Error>> {
    if !core.is_initialized() {
        println!("Initializing Shadow Ghost...");

        print!("Enter your name (default: user): ");
        io::stdout().flush()?;

        let mut name = String::new();
        io::stdin().read_line(&mut name)?;
        let name = name.trim();

        let user_name = if name.is_empty() {
            "user".to_string()
        } else {
            name.to_string()
        };

        match core.initialize(Some(user_name.clone())).await {
            Ok(()) => println!("Initialized successfully as '{}'!", user_name),
            Err(e) => {
                println!("Failed to initialize: {}", e);
                return Err(Box::new(e));
            }
        }
    }

    println!("\nStarting server...");
    print!("Would you like to start the server now? (Y/n): ");
    io::stdout().flush()?;

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;
    let response = response.trim().to_lowercase();

    if response.is_empty() || response == "y" || response == "yes" {
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
    } else {
        println!("Server not started. Use 'start' command when ready.");
        println!("⚠ Note: You won't receive messages until the server is started.");
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
