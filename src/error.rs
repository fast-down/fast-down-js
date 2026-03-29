use std::fmt::Display;

pub trait ToNapiError<T> {
  fn convert_err(self, topic: &str) -> napi::Result<T>;
}

impl<T, E: Display> ToNapiError<T> for Result<T, E> {
  fn convert_err(self, topic: &str) -> napi::Result<T> {
    self.map_err(|err| napi::Error::from_reason(format!("{topic}: {err}")))
  }
}

impl<T> ToNapiError<T> for Option<T> {
  fn convert_err(self, topic: &str) -> napi::Result<T> {
    self.ok_or_else(|| napi::Error::from_reason(topic))
  }
}
