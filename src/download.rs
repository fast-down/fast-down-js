use crate::{Event, ForceSendExt, ToNapiError, UrlInfo};
use fast_down_ffi::Rx;
use napi::{
  Status,
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};
use napi_derive::napi;
use parking_lot::Mutex;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

#[napi]
pub struct DownloadTask {
  info: UrlInfo,
  inner: Mutex<Option<(fast_down_ffi::DownloadTask, Rx)>>,
  token: CancellationToken,
}

pub type DownloadCallback = ThreadsafeFunction<Event, (), Event, Status, false>;

#[napi]
impl DownloadTask {
  pub fn new(task: fast_down_ffi::DownloadTask, rx: Rx, token: CancellationToken) -> Self {
    let info = (&task.info).into();
    let inner = Mutex::new(Some((task, rx)));
    Self { info, inner, token }
  }

  #[napi]
  pub fn cancel(&self) {
    self.token.cancel();
  }

  #[napi(getter)]
  pub fn info(&self) -> UrlInfo {
    self.info.clone()
  }

  /// 开始下载任务
  /// @param `save_path` 存储路径
  /// @param `callback` 进度与事件回调函数
  #[napi]
  pub async fn start(
    &self,
    save_path: String,
    #[napi(ts_arg_type = "(event: Event) => void")] callback: DownloadCallback,
  ) -> napi::Result<()> {
    let (task, rx) = self
      .inner
      .lock()
      .take()
      .convert_err("Download task has already been started or is invalid")?;
    let save_path: PathBuf = save_path.into();
    let token = self.token.clone();
    download_inner(task, rx, save_path, token, callback)
      .force_send()
      .await
  }
}

async fn download_inner(
  task: fast_down_ffi::DownloadTask,
  rx: Rx,
  save_path: PathBuf,
  token: CancellationToken,
  callback: DownloadCallback,
) -> napi::Result<()> {
  let download_fut = task.start(save_path, token.clone());
  tokio::pin!(download_fut);
  loop {
    tokio::select! {
      res = &mut download_fut => return res.convert_err("Download Task Error"),
      event = rx.recv() => {
        match event {
          Ok(e) => {
            callback.call(
              Event::from(e),
              ThreadsafeFunctionCallMode::NonBlocking,
            );
          }
          Err(_) => break,
        }
      }
    }
  }
  Ok(())
}
