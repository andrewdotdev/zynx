// SPDX-License-Identifier: MIT
// Adapted from discord-api-types (c) vladfrangu
// Rust port and modifications (c) 2025 andrewdotdev

use crate::models::payloads::responses::InteractionType;
use serde::{Deserialize, Serialize};

/// Verification PING does not contain the fields of a command interaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct APIPingInteraction {
    pub id: String,
    pub application_id: String,
    pub r#type: InteractionType,
    pub token: String,
    pub version: u8,
}
