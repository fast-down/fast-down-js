#![deny(clippy::all)]

use napi_derive::napi;

mod cancel;
mod config;
mod download;
mod error;
mod event;
mod force_send;
mod prefetch;
pub use cancel::*;
pub use config::*;
pub use download::*;
pub use error::*;
pub use event::*;
pub use force_send::*;
pub use prefetch::*;

#[napi(object, js_name = "UrlInfo")]
pub struct JsUrlInfo {
  pub size: i64,
  pub raw_name: String,
  pub supports_range: bool,
  pub fast_download: bool,
  pub final_url: String,
  pub etag: Option<String>,
  pub last_modified: Option<String>,
}
