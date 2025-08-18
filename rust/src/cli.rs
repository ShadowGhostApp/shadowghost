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
        println!("Введите 'help' для просмотра доступных команд");
        println!();

        if let Some(peer_info) = self.core.get_peer_info() {
            println!("👤 Готов как: {}", peer_info);
        }

        if !self.core.is_server_started() {
            println!(
                "⚠️  Сервер еще не запущен. Используйте команду 'start' для приема соединений."
            );
        }

        let event_bus = self.core.get_event_bus().clone();
        tokio::spawn(async move {
            let mut receiver = event_bus.subscribe();
            while let Ok(event) = receiver.recv().await {
                match event {
                    crate::events::AppEvent::Network(net_event) => match net_event {
                        crate::events::NetworkEvent::MessageReceived { message } => {
                            println!(
                                "\n💬 Новое сообщение от {}: {}",
                                message.from, message.content
                            );
                            print!("> ");
                            io::stdout().flush().unwrap();
                        }
                        crate::events::NetworkEvent::ContactAdded { contact } => {
                            println!("\n👥 Добавлен новый контакт: {}", contact.name);
                            print!("> ");
                            io::stdout().flush().unwrap();
                        }
                        crate::events::NetworkEvent::Error { error, context } => {
                            if let Some(ctx) = context {
                                println!("\n❌ Ошибка [{}]: {}", ctx, error);
                            } else {
                                println!("\n❌ Ошибка: {}", error);
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
                "help" | "помощь" => self.show_help(),
                "link" | "ссылка" => self.handle_link_command().await?,
                "contacts" | "контакты" => self.list_contacts().await?,
                "init" | "инит" => self.initialize_core().await?,
                "start" | "запуск" => self.start_server().await?,
                "stop" | "остановка" => self.stop_server().await?,
                "restart" | "перезапуск" => self.restart_server().await?,
                "quit" | "exit" | "q" | "выход" => {
                    println!("👋 До свидания!");
                    break;
                }
                "chat" | "чат" => self.handle_chat_command(args).await?,
                "add" | "добавить" => self.handle_add_command(args).await?,
                "ping" | "пинг" => self.handle_ping_command(args).await?,
                "status" | "статус" => self.show_status().await?,
                "stats" | "статистика" => self.show_network_stats().await?,
                "clear" | "очистить" => {
                    print!("\x1B[2J\x1B[1;1H");
                    io::stdout().flush()?;
                }
                "" => continue,
                _ => println!(
                    "❓ Неизвестная команда '{}'. Введите 'help' для просмотра доступных команд.",
                    command
                ),
            }
        }

        self.core.shutdown().await?;
        Ok(())
    }

    fn show_help(&self) {
        println!("\n📋 Доступные команды:");
        println!("┌─────────────────────────┬───────────────────────────────────────────────┐");
        println!("│ Команда                 │ Описание                                    │");
        println!("├─────────────────────────┼───────────────────────────────────────────────┤");
        println!("│ init                    │ Инициализировать приложение                 │");
        println!("│ start                   │ Запустить сервер (необходимо для приема)   │");
        println!("│ stop                    │ Остановить сервер                           │");
        println!("│ restart                 │ Перезапустить сервер                        │");
        println!("│ link                    │ Создать вашу ссылку для подключения        │");
        println!("│ add <sg-ссылка>         │ Добавить контакт по SG ссылке               │");
        println!("│ contacts                │ Показать все контакты                       │");
        println!("│ chat <имя-контакта>     │ Войти в чат с контактом                     │");
        println!("│ ping <имя-контакта>     │ Проверить, онлайн ли контакт                │");
        println!("│ status                  │ Показать текущий статус                     │");
        println!("│ stats                   │ Показать сетевую статистику                 │");
        println!("│ clear                   │ Очистить экран                              │");
        println!("│ help                    │ Показать эту справку                        │");
        println!("│ quit/exit/q             │ Выйти из приложения                         │");
        println!("└─────────────────────────┴───────────────────────────────────────────────┘");
    }

    async fn handle_link_command(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Ошибка: Система не инициализирована. Сначала выполните 'init'.");
            return Ok(());
        }

        match self.core.generate_sg_link().await {
            Ok(link) => self.display_link_for_copying(&link),
            Err(e) => println!("❌ Ошибка создания ссылки: {}", e),
        }

        Ok(())
    }

    fn display_link_for_copying(&self, link: &str) {
        println!("\n{}", "═".repeat(80));
        println!("🔗 ВАША ССЫЛКА ДЛЯ ПОДКЛЮЧЕНИЯ:");
        println!("{}", "═".repeat(80));
        println!();
        println!("   {}", link);
        println!();
        println!("{}", "═".repeat(80));
        println!("📋 ИНСТРУКЦИИ:");
        println!("1. Выделите ссылку выше мышью (тройной клик для выделения всего)");
        println!("2. Скопируйте с помощью Ctrl+C (Windows/Linux) или Cmd+C (Mac)");
        println!("3. Отправьте эту ссылку человеку, с которым хотите связаться");
        println!("4. Они должны использовать команду 'add <ваша-ссылка>' для добавления вас");
        println!("{}", "═".repeat(80));
        println!();

        print!("Нажмите Enter для продолжения...");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        println!("💡 Ссылка все еще доступна выше, если нужно скопировать ее снова.");
    }

    async fn initialize_core(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        print!("Введите ваше имя (по умолчанию: user): ");
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
                println!("✅ Система успешно инициализирована!");
                if let Some(peer_info) = self.core.get_peer_info() {
                    println!("👤 Вы теперь: {}", peer_info);
                }
            }
            Err(e) => println!("❌ Ошибка инициализации: {}", e),
        }

        Ok(())
    }

    async fn start_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Ошибка: Система не инициализирована. Сначала выполните 'init'.");
            return Ok(());
        }

        if self.core.is_server_started() {
            println!("ℹ️  Сервер уже работает.");
            return Ok(());
        }

        print!("🚀 Запуск сервера...");
        io::stdout().flush()?;

        match self.core.start_server().await {
            Ok(()) => {
                println!(" ✅ Сервер успешно запущен!");
                println!("📨 Теперь вы можете получать сообщения от других пользователей.");
            }
            Err(e) => println!(" ❌ Ошибка запуска сервера: {}", e),
        }

        Ok(())
    }

    async fn stop_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Ошибка: Система не инициализирована.");
            return Ok(());
        }

        if !self.core.is_server_started() {
            println!("ℹ️  Сервер уже остановлен.");
            return Ok(());
        }

        print!("🛑 Остановка сервера...");
        io::stdout().flush()?;

        match self.core.stop_server().await {
            Ok(()) => {
                println!(" ✅ Сервер успешно остановлен!");
                println!("⚠️  Вы больше не будете получать сообщения до перезапуска.");
            }
            Err(e) => println!(" ❌ Ошибка остановки сервера: {}", e),
        }

        Ok(())
    }

    async fn restart_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Ошибка: Система не инициализирована. Сначала выполните 'init'.");
            return Ok(());
        }

        print!("🔄 Перезапуск сервера...");
        io::stdout().flush()?;

        match self.core.restart_server().await {
            Ok(()) => {
                println!(" ✅ Сервер успешно перезапущен!");
            }
            Err(e) => println!(" ❌ Ошибка перезапуска сервера: {}", e),
        }

        Ok(())
    }

    async fn show_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n📊 Текущий статус:");
        println!("┌─────────────────────┬─────────────────────────────────┐");

        let init_status = if self.core.is_initialized() {
            "✅ Да"
        } else {
            "❌ Нет"
        };
        println!("│ Инициализировано    │ {:<31} │", init_status);

        let server_status = self.core.get_server_status().await;
        println!("│ Сервер              │ {:<31} │", server_status);

        if let Some(peer_info) = self.core.get_peer_info() {
            println!("│ Идентификация       │ {:<31} │", peer_info);
        }

        if self.core.is_initialized() {
            match self.core.get_contacts().await {
                Ok(contacts) => {
                    let online_count = contacts
                        .iter()
                        .filter(|c| matches!(c.status, crate::network::ContactStatus::Online))
                        .count();
                    println!(
                        "│ Контакты            │ {} (онлайн: {})              │",
                        contacts.len(),
                        online_count
                    );
                }
                Err(_) => println!("│ Контакты            │ Ошибка загрузки                 │"),
            }
        }

        println!("└─────────────────────┴─────────────────────────────────┘");

        Ok(())
    }

    async fn show_network_stats(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Ошибка: Система не инициализирована.");
            return Ok(());
        }

        match self.core.get_network_stats().await {
            Ok(stats) => {
                println!("\n📈 Сетевая статистика:");
                println!("┌─────────────────────┬─────────────────────┐");
                println!("│ Отправлено сообщений│ {:<19} │", stats.messages_sent);
                println!("│ Получено сообщений  │ {:<19} │", stats.messages_received);
                println!("│ Отправлено байт     │ {:<19} │", stats.bytes_sent);
                println!("│ Получено байт       │ {:<19} │", stats.bytes_received);
                println!("│ Всего соединений    │ {:<19} │", stats.total_connections);
                println!("│ Активных соединений │ {:<19} │", stats.active_connections);
                println!("└─────────────────────┴─────────────────────┘");
            }
            Err(e) => println!("❌ Ошибка получения статистики: {}", e),
        }

        Ok(())
    }

    async fn handle_add_command(&self, args: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Ошибка: Система не инициализирована. Сначала выполните 'init'.");
            return Ok(());
        }

        if args.is_empty() {
            println!("💡 Использование: add <sg-ссылка>");
            return Ok(());
        }

        let sg_link = args.trim();

        if !sg_link.starts_with("sg://") {
            println!("❌ Ошибка: Неверный формат SG ссылки. Ссылка должна начинаться с 'sg://'");
            return Ok(());
        }

        if sg_link.len() < 10 {
            println!("❌ Ошибка: SG ссылка слишком короткая для корректной");
            return Ok(());
        }

        print!("🔄 Обработка SG ссылки...");
        io::stdout().flush()?;

        match self.core.add_contact_by_sg_link(sg_link).await {
            Ok(()) => {
                println!(" ✅ Контакт успешно добавлен!");
                println!("💡 Используйте команду 'contacts' для просмотра всех контактов, или 'chat <имя>' для начала чата.");
            }
            Err(e) => match e {
                crate::core::CoreError::Contact(msg) => {
                    if msg.contains("UTF-8 conversion failed") {
                        println!(" ❌ Ошибка: SG ссылка повреждена или недействительна");
                        println!("Попросите контакт создать новую ссылку");
                    } else if msg.contains("Decode error") {
                        println!(" ❌ Ошибка: Не удалось декодировать SG ссылку");
                        println!("Проверьте, что ссылка была скопирована правильно");
                    } else if msg.contains("Cannot add yourself") {
                        println!(" ❌ Ошибка: Вы не можете добавить себя в качестве контакта");
                    } else if msg.contains("already exists") {
                        println!(" ℹ️  Примечание: Контакт уже существует, обновлена существующая запись");
                    } else if msg.contains("JSON parse failed") {
                        println!(" ❌ Ошибка: SG ссылка содержит недействительные данные");
                        println!("Попросите контакт создать новую ссылку");
                    } else {
                        println!(" ❌ Ошибка контакта: {}", msg);
                    }
                }
                _ => println!(" ❌ Ошибка добавления контакта: {}", e),
            },
        }

        Ok(())
    }

    async fn list_contacts(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Ошибка: Система не инициализирована. Сначала выполните 'init'.");
            return Ok(());
        }

        match self.core.get_contacts().await {
            Ok(contacts) => {
                if contacts.is_empty() {
                    println!("🔭 Контакты не найдены. Используйте 'add <sg-ссылка>' для добавления контактов.");
                } else {
                    println!("\n👥 Ваши контакты:");
                    println!("{}", "═".repeat(80));
                    for contact in contacts {
                        let status = match contact.status {
                            crate::network::ContactStatus::Online => "🟢 Онлайн",
                            crate::network::ContactStatus::Offline => "🔴 Оффлайн",
                            crate::network::ContactStatus::Blocked => "🚫 Заблокирован",
                        };

                        let message_count =
                            match self.core.get_unread_message_count(&contact.name).await {
                                Ok(count) => {
                                    if count > 0 {
                                        format!(" 💬 ({} сообщений)", count)
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
                    println!("💡 Используйте 'chat <имя-контакта>' для начала чата");
                }
            }
            Err(e) => println!("❌ Ошибка получения контактов: {}", e),
        }

        Ok(())
    }

    async fn handle_chat_command(&self, args: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Ошибка: Система не инициализирована. Сначала выполните 'init'.");
            return Ok(());
        }

        if args.is_empty() {
            println!("💡 Использование: chat <имя-контакта>");
            return Ok(());
        }

        let contact_name = args.trim();

        match self.core.get_contacts().await {
            Ok(contacts) => {
                if !contacts.iter().any(|c| c.name == contact_name) {
                    println!("❌ Ошибка: Контакт '{}' не найден", contact_name);
                    println!("💡 Используйте команду 'contacts' для просмотра доступных контактов");
                    return Ok(());
                }

                println!(
                    "💬 Вход в чат с {} (введите 'exit' для выхода)",
                    contact_name
                );
                println!("💡 Специальные команды в чате:");
                println!("   /history - показать полную историю");
                println!("   exit - выйти из чата");

                if !self.core.is_server_started() {
                    println!("⚠️  Предупреждение: Сервер не запущен. Вы не будете получать ответы до выполнения команды 'start'");
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
                        println!("👋 Выход из чата с {}", contact_name);
                        break;
                    }

                    if message == "/history" {
                        self.show_chat_history(contact_name).await;
                        continue;
                    }

                    match self.core.send_message(contact_name, message).await {
                        Ok(()) => {
                            println!("✅ Отправлено");
                        }
                        Err(e) => match e {
                            crate::core::CoreError::InvalidState(msg)
                                if msg.contains("Сервер не запущен") =>
                            {
                                println!("❌ Ошибка: Сервер не запущен. Сначала используйте команду 'start'.");
                            }
                            crate::core::CoreError::Network(msg) => {
                                if msg.contains("Connection refused") || msg.contains("недоступен")
                                {
                                    println!("❌ Неудача: {} недоступен", contact_name);
                                    println!("  (Возможно, они еще не запустили свой сервер)");
                                } else if msg.contains("timeout") || msg.contains("Таймаут")
                                {
                                    println!("❌ Неудача: Таймаут соединения");
                                } else {
                                    println!("❌ Неудача: {}", msg);
                                }
                            }
                            _ => println!("❌ Неудача: {}", e),
                        },
                    }
                }
            }
            Err(e) => println!("❌ Ошибка получения контактов: {}", e),
        }

        Ok(())
    }

    async fn show_chat_history(&self, contact_name: &str) {
        match self.core.get_chat_messages(contact_name).await {
            Ok(messages) => {
                if messages.is_empty() {
                    println!("🔭 Нет предыдущих сообщений с {}", contact_name);
                } else {
                    println!("\n{}", "═".repeat(60));
                    println!("💬 История чата с {}", contact_name);
                    println!("{}", "═".repeat(60));

                    for msg in messages.iter().rev().take(10).rev() {
                        let time = chrono::DateTime::from_timestamp(msg.timestamp as i64, 0)
                            .map(|dt| dt.format("%H:%M:%S").to_string())
                            .unwrap_or_else(|| "??:??:??".to_string());

                        let status_indicator = match msg.delivery_status {
                            crate::network::DeliveryStatus::Sent => "📤",
                            crate::network::DeliveryStatus::Delivered => "✅",
                            crate::network::DeliveryStatus::Failed => "❌",
                        };

                        println!(
                            "[{}] {}: {} {}",
                            time, msg.from, msg.content, status_indicator
                        );
                    }

                    if messages.len() > 10 {
                        println!(
                            "... (еще {} сообщений, введите '/history' для полной истории)",
                            messages.len() - 10
                        );
                    }

                    println!("{}", "═".repeat(60));
                }
            }
            Err(e) => println!("❌ Ошибка загрузки истории чата: {}", e),
        }
    }

    async fn handle_ping_command(&self, args: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.core.is_initialized() {
            println!("❌ Ошибка: Система не инициализирована. Сначала выполните 'init'.");
            return Ok(());
        }

        if args.is_empty() {
            println!("💡 Использование: ping <имя-контакта>");
            return Ok(());
        }

        let contact_name = args.trim();

        print!("🔍 Пинг {}...", contact_name);
        io::stdout().flush()?;

        match self.core.get_contacts().await {
            Ok(contacts) => {
                if let Some(contact) = contacts.iter().find(|c| c.name == contact_name) {
                    use std::time::Duration;
                    use tokio::net::TcpStream;

                    let start = std::time::Instant::now();
                    let result = tokio::time::timeout(
                        Duration::from_secs(3),
                        TcpStream::connect(&contact.address),
                    )
                    .await;

                    match result {
                        Ok(Ok(_)) => {
                            let elapsed = start.elapsed();
                            println!(" ✅ {} онлайн ({}мс)", contact_name, elapsed.as_millis());
                        }
                        Ok(Err(_)) => {
                            println!(" ❌ {} недоступен", contact_name);
                            println!("  💡 (Проверьте, запустили ли они сервер командой 'start')");
                        }
                        Err(_) => {
                            println!(" ⏰ {} таймаут соединения", contact_name);
                        }
                    }
                } else {
                    println!(" ❌ Контакт '{}' не найден", contact_name);
                }
            }
            Err(e) => println!(" ❌ Ошибка получения контактов: {}", e),
        }

        Ok(())
    }
}
