// SPDX-License-Identifier: MIT
// Adapted from discord-api-types (c) vladfrangu
// Rust port and modifications (c) 2025 andrewdotdev

use super::webhook::{
    RESTDeleteAPIWebhookWithTokenMessageResult, RESTGetAPIWebhookWithTokenMessageResult,
    RESTPatchAPIWebhookWithTokenMessageFormDataBody, RESTPatchAPIWebhookWithTokenMessageJSONBody,
    RESTPatchAPIWebhookWithTokenMessageResult, RESTPostAPIWebhookWithTokenWaitResult,
};
use crate::models::payloads::{
    APIApplicationCommand, APIApplicationCommandPermission, APIGuildApplicationCommandPermissions,
    APIInteractionResponse, APIInteractionResponseCallbackData, APIMessage, ApplicationCommandType,
    InteractionResponseType, InteractionType,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// See <https://discord.com/developers/docs/interactions/application-commands#get-global-application-commands>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RESTGetAPIApplicationCommandsQuery {
    /// Whether to include full localization dictionaries (name_localizations and description_localizations)
    /// in the returned objects, instead of the name_localized and description_localized fields.
    ///
    /// **Default:** `false`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_localizations: Option<bool>,
}

/// See <https://discord.com/developers/docs/interactions/application-commands#get-global-application-commands>
pub type RESTGetAPIApplicationCommandsResult = Vec<APIApplicationCommand>;

/// See <https://discord.com/developers/docs/interactions/application-commands#get-global-application-command>
pub type RESTGetAPIApplicationCommandResult = APIApplicationCommand;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RESTPostAPIBaseApplicationCommandsJSONBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contexts: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_member_permissions: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integration_types: Option<Value>,
    #[serde(flatten)]
    pub other: Value,
}

/// See <https://discord.com/developers/docs/interactions/application-commands#create-global-application-command>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTPostAPIChatInputApplicationCommandsJSONBody {
    #[serde(flatten)]
    pub base: RESTPostAPIBaseApplicationCommandsJSONBody,
    pub description: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<ApplicationCommandType>,
}

/// See <https://discord.com/developers/docs/interactions/application-commands#create-global-application-command>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTPostAPIContextMenuApplicationCommandsJSONBody {
    #[serde(flatten)]
    pub base: RESTPostAPIBaseApplicationCommandsJSONBody,
    #[serde(rename = "type")]
    pub r#type: ApplicationCommandType,
}

/// See <https://discord.com/developers/docs/interactions/application-commands#create-global-application-command>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTPostAPIPrimaryEntryPointApplicationCommandJSONBody {
    #[serde(flatten)]
    pub base: RESTPostAPIBaseApplicationCommandsJSONBody,
    #[serde(rename = "type")]
    pub r#type: ApplicationCommandType,
}

/// See <https://discord.com/developers/docs/interactions/application-commands#create-global-application-command>
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum RESTPostAPIApplicationCommandsJSONBody {
    ChatInput(RESTPostAPIChatInputApplicationCommandsJSONBody),
    ContextMenu(RESTPostAPIContextMenuApplicationCommandsJSONBody),
    PrimaryEntryPoint(RESTPostAPIPrimaryEntryPointApplicationCommandJSONBody),
}

impl<'de> Deserialize<'de> for RESTPostAPIApplicationCommandsJSONBody {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let value = Value::deserialize(d)?;
        let kind = match value.get("type") {
            None => 1,
            Some(tag) => tag
                .as_u64()
                .ok_or_else(|| D::Error::custom("command type must be numeric"))?,
        };
        match kind {
            1 => serde_json::from_value(value).map(Self::ChatInput),
            2 | 3 => serde_json::from_value(value).map(Self::ContextMenu),
            4 => serde_json::from_value(value).map(Self::PrimaryEntryPoint),
            _ => return Err(D::Error::custom("unsupported application command type")),
        }
        .map_err(D::Error::custom)
    }
}

/// See <https://discord.com/developers/docs/interactions/application-commands#create-global-application-command>
pub type RESTPostAPIApplicationCommandsResult = APIApplicationCommand;

