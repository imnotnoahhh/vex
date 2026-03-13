use crate::error::{Result, VexError};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Text,
    Json,
}

impl OutputMode {
    pub fn from_json_flag(json: bool) -> Self {
        if json {
            Self::Json
        } else {
            Self::Text
        }
    }
}

pub fn print_json<T: Serialize>(value: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| VexError::Parse(format!("Failed to serialize JSON output: {}", e)))?;
    println!("{}", json);
    Ok(())
}
