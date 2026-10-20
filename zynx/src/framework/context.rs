use std::fmt;
use std::sync::Arc;

use crate::http::{Http, HttpError};
use crate::models::gateway::{GatewayDispatch, GatewayDispatchEvents as GwEvt};
use crate::models::payloads::{
    channel::{APIBaseMessage, APIChannel},
    emoji::APIEmoji,
    user::APIUser,
};
use crate::structs::gateway::{
    ChannelCreate, MessageCreate, MessageDelete, MessageReactionAdd, MessageReactionRemove, Ready,
    TypingStart,
};
use serde::de::DeserializeOwned;
use serde_json::value::RawValue;

/// Services shared by every handler for a client.
#[derive(Clone)]
pub struct Context {
    pub http: Arc<Http>,
}

impl Context {
    pub fn new(http: Arc<Http>) -> Self {
        Self { http }
    }
    pub fn http(&self) -> &Arc<Http> {
        &self.http
    }
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Context")
            .field("http", &"Http { redacted }")
            .finish()
    }
}

/// Marker for an untyped context, used by catch-all handlers.
#[derive(Debug)]
pub struct RawEvent;

/// Handler context containing shared services and a decoded event payload.
/// Clones share the payload; helpers borrow it without reparsing JSON.
///
/// Event-specific methods are only available for the relevant payload type:
///
/// ```compile_fail
/// use zynx::prelude::ReadyContext;
/// async fn invalid(ctx: ReadyContext) {
///     ctx.reply("Ready has no message to reply to").await;
/// }
/// ```
///
/// ```compile_fail
/// use zynx::prelude::ReactionContext;
/// fn invalid(ctx: ReactionContext) {
///     ctx.text(); // A reaction does not contain the message's content.
/// }
/// ```
///
/// The macro also rejects a context belonging to a different event:
///
/// ```compile_fail
/// use zynx::prelude::*;
/// #[event_handler]
/// async fn on_ready(ctx: MessageContext, message: MessageCreate) {}
/// ```
///
/// Inferred contexts retain the same restrictions:
///
/// ```compile_fail
/// use zynx::prelude::*;
/// #[event_handler]
/// async fn on_ready(ctx: Ctx) {
///     ctx.reply("Ready has no message to reply to").await;
/// }
/// ```
#[derive(Debug)]
pub struct Ctx<T = RawEvent> {
    pub(crate) inner: Arc<Context>,
    pub(crate) event: Option<Arc<GatewayDispatch<Box<RawValue>>>>,
    data: Arc<T>,
}

impl<T> Clone for Ctx<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            event: self.event.clone(),
            data: self.data.clone(),
        }
    }
}

pub type MessageContext = Ctx<MessageCreate>;
pub type ReadyContext = Ctx<Ready>;
pub type ReactionContext = Ctx<MessageReactionAdd>;
pub type ReactionRemoveContext = Ctx<MessageReactionRemove>;
/// Channel create/update/delete share the same payload model.
pub type ChannelContext = Ctx<ChannelCreate>;
pub type MessageDeleteContext = Ctx<MessageDelete>;
pub type TypingContext = Ctx<TypingStart>;

#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("context has no event")]
    MissingEvent,
    #[error("could not decode event payload: {0}")]
    Decode(#[from] serde_json::Error),
}

impl Ctx {
    pub fn new(inner: Arc<Context>) -> Self {
        Self {
            inner,
            event: None,
            data: Arc::new(RawEvent),
        }
    }

    pub(crate) fn with_event(
        inner: Arc<Context>,
        event: Arc<GatewayDispatch<Box<RawValue>>>,
    ) -> Self {
        Self {
            inner,
            event: Some(event),
            data: Arc::new(RawEvent),
        }
    }

    /// Decode a raw context. Low-level callers must select the payload type that
    /// corresponds to `event_name()`. Typed client shortcuts do this automatically.
    pub fn decode<T: DeserializeOwned + 'static>(self) -> Result<Ctx<T>, ContextError> {
        let event = self.event.as_ref().ok_or(ContextError::MissingEvent)?;
        let raw = if event.t == GwEvt::Resumed
            && std::any::TypeId::of::<T>() == std::any::TypeId::of::<()>()
        {
            "null"
        } else {
            event.d.get()
        };
        let data = Arc::new(serde_json::from_str(raw)?);
        Ok(Ctx {
            inner: self.inner,
            event: self.event,
            data,
        })
    }

    /// Read a payload from a catch-all handler. Prefer typed shortcuts when possible.
    pub fn data_as<T: DeserializeOwned>(&self) -> Result<T, ContextError> {
        let event = self.event.as_ref().ok_or(ContextError::MissingEvent)?;
        Ok(serde_json::from_str(event.d.get())?)
    }

    /// Check the event kind before decoding, retaining decode errors for the caller.
    pub fn event_as<T: DeserializeOwned>(&self, kind: GwEvt) -> Result<Option<T>, ContextError> {
        if self.is(kind) {
            self.data_as().map(Some)
        } else {
            Ok(None)
        }
    }
}