/// See <https://discord.com/developers/docs/interactions/application-commands#edit-global-application-command>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RESTPatchAPIApplicationCommandJSONBody {
    #[serde(flatten)]
    pub data: Value,
}

/// See <https://discord.com/developers/docs/interactions/application-commands#edit-global-application-command>
pub type RESTPatchAPIApplicationCommandResult = APIApplicationCommand;

/// See <https://discord.com/developers/docs/interactions/application-commands#bulk-overwrite-global-application-commands>
pub type RESTPutAPIApplicationCommandsJSONBody = Vec<RESTPostAPIApplicationCommandsJSONBody>;

/// See <https://discord.com/developers/docs/interactions/application-commands#bulk-overwrite-global-application-commands>
pub type RESTPutAPIApplicationCommandsResult = Vec<APIApplicationCommand>;

/// See <https://discord.com/developers/docs/interactions/application-commands#get-guild-application-commands>
pub type RESTGetAPIApplicationGuildCommandsQuery = RESTGetAPIApplicationCommandsQuery;

/// See <https://discord.com/developers/docs/interactions/application-commands#get-guild-application-commands>
pub type RESTGetAPIApplicationGuildCommandsResult = Vec<APIApplicationCommand>;

/// See <https://discord.com/developers/docs/interactions/application-commands#get-guild-application-commands>
pub type RESTGetAPIApplicationGuildCommandResult = APIApplicationCommand;

/// See <https://discord.com/developers/docs/interactions/application-commands#create-guild-application-command>
pub type RESTPostAPIApplicationGuildCommandsJSONBody = Value;

/// See <https://discord.com/developers/docs/interactions/application-commands#create-guild-application-command>
pub type RESTPostAPIApplicationGuildCommandsResult = APIApplicationCommand;

/// See <https://discord.com/developers/docs/interactions/application-commands#edit-guild-application-command>
pub type RESTPatchAPIApplicationGuildCommandJSONBody = Value;

/// See <https://discord.com/developers/docs/interactions/application-commands#edit-guild-application-command>
pub type RESTPatchAPIApplicationGuildCommandResult = APIApplicationCommand;

/// See <https://discord.com/developers/docs/interactions/application-commands#bulk-overwrite-guild-application-commands>
pub type RESTPutAPIApplicationGuildCommandsJSONBody = Vec<Value>;

/// See <https://discord.com/developers/docs/interactions/application-commands#bulk-overwrite-guild-application-commands>
pub type RESTPutAPIApplicationGuildCommandsResult = Vec<APIApplicationCommand>;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#create-interaction-response>
pub type RESTPostAPIInteractionCallbackJSONBody = APIInteractionResponse;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#create-interaction-response>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RESTPostAPIInteractionCallbackQuery {
    /// Whether to include a interaction callback response as the response instead of a 204
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_response: Option<bool>,
}

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#create-interaction-response>
pub type RESTPostAPIInteractionCallbackFormDataBody = Value;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#create-interaction-response>
pub type RESTPostAPIInteractionCallbackResult = ();

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-callback-interaction-callback-response-object>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTPostAPIInteractionCallbackWithResponseResult {
    /// The interaction object associated with the interaction
    pub interaction: RESTAPIInteractionCallbackObject,
    /// The resource that was created by the interaction response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<RESTAPIInteractionCallbackResourceObject>,
}

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-callback-interaction-callback-object>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTAPIInteractionCallbackObject {
    /// ID of the interaction
    pub id: String,
    /// Interaction type
    pub r#type: InteractionType,
    /// Instance ID of the Activity if one was launched or joined
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_instance_id: Option<String>,
    /// ID of the message that was created by the interaction
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_message_id: Option<String>,
    /// Whether or not the message is in a loading state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_message_loading: Option<bool>,
    /// Whether or not the response message was ephemeral
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_message_ephemeral: Option<bool>,
}

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-callback-interaction-callback-resource-object>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTAPIInteractionCallbackResourceObject {
    /// Interaction callback type
    pub r#type: InteractionResponseType,
    /// Represents the Activity launched by this interaction
    ///
    /// **Note:**
    /// Only present if `type` is [`InteractionResponseType::LaunchActivity`](crate::models::payloads::_interactions::responses::InteractionResponseType::LaunchActivity)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_instance: Option<RESTAPIInteractionCallbackActivityInstanceResource>,
    /// Message created by the interaction
    ///
    /// **Note:**
    /// Only present if `type` is [`InteractionResponseType::ChannelMessageWithSource`](crate::models::payloads::_interactions::responses::InteractionResponseType::ChannelMessageWithSource)
    /// or [`InteractionResponseType::UpdateMessage`](crate::models::payloads::_interactions::responses::InteractionResponseType::UpdateMessage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<APIMessage>,
}

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#interaction-callback-interaction-callback-activity-instance-resource>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTAPIInteractionCallbackActivityInstanceResource {
    /// Instance ID of the Activity if one was launched or joined.
    pub id: String,
}

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#get-original-interaction-response>
pub type RESTGetAPIInteractionOriginalResponseResult = RESTGetAPIWebhookWithTokenMessageResult;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#edit-original-interaction-response>
pub type RESTPatchAPIInteractionOriginalResponseJSONBody =
    RESTPatchAPIWebhookWithTokenMessageJSONBody;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#edit-original-interaction-response>
