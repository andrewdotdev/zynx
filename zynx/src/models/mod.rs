//! Discord API models for Gateway events, REST requests, and voice connections.
//!
//! - [`gateway`] contains Gateway events, opcodes, and intents.
//! - [`payloads`] contains shared Discord objects and interaction payloads.
//! - [`rest`] contains REST request bodies, query parameters, and responses.
//! - [`voice`] contains voice protocol payloads.
//! - [`utils`] contains helpers for inspecting interaction types.
//!
//! These models are adapted from [discord-api-types](https://github.com/discordjs/discord-api-types)
//! and retain the upstream MIT license and attribution.

pub mod gateway;
pub mod payloads;
pub mod rest;
pub mod utils;
pub mod voice;
