use crate::core::ShadowGhostCore;
use std::io::{self, Write};

pub struct CliInterface {
    core: ShadowGhostCore,
}

impl CliInterface {
    pub fn new(core: ShadowGhostCore) -> Self {
        Self { core }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🌟 Shadow Ghost Messenger v0.1.0");
        println!("Type 'help' to see available commands");
        println!();

        if let Some(peer_info) = self.core.get_peer_info().await {
            println!("👤 Ready as: {}", peer_info);
        }

        if !self.core.is_server_started() {
            println!("⚠️ Server not started yet. Use 'start' command to receive connections.");
        }

        let event_bus = self.core.get_event_bus();
        tokio::spawn(async move {
            let mut receiver = event_bus.subscribe();
            while let Ok(event) = receiver.recv().await {
                match event {
                    crate::events::AppEvent::Network(net_event) => match net_event {
                        crate::events::NetworkEvent::MessageReceived { message } => {
                            println!(
                                "\n💬 New message from {}: {}",
                                message.from, message.content
                            );
                            print!("> ");
                            io::stdout().flush().unwrap();
                        }
                        crate::events::NetworkEvent::ContactAdded { contact } => {
                            println!("\n👥 New contact added: {}", contact.name);
                            print!("> ");
                            io::stdout().flush().unwrap();
                        }
                        crate::events::NetworkEvent::Error { error, context } => {
                            if let Some(ctx) = context {
                                println!("\n❌ Error [{}]: {}", ctx, error);
                            } else {
                                println!("\n❌ Error: {}", error);
                            }
                            print!("> ");
                            io::stdout().flush().unwrap();
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        });

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
                "link" => self.handle_link_command().await?,
                "contacts" => self.list_contacts().await?,
                "init" => self.initialize_core().await?,
                "start" => self.start_server().await?,
                "stop" => self.stop_server().await?,
                "restart" => self.restart_server().await?,
                "quit" | "exit" | "q" => {
                    println!("👋 Goodbye!");
                    break;
                }
                "chat" => self.handle_chat_command(args).await?,
                "add" => self.handle_add_command(args).await?,
                "ping" => self.handle_ping_command(args).await?,
                "status" => self.show_status().await?,
                "stats" => self.show_network_stats().await?,
                "name" => self.handle_name_command(args).await?,
                "connection" => self.handle_connection_command().await?,
                "update-ip" => self.handle_update_ip_command().await?,
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

        self.core.shutdown().await?;
        Ok(())
    }

    async fn handle_connection_command(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Error: System not initialized. Execute 'init' first.");
            return Ok(());
        }

        match self.core.get_connection_info().await {
            Ok(info) => {
                println!("\n🔗 Connection Information:");
                println!("{}", "═".repeat(50));
                for line in info.lines() {
                    println!("  {}", line);
                }
                println!("{}", "═".repeat(50));
                println!("💡 Share your SG link (use 'link' command) for others to connect");
            }
            Err(e) => println!("❌ Error getting connection info: {}", e),
        }

        Ok(())
    }

    async fn handle_update_ip_command(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Error: System not initialized. Execute 'init' first.");
            return Ok(());
        }

        print!("🔄 Updating external IP address...");
        io::stdout().flush()?;

        match self.core.update_external_address().await {
            Ok(()) => {
                println!(" ✅ External IP updated successfully!");
                println!("💡 Use 'connection' command to see current addresses");
                println!("⚠️ You may need to share a new SG link if your IP changed");
            }
            Err(e) => println!(" ❌ Error updating IP: {}", e),
        }

        Ok(())
    }

    fn show_help(&self) {
        println!("\n📋 Available commands:");
        println!("┌─────────────────────────┬──────────────────────────────────────────────┐");
        println!("│ Command                 │ Description                                  │");
        println!("├─────────────────────────┼──────────────────────────────────────────────┤");
        println!("│ init                    │ Initialize application                       │");
        println!("│ start                   │ Start server (required for receiving)       │");
        println!("│ stop                    │ Stop server                                  │");
        println!("│ restart                 │ Restart server                               │");
        println!("│ link                    │ Generate your connection link                │");
        println!("│ add <sg-link>           │ Add contact by SG link                       │");
        println!("│ contacts                │ Show all contacts                            │");
        println!("│ chat <contact-name>     │ Enter chat with contact                      │");
        println!("│ ping <contact-name>     │ Check if contact is online                   │");
        println!("│ status                  │ Show current status                          │");
        println!("│ stats                   │ Show network statistics                      │");
        println!("│ name <new-name>         │ Change your name                             │");
        println!("│ connection              │ Show connection information                  │");
        println!("│ update-ip               │ Update external IP address                   │");
        println!("│ clear                   │ Clear screen                                 │");
        println!("│ help                    │ Show this help                               │");
        println!("│ quit/exit/q             │ Exit application                             │");
        println!("└─────────────────────────┴──────────────────────────────────────────────┘");
    }

    async fn handle_link_command(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Error: System not initialized. Execute 'init' first.");
            return Ok(());
        }

        match self.core.generate_sg_link().await {
            Ok(link) => self.display_link_for_copying(&link),
            Err(e) => println!("❌ Error creating link: {}", e),
        }

        Ok(())
    }

    fn display_link_for_copying(&self, link: &str) {
        println!("\n{}", "═".repeat(80));
        println!("🔗 YOUR CONNECTION LINK:");
        println!("{}", "═".repeat(80));
        println!();
        println!("   {}", link);
        println!();
        println!("{}", "═".repeat(80));
        println!("📋 INSTRUCTIONS:");
        println!("1. Select the link above with mouse (triple-click to select all)");
        println!("2. Copy with Ctrl+C (Windows/Linux) or Cmd+C (Mac)");
        println!("3. Send this link to the person you want to connect with");
        println!("4. They should use 'add <your-link>' command to add you");
        println!("{}", "═".repeat(80));
        println!();

        print!("Press Enter to continue...");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        println!("💡 Link is still available above if you need to copy it again.");
    }

    async fn initialize_core(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        print!("Enter your name (default: user): ");
        io::stdout().flush()?;

        let mut name = String::new();
        io::stdin().read_line(&mut name)?;
        let name = name.trim();

        let user_name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };

        match self.core.initialize(user_name).await {
            Ok(()) => {
                println!("✅ System initialized successfully!");
                if let Some(peer_info) = self.core.get_peer_info().await {
                    println!("👤 You are now: {}", peer_info);
                }
            }
            Err(e) => println!("❌ Initialization error: {}", e),
        }

        Ok(())
    }

