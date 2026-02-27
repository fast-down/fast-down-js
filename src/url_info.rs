use napi_derive::napi;
use std::string::ToString;

#[derive(Debug, Clone)]
#[napi]
pub struct UrlInfo {
  pub size: i64,
  /// 服务器返回的原始文件名，必须清洗掉不合法字符才能安全使用
  ///
  /// 使用 `UrlInfo.filename()` 可用直接获取安全的文件名
  pub raw_name: String,
  pub supports_range: bool,
  pub fast_download: bool,
  pub final_url: String,
  pub etag: Option<String>,
  pub last_modified: Option<String>,
}

#[napi]
impl UrlInfo {
  #[napi]
  #[must_use]
  /// 返回清洗后的安全文件名
  pub fn filename(&self) -> String {
    sanitize_filename::sanitize_with_options(
      &self.raw_name,
      sanitize_filename::Options {
        windows: cfg!(windows),
        truncate: true,
        replacement: "_",
      },
    )
  }
}

impl From<&fast_down_ffi::UrlInfo> for UrlInfo {
  fn from(v: &fast_down_ffi::UrlInfo) -> Self {
    Self {
      #[allow(clippy::cast_possible_wrap)]
      size: v.size as i64,
      raw_name: v.raw_name.clone(),
      supports_range: v.supports_range,
      fast_download: v.fast_download,
      final_url: v.final_url.to_string(),
      etag: v.file_id.etag.as_ref().map(ToString::to_string),
      last_modified: v.file_id.last_modified.as_ref().map(ToString::to_string),
    }
  }
}
