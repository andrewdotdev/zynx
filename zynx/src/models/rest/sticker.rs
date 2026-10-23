// SPDX-License-Identifier: MIT
// Adapted from discord-api-types (c) vladfrangu
// Rust port and modifications (c) 2025 andrewdotdev

use crate::models::payloads::{APISticker, APIStickerPack};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// See <https://discord.com/developers/docs/resources/sticker#get-sticker>
pub type RESTGetAPIStickerResult = APISticker;

/// See <https://discord.com/developers/docs/resources/sticker#list-sticker-packs>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTGetStickerPacksResult {
    #[serde(rename = "sticker_packs")]
    pub sticker_packs: Vec<APIStickerPack>,
}

/// See <https://discord.com/developers/docs/resources/sticker#get-sticker-pack>
pub type RESTGetAPIStickerPackResult = APIStickerPack;

/// **Deprecated:** Use [`RESTGetAPIStickerPackResult`](crate::models::rest::sticker::RESTGetAPIStickerPackResult) instead
#[deprecated(note = "Use RESTGetAPIStickerPackResult instead")]
pub type RESTGetAPIStickerPack = RESTGetAPIStickerPackResult;

/// See <https://discord.com/developers/docs/resources/sticker#list-sticker-packs>

///
/// **Deprecated:** Use [`RESTGetStickerPacksResult`](crate::models::rest::sticker::RESTGetStickerPacksResult) instead
#[deprecated(note = "Use RESTGetStickerPacksResult instead")]
pub type RESTGetNitroStickerPacksResult = RESTGetStickerPacksResult;

/// See <https://discord.com/developers/docs/resources/sticker#list-guild-stickers>
pub type RESTGetAPIGuildStickersResult = Vec<APISticker>;

/// See <https://discord.com/developers/docs/resources/sticker#get-guild-sticker>
pub type RESTGetAPIGuildStickerResult = APISticker;

/// See <https://discord.com/developers/docs/resources/sticker#create-guild-sticker>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTPostAPIGuildStickerFormDataBody {
    /// Name of the sticker (2-30 characters)
    pub name: String,
    /// Description of the sticker (empty or 2-100 characters)
    pub description: String,
    /// The Discord name of a unicode emoji representing the sticker's expression (2-200 characters)
    pub tags: String,
    /// The sticker file to upload, must be a PNG, APNG, GIF, or Lottie JSON file, max 512 KB
    ///
    /// Uploaded stickers are constrained to 5 seconds in length for animated stickers, and 320 x 320 pixels.
    pub file: Value,
}

/// See <https://discord.com/developers/docs/resources/sticker#create-guild-sticker>
pub type RESTPostAPIGuildStickerResult = APISticker;

/// See <https://discord.com/developers/docs/resources/sticker#modify-guild-sticker>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RESTPatchAPIGuildStickerJSONBody {
    /// Name of the sticker (2-30 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Description of the sticker (2-100 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Option<String>>,
    /// The Discord name of a unicode emoji representing the sticker's expression (2-200 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
}

/// See <https://discord.com/developers/docs/resources/sticker#modify-guild-sticker>
pub type RESTPatchAPIGuildStickerResult = APISticker;

/// See <https://discord.com/developers/docs/resources/sticker#delete-guild-sticker>
pub type RESTDeleteAPIGuildStickerResult = ();
