use reqwest::Client as ReqClient;
use reqwest::header::{AUTHORIZATION, HeaderMap};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::{Instant, sleep_until};

const DISCORD_API_BASE: &str = "https://discord.com/api/v10";
const MAX_RATE_LIMIT_RETRIES: usize = 3;

/// Errors returned by the message endpoint. Only explicit 429s are retried.
#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error(transparent)]
    Transport(#[from] reqwest::Error),
    #[error("Discord HTTP error {status}: {body}")]
    Api {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("Discord rate limit could not be retried (retry_after={retry_after:?})")]
    RateLimited { retry_after: Option<Duration> },
}

#[derive(Default)]
struct RateLimits {
    global: Option<Instant>,
    channels: HashMap<String, Instant>,
}

/// Shared HTTP client with timeouts and conservative, serialized message requests.
/// Share one instance per bot token so all callers observe the same cooldowns.
pub struct Http {
    token: String,
    client: ReqClient,
    api_base: String,
    limits: Mutex<RateLimits>,
}

#[derive(Debug, Serialize)]
struct CreateMsg<'a> {
    content: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    message_reference: Option<MessageRef<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    allowed_mentions: Option<AllowedMentions>,
}

#[derive(Debug, Serialize)]
struct MessageRef<'a> {
    message_id: &'a str,
    channel_id: &'a str,
}

#[derive(Debug, Serialize)]
struct AllowedMentions {
    replied_user: bool,
}

#[derive(Deserialize)]
struct RateLimitBody {
    retry_after: Option<f64>,
    #[serde(default)]
    global: bool,
}

impl Http {
    pub fn new(token: impl Into<String>) -> Self {
        Self::try_new(token).expect("failed to build HTTP client")
    }

    /// Fallible constructor for applications that want to handle TLS/client setup errors.
    pub fn try_new(token: impl Into<String>) -> Result<Self, HttpError> {
        Ok(Self {
            token: token.into(),
            client: ReqClient::builder()
                .user_agent("zynx (https://github.com/andrewdotdev/zynx)")
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(30))
                .build()?,
            api_base: DISCORD_API_BASE.to_owned(),
            limits: Mutex::new(RateLimits::default()),
        })
    }

    /// Send a message, retrying at most three times on explicit rate limits.
    /// Network errors and other HTTP statuses are returned without retrying.
    pub async fn send_message(
        &self,
        channel_id: &str,
        content: &str,
        reply_to: Option<&str>,
        mention_replied_user: bool,
    ) -> Result<(), HttpError> {
        let url = format!("{}/channels/{}/messages", self.api_base, channel_id);
        let body = CreateMsg {
            content,
            message_reference: reply_to.map(|message_id| MessageRef {
                message_id,
                channel_id,
            }),
            allowed_mentions: reply_to.map(|_| AllowedMentions {
                replied_user: mention_replied_user,
            }),
        };
        // Serializing requests is deliberate for this small HTTP surface: parallel
        // callers cannot race past a newly observed route or global limit.
        let mut limits = self.limits.lock().await;
        for attempt in 0..=MAX_RATE_LIMIT_RETRIES {
            let now = Instant::now();
            limits.channels.retain(|_, until| *until > now);
            let until = limits
                .global
                .into_iter()
                .chain(limits.channels.get(channel_id).copied())
                .max();
            if let Some(until) = until {
                sleep_until(until).await;
            }

            let response = self
                .client
                .post(&url)
                .header(AUTHORIZATION, format!("Bot {}", self.token))
                .json(&body)
                .send()
                .await?;
            let status = response.status();
            let headers = response.headers().clone();
            // Record exhausted buckets even when the caller cancels while reading the body.
            if headers
                .get("x-ratelimit-remaining")
                .and_then(|v| v.to_str().ok())
                == Some("0")
                && let Some(delay) = header_delay(&headers, "x-ratelimit-reset-after")
                && let Some(until) = Instant::now().checked_add(delay)
            {
                limits.channels.insert(channel_id.to_owned(), until);
            }
            if status.is_success() {
                return Ok(());
            }
            let text = response.text().await?;
            if status != reqwest::StatusCode::TOO_MANY_REQUESTS {
                return Err(HttpError::Api { status, body: text });
            }

            let parsed = serde_json::from_str::<RateLimitBody>(&text).ok();
            let retry_after = header_delay(&headers, "retry-after")
                .into_iter()
                .chain(
                    parsed
                        .as_ref()
                        .and_then(|body| body.retry_after)
                        .and_then(valid_delay),
                )
                .max();
            let Some(delay) = retry_after else {
                return Err(HttpError::RateLimited { retry_after });
            };
            let Some(until) = Instant::now().checked_add(delay) else {
                return Err(HttpError::RateLimited { retry_after });
            };
            let global = parsed.as_ref().is_some_and(|body| body.global)
                || headers
                    .get("x-ratelimit-global")
                    .is_some_and(|v| v == "true")
                || headers
                    .get("x-ratelimit-scope")
                    .is_some_and(|v| v == "global");
            if global {
                limits.global = Some(limits.global.map_or(until, |old| old.max(until)));
            } else {
                limits
                    .channels
                    .entry(channel_id.to_owned())
                    .and_modify(|old| *old = (*old).max(until))
                    .or_insert(until);
            }
            // Persist cooldowns even after retry exhaustion or cancellation.
            if attempt == MAX_RATE_LIMIT_RETRIES {
                return Err(HttpError::RateLimited { retry_after });
            }
        }
        unreachable!("every final attempt returns")
    }
}

