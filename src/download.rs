use crate::{Event, FlushFn, ForceSendExt, JsPusher, PushFn, ToNapiError, UrlInfo};
use fast_down_ffi::{BoxPusher, Error, Rx};
use napi::{
  bindgen_prelude::Uint8Array,
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Status,
};
use napi_derive::napi;
use parking_lot::Mutex;
use std::{future::Future, sync::Arc};
use tokio_util::sync::CancellationToken;

#[napi]
pub struct DownloadTask {
  info: UrlInfo,
  task: fast_down_ffi::DownloadTask,
  rx: Mutex<Option<Rx>>,
  token: CancellationToken,
  child_token: Mutex<CancellationToken>,
}

pub type DownloadCallback = ThreadsafeFunction<Event, (), Event, Status, false>;

#[napi]
impl DownloadTask {
  pub fn new(task: fast_down_ffi::DownloadTask, rx: Rx, token: CancellationToken) -> Self {
    let child_token = token.child_token();
    child_token.cancel();
    Self {
      info: (&task.info).into(),
      task,
      rx: Mutex::new(Some(rx)),
      child_token: Mutex::new(child_token),
      token,
    }
  }

  #[napi]
  /// 彻底取消下载任务，不可恢复
  pub fn cancel(&self) {
    self.token.cancel();
  }

  #[napi]
  pub fn is_cancelled(&self) -> bool {
    self.token.is_cancelled()
  }

  #[napi]
  /// 暂停下载任务，可恢复
  pub fn pause(&self) {
    self.child_token.lock().cancel();
  }

  #[napi]
  pub fn is_paused(&self) -> bool {
    self.child_token.lock().is_cancelled()
  }

  #[napi(getter)]
  pub fn info(&self) -> UrlInfo {
    self.info.clone()
  }

  fn rx(&self) -> napi::Result<Rx> {
    self
      .rx
      .lock()
      .take()
      .convert_err("Download task is running")
  }

  fn child_token(&self) -> CancellationToken {
    let child_token = self.token.child_token();
    *self.child_token.lock() = child_token.clone();
    child_token
  }

  /// 开始下载任务写入到指定路径
  /// @param `save_path` 存储路径
  /// @param `callback` 进度与事件回调函数
  #[napi]
  pub async fn start(
    &self,
    save_path: String,
    #[napi(ts_arg_type = "(event: Event) => void")] callback: Option<DownloadCallback>,
  ) -> napi::Result<()> {
    let rx = self.rx()?;
    let child_token = self.child_token();
    let download_fut = self.task.start(save_path.into(), child_token.clone());
    let (res, rx) = download_inner(download_fut, rx, callback)
      .force_send()
      .await;
    *self.rx.lock() = Some(rx);
    child_token.cancel();
    res
  }

  /// 开始下载任务并返回内存中的数据
  /// @param `callback` 进度与事件回调函数
  #[napi]
  pub async fn start_in_memory(
    &self,
    #[napi(ts_arg_type = "(event: Event) => void")] callback: Option<DownloadCallback>,
  ) -> napi::Result<Uint8Array> {
    let rx = self.rx()?;
    let child_token = self.child_token();
    let download_fut = self.task.start_in_memory(child_token.clone());
    let (res, rx) = download_inner(download_fut, rx, callback)
      .force_send()
      .await;
    *self.rx.lock() = Some(rx);
    child_token.cancel();
    res.map(Uint8Array::new)
  }

  /// 开始下载任务并使用自定义的 pusher
  /// @param `push_fn` 数据推送回调函数
  /// @param `flush_fn` 缓冲区刷新回调函数
  /// @param `callback` 进度与事件回调函数
  #[napi]
  pub async fn start_with_pusher(
    &self,
    #[napi(ts_arg_type = "(data: [number, Uint8Array]) => Promise<void>")] push_fn: Arc<PushFn>,
    #[napi(ts_arg_type = "() => Promise<void>")] flush_fn: Option<Arc<FlushFn>>,
    #[napi(ts_arg_type = "(event: Event) => void")] callback: Option<DownloadCallback>,
  ) -> napi::Result<()> {
    let rx = self.rx()?;
    let child_token = self.child_token();
    let pusher = JsPusher::new(push_fn, flush_fn, self.task.config.write_buffer_size);
    let download_fut = self
      .task
      .start_with_pusher(BoxPusher::new(pusher), child_token.clone());
    let (res, rx) = download_inner(download_fut, rx, callback)
      .force_send()
      .await;
    *self.rx.lock() = Some(rx);
    child_token.cancel();
    res
  }
}

async fn download_inner<R>(
  download_fut: impl Future<Output = Result<R, Error>>,
  rx: Rx,
  callback: Option<DownloadCallback>,
) -> (napi::Result<R>, Rx) {
  tokio::pin!(download_fut);
  let res = loop {
    tokio::select! {
      res = &mut download_fut => break res,
      event = rx.recv() => {
        match event {
          Ok(e) => {
            if let Some(ref cb) = callback {
              cb.call(
                Event::from(e),
                ThreadsafeFunctionCallMode::NonBlocking,
              );
            }
          }
          Err(_) => break download_fut.await,
        }
      }
    }
  };
  while let Ok(e) = rx.try_recv() {
    if let Some(ref cb) = callback {
      cb.call(Event::from(e), ThreadsafeFunctionCallMode::NonBlocking);
    }
  }
  let res = res.convert_err("Download task failed");
  (res, rx)
}
