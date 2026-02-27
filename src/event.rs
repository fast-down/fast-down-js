use napi_derive::napi;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub enum Event {
  PrefetchError(String),
  Pulling(u32),
  PullError(u32, String),
  PullTimeout(u32),
  PullProgress(u32, Range),
  PushError(u32, String),
  PushProgress(u32, Range),
  FlushError(String),
  Finished(u32),
}

impl From<fast_down_ffi::Event> for Event {
  #[allow(clippy::cast_possible_truncation)]
  fn from(value: fast_down_ffi::Event) -> Self {
    match value {
      fast_down_ffi::Event::PrefetchError(e) => Self::PrefetchError(e),
      fast_down_ffi::Event::Pulling(id) => Self::Pulling(id as u32),
      fast_down_ffi::Event::PullError(id, e) => Self::PullError(id as u32, e),
      fast_down_ffi::Event::PullTimeout(id) => Self::PullTimeout(id as u32),
      fast_down_ffi::Event::PullProgress(id, range) => Self::PullProgress(id as u32, range.into()),
      fast_down_ffi::Event::PushError(id, e) => Self::PushError(id as u32, e),
      fast_down_ffi::Event::PushProgress(id, range) => Self::PushProgress(id as u32, range.into()),
      fast_down_ffi::Event::FlushError(e) => Self::FlushError(e),
      fast_down_ffi::Event::Finished(id) => Self::Finished(id as u32),
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[napi(object)]
/// 左闭右开
pub struct Range {
  /// 包括 start
  pub start: i64,
  /// 不包括 end
  pub end: i64,
}

impl From<core::ops::Range<u64>> for Range {
  #[allow(clippy::cast_possible_wrap)]
  fn from(r: core::ops::Range<u64>) -> Self {
    Self {
      start: r.start as i64,
      end: r.end as i64,
    }
  }
}
