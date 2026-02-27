use fast_down_ffi::Event;
use napi_derive::napi;
use std::ops::Range;

#[derive(Debug, Clone)]
#[napi(object, js_name = "Range")]
/// 左闭右开
pub struct JsRange {
  pub start: i64,
  pub end: i64,
}

impl From<Range<u64>> for JsRange {
  #[allow(clippy::cast_possible_wrap)]
  fn from(r: Range<u64>) -> Self {
    Self {
      start: r.start as i64,
      end: r.end as i64,
    }
  }
}

#[derive(Debug, Clone)]
#[napi(object)]
pub struct JsEvent {
  /// 事件类型：`PrefetchError` | `Pulling` | `PullError` | `PullTimeout` | `PullProgress` | `PushError` | `PushProgress` | `FlushError` | `Finished`
  #[napi(js_name = "type")]
  pub event_type: String,
  /// 关联的任务/线程 ID
  pub id: Option<u32>,
  /// 错误消息或描述
  pub message: Option<String>,
  /// 进度范围数据
  pub range: Option<JsRange>,
}

impl From<Event> for JsEvent {
  #[allow(clippy::cast_possible_truncation)]
  fn from(value: Event) -> Self {
    let mut event = Self {
      event_type: String::with_capacity(20),
      id: None,
      message: None,
      range: None,
    };
    match value {
      Event::PrefetchError(e) => {
        event.event_type.push_str("PrefetchError");
        event.message = Some(e);
      }
      Event::Pulling(id) => {
        event.event_type.push_str("Pulling");
        event.id = Some(id as u32);
      }
      Event::PullError(id, e) => {
        event.event_type.push_str("PullError");
        event.id = Some(id as u32);
        event.message = Some(e);
      }
      Event::PullTimeout(id) => {
        event.event_type.push_str("PullTimeout");
        event.id = Some(id as u32);
      }
      Event::PullProgress(id, range) => {
        event.event_type.push_str("PullProgress");
        event.id = Some(id as u32);
        event.range = Some(range.into());
      }
      Event::PushError(id, e) => {
        event.event_type.push_str("PushError");
        event.id = Some(id as u32);
        event.message = Some(e);
      }
      Event::PushProgress(id, range) => {
        event.event_type.push_str("PushProgress");
        event.id = Some(id as u32);
        event.range = Some(range.into());
      }
      Event::FlushError(e) => {
        event.event_type.push_str("FlushError");
        event.message = Some(e);
      }
      Event::Finished(id) => {
        event.event_type.push_str("Finished");
        event.id = Some(id as u32);
      }
    }
    event
  }
}
