// SPDX-License-Identifier: MIT
// Adapted from discord-api-types (c) vladfrangu
// Rust port and modifications (c) 2025 andrewdotdev

use crate::models::payloads::_application_commands::internals::APIBaseApplicationCommandInteractionData;
use crate::models::payloads::_interactions::application_commands::APIApplicationCommandInteractionWrapper;
use crate::models::payloads::base::{APIDMInteractionWrapper, APIGuildInteractionWrapper};

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object-interaction-data>
pub type APIPrimaryEntryPointCommandInteractionData = APIBaseApplicationCommandInteractionData;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object>
pub type APIPrimaryEntryPointCommandInteraction =
    APIApplicationCommandInteractionWrapper<APIPrimaryEntryPointCommandInteractionData>;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object>
pub type APIPrimaryEntryPointCommandDMInteraction =
    APIDMInteractionWrapper<APIPrimaryEntryPointCommandInteraction>;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-object>
pub type APIPrimaryEntryPointCommandGuildInteraction =
    APIGuildInteractionWrapper<APIPrimaryEntryPointCommandInteraction>;
