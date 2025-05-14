use std::fmt;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

// Define your enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum UserType {
    Member,
    Proctor,
    Admin,
}

// Implement the Display trait to convert enum variants to strings
impl fmt::Display for UserType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserType::Member => write!(f, "member"),
            UserType::Proctor => write!(f, "proctor"),
            UserType::Admin => write!(f, "admin"),
        }
    }
}
