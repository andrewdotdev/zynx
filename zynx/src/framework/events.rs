use std::sync::Arc;

use crate::{
    framework::context::Ctx,
    models::gateway::{GatewayDispatch, GatewayDispatchEvents as E},
};

pub mod registry {
    use super::*;
    use std::{future::Future, pin::Pin};

    pub type DynFut = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
    pub type HandlerFn = fn(Ctx) -> DynFut;

    pub struct Handler {
        pub event: E,
        pub f: HandlerFn,
        pub name: &'static str,
    }

    inventory::collect!(Handler);

    pub fn iter_for(event: E) -> impl Iterator<Item = &'static Handler> {
        inventory::iter::<Handler>
            .into_iter()
            .filter(move |handler| handler.event == event)
    }
}

/// Dispatch a decoded envelope through macro handlers, attaching its actual
/// event metadata to the context just as the Gateway router does.
pub async fn dispatch(ctx: Ctx, event: GatewayDispatch<serde_json::Value>) {
    let raw = match serde_json::value::to_raw_value(&event.d) {
        Ok(raw) => raw,
        Err(error) => {
            crate::log!("WARN", "could not encode dispatch payload: {error}");
            return;
        }
    };
    let envelope = Arc::new(GatewayDispatch {
        op: event.op,
        t: event.t,
        s: event.s,
        d: raw,
    });
    dispatch_inventory(Ctx::with_event(ctx.inner, envelope)).await;
}

pub async fn dispatch_inventory(ctx: Ctx) -> usize {
    let Some(kind) = ctx.event_name() else {
        return 0;
    };
    let mut count = 0;
    for handler in registry::iter_for(kind) {
        (handler.f)(ctx.clone()).await;
        count += 1;
    }
    count
}
