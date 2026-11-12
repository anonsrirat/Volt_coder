//! Server-Sent Events (SSE) helpers shared by streaming adapters.

use crate::error::{AppError, Result};
use eventsource_stream::Eventsource;
use futures::StreamExt;
use reqwest::Response;
use std::pin::Pin;

/// A boxed, `Send`, `Unpin` stream of parsed SSE `data:` payloads.
///
/// Each item is the raw JSON string found in a `data:` line (ignoring keep-alive
/// comments and the terminating `[DONE]` sentinel, which is surfaced as `None`
/// to signal end-of-stream).
pub type SseStream = Pin<Box<dyn futures::Stream<Item = Result<Option<String>>> + Send>>;

/// Turn an HTTP response body into a stream of parsed SSE `data:` payloads.
pub fn sse_json_stream(resp: Response) -> SseStream {
    let byte_stream = resp.bytes_stream();
    let mapped = byte_stream.eventsource().map(|event| match event {
        Ok(ev) => {
            let data = ev.data;
            if data.trim() == "[DONE]" {
                Ok(None)
            } else if data.trim().is_empty() {
                Ok(Some(String::new()))
            } else {
                Ok(Some(data))
            }
        }
        Err(e) => Err(AppError::Provider(format!("sse error: {e}"))),
    });
    Box::pin(mapped)
}