fn valid_delay(seconds: f64) -> Option<Duration> {
    Duration::try_from_secs_f64(seconds).ok()
}

fn header_delay(headers: &HeaderMap, name: &str) -> Option<Duration> {
    headers
        .get(name)?
        .to_str()
        .ok()?
        .parse()
        .ok()
        .and_then(valid_delay)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::sync::mpsc;

    pub(crate) async fn mock(
        responses: Vec<(u16, &'static str, &'static str)>,
    ) -> (Http, mpsc::UnboundedReceiver<(Instant, String)>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            for (status, headers, body) in responses {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut request = Vec::new();
                loop {
                    let mut buffer = [0; 4096];
                    let len = socket.read(&mut buffer).await.unwrap();
                    assert!(len > 0);
                    request.extend_from_slice(&buffer[..len]);
                    let text = String::from_utf8_lossy(&request);
                    if let Some(end) = text.find("\r\n\r\n") {
                        let length: usize = text[..end]
                            .lines()
                            .find_map(|line| {
                                let (key, value) = line.split_once(':')?;
                                key.eq_ignore_ascii_case("content-length")
                                    .then(|| value.trim().parse().unwrap())
                            })
                            .unwrap_or(0);
                        if request.len() >= end + 4 + length {
                            break;
                        }
                    }
                }
                let _ = tx.send((Instant::now(), String::from_utf8(request).unwrap()));
                let response = format!(
                    "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n{body}",
                    body.len()
                );
                socket.write_all(response.as_bytes()).await.unwrap();
            }
        });
        let mut http = Http::new("test-token");
        http.api_base = format!("http://{addr}");
        (http, rx)
    }

    #[tokio::test]
    async fn retries_429_after_fractional_delay_and_preserves_payload() {
        let (http, mut requests) = mock(vec![
            (429, "", r#"{"retry_after":0.03,"global":false}"#),
            (200, "", "{}"),
        ])
        .await;
        http.send_message("123", "hello", Some("456"), false)
            .await
            .unwrap();
        let (first, request) = requests.recv().await.unwrap();
        let (second, retry) = requests.recv().await.unwrap();
        assert!(second.duration_since(first) >= Duration::from_millis(30));
        assert_eq!(request, retry);
        let body: serde_json::Value =
            serde_json::from_str(request.split_once("\r\n\r\n").unwrap().1).unwrap();
        assert_eq!(body["message_reference"]["message_id"], "456");
        assert_eq!(body["allowed_mentions"]["replied_user"], false);
    }

    #[tokio::test]
    async fn exhausted_bucket_delays_the_next_request() {
        let (http, mut requests) = mock(vec![
            (
                200,
                "X-RateLimit-Remaining: 0\r\nX-RateLimit-Reset-After: 0.03\r\n",
                "{}",
            ),
            (200, "", "{}"),
        ])
        .await;
        http.send_message("123", "one", None, false).await.unwrap();
        http.send_message("123", "two", None, false).await.unwrap();
        let first = requests.recv().await.unwrap().0;
        let second = requests.recv().await.unwrap().0;
        assert!(second.duration_since(first) >= Duration::from_millis(30));
    }

    #[tokio::test]
    async fn global_limit_survives_retry_exhaustion_and_applies_to_other_channels() {
        let mut responses = vec![(429, "Retry-After: 0\r\n", "{}"); 3];
        responses.push((
            429,
            "Retry-After: 0.03\r\nX-RateLimit-Global: true\r\n",
            "{}",
        ));
        responses.push((200, "", "{}"));
        let (http, mut requests) = mock(responses).await;
        assert!(matches!(
            http.send_message("123", "one", None, false).await,
            Err(HttpError::RateLimited { .. })
        ));
        for _ in 0..3 {
            requests.recv().await.unwrap();
        }
        let first = requests.recv().await.unwrap().0;
        assert!(http.limits.lock().await.global.is_some());
        http.send_message("789", "two", None, false).await.unwrap();
        let second = requests.recv().await.unwrap().0;
        assert!(second.duration_since(first) >= Duration::from_millis(30));
    }

    #[tokio::test]
    async fn retries_are_bounded_and_other_errors_are_not_retried() {
        let (http, mut requests) = mock(vec![(429, "Retry-After: 0\r\n", "{}"); 4]).await;
        assert!(matches!(
            http.send_message("123", "hello", None, false).await,
            Err(HttpError::RateLimited { .. })
        ));
        let mut count = 0;
        while requests.recv().await.is_some() {
            count += 1;
        }
        assert_eq!(count, 4);
        let (http, _) = mock(vec![(500, "", "failure")]).await;
        assert!(
            matches!(http.send_message("123", "hello", None, false).await,
            Err(HttpError::Api { status, .. }) if status == reqwest::StatusCode::INTERNAL_SERVER_ERROR)
        );
    }

    #[tokio::test]
    async fn invalid_retry_delay_returns_error_instead_of_looping() {
        let (http, _) = mock(vec![(429, "Retry-After: -1\r\n", r#"{"retry_after":-2}"#)]).await;
        assert!(matches!(
            http.send_message("123", "hello", None, false).await,
            Err(HttpError::RateLimited { retry_after: None })
        ));
        assert!(valid_delay(f64::NAN).is_none());
        assert!(valid_delay(f64::INFINITY).is_none());
    }
}
