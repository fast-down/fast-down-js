use crate::{JsCancellationToken, JsConfig, JsDownloadTask, ToNapiError};
use fast_down_ffi::{create_channel, prefetch};
use napi_derive::napi;
use url::Url;

#[napi(js_name = "prefetch")]
#[allow(clippy::trailing_empty_array)]
pub async fn js_prefetch(
  url: String,
  config: Option<JsConfig>,
  token: Option<&JsCancellationToken>,
) -> napi::Result<JsDownloadTask> {
  let url: Url = url.parse().convert_err("Invalid URL")?;
  let config = config.map(|c| c.to_ffi_config()).unwrap_or_default();
  let token = token
    .map(JsCancellationToken::get_token)
    .unwrap_or_default();
  let (tx, rx) = create_channel();
  let task = tokio::select! {
    () = token.cancelled() => Err(napi::Error::new(napi::Status::Cancelled, "Prefetch Cancelled"))?,
    t = prefetch(url, config, tx) => t.convert_err("Prefetch Failed")?
  };
  Ok(JsDownloadTask::new(task, rx, token))
}
