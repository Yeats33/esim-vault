//! eSIM profile data structure

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::ParsedLpa;

/// Status of an eSIM profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProfileStatus {
    /// Not yet installed/used
    #[default]
    Unused,
    /// Currently active or previously used
    Used,
    /// Expired or depleted
    Expired,
}

impl std::fmt::Display for ProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileStatus::Unused => write!(f, "unused"),
            ProfileStatus::Used => write!(f, "used"),
            ProfileStatus::Expired => write!(f, "expired"),
        }
    }
}

impl std::str::FromStr for ProfileStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "unused" => Ok(ProfileStatus::Unused),
            "used" => Ok(ProfileStatus::Used),
            "expired" => Ok(ProfileStatus::Expired),
            _ => Err(format!("Invalid status: {}", s)),
        }
    }
}

/// An eSIM profile containing activation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Unique identifier (UUID v4)
    pub id: String,
    /// User-friendly label (e.g., "Airalo Japan 10GB")
    pub label: String,
    /// Service provider name
    pub provider: Option<String>,
    /// Region/country tags
    pub region_tags: Vec<String>,
    /// Current status
    pub status: ProfileStatus,
    /// Creation timestamp (RFC3339)
    pub created_at: DateTime<Utc>,
    /// Last update timestamp (RFC3339)
    pub updated_at: DateTime<Utc>,
    /// Raw LPA payload string
    pub lpa_payload_raw: String,
    /// Parsed LPA fields (optional, computed on demand)
    pub parsed: Option<ParsedLpa>,
    /// User notes
    pub notes: Option<String>,
}

impl Profile {
    /// Create a new profile with a generated UUID and current timestamp
    pub fn new(label: String, lpa_payload: String) -> Self {
        let now = Utc::now();
        let parsed = crate::parser::parse_lpa(&lpa_payload).ok();

        Self {
            id: Uuid::new_v4().to_string(),
            label,
            provider: parsed.as_ref().and_then(|p| p.smdp.clone()),
            region_tags: Vec::new(),
            status: ProfileStatus::Unused,
            created_at: now,
            updated_at: now,
            lpa_payload_raw: lpa_payload,
            parsed,
            notes: None,
        }
    }

    /// Update the profile's label
    pub fn set_label(&mut self, label: String) {
        self.label = label;
        self.updated_at = Utc::now();
    }

    /// Update the profile's status
    pub fn set_status(&mut self, status: ProfileStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Add a region tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.region_tags.contains(&tag) {
            self.region_tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// Remove a region tag
    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(pos) = self.region_tags.iter().position(|t| t == tag) {
            self.region_tags.remove(pos);
            self.updated_at = Utc::now();
        }
    }

    /// Set notes
    pub fn set_notes(&mut self, notes: Option<String>) {
        self.notes = notes;
        self.updated_at = Utc::now();
    }

    /// Re-parse the LPA payload
    pub fn reparse(&mut self) {
        self.parsed = crate::parser::parse_lpa(&self.lpa_payload_raw).ok();
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_creation() {
        let profile = Profile::new(
            "Test Profile".to_string(),
            "LPA:1$sm-dp.plus$ABC123".to_string(),
        );

        assert!(!profile.id.is_empty());
        assert_eq!(profile.label, "Test Profile");
        assert_eq!(profile.status, ProfileStatus::Unused);
        assert!(profile.parsed.is_some());
    }

    #[test]
    fn test_profile_status_from_str() {
        assert_eq!(
            "unused".parse::<ProfileStatus>().unwrap(),
            ProfileStatus::Unused
        );
        assert_eq!(
            "USED".parse::<ProfileStatus>().unwrap(),
            ProfileStatus::Used
        );
        assert!("invalid".parse::<ProfileStatus>().is_err());
    }
}
