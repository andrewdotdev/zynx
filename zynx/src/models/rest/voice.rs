// SPDX-License-Identifier: MIT
// Adapted from discord-api-types (c) vladfrangu
// Rust port and modifications (c) 2025 andrewdotdev

use crate::models::payloads::{APIVoiceRegion, APIVoiceState};
use serde::{Deserialize, Serialize};

/// See <https://discord.com/developers/docs/resources/voice#list-voice-regions>
pub type RESTGetAPIVoiceRegionsResult = Vec<APIVoiceRegion>;

/// **Deprecated:** This was exported with the wrong name, use [`RESTGetAPIVoiceRegionsResult`](crate::models::rest::voice::RESTGetAPIVoiceRegionsResult) instead
#[deprecated(
    note = "This was exported with the wrong name, use RESTGetAPIVoiceRegionsResult instead"
)]
pub type GetAPIVoiceRegionsResult = RESTGetAPIVoiceRegionsResult;

/// See <https://discord.com/developers/docs/resources/voice#get-current-user-voice-state>
pub type RESTGetAPIGuildVoiceStateCurrentMemberResult = APIVoiceState;

/// See <https://discord.com/developers/docs/resources/voice#get-user-voice-state>
pub type RESTGetAPIGuildVoiceStateUserResult = APIVoiceState;

/// See <https://discord.com/developers/docs/resources/voice#modify-current-user-voice-state>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RESTPatchAPIGuildVoiceStateCurrentMemberJSONBody {
    /// The id of the channel the user is currently in
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    /// Toggles the user's suppress state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress: Option<bool>,
    /// Sets the user's request to speak
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_to_speak_timestamp: Option<Option<String>>,
}

/// See <https://discord.com/developers/docs/resources/voice#modify-current-user-voice-state>
pub type RESTPatchAPIGuildVoiceStateCurrentMemberResult = ();

/// See <https://discord.com/developers/docs/resources/voice#modify-user-voice-state>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RESTPatchAPIGuildVoiceStateUserJSONBody {
    /// The id of the channel the user is currently in
    pub channel_id: String,
    /// Toggles the user's suppress state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress: Option<bool>,
}

/// See <https://discord.com/developers/docs/resources/voice#modify-user-voice-state>
pub type RESTPatchAPIGuildVoiceStateUserResult = ();