pub type RESTPatchAPIInteractionOriginalResponseFormDataBody =
    RESTPatchAPIWebhookWithTokenMessageFormDataBody;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#edit-original-interaction-response>
pub type RESTPatchAPIInteractionOriginalResponseResult = RESTPatchAPIWebhookWithTokenMessageResult;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#delete-original-interaction-response>
pub type RESTDeleteAPIInteractionOriginalResponseResult =
    RESTDeleteAPIWebhookWithTokenMessageResult;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#create-followup-message>
pub type RESTPostAPIInteractionFollowupJSONBody = APIInteractionResponseCallbackData;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#create-followup-message>
pub type RESTPostAPIInteractionFollowupFormDataBody = Value;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#create-followup-message>
pub type RESTPostAPIInteractionFollowupResult = RESTPostAPIWebhookWithTokenWaitResult;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#get-followup-message>
pub type RESTGetAPIInteractionFollowupResult = RESTGetAPIWebhookWithTokenMessageResult;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#edit-followup-message>
pub type RESTPatchAPIInteractionFollowupJSONBody = RESTPatchAPIWebhookWithTokenMessageJSONBody;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#edit-followup-message>
pub type RESTPatchAPIInteractionFollowupFormDataBody =
    RESTPatchAPIWebhookWithTokenMessageFormDataBody;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#edit-followup-message>
pub type RESTPatchAPIInteractionFollowupResult = RESTPatchAPIWebhookWithTokenMessageResult;

/// See <https://discord.com/developers/docs/interactions/receiving-and-responding#delete-followup-message>
pub type RESTDeleteAPIInteractionFollowupResult = RESTDeleteAPIWebhookWithTokenMessageResult;

/// See <https://discord.com/developers/docs/interactions/application-commands#get-guild-application-command-permissions>
pub type RESTGetAPIGuildApplicationCommandsPermissionsResult =
    Vec<APIGuildApplicationCommandPermissions>;

/// See <https://discord.com/developers/docs/interactions/application-commands#get-application-command-permissions>
pub type RESTGetAPIApplicationCommandPermissionsResult = APIGuildApplicationCommandPermissions;

/// See <https://discord.com/developers/docs/interactions/application-commands#edit-application-command-permissions>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTPutAPIApplicationCommandPermissionsJSONBody {
    pub permissions: Vec<APIApplicationCommandPermission>,
}

/// See <https://discord.com/developers/docs/interactions/application-commands#edit-application-command-permissions>
pub type RESTPutAPIApplicationCommandPermissionsResult = APIGuildApplicationCommandPermissions;

/// See <https://discord.com/developers/docs/interactions/application-commands#batch-edit-application-command-permissions>
pub type RESTPutAPIGuildApplicationCommandsPermissionsJSONBody =
    Vec<APIGuildApplicationCommandPermissions>;

/// See <https://discord.com/developers/docs/interactions/application-commands#batch-edit-application-command-permissions>
pub type RESTPutAPIGuildApplicationCommandsPermissionsResult =
    Vec<APIGuildApplicationCommandPermissions>;
