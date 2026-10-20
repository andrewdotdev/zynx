use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde_json::value::RawValue;
use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use crate::framework::context::{Context, Ctx};
use crate::models::gateway::{GatewayDispatch, GatewayDispatchEvents as GwEvt};

#[async_trait]
pub trait DynHandler: Send + Sync {
    // Keep raw `d` and decode lazily.
    async fn call(&self, ctx: Arc<Context>, ev: Arc<GatewayDispatch<Box<RawValue>>>);
    fn event(&self) -> GwEvt;
    fn is_once(&self) -> bool {
        false
    }
}

struct Handler<T, F> {
    event: GwEvt,
    f: F,
    once: bool,
    _phantom: PhantomData<fn() -> T>,
}

#[async_trait]
impl<T, F, Fut> DynHandler for Handler<T, F>
where
    T: DeserializeOwned + Clone + Send + Sync + 'static,
    F: Send + Sync + 'static + Fn(Ctx<T>, T) -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    async fn call(&self, ctx: Arc<Context>, ev: Arc<GatewayDispatch<Box<RawValue>>>) {
        if ev.t != self.event {
            return;
        }
        match Ctx::with_event(ctx, ev).decode::<T>() {
            Ok(context) => {
                let payload = context.data().clone();
                (self.f)(context, payload).await;
            }
            Err(err) => {
                crate::log!("WARN", "decode failed for {:?}: {}", self.event, err);
            }
        }
    }

    fn event(&self) -> GwEvt {
        self.event.clone()
    }
    fn is_once(&self) -> bool {
        self.once
    }
}

#[async_trait]
pub trait DynAnyHandler: Send + Sync {
    async fn call(&self, ctx: Arc<Context>, ev: Arc<GatewayDispatch<Box<RawValue>>>);
}

#[async_trait]
impl<F, Fut> DynAnyHandler for F
where
    F: Send + Sync + 'static + Fn(Ctx) -> Fut,
    Fut: Future<Output = ()> + Send + 'static,
{
    async fn call(&self, ctx: Arc<Context>, ev: Arc<GatewayDispatch<Box<RawValue>>>) {
        let c = Ctx::with_event(Arc::clone(&ctx), Arc::clone(&ev));
        (self)(c).await;
    }
}

