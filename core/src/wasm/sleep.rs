//!
//! `Sleep` future and an `async sleep()` function backed by
//! the JavaScript `setTimeout()` and `clearTimeout()` APIs.
//!

#![allow(dead_code)]

use futures::task::AtomicWaker;
use instant::Duration;
use std::future::Future;
use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    task::{Context, Poll},
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen (catch, js_name = setTimeout)]
    pub fn set_timeout(
        closure: &Closure<dyn FnMut()>,
        timeout: u32,
    ) -> std::result::Result<JsValue, JsValue>;
    #[wasm_bindgen (catch, js_name = clearTimeout)]
    pub fn clear_timeout(interval: &JsValue) -> std::result::Result<(), JsValue>;
}

type SleepClosure = Closure<dyn FnMut()>;

struct SleepContext {
    instance: JsValue,
    // this closue, while not read
    // must be retained for the lifetime
    // of this context.
    #[allow(dead_code)]
    closure: SleepClosure,
}

struct Inner {
    ready: AtomicBool,
    waker: AtomicWaker,
    ctx: Mutex<Option<SleepContext>>,
}

/// `Sleep` future used by the `sleep()` function to provide a
/// timeout future that is backed by the JavaScript `createTimeout()`
/// and `clearTimeout()` APIs. The `Sleep` future is meant only for
/// use in WASM32 browser environments. It has an advantage of having
/// `Send` and `Sync` markers.
#[derive(Clone)]
pub struct Sleep {
    inner: Arc<Inner>,
}

unsafe impl Sync for Sleep {}
unsafe impl Send for Sleep {}

impl Sleep {
    /// Create a new `Sleep` future that will resolve after the given duration.
    pub fn new(duration: Duration) -> Self {
        let inner = Arc::new(Inner {
            ready: AtomicBool::new(false),
            waker: AtomicWaker::new(),
            ctx: Mutex::new(None),
        });

        let inner_ = inner.clone();
        let closure = Closure::new(move || {
            inner_.ready.store(true, Ordering::SeqCst);
            if let Some(waker) = inner_.waker.take() {
                waker.wake();
            }
        });

        let instance = set_timeout(&closure, duration.as_millis() as u32).unwrap();

        inner
            .ctx
            .lock()
            .unwrap()
            .replace(SleepContext { instance, closure });

        Sleep { inner }
    }

    #[inline]
    fn clear(&self) {
        if let Some(ctx) = self.inner.ctx.lock().unwrap().take() {
            clear_timeout(ctx.instance.as_ref()).unwrap();
        }
    }

    /// Cancel the current timeout.
    pub fn cancel(&self) {
        self.clear();
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.inner.ready.load(Ordering::SeqCst) {
            true => {
                self.inner.ctx.lock().unwrap().take();
                Poll::Ready(())
            }
            false => {
                self.inner.waker.register(cx.waker());
                if self.inner.ready.load(Ordering::SeqCst) {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

impl Drop for Sleep {
    fn drop(&mut self) {
        self.clear();
    }
}

/// `async sleep()` function backed by the JavaScript `createTimeout()`
pub fn sleep(duration: Duration) -> Sleep {
    Sleep::new(duration)
}
