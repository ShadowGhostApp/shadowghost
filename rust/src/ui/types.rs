use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum CliCommand {
    Help,
    Init,
    Start,
    Stop,
    Restart,
    Link,
    Add(String),
    Chat(String),
    Contacts,
    Ping(String),
    Status,
    Stats,
    Name(String),
    Connection,
    UpdateIp,
    Clear,
    Quit,
    Unknown(String),
}

impl CliCommand {
    pub fn parse(input: &str) -> Self {
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let command = parts[0].to_lowercase();
        let args = if parts.len() > 1 { parts[1] } else { "" };

        match command.as_str() {
            "help" => CliCommand::Help,
            "init" => CliCommand::Init,
            "start" => CliCommand::Start,
            "stop" => CliCommand::Stop,
            "restart" => CliCommand::Restart,
            "link" => CliCommand::Link,
            "add" => CliCommand::Add(args.to_string()),
            "chat" => CliCommand::Chat(args.to_string()),
            "contacts" => CliCommand::Contacts,
            "ping" => CliCommand::Ping(args.to_string()),
            "status" => CliCommand::Status,
            "stats" => CliCommand::Stats,
            "name" => CliCommand::Name(args.to_string()),
            "connection" => CliCommand::Connection,
            "update-ip" => CliCommand::UpdateIp,
            "clear" => CliCommand::Clear,
            "quit" | "exit" | "q" => CliCommand::Quit,
            _ => CliCommand::Unknown(command),
        }
    }
}

