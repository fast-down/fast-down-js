use crate::{ForceSendExt, JsEvent, ToNapiError};
use fast_down_ffi::{DownloadTask, Rx};
use napi::{
  Status,
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};
use napi_derive::napi;
use parking_lot::Mutex;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;

#[napi(js_name = "DownloadTask")]
pub struct JsDownloadTask {
  inner: Mutex<Option<(DownloadTask, Rx)>>,
  token: CancellationToken,
}

pub type JsDownloadCallback = ThreadsafeFunction<JsEvent, (), JsEvent, Status, false>;

#[napi]
impl JsDownloadTask {
  pub const fn new(task: DownloadTask, rx: Rx, token: CancellationToken) -> Self {
    let inner = Mutex::new(Some((task, rx)));
    Self { inner, token }
  }

  #[napi]
  pub fn cancel(&self) {
    self.token.cancel();
  }

  /// 开始下载任务
  /// @param `save_path` 存储路径
  /// @param `callback` 进度与事件回调函数
  #[napi]
  pub async fn start(
    &self,
    save_path: String,
    #[napi(ts_arg_type = "(event: Event) => void")] callback: JsDownloadCallback,
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
  task: DownloadTask,
  rx: Rx,
  save_path: PathBuf,
  token: CancellationToken,
  callback: JsDownloadCallback,
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
              JsEvent::from(e),
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
