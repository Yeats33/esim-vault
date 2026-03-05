//! Core data models for eSIM profiles

mod lpa;
mod profile;

pub use lpa::{ParsedLpa, LpaField};
pub use profile::{Profile, ProfileStatus};

use serde::{Deserialize, Serialize};

/// The main vault containing all eSIM profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    /// Vault format version
    pub version: u32,
    /// All stored eSIM profiles
    pub profiles: Vec<Profile>,
}

impl Default for Vault {
    fn default() -> Self {
        Self {
            version: 1,
            profiles: Vec::new(),
        }
    }
}

impl Vault {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_profile(&mut self, profile: Profile) {
        self.profiles.push(profile);
    }

    pub fn get_profile(&self, id: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.id == id)
    }

    pub fn get_profile_mut(&mut self, id: &str) -> Option<&mut Profile> {
        self.profiles.iter_mut().find(|p| p.id == id)
    }

    pub fn remove_profile(&mut self, id: &str) -> Option<Profile> {
        if let Some(pos) = self.profiles.iter().position(|p| p.id == id) {
            Some(self.profiles.remove(pos))
        } else {
            None
        }
    }
}
