use bytes::Bytes;
use fast_down_ffi::ProgressEntry;
use napi::{
  bindgen_prelude::{Promise, Uint8Array},
  threadsafe_function::ThreadsafeFunction,
  Status,
};
use std::sync::Arc;

pub type PushFn =
  ThreadsafeFunction<(i64, Uint8Array), Promise<()>, (i64, Uint8Array), Status, false>;
pub type FlushFn = ThreadsafeFunction<(), Promise<()>, (), Status, false>;

pub struct JsPusher {
  pub push_fn: Arc<PushFn>,
  pub flush_fn: Option<Arc<FlushFn>>,
}

impl fast_down_ffi::Pusher for JsPusher {
  type Error = String;

  fn push(&mut self, range: &ProgressEntry, content: Bytes) -> Result<(), (Self::Error, Bytes)> {
    let push_fn = self.push_fn.clone();
    #[allow(clippy::cast_possible_wrap)]
    let start = range.start as i64;
    let data = Uint8Array::from(content.clone());
    let res = tokio::runtime::Handle::current().block_on(async move {
      let res = push_fn.call_async((start, data)).await;
      match res {
        Ok(promise) => promise.await.map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
      }
    });
    res.map_err(|e| (e, content))
  }

  fn flush(&mut self) -> Result<(), Self::Error> {
    let Some(flush_fn) = self.flush_fn.clone() else {
      return Ok(());
    };
    tokio::runtime::Handle::current().block_on(async move {
      let res = flush_fn.call_async(()).await;
      match res {
        Ok(promise) => promise.await.map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
      }
    })
  }
}
