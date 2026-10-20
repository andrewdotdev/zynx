#![doc = include_str!("../README.md")]

pub mod client;
pub mod framework;
pub mod gateway;
pub mod http;
pub mod models;
pub mod structs;
pub mod utils;

pub use crate::utils::log;
pub use zynx_macros::event_handler;

pub mod prelude {
    pub use crate::framework::context::{
        ChannelContext, Context, ContextError, Ctx, MessageContext, MessageDeleteContext, RawEvent,
        ReactionContext, ReactionRemoveContext, ReadyContext, TypingContext,
    };
    pub use crate::structs::gateway::*;
    pub use zynx_macros::event_handler;
}

#[doc(hidden)]
pub use crate as zynx;

#[doc(hidden)]
pub mod __private {
    pub use crate::framework::context::Ctx;
    pub use crate::framework::events::registry;
    pub use crate::models::gateway::GatewayDispatchEvents;
    pub use crate::models::gateway::*;
    pub use ::inventory;
    pub use ::serde_json;
}
