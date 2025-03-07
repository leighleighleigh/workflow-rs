//!
//! Subscription-based channel multiplexer - WASM client.
//!

use crate::result::Result;
use futures::{select, FutureExt};
use js_sys::Function;
use serde::Serialize;
use serde_wasm_bindgen::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use workflow_core::channel::{DuplexChannel, Multiplexer, MultiplexerChannel};
use workflow_core::sendable::Sendable;
use workflow_core::task::*;
use workflow_log::log_error;

pub struct Inner {
    callback: Mutex<Option<Sendable<Function>>>,
    task_running: AtomicBool,
    task_ctl: DuplexChannel,
}

///
/// [`MultiplexerClient`] is an object meant to be used in WASM environment to
/// process channel events.
///
#[wasm_bindgen(inspectable)]
#[derive(Clone)]
pub struct MultiplexerClient {
    inner: Arc<Inner>,
}

impl Default for MultiplexerClient {
    fn default() -> Self {
        MultiplexerClient::new()
    }
}

impl MultiplexerClient {
    pub async fn start_notification_task<T>(&self, multiplexer: &Multiplexer<T>) -> Result<()>
    where
        T: Clone + Serialize + Send + Sync + 'static,
    {
        let inner = self.inner.clone();

        if inner.task_running.load(Ordering::SeqCst) {
            panic!("ReflectorClient task is already running");
        }
        let ctl_receiver = inner.task_ctl.request.receiver.clone();
        let ctl_sender = inner.task_ctl.response.sender.clone();
        inner.task_running.store(true, Ordering::SeqCst);

        let channel = MultiplexerChannel::from(multiplexer);

        spawn(async move {
            loop {
                select! {
                    _ = ctl_receiver.recv().fuse() => {
                        break;
                    },
                    msg = channel.receiver.recv().fuse() => {
                        // log_info!("notification: {:?}",msg);
                        if let Ok(notification) = &msg {
                            if let Some(callback) = inner.callback.lock().unwrap().as_ref() {
                                // if let Ok(event) = JsValue::try_from(notification) {
                                if let Ok(event) = to_value(notification) {
                                    if let Err(err) = callback.0.call1(&JsValue::undefined(), &event) {
                                        log_error!("Error while executing notification callback: {:?}", err);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            channel.close();
            ctl_sender.send(()).await.ok();
        });

        Ok(())
    }
}

#[wasm_bindgen]
impl MultiplexerClient {
    #[wasm_bindgen(constructor)]
    pub fn new() -> MultiplexerClient {
        MultiplexerClient {
            inner: Arc::new(Inner {
                callback: Mutex::new(None),
                task_running: AtomicBool::new(false),
                task_ctl: DuplexChannel::oneshot(),
            }),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn handler(&self) -> JsValue {
        if let Some(callback) = self.inner.callback.lock().unwrap().as_ref() {
            callback.as_ref().clone().into()
        } else {
            JsValue::UNDEFINED
        }
    }

    #[wasm_bindgen(js_name = "setHandler")]
    pub fn set_handler(&self, callback: JsValue) -> Result<()> {
        if callback.is_function() {
            let fn_callback: Function = callback.into();
            self.inner
                .callback
                .lock()
                .unwrap()
                .replace(fn_callback.into());
        } else {
            self.remove_handler()?;
        }
        Ok(())
    }

    /// `removeHandler` must be called when releasing ReflectorClient
    /// to stop the background event processing task
    #[wasm_bindgen(js_name = "removeHandler")]
    pub fn remove_handler(&self) -> Result<()> {
        *self.inner.callback.lock().unwrap() = None;
        Ok(())
    }

    #[wasm_bindgen(js_name = "stop")]
    pub async fn stop_notification_task(&self) -> Result<()> {
        let inner = &self.inner;
        if inner.task_running.load(Ordering::SeqCst) {
            inner.task_running.store(false, Ordering::SeqCst);
            inner
                .task_ctl
                .signal(())
                .await
                .map_err(|err| JsValue::from_str(&err.to_string()))?;
        }
        Ok(())
    }
}
