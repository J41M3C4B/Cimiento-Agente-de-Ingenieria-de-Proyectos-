//! Anthropic Messages API over plain HTTPS (there is no official Rust SDK).
//! Request shape: structured outputs (`output_config.format`), a cacheable system
//! prompt, no sampling parameters, no forced tool use.

use super::settings::{AiSettings, ProviderKind};
use super::{AiError, AiProvider, AiRequest, AiResponse, ModelTier, Usage};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const API_VERSION: &str = "2023-06-01";
const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";
/// Models on which the server-side `fallbacks: "default"` option exists.
const FALLBACK_MODELS: &[&str] = &["claude-fable-5-1", "claude-opus-5-5", "claude-opus-5", "claude-sonnet-5-5"];

pub struct AnthropicProvider {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    settings: AiSettings,
}

impl AnthropicProvider {
    pub fn new(api_key: String, settings: AiSettings) -> Self {
        Self::with_base_url(api_key, settings, DEFAULT_BASE_URL.to_string())
    }

    pub fn with_base_url(api_key: String, settings: AiSettings, base_url: String) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .connect_timeout(Duration::from_secs(15))
            .build()
            .expect("http client");
        AnthropicProvider { http, base_url, api_key, settings }
    }

    fn model(&self, tier: ModelTier) -> &str {
        self.settings.model_of(ProviderKind::Anthropic, tier)
    }

    /// The JSON body for a request. Public inside the crate so it can be unit-tested.
    pub(crate) fn build_body(&self, req: &AiRequest) -> Value {
        let model = self.model(req.tier);
        let mut content: Vec<Value> = req
            .context
            .iter()
            .map(|c| json!({ "type": "text", "text": c }))
            .collect();
        content.push(json!({ "type": "text", "text": req.user }));

        let mut output_config = json!({ "format": { "type": "json_schema", "schema": req.output_schema } });
        // Haiku 4.5 rejects `effort`; only the strong tier carries it.
        if req.tier == ModelTier::Strong && !self.settings.effort_strong.is_empty() && !model.contains("haiku") {
            output_config["effort"] = json!(self.settings.effort_strong);
        }
        let headroom = if req.tier == ModelTier::Strong { self.settings.thinking_headroom } else { 0 };

        let mut body = json!({
            "model": model,
            "max_tokens": req.max_output_tokens + headroom,
            "system": [{ "type": "text", "text": req.system, "cache_control": { "type": "ephemeral" } }],
            "messages": [{ "role": "user", "content": content }],
            "output_config": output_config,
        });
        if self.uses_fallbacks(model) {
            body["fallbacks"] = json!("default");
        }
        body
    }

    fn uses_fallbacks(&self, model: &str) -> bool {
        self.settings.use_fallbacks && FALLBACK_MODELS.contains(&model)
    }
}

/// Turns a successful response body into an `AiResponse`.
pub(crate) fn parse_response(body: &Value) -> Result<AiResponse, AiError> {
    match body["stop_reason"].as_str() {
        Some("refusal") => return Err(AiError::Refused),
        Some("max_tokens") => return Err(AiError::Truncated),
        _ => {}
    }
    let text = body["content"]
        .as_array()
        .and_then(|blocks| blocks.iter().find(|b| b["type"] == "text"))
        .and_then(|b| b["text"].as_str())
        .ok_or_else(|| AiError::BadOutput("no text block in the response".into()))?;
    let value: Value =
        serde_json::from_str(text).map_err(|e| AiError::BadOutput(format!("not valid JSON: {e}")))?;
    let u = &body["usage"];
    let n = |k: &str| u[k].as_u64().unwrap_or(0);
    Ok(AiResponse {
        value,
        model: body["model"].as_str().unwrap_or_default().to_string(),
        usage: Usage {
            input_tokens: n("input_tokens"),
            output_tokens: n("output_tokens"),
            cache_read_tokens: n("cache_read_input_tokens"),
            cache_write_tokens: n("cache_creation_input_tokens"),
            // Anthropic does not report thinking apart from the output it bills.
            thought_tokens: 0,
        },
    })
}

fn error_message(body: &Value) -> String {
    body["error"]["message"].as_str().unwrap_or("").chars().take(300).collect()
}

