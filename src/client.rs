//! A shared HTTP client for all GoaT API requests.
//!
//! [`GoatClient`] wraps [`reqwest::Client`] so the underlying connection pool
//! is created once and reused across all concurrent requests instead of being
//! rebuilt on every call. Clone it freely — the inner client is
//! reference-counted.

use crate::error::{Error, ErrorKind, Result};
use reqwest::header::ACCEPT;
use reqwest::Client;
use serde_json::Value;

/// Shared HTTP client for the GoaT API.
#[derive(Clone)]
pub struct GoatClient {
    inner: Client,
}

impl GoatClient {
    /// Construct a new [`GoatClient`].
    ///
    /// Create once per program invocation, then clone into async tasks as
    /// needed — cloning is cheap because the inner client is `Arc`-backed.
    pub fn new() -> Self {
        Self {
            inner: Client::new(),
        }
    }

    /// GET `url`, setting the `Accept` header to `accept`, and return the
    /// response body as a [`String`].
    ///
    /// Retries on transient failures using [`again::retry`] with the
    /// default policy.
    pub async fn get_text(&self, url: &str, accept: &str) -> Result<String> {
        // Clone client and own the strings so the closure is 'static and Fn.
        let client = self.inner.clone();
        let url = url.to_owned();
        let accept = accept.to_owned();

        let resp = again::retry(move || client.get(&url).header(ACCEPT, accept.as_str()).send())
            .await
            .map_err(|e| Error::new(ErrorKind::Reqwest(e)))?;

        resp.text()
            .await
            .map_err(|e| Error::new(ErrorKind::Reqwest(e)))
    }

    /// GET `url` expecting a JSON response body; parse and return a
    /// [`serde_json::Value`].
    pub async fn get_json(&self, url: &str) -> Result<Value> {
        let body = self.get_text(url, "application/json").await?;
        serde_json::from_str(&body).map_err(|e| Error::new(ErrorKind::SerdeJSON(e)))
    }
}

impl Default for GoatClient {
    fn default() -> Self {
        Self::new()
    }
}
