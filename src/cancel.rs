use napi_derive::napi;
use tokio_util::sync::CancellationToken;

#[napi(js_name = "CancellationToken")]
#[derive(Debug, Default, Clone)]
pub struct JsCancellationToken {
  token: CancellationToken,
}

#[napi]
impl JsCancellationToken {
  #[napi(factory)]
  #[must_use]
  pub fn new() -> Self {
    let token = CancellationToken::new();
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

  #[must_use]
  pub fn get_token(&self) -> CancellationToken {
    self.token.clone()
  }
}