// UI Display types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactDisplay {
    pub name: String,
    pub address: String,
    pub status: ContactStatusDisplay,
    pub unread_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContactStatusDisplay {
    Online,
    Offline,
    Away,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDisplay {
    pub timestamp: String,
    pub sender: String,
    pub content: String,
    pub status: MessageStatusDisplay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageStatusDisplay {
    Pending,
    Sent,
    Delivered,
    Failed,
    Read,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatsDisplay {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connected_peers: u32,
    pub uptime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusDisplay {
    pub initialized: bool,
    pub server_running: bool,
    pub identity: Option<String>,
    pub contacts_count: usize,
    pub online_contacts: usize,
}

// Chat session types
#[derive(Debug, Clone)]
pub struct ChatSession {
    pub contact_name: String,
    pub start_time: std::time::Instant,
    pub message_count: u32,
    pub last_activity: std::time::Instant,
}

impl ChatSession {
    pub fn new(contact_name: String) -> Self {
        let now = std::time::Instant::now();
        Self {
            contact_name,
            start_time: now,
            message_count: 0,
            last_activity: now,
        }
    }

    pub fn update_activity(&mut self) {
        self.last_activity = std::time::Instant::now();
        self.message_count += 1;
    }

    pub fn get_duration(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn get_idle_time(&self) -> Duration {
        self.last_activity.elapsed()
    }
}

// Command result types
#[derive(Debug, Clone)]
pub enum CommandResult {
    Success(String),
    Warning(String),
    Error(String),
    Info(String),
    None,
}

impl CommandResult {
    pub fn format(&self) -> String {
        match self {
            CommandResult::Success(msg) => format!("✅ {}", msg),
            CommandResult::Warning(msg) => format!("⚠️ {}", msg),
            CommandResult::Error(msg) => format!("❌ {}", msg),
            CommandResult::Info(msg) => format!("ℹ️ {}", msg),
            CommandResult::None => String::new(),
        }
    }
}

pub struct CliFormatter;

impl CliFormatter {
    pub fn format_table_header(title: &str, width: usize) -> String {
        let border = "═".repeat(width);
        format!("{}\n{}\n{}", border, title, border)
    }

    pub fn format_table_row(left: &str, right: &str, width: usize) -> String {
        let padding = width.saturating_sub(left.len() + right.len() + 4);
        format!("│ {} │ {}{} │", left, right, " ".repeat(padding))
    }

    pub fn format_table_footer(width: usize) -> String {
        format!("{}", "└".repeat(width))
    }

    pub fn format_link_display(link: &str) -> String {
        let border = "═".repeat(80);
        format!(
            "\n{}\n🔗 YOUR CONNECTION LINK:\n{}\n\n   {}\n\n{}\n📋 INSTRUCTIONS:\n1. Select the link above with mouse (triple-click to select all)\n2. Copy with Ctrl+C (Windows/Linux) or Cmd+C (Mac)\n3. Send this link to the person you want to connect with\n4. They should use 'add <your-link>' command to add you\n{}\n",
            border, border, link, border, border
        )
    }

    pub fn format_contact_list(contacts: &[ContactDisplay]) -> String {
        if contacts.is_empty() {
            return "🔭 No contacts found. Use 'add <sg-link>' to add contacts.".to_string();
        }

        let mut output = String::from("\n👥 Your contacts:\n");
        output.push_str(&"═".repeat(80));
        output.push('\n');

        for contact in contacts {
            let status_icon = match contact.status {
                ContactStatusDisplay::Online => "🟢 Online",
                ContactStatusDisplay::Offline => "🔴 Offline",
                _ => "⚪ Unknown",
            };

            let unread_indicator = if contact.unread_count > 0 {
                format!(" 💬 ({} messages)", contact.unread_count)
            } else {
                String::new()
            };

            output.push_str(&format!(
                "  📞 {} - {} ({}){}",
                contact.name, contact.address, status_icon, unread_indicator
            ));
            output.push('\n');
        }

        output.push_str(&"═".repeat(80));
        output.push_str("\n💡 Use 'chat <contact-name>' to start chatting");
        output
    }

    pub fn format_chat_history(messages: &[MessageDisplay], contact_name: &str) -> String {
        if messages.is_empty() {
            return format!("🔭 No previous messages with {}", contact_name);
        }

        let mut output = String::new();
        output.push_str(&"═".repeat(60));
        output.push('\n');
        output.push_str(&format!("💬 Chat history with {}", contact_name));
        output.push('\n');
        output.push_str(&"═".repeat(60));
        output.push('\n');

        // Show last 10 messages
        let display_messages = if messages.len() > 10 {
            &messages[messages.len() - 10..]
        } else {
            messages
        };

        for msg in display_messages {
            let status_indicator = match msg.status {
                MessageStatusDisplay::Pending => "⏳",
                MessageStatusDisplay::Sent => "📤",
                MessageStatusDisplay::Delivered => "✅",
                MessageStatusDisplay::Failed => "❌",
                MessageStatusDisplay::Read => "👁️",
            };

            output.push_str(&format!(
                "[{}] {}: {} {}",
                msg.timestamp, msg.sender, msg.content, status_indicator
            ));
            output.push('\n');
        }

        if messages.len() > 10 {
            output.push_str(&format!(
                "... ({} more messages, type '/history' for full history)",
                messages.len() - 10
            ));
            output.push('\n');
        }

        output.push_str(&"═".repeat(60));
        output
    }

    pub fn format_network_stats(stats: &NetworkStatsDisplay) -> String {
        format!(
            "📈 Network statistics:\n\
             ┌─────────────────────┬─────────────────────┐\n\
             │ Messages sent       │ {:<19} │\n\
             │ Messages received   │ {:<19} │\n\
             │ Bytes sent          │ {:<19} │\n\
             │ Bytes received      │ {:<19} │\n\
             │ Connected peers     │ {:<19} │\n\
             │ Uptime              │ {:<19} │\n\
             └─────────────────────┴─────────────────────┘",
            stats.messages_sent,
            stats.messages_received,
            stats.bytes_sent,
            stats.bytes_received,
            stats.connected_peers,
            stats.uptime
        )
    }
}

pub struct InputValidator;

impl InputValidator {
    pub fn validate_contact_name(name: &str) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Contact name cannot be empty".to_string());
        }

        if name.len() > 50 {
            return Err("Contact name too long (max 50 characters)".to_string());
        }

        if name.contains(&['\n', '\r', '\t']) {
            return Err("Contact name contains invalid characters".to_string());
        }

        Ok(())
    }

    pub fn validate_sg_link(link: &str) -> Result<(), String> {
        if !link.starts_with("sg://") {
            return Err("SG link must start with sg://".to_string());
        }

        if link.len() < 10 {
            return Err("SG link too short".to_string());
        }

        let encoded_part = &link[5..];
        if encoded_part.is_empty() {
            return Err("SG link missing data".to_string());
        }

        Ok(())
    }

    pub fn validate_message_content(content: &str) -> Result<(), String> {
        if content.trim().is_empty() {
            return Err("Message cannot be empty".to_string());
        }

        if content.len() > 1000 {
            return Err("Message too long (max 1000 characters)".to_string());
        }

        Ok(())
    }
}