pub struct Router {
    routes: HashMap<GwEvt, Vec<Arc<dyn DynHandler>>>,
    once_routes: Mutex<HashMap<GwEvt, Vec<Arc<dyn DynHandler>>>>,
    any: Vec<Arc<dyn DynAnyHandler>>,
    unknown: Vec<Arc<dyn DynAnyHandler>>,
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            once_routes: Mutex::new(HashMap::new()),
            any: vec![],
            unknown: vec![],
        }
    }

    pub fn on<T, F, Fut>(&mut self, kind: GwEvt, handler: F)
    where
        T: DeserializeOwned + Clone + Send + Sync + 'static,
        F: Send + Sync + 'static + Fn(Ctx<T>, T) -> Fut,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let h: Arc<dyn DynHandler> = Arc::new(Handler::<T, F> {
            event: kind.clone(),
            f: handler,
            once: false,
            _phantom: PhantomData,
        });
        self.routes.entry(kind).or_default().push(h);
    }

    pub fn once<T, F, Fut>(&mut self, kind: GwEvt, handler: F)
    where
        T: DeserializeOwned + Clone + Send + Sync + 'static,
        F: Send + Sync + 'static + Fn(Ctx<T>, T) -> Fut,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let h: Arc<dyn DynHandler> = Arc::new(Handler::<T, F> {
            event: kind.clone(),
            f: handler,
            once: true,
            _phantom: PhantomData,
        });
        let mut map = self.once_routes.lock().unwrap();
        map.entry(kind).or_default().push(h);
    }

    pub fn on_all<F, Fut>(&mut self, handler: F)
    where
        F: Send + Sync + 'static + Fn(Ctx) -> Fut,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.any.push(Arc::new(handler));
    }

    pub fn on_unknown<F, Fut>(&mut self, handler: F)
    where
        F: Send + Sync + 'static + Fn(Ctx) -> Fut,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.unknown.push(Arc::new(handler));
    }

    pub async fn dispatch(&self, base: &Ctx, ev: Arc<GatewayDispatch<Box<RawValue>>>) {
        let ctx_arc = Arc::clone(&base.inner);

        // on_all
        for h in &self.any {
            h.call(Arc::clone(&ctx_arc), Arc::clone(&ev)).await;
        }

        let kind = ev.t.clone();

        // --- once handlers
        let mut had_handlers = false;
        // Release the synchronous mutex before invoking any async user code.
        let once_handlers = self.once_routes.lock().unwrap().remove(&kind);
        if let Some(list) = once_handlers {
            had_handlers |= !list.is_empty();
            for h in list {
                h.call(Arc::clone(&ctx_arc), Arc::clone(&ev)).await;
            }
        }

        // --- persistent handlers
        if let Some(list) = self.routes.get(&kind) {
            had_handlers |= !list.is_empty();
            for h in list {
                h.call(Arc::clone(&ctx_arc), Arc::clone(&ev)).await;
            }
        }

        use crate::framework::events;
        let ran_macros =
            events::dispatch_inventory(Ctx::with_event(Arc::clone(&ctx_arc), Arc::clone(&ev)))
                .await;
        had_handlers |= ran_macros > 0;

        if !had_handlers {
            for h in &self.unknown {
                h.call(Arc::clone(&ctx_arc), Arc::clone(&ev)).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{http::Http, models::gateway::GatewayOpcodes};
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn context() -> Ctx {
        Ctx::new(Arc::new(Context::new(Arc::new(Http::new("test-token")))))
    }

    fn event(kind: GwEvt, data: &str) -> Arc<GatewayDispatch<Box<RawValue>>> {
        Arc::new(GatewayDispatch {
            op: GatewayOpcodes::Dispatch,
            t: kind,
            s: 1,
            d: RawValue::from_string(data.to_owned()).unwrap(),
        })
    }

    #[tokio::test]
    async fn once_handlers_are_removed_before_awaiting_user_code() {
        let mut router = Router::new();
        router.once::<serde_json::Value, _, _>(GwEvt::MessageCreate, |_, _| async {
            tokio::task::yield_now().await;
        });
        let router = Arc::new(router);
        // The future must be Send, and no mutex may remain locked across await.
        let ctx = context();
        let task = tokio::spawn(async move {
            router
                .dispatch(&ctx, event(GwEvt::MessageCreate, "{}"))
                .await;
            router
        });
        let router = task.await.unwrap();
        assert!(router.once_routes.try_lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn routes_once_persistent_and_unhandled_events() {
        let once = Arc::new(AtomicUsize::new(0));
        let persistent = Arc::new(AtomicUsize::new(0));
        let unknown = Arc::new(AtomicUsize::new(0));
        let mut router = Router::new();
        let count = once.clone();
        router.once::<serde_json::Value, _, _>(GwEvt::MessageCreate, move |_, _| {
            count.fetch_add(1, Ordering::SeqCst);
            async {}
        });
        let count = persistent.clone();
        router.on::<serde_json::Value, _, _>(GwEvt::MessageCreate, move |_, _| {
            count.fetch_add(1, Ordering::SeqCst);
            async {}
        });
        let count = unknown.clone();
        router.on_unknown(move |_| {
            count.fetch_add(1, Ordering::SeqCst);
            async {}
        });
        let ctx = context();
        router
            .dispatch(&ctx, event(GwEvt::MessageCreate, "{}"))
            .await;
        router
            .dispatch(&ctx, event(GwEvt::MessageCreate, "{}"))
            .await;
        router.dispatch(&ctx, event(GwEvt::Resumed, "{}")).await;
        assert_eq!(once.load(Ordering::SeqCst), 1);
        assert_eq!(persistent.load(Ordering::SeqCst), 2);
        assert_eq!(unknown.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn resumed_object_invokes_unit_handler() {
        let called = Arc::new(AtomicUsize::new(0));
        let count = called.clone();
        let mut router = Router::new();
        router.on::<(), _, _>(GwEvt::Resumed, move |_, ()| {
            count.fetch_add(1, Ordering::SeqCst);
            async {}
        });
        router
            .dispatch(&context(), event(GwEvt::Resumed, "{}"))
            .await;
        assert_eq!(called.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn typed_closure_and_catch_all_receive_the_same_event() {
        use crate::prelude::MessageCreate;
        let called = Arc::new(AtomicUsize::new(0));
        let count = called.clone();
        let mut router = Router::new();
        router.on::<MessageCreate, _, _>(GwEvt::MessageCreate, move |ctx, payload| {
            assert_eq!(ctx.text(), "hello");
            assert_eq!(ctx.message(), &payload.message);
            assert_eq!(ctx.sequence(), Some(1));
            count.fetch_add(1, Ordering::SeqCst);
            async {}
        });
        let count = called.clone();
        router.on_all(move |ctx| {
            assert!(ctx.is(GwEvt::MessageCreate));
            let message = ctx.decode::<MessageCreate>().unwrap();
            assert_eq!(message.channel_id(), "200");
            count.fetch_add(1, Ordering::SeqCst);
            async {}
        });
        router
            .dispatch(
                &context(),
                event(
                    GwEvt::MessageCreate,
                    include_str!("../../tests/fixtures/message_create.json"),
                ),
            )
            .await;
        assert_eq!(called.load(Ordering::SeqCst), 2);
    }
}
