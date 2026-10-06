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
use std::time::Duration;

/// Identify ourselves to the GoaT API.
const USER_AGENT: &str = concat!("goat-cli/", env!("CARGO_PKG_VERSION"));

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
        let inner = Client::builder()
            .user_agent(USER_AGENT)
            // no overall timeout, as large searches can legitimately take a while.
            .connect_timeout(Duration::from_secs(30))
            .build()
            .expect("static reqwest client configuration is valid");
        Self { inner }
    }

    /// GET `url`, setting the `Accept` header to `accept`, and return the
    /// response body as a [`String`].
    ///
    /// Connection failures and 5xx responses are retried using
    /// [`again::retry`] with the default policy. Any other non-2xx response,
    /// or a 2xx JSON body in which GoaT reports `"success": false`, is
    /// returned as an [`ErrorKind::Api`] error.
    pub async fn get_text(&self, url: &str, accept: &str) -> Result<String> {
        let body = self.fetch(url, accept).await?;
        // GoaT reports query errors as a JSON body with a 200 status, even
        // when another format (e.g. TSV) was requested.
        if body.trim_start().starts_with('{') {
            if let Ok(v) = serde_json::from_str::<Value>(&body) {
                check_status(&v)?;
            }
        }
        Ok(body)
    }

    /// GET `url` expecting a JSON response body; parse and return a
    /// [`serde_json::Value`].
    pub async fn get_json(&self, url: &str) -> Result<Value> {
        let body = self.fetch(url, "application/json").await?;
        let v = serde_json::from_str(&body).map_err(|e| Error::new(ErrorKind::SerdeJSON(e)))?;
        check_status(&v)?;
        Ok(v)
    }

    /// GET `url` and return the body, erroring on a non-2xx status.
    async fn fetch(&self, url: &str, accept: &str) -> Result<String> {
        // Clone client and own the strings so the closure is 'static and Fn.
        let client = self.inner.clone();
        let url = url.to_owned();
        let accept = accept.to_owned();

        let resp = again::retry(move || {
            let request = client.get(&url).header(ACCEPT, accept.as_str());
            async move {
                let resp = request.send().await?;
                // only server errors are worth retrying
                if resp.status().is_server_error() {
                    resp.error_for_status()
                } else {
                    Ok(resp)
                }
            }
        })
        .await
        .map_err(|e| Error::new(ErrorKind::Reqwest(e)))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| Error::new(ErrorKind::Reqwest(e)))?;

        if !status.is_success() {
            return Err(Error::new(ErrorKind::Api(format!(
                "HTTP {}: {}",
                status,
                error_message(&body)
            ))));
        }
        Ok(body)
    }
}

impl Default for GoatClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Error if a GoaT JSON response has `"status": {"success": false}`,
/// either at the top level or, for `/report`, nested under `report`.
fn check_status(v: &Value) -> Result<()> {
    for status in [&v["status"], &v["report"]["status"]] {
        if status["success"] == Value::Bool(false) {
            let message = status["error"]
                .as_str()
                .unwrap_or("the request was unsuccessful");
            return Err(Error::new(ErrorKind::Api(message.to_string())));
        }
    }
    Ok(())
}

/// Pull a human readable message out of an error response body.
fn error_message(body: &str) -> String {
    if let Ok(v) = serde_json::from_str::<Value>(body) {
        if let Some(m) = v["message"].as_str().or_else(|| v["status"]["error"].as_str()) {
            return m.to_string();
        }
    }
    let body = body.trim();
    match body.char_indices().nth(200) {
        Some((i, _)) => format!("{}...", &body[..i]),
        None => body.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_check_status_success() {
        assert!(check_status(&json!({"status": {"success": true}, "count": 3})).is_ok());
    }

    #[test]
    fn test_check_status_missing_status_is_ok() {
        assert!(check_status(&json!({"count": 3})).is_ok());
    }

    #[test]
    fn test_check_status_failure_reports_api_message() {
        let v = json!({"status": {"success": false, "error": "invalid attribute name in foo > 1"}});
        let err = check_status(&v).unwrap_err();
        assert!(matches!(err.kind(), ErrorKind::Api(_)));
        assert!(err.to_string().contains("invalid attribute name in foo > 1"));
    }

    #[test]
    fn test_check_status_nested_report_failure() {
        let v = json!({"status": {"success": true}, "report": {"status": {"success": false, "error": "unable to load report"}}});
        let err = check_status(&v).unwrap_err();
        assert!(err.to_string().contains("unable to load report"));
    }

    #[test]
    fn test_error_message_prefers_json_message() {
        let body = r#"{"message":"Parameter 'query' must be url encoded.","errors":[]}"#;
        assert_eq!(error_message(body), "Parameter 'query' must be url encoded.");
    }

    #[test]
    fn test_error_message_truncates_non_json() {
        let body = "x".repeat(500);
        assert_eq!(error_message(&body).len(), 203);
    }
}
