//! Parsed LPA (Local Profile Assistant) data structure

use serde::{Deserialize, Serialize};

/// Individual LPA field
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LpaField {
    /// Field index (0-based)
    pub index: usize,
    /// Field name (if known)
    pub name: Option<String>,
    /// Field value
    pub value: String,
}

/// Parsed LPA payload
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ParsedLpa {
    /// SM-DP+ (Service Module Descriptor - Provisioning) address
    pub smdp: Option<String>,
    /// Activation code
    pub activation_code: Option<String>,
    /// Confirmation code (if provided)
    pub confirmation_code: Option<String>,
    /// Other optional fields
    pub other: Vec<LpaField>,
    /// Original raw format detected
    pub format: Option<String>,
}

impl std::fmt::Display for ParsedLpa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(smdp) = &self.smdp {
            writeln!(f, "SM-DP+: {}", smdp)?;
        }
        if let Some(ac) = &self.activation_code {
            writeln!(f, "Activation Code: {}", ac)?;
        }
        if let Some(cc) = &self.confirmation_code {
            writeln!(f, "Confirmation Code: {}", cc)?;
        }
        for field in &self.other {
            if let Some(name) = &field.name {
                writeln!(f, "{}: {}", name, field.value)?;
            } else {
                writeln!(f, "Field[{}]: {}", field.index, field.value)?;
            }
        }
        Ok(())
    }
}

impl ParsedLpa {
    /// Create a simple parsed LPA with just the activation code
    pub fn simple(activation_code: String) -> Self {
        Self {
            activation_code: Some(activation_code),
            ..Default::default()
        }
    }

    /// Check if this is a valid (non-empty) parsed LPA
    pub fn is_valid(&self) -> bool {
        self.smdp.is_some() || self.activation_code.is_some()
    }
}
