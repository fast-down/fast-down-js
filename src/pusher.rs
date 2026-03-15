use bytes::{Bytes, BytesMut};
use fast_down_ffi::ProgressEntry;
use napi::{
  bindgen_prelude::{Promise, Uint8Array},
  threadsafe_function::ThreadsafeFunction,
  Status,
};
use std::{collections::BTreeMap, sync::Arc};

pub type PushFn =
  ThreadsafeFunction<(i64, Uint8Array), Promise<()>, (i64, Uint8Array), Status, false>;
pub type FlushFn = ThreadsafeFunction<(), Promise<()>, (), Status, false>;

pub struct JsPusher {
  pub push_fn: Arc<PushFn>,
  pub flush_fn: Option<Arc<FlushFn>>,
  pub cache: BTreeMap<u64, Bytes>,
  pub cache_size: usize,
  pub buffer_size: usize,
}

impl JsPusher {
  #[must_use]
  pub const fn new(
    push_fn: Arc<PushFn>,
    flush_fn: Option<Arc<FlushFn>>,
    buffer_size: usize,
  ) -> Self {
    Self {
      push_fn,
      flush_fn,
      cache: BTreeMap::new(),
      cache_size: 0,
      buffer_size,
    }
  }

  fn send_to_js(push_fn: &PushFn, start: u64, content: Bytes) -> Result<(), String> {
    #[allow(clippy::cast_possible_wrap)]
    let start_i64 = start as i64;
    let data = Uint8Array::from(content);
    tokio::runtime::Handle::current().block_on(async move {
      let res = push_fn.call_async((start_i64, data)).await;
      match res {
        Ok(promise) => promise.await.map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
      }
    })
  }

  /// 内部 flush：取出 `cache` 中的小块，合并连续的小块，然后调用 `send_to_js`
  fn flush_buffer(&mut self) -> Result<(), String> {
    let mut merged_start: Option<u64> = None;
    let mut merged_bytes = BytesMut::new();
    while let Some((start, chunk)) = self.cache.pop_first() {
      let len = chunk.len();
      self.cache_size -= len;
      if let Some(m_start) = merged_start {
        if m_start + (merged_bytes.len() as u64) == start {
          merged_bytes.extend_from_slice(&chunk);
          continue;
        }
        let data_to_send = merged_bytes.split().freeze();
        if let Err(e) = Self::send_to_js(&self.push_fn, m_start, data_to_send.clone()) {
          let len_to_send = data_to_send.len();
          self.cache.insert(m_start, data_to_send);
          self.cache.insert(start, chunk);
          self.cache_size += len_to_send + len;
          return Err(e);
        }
      }
      merged_start = Some(start);
      merged_bytes.extend_from_slice(&chunk);
    }
    if let Some(m_start) = merged_start {
      if !merged_bytes.is_empty() {
        let data_to_send = merged_bytes.freeze();
        if let Err(e) = Self::send_to_js(&self.push_fn, m_start, data_to_send.clone()) {
          let len_to_send = data_to_send.len();
          self.cache.insert(m_start, data_to_send);
          self.cache_size += len_to_send;
          return Err(e);
        }
      }
    }
    Ok(())
  }
}

impl fast_down_ffi::Pusher for JsPusher {
  type Error = String;

  fn push(&mut self, range: &ProgressEntry, content: Bytes) -> Result<(), (Self::Error, Bytes)> {
    let start = range.start;
    let new_len = content.len();
    match self.cache.get(&start) {
      Some(old) if new_len <= old.len() => return Ok(()),
      Some(old) => self.cache_size -= old.len(),
      None => {}
    }
    self.cache.insert(start, content);
    self.cache_size += new_len;
    if self.cache_size >= self.buffer_size {
      self.flush_buffer().map_err(|e| {
        let failed_bytes = self.cache.remove(&range.start).unwrap_or_default();
        self.cache_size -= failed_bytes.len();
        (e, failed_bytes)
      })?;
    }
    Ok(())
  }

  fn flush(&mut self) -> Result<(), Self::Error> {
    self.flush_buffer()?;
    if let Some(flush_fn) = &self.flush_fn {
      tokio::runtime::Handle::current().block_on(async move {
        let res = flush_fn.call_async(()).await;
        match res {
          Ok(promise) => promise.await.map_err(|e| e.to_string()),
          Err(e) => Err(e.to_string()),
        }
      })?;
    }
    Ok(())
  }
}
