#![deny(clippy::all)]

use napi_derive::napi;

mod config;
mod download;
mod error;
mod event;
pub use config::*;
pub use download::*;
pub use error::*;
pub use event::*;

#[napi(object)]
pub struct UrlInfo {
  pub size: i64,
  pub raw_name: String,
  pub supports_range: bool,
  pub fast_download: bool,
  pub final_url: String,
  pub etag: Option<String>,
  pub last_modified: Option<String>,
}
