pub trait ToNapiError<T> {
  fn convert_err(self, topic: &str) -> napi::Result<T>;
}

impl<T, E: ToString> ToNapiError<T> for Result<T, E> {
  fn convert_err(self, topic: &str) -> napi::Result<T> {
    self.map_err(|err| napi::Error::from_reason(format!("{topic}: {}", err.to_string())))
  }
}

impl<T> ToNapiError<T> for Option<T> {
  fn convert_err(self, topic: &str) -> napi::Result<T> {
    self.ok_or_else(|| napi::Error::from_reason(topic))
  }
}
