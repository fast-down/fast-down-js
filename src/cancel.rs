use napi_derive::napi;

#[derive(Debug, Default, Clone)]
#[napi]
pub struct CancellationToken {
  pub(crate) token: tokio_util::sync::CancellationToken,
}

#[napi]
impl CancellationToken {
  #[napi(constructor)]
  #[must_use]
  pub fn new() -> Self {
    let token = tokio_util::sync::CancellationToken::new();
    Self { token }
  }

  #[napi]
  pub fn cancel(&self) {
    self.token.cancel();
  }

  #[napi]
  #[must_use]
  pub fn is_cancelled(&self) -> bool {
    self.token.is_cancelled()
  }
}
