use crate::errors::{ApiError, Error};

static BASE_URL: &'static str = "https://api.pinata.cloud";

/// Checks to ensure keys are not empty
pub(crate) fn validate_keys(api_key: &str, secret_api_key: &str) -> Result<(), Error> {
  if api_key.is_empty() {
    Err(ApiError::InvalidApiKey())?
  }

  if secret_api_key.is_empty() {
    Err(ApiError::InvalidSecretApiKey())?
  }

  Ok(())
}

pub(crate) fn api_url(path: &str) -> String {
  format!("{}{}", BASE_URL, path)
}