    async fn start_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Error: System not initialized. Execute 'init' first.");
            return Ok(());
        }

        if self.core.is_server_started() {
            println!("ℹ️ Server is already running.");
            return Ok(());
        }

        print!("🚀 Starting server...");
        io::stdout().flush()?;

        match self.core.start_server().await {
            Ok(()) => {
                println!(" ✅ Server started successfully!");
                println!("📨 You can now receive messages from other users.");
            }
            Err(e) => println!(" ❌ Server start error: {}", e),
        }

        Ok(())
    }

    async fn stop_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Error: System not initialized.");
            return Ok(());
        }

        if !self.core.is_server_started() {
            println!("ℹ️ Server is already stopped.");
            return Ok(());
        }

        print!("🛑 Stopping server...");
        io::stdout().flush()?;

        match self.core.stop_server().await {
            Ok(()) => {
                println!(" ✅ Server stopped successfully!");
                println!("⚠️ You will no longer receive messages until restart.");
            }
            Err(e) => println!(" ❌ Server stop error: {}", e),
        }

        Ok(())
    }

    async fn restart_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Error: System not initialized. Execute 'init' first.");
            return Ok(());
        }

        print!("🔄 Restarting server...");
        io::stdout().flush()?;

        match self.core.restart_server().await {
            Ok(()) => {
                println!(" ✅ Server restarted successfully!");
            }
            Err(e) => println!(" ❌ Server restart error: {}", e),
        }

        Ok(())
    }

    async fn show_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n📊 Current status:");
        println!("┌─────────────────────┬─────────────────────────────────┐");

        let init_status = if self.core.is_initialized() {
            "✅ Yes"
        } else {
            "❌ No"
        };
        println!("│ Initialized         │ {:<31} │", init_status);

        let server_status = self.core.get_server_status().await;
        println!("│ Server              │ {:<31} │", server_status);

        if let Some(peer_info) = self.core.get_peer_info().await {
            println!("│ Identity            │ {:<31} │", peer_info);
        }

        if self.core.is_initialized() {
            match self.core.get_contacts().await {
                Ok(contacts) => {
                    let mut online_count = 0;
                    for contact in &contacts {
                        if self.core.check_contact_online(&contact.name).await {
                            online_count += 1;
                        }
                    }
                    println!(
                        "│ Contacts            │ {} (online: {})              │",
                        contacts.len(),
                        online_count
                    );
                }
                Err(_) => println!("│ Contacts            │ Load error                       │"),
            }
        }

        println!("└─────────────────────┴─────────────────────────────────┘");

        Ok(())
    }

    async fn show_network_stats(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Error: System not initialized.");
            return Ok(());
        }

        match self.core.get_network_stats().await {
            Ok(stats) => {
                println!("\n📈 Network statistics:");
                println!("┌─────────────────────┬─────────────────────┐");
                println!(
                    "│ Messages sent       │ {:<19} │",
                    stats.total_messages_sent
                );
                println!(
                    "│ Messages received   │ {:<19} │",
                    stats.total_messages_received
                );
                println!("│ Bytes sent          │ {:<19} │", stats.bytes_sent);
                println!("│ Bytes received      │ {:<19} │", stats.bytes_received);
                println!("│ Connected peers     │ {:<19} │", stats.connected_peers);
                println!("│ Uptime (seconds)    │ {:<19} │", stats.uptime_seconds);
                println!("└─────────────────────┴─────────────────────┘");
            }
            Err(e) => println!("❌ Error getting statistics: {}", e),
        }

        Ok(())
    }

    async fn handle_add_command(&self, args: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Error: System not initialized. Execute 'init' first.");
            return Ok(());
        }

        if args.is_empty() {
            println!("💡 Usage: add <sg-link>");
            return Ok(());
        }

        let sg_link = args.trim();

        if !sg_link.starts_with("sg://") {
            println!("❌ Error: Invalid SG link format. Link must start with 'sg://'");
            return Ok(());
        }

        if sg_link.len() < 10 {
            println!("❌ Error: SG link too short to be valid");
            return Ok(());
        }

        print!("🔄 Processing SG link...");
        io::stdout().flush()?;

        match self.core.add_contact_by_sg_link(sg_link).await {
            Ok(()) => {
                println!(" ✅ Contact added successfully!");
                println!("💡 Use 'contacts' command to view all contacts, or 'chat <name>' to start chatting.");
            }
            Err(e) => match e {
                crate::core::CoreError::Contact(msg) => {
                    if msg.contains("UTF-8 conversion failed") {
                        println!(" ❌ Error: SG link is corrupted or invalid");
                        println!("Ask the contact to create a new link");
                    } else if msg.contains("Decode error") {
                        println!(" ❌ Error: Failed to decode SG link");
                        println!("Check that the link was copied correctly");
                    } else if msg.contains("Cannot add yourself") {
                        println!(" ❌ Error: You cannot add yourself as a contact");
                    } else if msg.contains("already exists") {
                        println!(" ℹ️ Note: Contact already exists, updated existing record");
                    } else if msg.contains("JSON parse failed") {
                        println!(" ❌ Error: SG link contains invalid data");
                        println!("Ask the contact to create a new link");
                    } else {
                        println!(" ❌ Contact error: {}", msg);
                    }
                }
                _ => println!(" ❌ Error adding contact: {}", e),
            },
        }

        Ok(())
    }

    async fn list_contacts(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Error: System not initialized. Execute 'init' first.");
            return Ok(());
        }

        match self.core.get_contacts().await {
            Ok(contacts) => {
                if contacts.is_empty() {
                    println!("🔭 No contacts found. Use 'add <sg-link>' to add contacts.");
                } else {
                    println!("\n👥 Your contacts:");
                    println!("{}", "═".repeat(80));
                    for contact in contacts {
                        let is_online = self.core.check_contact_online(&contact.name).await;
                        let status = if is_online {
                            "🟢 Online"
                        } else {
                            "🔴 Offline"
                        };

                        let message_count =
                            match self.core.get_unread_message_count(&contact.name).await {
                                Ok(count) => {
                                    if count > 0 {
                                        format!(" 💬 ({} messages)", count)
                                    } else {
                                        String::new()
                                    }
                                }
                                Err(_) => String::new(),
                            };

                        println!(
                            "  📞 {} - {} ({}){}",
                            contact.name, contact.address, status, message_count
                        );
                    }
                    println!("{}", "═".repeat(80));
                    println!("💡 Use 'chat <contact-name>' to start chatting");
                }
            }
            Err(e) => println!("❌ Error getting contacts: {}", e),
        }

        Ok(())
    }

    async fn handle_chat_command(&self, args: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Error: System not initialized. Execute 'init' first.");
            return Ok(());
        }

        if args.is_empty() {
            println!("💡 Usage: chat <contact-name>");
            return Ok(());
        }

        let contact_name = args.trim();

        match self.core.get_contacts().await {
            Ok(contacts) => {
                if !contacts.iter().any(|c| c.name == contact_name) {
                    println!("❌ Error: Contact '{}' not found", contact_name);
                    println!("💡 Use 'contacts' command to view available contacts");
                    return Ok(());
                }

                let is_online = self.core.check_contact_online(contact_name).await;
                let status_msg = if is_online {
                    "🟢 Online"
                } else {
                    "🔴 Offline"
                };

                println!("💬 Entering chat with {} ({})", contact_name, status_msg);
                if !is_online {
                    println!(
                        "⚠️ Contact is offline. Messages will be delivered when they come online."
                    );
                }
                println!("💡 Special chat commands:");
                println!("   /history - show full history");
                println!("   exit - exit chat");

                if !self.core.is_server_started() {
                    println!("⚠️ Warning: Server not running. You won't receive replies until you execute 'start' command");
                }

                self.show_chat_history(contact_name).await;

                loop {
                    print!("{}> ", contact_name);
                    io::stdout().flush()?;

                    let mut message = String::new();
                    io::stdin().read_line(&mut message)?;
                    let message = message.trim();

                    if message.is_empty() {
                        continue;
                    }

                    if message.to_lowercase() == "exit" {
                        println!("👋 Exiting chat with {}", contact_name);
                        break;
                    }

                    if message == "/history" {
                        self.show_chat_history(contact_name).await;
                        continue;
                    }

                    match self.core.send_message(contact_name, message).await {
                        Ok(()) => {
                            println!("✅ Sent (checking delivery...)");

                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                            if let Ok(messages) = self.core.get_chat_messages(contact_name).await {
                                if let Some(last_msg) = messages.last() {
                                    match last_msg.delivery_status {
                                        crate::network::DeliveryStatus::Delivered => {
                                            println!("📨 Message delivered");
                                        }
                                        crate::network::DeliveryStatus::Failed => {
                                            println!("❌ Message failed to deliver");
                                        }
                                        crate::network::DeliveryStatus::Pending => {
                                            println!("⏳ Message pending...");
                                        }
                                        crate::network::DeliveryStatus::Sent => {
                                            println!("📤 Message sent, waiting for confirmation");
                                        }
                                        crate::network::DeliveryStatus::Read => {
                                            println!("✔️ Message read");
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => match e {
                            crate::core::CoreError::InvalidState(msg)
                                if msg.contains("Server not running") =>
                            {
                                println!(
                                    "❌ Error: Server not running. Execute 'start' command first."
                                );
                            }
                            crate::core::CoreError::Network(msg) => {
                                if msg.contains("Connection refused") || msg.contains("unavailable")
                                {
                                    println!("❌ Failed: {} unavailable", contact_name);
                                    println!("  (They may not have started their server yet)");
                                } else if msg.contains("timeout") {
                                    println!("❌ Failed: Connection timeout");
                                } else {
                                    println!("❌ Failed: {}", msg);
                                }
                            }
                            _ => println!("❌ Failed: {}", e),
                        },
                    }
                }
            }
            Err(e) => println!("❌ Error getting contacts: {}", e),
        }

        Ok(())
    }

    async fn show_chat_history(&self, contact_name: &str) {
        match self.core.get_chat_messages(contact_name).await {
            Ok(messages) => {
                if messages.is_empty() {
                    println!("🔭 No previous messages with {}", contact_name);
                } else {
                    println!("\n{}", "═".repeat(60));
                    println!("💬 Chat history with {}", contact_name);
                    println!("{}", "═".repeat(60));

                    for msg in messages.iter().rev().take(10).rev() {
                        let time = chrono::DateTime::from_timestamp(msg.timestamp as i64, 0)
                            .map(|dt| dt.format("%H:%M:%S").to_string())
                            .unwrap_or_else(|| "??:??:??".to_string());

                        let status_indicator = match msg.delivery_status {
                            crate::network::DeliveryStatus::Pending => "⏳",
                            crate::network::DeliveryStatus::Sent => "📤",
                            crate::network::DeliveryStatus::Delivered => "✅",
                            crate::network::DeliveryStatus::Failed => "❌",
                            crate::network::DeliveryStatus::Read => "👁️",
                        };

                        println!(
                            "[{}] {}: {} {}",
                            time, msg.from, msg.content, status_indicator
                        );
                    }

                    if messages.len() > 10 {
                        println!(
                            "... ({} more messages, type '/history' for full history)",
                            messages.len() - 10
                        );
                    }

                    println!("{}", "═".repeat(60));
                }
            }
            Err(e) => println!("❌ Error loading chat history: {}", e),
        }
    }

    async fn handle_ping_command(&self, args: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Error: System not initialized. Execute 'init' first.");
            return Ok(());
        }

        if args.is_empty() {
            println!("💡 Usage: ping <contact-name>");
            return Ok(());
        }

        let contact_name = args.trim();

        print!("📡 Pinging {}...", contact_name);
        io::stdout().flush()?;

        match self.core.get_contacts().await {
            Ok(contacts) => {
                if let Some(_contact) = contacts.iter().find(|c| c.name == contact_name) {
                    let start = std::time::Instant::now();
                    let is_online = self.core.check_contact_online(contact_name).await;
                    let elapsed = start.elapsed();

                    if is_online {
                        println!(" ✅ {} is online ({}ms)", contact_name, elapsed.as_millis());
                    } else {
                        println!(" ❌ {} is offline or unavailable", contact_name);
                        println!("  (Check if they started their server with 'start' command)");
                    }
                } else {
                    println!(" ❌ Contact '{}' not found", contact_name);
                }
            }
            Err(e) => println!(" ❌ Error getting contacts: {}", e),
        }

        Ok(())
    }

    async fn handle_name_command(&mut self, args: &str) -> Result<(), Box<dyn std::error::Error>> {
        if args.is_empty() {
            println!("💡 Usage: name <new-name>");
            return Ok(());
        }

        let new_name = args.trim().to_string();

        match self.core.update_user_name(new_name.clone()).await {
            Ok(()) => {
                println!("✅ Name updated to '{}'", new_name);
                println!("💡 Your new identity will be used for new connections");
            }
            Err(e) => println!("❌ Error updating name: {}", e),
        }

        Ok(())
    }
}
