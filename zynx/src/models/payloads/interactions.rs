// SPDX-License-Identifier: MIT
// Adapted from discord-api-types (c) vladfrangu
// Rust port and modifications (c) 2025 andrewdotdev

use serde::Serialize;

pub use crate::models::payloads::_interactions::application_commands::*;
pub use crate::models::payloads::_interactions::autocomplete::*;
pub use crate::models::payloads::_interactions::base::*;
pub use crate::models::payloads::_interactions::message_components::*;
pub use crate::models::payloads::_interactions::modal_submit::*;
pub use crate::models::payloads::_interactions::ping::*;
pub use crate::models::payloads::_interactions::responses::*;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object>
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum APIInteraction {
    APIApplicationCommandAutocompleteInteraction(APIApplicationCommandAutocompleteInteraction),
    APIApplicationCommandInteraction(APIApplicationCommandInteraction),
    APIMessageComponentInteraction(APIMessageComponentInteraction),
    APIModalSubmitInteraction(APIModalSubmitInteraction),
    APIPingInteraction(APIPingInteraction),
}

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object>
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum APIDMInteraction {
    APIApplicationCommandAutocompleteDMInteraction(APIApplicationCommandAutocompleteDMInteraction),
    APIApplicationCommandDMInteraction(APIApplicationCommandDMInteraction),
    APIMessageComponentDMInteraction(APIMessageComponentDMInteraction),
    APIModalSubmitDMInteraction(APIModalSubmitDMInteraction),
}

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object>
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum APIGuildInteraction {
    APIApplicationCommandAutocompleteGuildInteraction(
        APIApplicationCommandAutocompleteGuildInteraction,
    ),
    APIApplicationCommandGuildInteraction(APIApplicationCommandGuildInteraction),
    APIMessageComponentGuildInteraction(APIMessageComponentGuildInteraction),
    APIModalSubmitGuildInteraction(APIModalSubmitGuildInteraction),
}

crate::utils::serde::numeric_union!(APIInteraction, "/type", {
    2 => APIApplicationCommandInteraction,
    3 => APIMessageComponentInteraction,
    4 => APIApplicationCommandAutocompleteInteraction,
    5 => APIModalSubmitInteraction,
    1 => APIPingInteraction,
});

crate::utils::serde::numeric_union!(APIDMInteraction, "/type", {
    2 => APIApplicationCommandDMInteraction,
    3 => APIMessageComponentDMInteraction,
    4 => APIApplicationCommandAutocompleteDMInteraction,
    5 => APIModalSubmitDMInteraction,
});

crate::utils::serde::numeric_union!(APIGuildInteraction, "/type", {
    2 => APIApplicationCommandGuildInteraction,
    3 => APIMessageComponentGuildInteraction,
    4 => APIApplicationCommandAutocompleteGuildInteraction,
    5 => APIModalSubmitGuildInteraction,
});
