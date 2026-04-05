use crate::ToNapiError;
use bytes::{Bytes, BytesMut};
use crossfire::{oneshot, spsc, Tx};
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
  pub cache: BTreeMap<u64, Bytes>,
  pub cache_size: usize,
  pub buffer_size: usize,
  pub tx: Tx<spsc::Array<(Action, oneshot::TxOneshot<napi::Result<()>>)>>,
}

pub enum Action {
  Push(i64, Bytes),
  Flush,
}

impl JsPusher {
  #[must_use]
  pub fn new(push_fn: PushFn, flush_fn: Option<FlushFn>, buffer_size: usize) -> Self {
    let (tx, rx) =
      spsc::bounded_blocking_async::<(Action, oneshot::TxOneshot<napi::Result<()>>)>(1);
    tokio::spawn(async move {
      while let Ok((action, tx)) = rx.recv().await {
        let res = match action {
          Action::Push(offset, data) => {
            match push_fn.call_async((offset, Uint8Array::from(data))).await {
              Ok(promise) => promise.await.map_err(|e| e.to_string()),
              Err(e) => Err(e.to_string()),
            }
          }
          Action::Flush => match &flush_fn {
            Some(flush_fn) => match flush_fn.call_async(()).await {
              Ok(promise) => promise.await.map_err(|e| e.to_string()),
              Err(e) => Err(e.to_string()),
            },
            None => Ok(()),
          },
        };
        tx.send(res.convert_err("JsPusher Error"));
      }
    });
    Self {
      cache: BTreeMap::new(),
      cache_size: 0,
      buffer_size,
      tx,
    }
  }

  fn send_to_js(&self, offset: u64, content: Bytes) -> napi::Result<()> {
    let (tx, rx) = oneshot::oneshot();
    #[allow(clippy::cast_possible_wrap)]
    self
      .tx
      .send((Action::Push(offset as i64, content), tx))
      .convert_err("JsPusher Error")?;
    rx.recv().convert_err("JsPusher Error").flatten()
  }

  /// 内部 flush：取出 `cache` 中的小块，合并连续的小块，然后调用 `send_to_js`
  fn flush_buffer(&mut self) -> napi::Result<()> {
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
  type Error = napi::Error;

  fn push(&mut self, range: &ProgressEntry, content: Bytes) -> Result<(), (Self::Error, Bytes)> {
    let start = range.start;
    let new_len = content.len();
    match self.cache.get(&start) {
      Some(old) if new_len <= old.len() => return Ok(()),
      Some(old) => self.cache_size -= old.len(),
      None => {}
    }
    self.cache.insert(start, content.clone());
    self.cache_size += new_len;
    if self.cache_size >= self.buffer_size {
      if let Err(e) = self.flush_buffer() {
        return Err((e, content));
      }
    }
    Ok(())
  }

  fn flush(&mut self) -> Result<(), Self::Error> {
    self.flush_buffer()?;
    let (tx, rx) = oneshot::oneshot();
    self
      .tx
      .send((Action::Flush, tx))
      .convert_err("JsPusher Error")?;
    rx.recv().convert_err("JsPusher Error").flatten()
  }
}