#[async_trait]
impl AiProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn model_name(&self, tier: ModelTier) -> String {
        self.model(tier).to_string()
    }

    async fn complete(&self, req: &AiRequest) -> Result<AiResponse, AiError> {
        let model = self.model(req.tier).to_string();
        let mut rb = self
            .http
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .timeout(Duration::from_secs(req.task.timeout_secs()))
            .json(&self.build_body(req));
        if self.uses_fallbacks(&model) {
            rb = rb.header("anthropic-beta", FALLBACK_BETA);
        }
        let resp = rb.send().await.map_err(|e| {
            // no connection is "offline"; a request that was sent and took too long is a timeout
            if e.is_connect() {
                AiError::Offline
            } else if e.is_timeout() {
                AiError::Timeout
            } else if e.is_request() {
                AiError::Offline
            } else {
                AiError::Internal(e.to_string())
            }
        })?;
        let status = resp.status().as_u16();
        let body: Value = resp.json().await.unwrap_or(Value::Null);
        match status {
            200 => parse_response(&body),
            401 | 403 => Err(AiError::Auth),
            429 => Err(AiError::RateLimited),
            s => Err(AiError::Http { status: s, message: error_message(&body) }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::AiTask;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn schema() -> Value {
        json!({"type":"object","properties":{"enough":{"type":"boolean"},"question":{"type":"string"}},
               "required":["enough","question"],"additionalProperties":false})
    }

    fn request(tier: ModelTier) -> AiRequest {
        AiRequest {
            task: AiTask::ConversationTurn,
            tier,
            system: "SISTEMA FIJO".into(),
            context: vec!["Contexto A".into()],
            user: "Pregunta".into(),
            output_schema: schema(),
            max_output_tokens: 150,
        }
    }

    fn ok_body(text: &str) -> Value {
        json!({
            "model": "claude-haiku-4-5", "stop_reason": "end_turn",
            "content": [{"type": "text", "text": text}],
            "usage": {"input_tokens": 120, "output_tokens": 30, "cache_read_input_tokens": 100, "cache_creation_input_tokens": 0}
        })
    }

    #[test]
    fn light_request_has_no_effort_and_no_sampling_params() {
        let p = AnthropicProvider::new("k".into(), AiSettings::default());
        let b = p.build_body(&request(ModelTier::Light));
        assert_eq!(b["model"], "claude-haiku-4-5");
        assert_eq!(b["max_tokens"], 150);
        assert!(b["output_config"].get("effort").is_none());
        assert_eq!(b["output_config"]["format"]["type"], "json_schema");
        for k in ["temperature", "top_p", "top_k", "thinking", "tool_choice"] {
            assert!(b.get(k).is_none(), "{k} must not be sent");
        }
        // the fixed prompt is cacheable and comes first; the question is last
        assert_eq!(b["system"][0]["cache_control"]["type"], "ephemeral");
        let content = b["messages"][0]["content"].as_array().unwrap();
        assert_eq!(content.first().unwrap()["text"], "Contexto A");
        assert_eq!(content.last().unwrap()["text"], "Pregunta");
        // fallbacks only exist on some models; haiku is not one
        assert!(b.get("fallbacks").is_none());
    }

    #[test]
    fn strong_request_has_effort_headroom_and_fallbacks() {
        let p = AnthropicProvider::new("k".into(), AiSettings::default());
        let b = p.build_body(&request(ModelTier::Strong));
        assert_eq!(b["model"], "claude-sonnet-5-5");
        assert_eq!(b["output_config"]["effort"], "medium");
        assert_eq!(b["max_tokens"], 150 + 4000);
        assert_eq!(b["fallbacks"], "default");
    }

    #[test]
    fn model_names_come_from_settings() {
        let s = AiSettings { model_light: "modelo-x".into(), use_fallbacks: false, ..Default::default() };
        let b = AnthropicProvider::new("k".into(), s).build_body(&request(ModelTier::Light));
        assert_eq!(b["model"], "modelo-x");
    }

    #[test]
    fn parses_text_json_usage_and_skips_thinking_blocks() {
        let mut body = ok_body("{\"enough\":true,\"question\":\"\"}");
        body["content"] = json!([
            {"type": "thinking", "thinking": ""},
            {"type": "text", "text": "{\"enough\":true,\"question\":\"\"}"}
        ]);
        let r = parse_response(&body).unwrap();
        assert_eq!(r.value["enough"], true);
        assert_eq!(
            r.usage,
            Usage { input_tokens: 120, output_tokens: 30, cache_read_tokens: 100, cache_write_tokens: 0, thought_tokens: 0 }
        );
    }

    #[test]
    fn refusal_truncation_and_bad_json_are_errors() {
        let mut b = ok_body("{}");
        b["stop_reason"] = json!("refusal");
        assert!(matches!(parse_response(&b), Err(AiError::Refused)));
        b["stop_reason"] = json!("max_tokens");
        assert!(matches!(parse_response(&b), Err(AiError::Truncated)));
        assert!(matches!(parse_response(&ok_body("no es json")), Err(AiError::BadOutput(_))));
    }

    #[tokio::test]
    async fn sends_headers_and_returns_the_parsed_answer() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .and(header("x-api-key", "clave-de-prueba"))
            .and(header("anthropic-version", "2023-06-01"))
            .respond_with(ResponseTemplate::new(200).set_body_json(ok_body("{\"enough\":false,\"question\":\"¿Cuántas?\"}")))
            .expect(1)
            .mount(&server)
            .await;
        let p = AnthropicProvider::with_base_url("clave-de-prueba".into(), AiSettings::default(), server.uri());
        let r = p.complete(&request(ModelTier::Light)).await.unwrap();
        assert_eq!(r.value["question"], "¿Cuántas?");
    }

    #[tokio::test]
    async fn http_errors_are_classified() {
        for (status, expect) in [(401u16, "auth"), (429, "rate"), (500, "http")] {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(ResponseTemplate::new(status).set_body_json(json!({"error": {"message": "x"}})))
                .mount(&server)
                .await;
            let p = AnthropicProvider::with_base_url("k".into(), AiSettings::default(), server.uri());
            let e = p.complete(&request(ModelTier::Light)).await.unwrap_err();
            let got = match e {
                AiError::Auth => "auth",
                AiError::RateLimited => "rate",
                AiError::Http { .. } => "http",
                _ => "other",
            };
            assert_eq!(got, expect);
        }
    }

    #[tokio::test]
    async fn unreachable_server_means_offline() {
        let p = AnthropicProvider::with_base_url("k".into(), AiSettings::default(), "http://127.0.0.1:1".into());
        assert!(matches!(p.complete(&request(ModelTier::Light)).await, Err(AiError::Offline)));
    }
}
