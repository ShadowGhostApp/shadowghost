use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

pub use crate::network::{Contact, ContactStatus, TrustLevel};

#[derive(Debug)]
pub enum ContactError {
    InvalidContact(String),
    ContactNotFound(String),
    ContactExists(String),
    SerializationError(String),
    IoError(String),
}

impl fmt::Display for ContactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContactError::InvalidContact(msg) => write!(f, "Invalid contact: {}", msg),
            ContactError::ContactNotFound(msg) => write!(f, "Contact not found: {}", msg),
            ContactError::ContactExists(msg) => write!(f, "Contact exists: {}", msg),
            ContactError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            ContactError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl Error for ContactError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInteractionStats {
    pub contact_id: String,
    pub days_since_last_seen: u32,
    pub is_trusted: bool,
    pub is_blocked: bool,
    pub current_status: ContactStatus,
    pub trust_level: TrustLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactOptimizationResult {
    pub original_contact_count: usize,
    pub final_contact_count: usize,
    pub duplicates_removed: u32,
    pub orphaned_blocks_cleaned: u32,
    pub optimization_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactValidationIssue {
    pub contact_id: String,
    pub issue_type: ContactIssueType,
    pub description: String,
    pub severity: IssueSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContactIssueType {
    EmptyName,
    EmptyAddress,
    InvalidAddress,
    DuplicateContact,
    OrphanedBlock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactSearchQuery {
    pub name: Option<String>,
    pub address: Option<String>,
    pub status: Option<ContactStatus>,
    pub trust_level: Option<TrustLevel>,
    pub is_blocked: Option<bool>,
    pub last_seen_after: Option<DateTime<Utc>>,
    pub last_seen_before: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactGroup {
    pub id: String,
    pub name: String,
    pub contact_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactImportResult {
    pub imported_count: u32,
    pub skipped_count: u32,
    pub updated_count: u32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactExportOptions {
    pub format: ContactExportFormat,
    pub include_blocked: bool,
    pub trust_level_filter: Option<TrustLevel>,
    pub status_filter: Option<ContactStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContactExportFormat {
    Json,
    Csv,
    Vcf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactStats {
    pub total_contacts: usize,
    pub online_contacts: usize,
    pub trusted_contacts: usize,
    pub blocked_contacts: usize,
    pub pending_contacts: usize,
}

impl Default for ContactSearchQuery {
    fn default() -> Self {
        Self {
            name: None,
            address: None,
            status: None,
            trust_level: None,
            is_blocked: None,
            last_seen_after: None,
            last_seen_before: None,
        }
    }
}

impl ContactGroup {
    pub fn new(name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            contact_ids: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn add_contact(&mut self, contact_id: String) {
        if !self.contact_ids.contains(&contact_id) {
            self.contact_ids.push(contact_id);
            self.updated_at = Utc::now();
        }
    }

    pub fn remove_contact(&mut self, contact_id: &str) {
        if let Some(pos) = self.contact_ids.iter().position(|id| id == contact_id) {
            self.contact_ids.remove(pos);
            self.updated_at = Utc::now();
        }
    }

    pub fn contains_contact(&self, contact_id: &str) -> bool {
        self.contact_ids.contains(&contact_id.to_string())
    }
}
