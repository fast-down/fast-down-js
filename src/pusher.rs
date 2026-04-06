use bytes::{Bytes, BytesMut};
use fast_down_ffi::ProgressEntry;
use napi::{
  bindgen_prelude::{Promise, Uint8Array},
  threadsafe_function::ThreadsafeFunction,
  Status,
};
use std::collections::BTreeMap;

pub type PushFn =
  ThreadsafeFunction<(i64, Uint8Array), Promise<()>, (i64, Uint8Array), Status, false>;
pub type FlushFn = ThreadsafeFunction<(), Promise<()>, (), Status, false>;

pub struct JsPusher {
  pub push_fn: PushFn,
  pub flush_fn: Option<FlushFn>,
  pub cache: BTreeMap<u64, Bytes>,
  pub cache_size: usize,
  pub buffer_size: usize,
}

impl JsPusher {
  #[must_use]
  pub const fn new(push_fn: PushFn, flush_fn: Option<FlushFn>, buffer_size: usize) -> Self {
    Self {
      push_fn,
      flush_fn,
      cache: BTreeMap::new(),
      cache_size: 0,
      buffer_size,
    }
  }

  fn send_to_js(&self, start: u64, content: Bytes) -> Result<(), String> {
    #[allow(clippy::cast_possible_wrap)]
    let start_i64 = start as i64;
    let data = Uint8Array::from(content);
    tokio::runtime::Handle::current().block_on(async move {
      let res = self.push_fn.call_async((start_i64, data)).await;
      match res {
        Ok(promise) => promise.await.map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
      }
    })
  }

  /// 内部 flush：取出 `cache` 中的小块，合并连续的小块，然后调用 `send_to_js`
  fn flush_buffer(&mut self) -> Result<(), String> {
    let mut curr_start: Option<u64> = None;
    let mut curr_end: u64 = 0;
    let mut buf = BytesMut::new();
    while let Some((start, chunk)) = self.cache.pop_first() {
      let len = chunk.len();
      self.cache_size -= len;
      if let Some(c_start) = curr_start {
        if start <= curr_end {
          let overlap = curr_end - start;
          if overlap < (len as u64) {
            #[allow(clippy::cast_possible_truncation)]
            let new_data = &chunk[(overlap as usize)..];
            buf.extend_from_slice(new_data);
            curr_end += new_data.len() as u64;
          }
          continue;
        }
        let data_to_send = buf.split().freeze();
        if let Err(e) = self.send_to_js(c_start, data_to_send.clone()) {
          self.cache_size += data_to_send.len() + len;
          self.cache.insert(c_start, data_to_send);
          self.cache.insert(start, chunk);
          return Err(e);
        }
      }
      curr_start = Some(start);
      curr_end = start + len as u64;
      buf.extend_from_slice(&chunk);
    }
    if let Some(c_start) = curr_start {
      if !buf.is_empty() {
        let data_to_send = buf.freeze();
        if let Err(e) = self.send_to_js(c_start, data_to_send.clone()) {
          self.cache_size += data_to_send.len();
          self.cache.insert(c_start, data_to_send);
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
