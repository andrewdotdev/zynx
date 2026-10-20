// SPDX-License-Identifier: MIT
// Adapted from discord-api-types (c) vladfrangu
// Rust port and modifications (c) 2025 andrewdotdev

use crate::models::payloads::{APIApplication, APIGuild, APIUser, APIWebhook, OAuth2Scopes};
use serde::{Deserialize, Serialize};

/**
 * @see {@link https://discord.com/developers/docs/topics/oauth2#get-current-bot-application-information}
 */
pub type RESTGetAPIOAuth2CurrentApplicationResult = APIApplication;

/**
 * @see {@link https://discord.com/developers/docs/topics/oauth2#get-current-authorization-information}
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTGetAPIOAuth2CurrentAuthorizationResult {
    /**
     * the current application
     */
    pub application: APIApplication,
    /**
     * the scopes the user has authorized the application for
     */
    pub scopes: Vec<OAuth2Scopes>,
    /**
     * when the access token expires
     */
    pub expires: String,
    /**
     * the user who has authorized, if the user has authorized with the `identify` scope
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<APIUser>,
}

/**
 * @see {@link https://discord.com/developers/docs/topics/oauth2#authorization-code-grant}
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTOAuth2AuthorizationQuery {
    pub response_type: String, // 'code'
    pub client_id: String,
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>, // 'consent' | 'none'
}

/**
 * @see {@link https://discord.com/developers/docs/topics/oauth2#authorization-code-grant-token-revocation-example}
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTPostOAuth2TokenRevocationQuery {
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type_hint: Option<String>, // 'access_token' | 'refresh_token'
}

/**
 * @see {@link https://discord.com/developers/docs/topics/oauth2#authorization-code-grant-redirect-url-example}
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTPostOAuth2AuthorizationQueryResult {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

/**
 * @deprecated Use {@link RESTPostOAuth2AuthorizationQueryResult} instead
 */
pub type RESTOAuth2AuthorizationQueryResult = RESTPostOAuth2AuthorizationQueryResult;

/**
 * @remarks
 * This endpoint requires either HTTP Basic authentication using `client_id:client_secret`,
 * or the `client_id` and `client_secret` must be provided in the form body.
 * @see {@link https://discord.com/developers/docs/topics/oauth2#authorization-code-grant-redirect-url-example}
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTPostOAuth2AccessTokenURLEncodedData {
    pub grant_type: String, // 'authorization_code'
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum RESTOAuth2TokenOptionalClientCredentials {
    With {
        client_id: String,
        client_secret: String,
    },
    Without {},
}

impl<'de> Deserialize<'de> for RESTOAuth2TokenOptionalClientCredentials {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let value = serde_json::Value::deserialize(d)?;
        let object = value
            .as_object()
            .ok_or_else(|| D::Error::custom("credentials must be an object"))?;
        match (object.get("client_id"), object.get("client_secret")) {
            (None, None) => Ok(Self::Without {}),
            (Some(id), Some(secret)) => Ok(Self::With {
                client_id: serde_json::from_value(id.clone()).map_err(D::Error::custom)?,
                client_secret: serde_json::from_value(secret.clone()).map_err(D::Error::custom)?,
            }),
            _ => Err(D::Error::custom(
                "client_id and client_secret must be provided together",
            )),
        }
    }
}

/**
 * @see {@link https://discord.com/developers/docs/topics/oauth2#authorization-code-grant-access-token-response}
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTPostOAuth2AccessTokenResult {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub refresh_token: String,
    pub scope: String,
}

/**
 * @remarks
 * This endpoint requires either HTTP Basic authentication using `client_id:client_secret`,
 * or the `client_id` and `client_secret` must be provided in the form body.
 * @see {@link https://discord.com/developers/docs/topics/oauth2#authorization-code-grant-refresh-token-exchange-example}
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTPostOAuth2RefreshTokenURLEncodedData {
    pub grant_type: String, // 'refresh_token'
    pub refresh_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
}

pub type RESTPostOAuth2RefreshTokenResult = RESTPostOAuth2AccessTokenResult;

/**
 * @see {@link https://discord.com/developers/docs/topics/oauth2#implicit-grant}
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTOAuth2ImplicitAuthorizationQuery {
    pub response_type: String, // 'token'
    pub client_id: String,
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>, // 'consent' | 'none'
}

/**
 * @see {@link https://discord.com/developers/docs/topics/oauth2#implicit-grant-redirect-url-example}
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTOAuth2ImplicitAuthorizationURLFragmentResult {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub scope: String,
}

/**
 * @see {@link https://discord.com/developers/docs/topics/oauth2#client-credentials-grant}
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTPostOAuth2ClientCredentialsURLEncodedData {
    pub grant_type: String, // 'client_credentials'
    pub scope: String,
}

pub type RESTPostOAuth2ClientCredentialsResult = RESTOAuth2ImplicitAuthorizationURLFragmentResult;

/**
 * @see {@link https://discord.com/developers/docs/topics/oauth2#bot-authorization-flow-bot-auth-parameters}
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTOAuth2BotAuthorizationQuery {
    /**
     * Your app's client id
     */
    pub client_id: String,
    /**
     * Needs to include bot for the bot flow
     */
    pub scope: String,
    /**
     * The permissions you're requesting
     *
     * @see {@link https://discord.com/developers/docs/topics/permissions}
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<String>,
    /**
     * Pre-fills the dropdown picker with a guild for the user
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<String>,
    /**
     * `true` or `false`—disallows the user from changing the guild dropdown
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_guild_select: Option<bool>,
}

/**
 * @see {@link https://discord.com/developers/docs/topics/oauth2#advanced-bot-authorization}
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTOAuth2AdvancedBotAuthorizationQuery {
    pub client_id: String,
    /**
     * This assumes you include the `bot` scope alongside others (like `identify` for example)
     */
    pub scope: String,
    /**
     * The required permissions bitfield, stringified
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guild_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_guild_select: Option<bool>,
    pub response_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTOAuth2AdvancedBotAuthorizationQueryResult {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    pub guild_id: String,
    pub permissions: String,
}

/**
 * @see {@link https://discord.com/developers/docs/topics/oauth2#advanced-bot-authorization-extended-bot-authorization-access-token-example}
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTPostOAuth2AccessTokenWithBotAndGuildsScopeResult {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub refresh_token: String,
    pub scope: String,
    pub guild: APIGuild,
}

/**
 * @see {@link https://discord.com/developers/docs/topics/oauth2#webhooks-webhook-token-response-example}
 */
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTPostOAuth2AccessTokenWithBotAndWebhookIncomingScopeResult {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub refresh_token: String,
    pub scope: String,
    pub webhook: APIWebhook,
}

pub type RESTPostOAuth2AccessTokenWithBotAndGuildsAndWebhookIncomingScopeResult =
    RESTPostOAuth2AccessTokenWithBotAndGuildsScopeResult;
