// SPDX-License-Identifier: MIT
// Adapted from discord-api-types (c) vladfrangu
// Rust port and modifications (c) 2025 andrewdotdev

use crate::models::payloads::{APIGatewayBotInfo, APIGatewayInfo};

/// See <https://discord.com/developers/docs/topics/gateway#get-gateway>
pub type RESTGetAPIGatewayResult = APIGatewayInfo;

/// See <https://discord.com/developers/docs/topics/gateway#get-gateway-bot>
pub type RESTGetAPIGatewayBotResult = APIGatewayBotInfo;
