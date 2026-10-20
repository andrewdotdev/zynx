use zynx::{
    framework::events, http::Http, models::gateway::GatewayDispatchEvents, prelude::*,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

static SINGLE: AtomicUsize = AtomicUsize::new(0);
static DOUBLE: AtomicUsize = AtomicUsize::new(0);
static RESUMED: AtomicUsize = AtomicUsize::new(0);

mod single {
    use super::*;

    #[event_handler]
    async fn on_message_create(ctx: Ctx) {
        assert_eq!(ctx.text(), "hello");
        assert_eq!(ctx.channel_id(), "200");
        assert_eq!(ctx.sequence(), Some(7));
        SINGLE.fetch_add(1, Ordering::SeqCst);
    }
}

mod double {
    use super::*;

    #[event_handler]
    async fn on_message_create(ctx: Ctx<MessageCreate>, payload: MessageCreate) {
        assert_eq!(ctx.message(), &payload.message);
        DOUBLE.fetch_add(1, Ordering::SeqCst);
    }
}

#[event_handler]
async fn on_resumed(ctx: Ctx) {
    let _: &() = ctx.data();
    assert_eq!(ctx.event_name(), Some(GatewayDispatchEvents::Resumed));
    RESUMED.fetch_add(1, Ordering::SeqCst);
}

// These event names share a payload type that cannot be inferred by simple concatenation.
#[event_handler]
async fn on_message_poll_vote_add(ctx: Ctx) {
    let _: &MessagePollVoteAdd = ctx.data();
}
#[event_handler]
async fn on_message_poll_vote_remove(ctx: Ctx) {
    let _: &MessagePollVoteRemove = ctx.data();
}

mod inferred_double {
    use super::*;

    #[event_handler]
    async fn on_message_create(ctx: zynx::prelude::Ctx, payload: MessageCreate) {
        assert_eq!(ctx.message(), &payload.message);
        INFERRED_DOUBLE.fetch_add(1, Ordering::SeqCst);
    }
}

static INFERRED_DOUBLE: AtomicUsize = AtomicUsize::new(0);

#[tokio::test]
async fn macro_dispatch_attaches_typed_context_for_both_handler_styles() {
    let base = Ctx::new(Arc::new(Context::new(Arc::new(Http::new("test-token")))));
    events::dispatch(
        base.clone(),
        GatewayDispatch {
            op: GatewayOpcodes::Dispatch,
            t: GatewayDispatchEvents::MessageCreate,
            s: 7,
            d: serde_json::from_str(include_str!("fixtures/message_create.json")).unwrap(),
        },
    )
    .await;
    assert_eq!(SINGLE.load(Ordering::SeqCst), 1);
    assert_eq!(DOUBLE.load(Ordering::SeqCst), 1);
    assert_eq!(INFERRED_DOUBLE.load(Ordering::SeqCst), 1);
    // A malformed payload must not invoke either handler with an invalid context.
    events::dispatch(
        base.clone(),
        GatewayDispatch {
            op: GatewayOpcodes::Dispatch,
            t: GatewayDispatchEvents::MessageCreate,
            s: 8,
            d: serde_json::json!({}),
        },
    )
    .await;
    assert_eq!(SINGLE.load(Ordering::SeqCst), 1);
    assert_eq!(DOUBLE.load(Ordering::SeqCst), 1);
    assert_eq!(INFERRED_DOUBLE.load(Ordering::SeqCst), 1);
    events::dispatch(
        base,
        GatewayDispatch {
            op: GatewayOpcodes::Dispatch,
            t: GatewayDispatchEvents::Resumed,
            s: 9,
            d: serde_json::json!({}),
        },
    )
    .await;
    assert_eq!(RESUMED.load(Ordering::SeqCst), 1);
}
