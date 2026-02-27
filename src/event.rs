use napi_derive::napi;

#[derive(Debug, Clone)]
#[napi(object)]
/// 左闭右开
pub struct Range {
  pub start: i64,
  pub end: i64,
}

impl From<std::ops::Range<u64>> for Range {
  #[allow(clippy::cast_possible_wrap)]
  fn from(r: std::ops::Range<u64>) -> Self {
    Self {
      start: r.start as i64,
      end: r.end as i64,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct Event {
  /// 事件类型
  #[napi(
    js_name = "type",
    ts_type = "'PrefetchError' | 'Pulling' | 'PullError' | 'PullTimeout' | 'PullProgress' | 'PushError' | 'PushProgress' | 'FlushError' | 'Finished'"
  )]
  pub event_type: String,
  /// 关联的线程 ID
  pub id: Option<u32>,
  /// 错误消息或描述
  pub message: Option<String>,
  /// 进度范围数据
  pub range: Option<Range>,
}

impl From<fast_down_ffi::Event> for Event {
  #[allow(clippy::cast_possible_truncation)]
  fn from(value: fast_down_ffi::Event) -> Self {
    let mut event = Self {
      event_type: String::with_capacity(20),
      id: None,
      message: None,
      range: None,
    };
    match value {
      fast_down_ffi::Event::PrefetchError(e) => {
        event.event_type.push_str("PrefetchError");
        event.message = Some(e);
      }
      fast_down_ffi::Event::Pulling(id) => {
        event.event_type.push_str("Pulling");
        event.id = Some(id as u32);
      }
      fast_down_ffi::Event::PullError(id, e) => {
        event.event_type.push_str("PullError");
        event.id = Some(id as u32);
        event.message = Some(e);
      }
      fast_down_ffi::Event::PullTimeout(id) => {
        event.event_type.push_str("PullTimeout");
        event.id = Some(id as u32);
      }
      fast_down_ffi::Event::PullProgress(id, range) => {
        event.event_type.push_str("PullProgress");
        event.id = Some(id as u32);
        event.range = Some(range.into());
      }
      fast_down_ffi::Event::PushError(id, e) => {
        event.event_type.push_str("PushError");
        event.id = Some(id as u32);
        event.message = Some(e);
      }
      fast_down_ffi::Event::PushProgress(id, range) => {
        event.event_type.push_str("PushProgress");
        event.id = Some(id as u32);
        event.range = Some(range.into());
      }
      fast_down_ffi::Event::FlushError(e) => {
        event.event_type.push_str("FlushError");
        event.message = Some(e);
      }
      fast_down_ffi::Event::Finished(id) => {
        event.event_type.push_str("Finished");
        event.id = Some(id as u32);
      }
    }
    event
  }
}
