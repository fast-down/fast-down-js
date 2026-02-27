use crate::{Config, Event, ToNapiError};
use fast_down_ffi::{Rx, create_channel};
use napi::{
  Env, Task,
  bindgen_prelude::{AbortSignal, AsyncTask},
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};
use napi_derive::napi;
use parking_lot::Mutex;
use std::path::PathBuf;
use tokio_util::sync::CancellationToken;
use url::Url;

#[napi(js_name = "prefetch")]
pub fn js_prefetch(
  url: String,
  config: Config,
  signal: Option<AbortSignal>,
) -> AsyncTask<AsyncPerfetch> {
  let token = CancellationToken::new();
  if let Some(signal) = &signal {
    let token = token.clone();
    signal.on_abort(move || token.cancel());
  }
  AsyncTask::with_optional_signal(AsyncPerfetch { url, config, token }, signal)
}

struct AsyncPerfetch {
  url: String,
  config: Config,
  token: CancellationToken,
}

impl Task for AsyncPerfetch {
  type Output = u32;
  type JsValue = JsNumber;

  fn compute(&mut self) -> napi::Result<Self::Output> {
    Ok(1)
  }

  fn resolve(&mut self, env: Env, output: u32) -> napi::Result<Self::JsValue> {
    env.create_uint32(output)
  }
}

// #[napi]
// pub async fn prefetch(
//   url: String,
//   config: Config,
//   mut abort_signal: Option<AbortSignal>,
// ) -> napi::Result<DownloadTask> {
//   let token = CancellationToken::new();
//   if let Some(signal) = abort_signal.take() {
//     let token = token.clone();
//     signal.on_abort(move || token.cancel());
//   }
//   todo!()
//   // prefetch_inner(url, config, token).await
// }

// pub async fn prefetch_inner(
//   url: String,
//   config: Config,
//   token: CancellationToken,
// ) -> napi::Result<DownloadTask> {
//   let url = Url::parse(&url).convert_err("Invalid URL")?;
//   let config = config.into();
//   let (tx, rx) = create_channel();
//   let task = tokio::select! {
//       () = token.cancelled() => Err(napi::Error::from_reason("Prefetch aborted by user"))?,
//       res = fast_down_ffi::prefetch(url, config, tx) => res.convert_err("Prefetch failed")?,
//   };
//   Ok(DownloadTask {
//     inner: Mutex::new(Some(task)),
//     rx: Mutex::new(Some(rx)),
//     token,
//   })
// }

// #[napi]
// pub struct DownloadTask {
//   inner: Mutex<Option<fast_down_ffi::DownloadTask>>,
//   rx: Mutex<Option<Rx>>,
//   token: CancellationToken,
// }

// #[napi]
// impl DownloadTask {
//   /// 正式开始下载
//   #[napi]
//   pub async fn start(
//     &self,
//     save_path: String,
//     #[napi(ts_arg_type = "(event: any) => void")] callback: ThreadsafeFunction<serde_json::Value>,
//   ) -> napi::Result<()> {
//     let task = self
//       .inner
//       .lock()
//       .take()
//       .ok_or_else(|| napi::Error::from_reason("Task already started or invalid"))?;
//     let rx = self
//       .rx
//       .lock()
//       .take()
//       .ok_or_else(|| napi::Error::from_reason("Event receiver already consumed"))?;
//     let token = self.token.clone();

//     // tokio::spawn(async move {
//     //   while let Ok(event) = rx.recv().await {
//     // callback.call(
//     //   serde_json::to_value(&Event::from(event)),
//     //   ThreadsafeFunctionCallMode::NonBlocking,
//     // );
//     //   }
//     // });
//     task
//       .start(PathBuf::from(save_path), token)
//       .await
//       .convert_err("Download error")
//   }
// }

// /// 获取 Task 中的元数据
// #[napi]
// pub fn get_task_info(task: &DownloadTask) -> napi::Result<JsUrlInfo> {
//   let lock = task.inner.lock();
//   let (_, info) = lock
//     .as_ref()
//     .ok_or_else(|| napi::Error::from_reason("Task info consumed"))?;

//   Ok(JsUrlInfo {
//     size: info.size as i64,
//     raw_name: info.raw_name.clone(),
//     fast_download: info.fast_download,
//     file_id: format!("{:?}", info.file_id),
//     final_url: info.final_url.to_string(),
//   })
// }