impl<T> Ctx<T> {
    /// The decoded payload. All events are available through this accessor, even
    /// when no specialized context helpers exist yet.
    pub fn data(&self) -> &T {
        &self.data
    }
    pub fn http(&self) -> &Arc<Http> {
        self.inner.http()
    }
    pub fn event_name(&self) -> Option<GwEvt> {
        self.event.as_ref().map(|event| event.t.clone())
    }
    pub fn sequence(&self) -> Option<i64> {
        self.event.as_ref().map(|event| event.s)
    }
    pub fn is(&self, kind: GwEvt) -> bool {
        self.event_name() == Some(kind)
    }

    /// Send to an explicitly selected channel, available from any event context.
    pub async fn say(&self, channel_id: &str, content: &str) -> Result<(), HttpError> {
        self.http()
            .send_message(channel_id, content, None, false)
            .await
    }
}

impl MessageContext {
    pub fn message(&self) -> &APIBaseMessage {
        &self.data().message
    }
    pub fn text(&self) -> &str {
        &self.message().content
    }
    pub fn message_id(&self) -> &str {
        &self.message().id
    }
    pub fn channel_id(&self) -> &str {
        &self.message().channel_id
    }
    pub fn guild_id(&self) -> Option<&str> {
        self.data().extra.guild_id.as_deref()
    }
    pub fn author(&self) -> &APIUser {
        &self.message().author
    }
    pub fn is_bot(&self) -> bool {
        self.data().is_bot()
    }

    /// Reply to this message without pinging its author.
    pub async fn reply(&self, content: &str) -> Result<(), HttpError> {
        self.http()
            .send_message(self.channel_id(), content, Some(self.message_id()), false)
            .await
    }
}

impl ReadyContext {
    pub fn user(&self) -> &APIUser {
        &self.data().user
    }
    pub fn session_id(&self) -> &str {
        &self.data().session_id
    }
    pub fn resume_gateway_url(&self) -> &str {
        &self.data().resume_gateway_url
    }
}

macro_rules! reaction_helpers {
    ($payload:ty) => {
        impl Ctx<$payload> {
            pub fn emoji(&self) -> &APIEmoji {
                &self.data().emoji
            }
            pub fn user_id(&self) -> &str {
                &self.data().user_id
            }
            pub fn message_id(&self) -> &str {
                &self.data().message_id
            }
            pub fn channel_id(&self) -> &str {
                &self.data().channel_id
            }
            pub fn guild_id(&self) -> Option<&str> {
                self.data().guild_id.as_deref()
            }
        }
    };
}
reaction_helpers!(MessageReactionAdd);
reaction_helpers!(MessageReactionRemove);

impl ChannelContext {
    pub fn channel_id(&self) -> &str {
        self.channel().id()
    }
    pub fn channel_name(&self) -> Option<&str> {
        self.channel().name()
    }
    pub fn channel(&self) -> &APIChannel {
        &self.data().channel
    }
    pub fn guild_id(&self) -> &str {
        &self.data().guild_id
    }
}

impl MessageDeleteContext {
    pub fn message_id(&self) -> &str {
        &self.data().id
    }
    pub fn channel_id(&self) -> &str {
        &self.data().channel_id
    }
    pub fn guild_id(&self) -> Option<&str> {
        self.data().guild_id.as_deref()
    }
}

