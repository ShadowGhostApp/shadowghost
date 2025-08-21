pub async fn send_message(&self, contact_name: &str, content: &str) -> Result<(), CoreError> {
    if !self.is_initialized {
        return Err(CoreError::InvalidState("Core not initialized".to_string()));
    }

    let mut network = self.network_manager.write().await;
    if !network.is_running() {
        return Err(CoreError::InvalidState("Server not running".to_string()));
    }

    let contacts = self.contact_manager.read().await;
    let contact = contacts.get_contact_by_name(contact_name).await;
    if contact.is_none() {
        return Err(CoreError::Contact(format!(
            "Contact '{}' not found",
            contact_name
        )));
    }
    drop(contacts);

    let message_id = network
        .send_chat_message_by_name(contact_name, content)
        .await?;

    let message = ChatMessage {
        id: message_id,
        from: self
            .user_name
            .as_ref()
            .unwrap_or(&"user".to_string())
            .clone(),
        to: contact_name.to_string(),
        content: content.to_string(),
        msg_type: crate::network::ChatMessageType::Text,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        delivery_status: crate::network::DeliveryStatus::Sent,
    };

    let mut storage = self.storage_manager.write().await;
    storage.save_message(contact_name, &message).await?;

    Ok(())
}

pub async fn start_server(&mut self) -> Result<(), CoreError> {
    if !self.is_initialized {
        return Err(CoreError::InvalidState("Core not initialized".to_string()));
    }

    let mut network = self.network_manager.write().await;
    network.start_server().await?;
    Ok(())
}

pub async fn stop_server(&mut self) -> Result<(), CoreError> {
    let mut network = self.network_manager.write().await;
    network.shutdown().await?;
    Ok(())
}
