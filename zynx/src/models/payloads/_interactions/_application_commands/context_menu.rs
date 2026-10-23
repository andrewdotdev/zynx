// SPDX-License-Identifier: MIT
// Adapted from discord-api-types (c) vladfrangu
// Rust port and modifications (c) 2025 andrewdotdev

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::payloads::_application_commands::internals::APIBaseApplicationCommandInteractionData;
use crate::models::payloads::application_commands::APIApplicationCommandInteractionWrapper;
use crate::models::payloads::base::{
    APIDMInteractionWrapper, APIGuildInteractionWrapper, APIUserInteractionDataResolved,
};
use crate::models::payloads::channel::APIMessage;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-data>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct APIUserApplicationCommandInteractionData {
    #[serde(flatten)]
    pub base: APIBaseApplicationCommandInteractionData,
    pub target_id: String,
    pub resolved: APIUserInteractionDataResolved,
}

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-data>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct APIMessageApplicationCommandInteractionData {
    #[serde(flatten)]
    pub base: APIBaseApplicationCommandInteractionData,
    pub target_id: String,
    pub resolved: APIMessageApplicationCommandInteractionDataResolved,
}

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-resolved-data-structure>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct APIMessageApplicationCommandInteractionDataResolved {
    pub messages: HashMap<String, APIMessage>,
}

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-data>
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum APIContextMenuInteractionData {
    APIMessageApplicationCommandInteractionData(APIMessageApplicationCommandInteractionData),
    APIUserApplicationCommandInteractionData(APIUserApplicationCommandInteractionData),
}

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object>
pub type APIUserApplicationCommandInteraction =
    APIApplicationCommandInteractionWrapper<APIUserApplicationCommandInteractionData>;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object>
pub type APIUserApplicationCommandDMInteraction =
    APIDMInteractionWrapper<APIUserApplicationCommandInteraction>;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object>
pub type APIUserApplicationCommandGuildInteraction =
    APIGuildInteractionWrapper<APIUserApplicationCommandInteraction>;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object>
pub type APIMessageApplicationCommandInteraction =
    APIApplicationCommandInteractionWrapper<APIMessageApplicationCommandInteractionData>;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object>
pub type APIMessageApplicationCommandDMInteraction =
    APIDMInteractionWrapper<APIMessageApplicationCommandInteraction>;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object>
pub type APIMessageApplicationCommandGuildInteraction =
    APIGuildInteractionWrapper<APIMessageApplicationCommandInteraction>;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object>
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum APIContextMenuInteraction {
    APIMessageApplicationCommandInteraction(APIMessageApplicationCommandInteraction),
    APIUserApplicationCommandInteraction(APIUserApplicationCommandInteraction),
}

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object>
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum APIContextMenuDMInteraction {
    APIMessageApplicationCommandDMInteraction(APIMessageApplicationCommandDMInteraction),
    APIUserApplicationCommandDMInteraction(APIUserApplicationCommandDMInteraction),
}

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object>
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum APIContextMenuGuildInteraction {
    APIMessageApplicationCommandGuildInteraction(APIMessageApplicationCommandGuildInteraction),
    APIUserApplicationCommandGuildInteraction(APIUserApplicationCommandGuildInteraction),
}

crate::utils::serde::numeric_union!(APIContextMenuInteractionData, "/type", {
    2 => APIUserApplicationCommandInteractionData,
    3 => APIMessageApplicationCommandInteractionData,
});

crate::utils::serde::numeric_union!(APIContextMenuInteraction, "/data/type", {
    2 => APIUserApplicationCommandInteraction,
    3 => APIMessageApplicationCommandInteraction,
});

crate::utils::serde::numeric_union!(APIContextMenuDMInteraction, "/data/type", {
    2 => APIUserApplicationCommandDMInteraction,
    3 => APIMessageApplicationCommandDMInteraction,
});

crate::utils::serde::numeric_union!(APIContextMenuGuildInteraction, "/data/type", {
    2 => APIUserApplicationCommandGuildInteraction,
    3 => APIMessageApplicationCommandGuildInteraction,
});