impl TypingContext {
    pub fn user_id(&self) -> &str {
        &self.data().user_id
    }
    pub fn channel_id(&self) -> &str {
        &self.data().channel_id
    }
    pub fn guild_id(&self) -> Option<&str> {
        self.data().guild_id.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::gateway::GatewayOpcodes;

    fn raw_with_http(kind: GwEvt, json: &str, http: Http) -> Ctx {
        Ctx::with_event(
            Arc::new(Context::new(Arc::new(http))),
            Arc::new(GatewayDispatch {
                op: GatewayOpcodes::Dispatch,
                t: kind,
                s: 42,
                d: RawValue::from_string(json.to_owned()).unwrap(),
            }),
        )
    }

    fn raw(kind: GwEvt, json: &str) -> Ctx {
        raw_with_http(kind, json, Http::new("test-token"))
    }

    #[test]
    fn message_helpers_borrow_shared_payload_and_preserve_metadata() {
        let ctx: MessageContext = raw(
            GwEvt::MessageCreate,
            include_str!("../../tests/fixtures/message_create.json"),
        )
        .decode()
        .unwrap();
        assert_eq!(ctx.text(), "hello");
        assert_eq!(ctx.message_id(), "100");
        assert_eq!(ctx.channel_id(), "200");
        assert_eq!(ctx.guild_id(), Some("300"));
        assert_eq!(ctx.author().id, "400");
        assert!(!ctx.is_bot());
        assert_eq!(ctx.event_name(), Some(GwEvt::MessageCreate));
        assert_eq!(ctx.sequence(), Some(42));
        let cloned = ctx.clone();
        assert!(std::ptr::eq(ctx.data(), cloned.data()));
        assert!(std::ptr::eq(ctx.text().as_ptr(), cloned.text().as_ptr()));
        assert!(Arc::ptr_eq(ctx.http(), cloned.http()));
    }

    #[test]
    fn dm_message_has_no_guild_and_retains_empty_content() {
        let mut value: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/message_create.json")).unwrap();
        value.as_object_mut().unwrap().remove("guild_id");
        value["content"] = "".into();
        let ctx: MessageContext = raw(GwEvt::MessageCreate, &value.to_string())
            .decode()
            .unwrap();
        assert_eq!(ctx.guild_id(), None);
        assert_eq!(ctx.text(), "");
    }

    #[test]
    fn channel_context_exposes_id_and_name_for_guild_events() {
        for kind in [
            GwEvt::ChannelCreate,
            GwEvt::ChannelUpdate,
            GwEvt::ChannelDelete,
        ] {
            let ctx: ChannelContext = raw(
                kind.clone(),
                r#"{
                "id":"123", "type":0, "guild_id":"456", "name":"hoka", "position":0
            }"#,
            )
            .decode()
            .unwrap();
            assert_eq!(ctx.channel_id(), "123");
            assert_eq!(ctx.channel_name(), Some("hoka"));
            assert_eq!(ctx.guild_id(), "456");
            assert_eq!(ctx.event_name(), Some(kind));
            assert!(matches!(ctx.channel(), APIChannel::APITextChannel(_)));
        }
    }

    #[test]
    fn ready_and_reaction_helpers_work_without_message_payloads() {
        let ready: ReadyContext = raw(
            GwEvt::Ready,
            r#"{
            "v":10,"user":{"id":"1","username":"bot","discriminator":"0"},
            "guilds":[],"session_id":"session","resume_gateway_url":"wss://gateway.example",
            "application":{"id":"2","flags":0}
        }"#,
        )
        .decode()
        .unwrap();
        assert_eq!(ready.user().username, "bot");
        assert_eq!(ready.session_id(), "session");
        let reaction: ReactionContext = raw(
            GwEvt::MessageReactionAdd,
            r#"{
            "user_id":"1","channel_id":"2","message_id":"3",
            "emoji":{"id":null,"name":"👍"},"burst":false,"type":0
        }"#,
        )
        .decode()
        .unwrap();
        assert_eq!(reaction.user_id(), "1");
        assert_eq!(reaction.channel_id(), "2");
        assert_eq!(reaction.message_id(), "3");
        assert_eq!(reaction.guild_id(), None);
        assert_eq!(reaction.emoji().base.name.as_deref(), Some("👍"));
    }

    #[test]
    fn raw_context_distinguishes_wrong_event_malformed_data_and_missing_event() {
        let ctx = raw(GwEvt::MessageCreate, "{}");
        assert!(ctx.event_as::<Ready>(GwEvt::Ready).unwrap().is_none());
        assert!(matches!(
            ctx.decode::<MessageCreate>(),
            Err(ContextError::Decode(_))
        ));
        let base = Ctx::new(Arc::new(Context::new(Arc::new(Http::new("test-token")))));
        assert!(matches!(
            base.decode::<Ready>(),
            Err(ContextError::MissingEvent)
        ));
        let resumed: Ctx<()> = raw(GwEvt::Resumed, "{}").decode().unwrap();
        assert_eq!(resumed.data(), &());
    }

    #[tokio::test]
    async fn reply_targets_the_context_message_and_say_does_not_reference_it() {
        let (http, mut requests) =
            crate::http::tests::mock(vec![(200, "", "{}"), (200, "", "{}")]).await;
        let ctx: MessageContext = raw_with_http(
            GwEvt::MessageCreate,
            include_str!("../../tests/fixtures/message_create.json"),
            http,
        )
        .decode()
        .unwrap();
        ctx.reply("reply").await.unwrap();
        ctx.say("999", "plain message").await.unwrap();
        let reply = requests.recv().await.unwrap().1;
        assert!(reply.starts_with("POST /channels/200/messages "));
        let body: serde_json::Value =
            serde_json::from_str(reply.split_once("\r\n\r\n").unwrap().1).unwrap();
        assert_eq!(body["message_reference"]["message_id"], "100");
        assert_eq!(body["allowed_mentions"]["replied_user"], false);
        let say = requests.recv().await.unwrap().1;
        assert!(say.starts_with("POST /channels/999/messages "));
        let body: serde_json::Value =
            serde_json::from_str(say.split_once("\r\n\r\n").unwrap().1).unwrap();
        assert!(body.get("message_reference").is_none());
    }
}
