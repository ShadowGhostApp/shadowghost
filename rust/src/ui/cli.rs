use crate::core::Engine;
use std::io::{self, Write};

pub struct CliInterface {
    // Store a reference or handle to the engine instead of the full ShadowGhostCore trait
}

impl CliInterface {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🌟 Shadow Ghost Messenger v0.1.0");
        println!("Type 'help' to see available commands");
        println!();

        // Get engine reference through static ENGINE
        let engine = crate::core::ENGINE.get();
        if engine.is_none() {
            println!("⚠️ Engine not initialized. Use 'init' command first.");
        }

        loop {
            print!("\n> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            let parts: Vec<&str> = input.splitn(2, ' ').collect();
            let command = parts[0].to_lowercase();
            let args = if parts.len() > 1 { parts[1] } else { "" };

            match command.as_str() {
                "help" => self.show_help(),
                "init" => self.initialize_engine().await?,
                "status" => self.show_status().await?,
                "quit" | "exit" | "q" => {
                    println!("👋 Goodbye!");
                    break;
                }
                "clear" => {
                    print!("\x1B[2J\x1B[1;1H");
                    io::stdout().flush()?;
                }
                "" => continue,
                _ => println!(
                    "❓ Unknown command '{}'. Type 'help' to see available commands.",
                    command
                ),
            }
        }

        Ok(())
    }

    fn show_help(&self) {
        println!("\n📋 Available commands:");
        println!("┌─────────────────────────┬──────────────────────────────────────────────┐");
        println!("│ Command                 │ Description                                  │");
        println!("├─────────────────────────┼──────────────────────────────────────────────┤");
        println!("│ init                    │ Initialize application                       │");
        println!("│ status                  │ Show current status                          │");
        println!("│ clear                   │ Clear screen                                 │");
        println!("│ help                    │ Show this help                               │");
        println!("│ quit/exit/q             │ Exit application                             │");
        println!("└─────────────────────────┴──────────────────────────────────────────────┘");
    }

    async fn initialize_engine(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 Engine initialization would happen here");
        println!("✅ Placeholder initialization complete");
        Ok(())
    }

    async fn show_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n📊 Current status:");
        println!("┌─────────────────────┬─────────────────────────────────┐");

        let engine_status = if crate::core::ENGINE.get().is_some() {
            "✅ Yes"
        } else {
            "❌ No"
        };
        println!("│ Engine Loaded       │ {:<31} │", engine_status);

        println!("└─────────────────────┴─────────────────────────────────┘");
        Ok(())
    }
}
