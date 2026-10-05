//! Google Gemini over plain HTTPS, `generateContent` (ADR-007). The request asks for a JSON
//! answer that follows our schema; code still validates it before trusting it.
//!
//! Why `generateContent` and not the newer Interactions API: Google documents it as
//! "legacy but fully supported", it is stateless (nothing is kept on Google's side), and
//! the Interactions API stores every call for a day unless told otherwise.

use super::settings::{AiSettings, ProviderKind};
use super::{AiError, AiProvider, AiRequest, AiResponse, ModelCheck, ModelTier, Usage};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com";

pub struct GeminiProvider {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    settings: AiSettings,
    /// Overrides the task's own time limit (tests use a few milliseconds).
    timeout: Option<Duration>,
    /// Overrides the sampling of the task (`AiTask::sampling`): `Some(None)` sends none, `Some(Some(..))`
    /// sends that, `None` follows the task. Only the tests of the rail set it.
    sampling: Option<Option<(f32, i64)>>,
    /// The schema goes in the instructions instead of `responseJsonSchema`: for a schema the service
    /// refuses as too complex (the whole canonical schema, ADR-015). The answer is still JSON and the
    /// code still validates it.
    schema_in_prompt: bool,
}

impl GeminiProvider {
    pub fn new(api_key: String, settings: AiSettings) -> Self {
        Self::with_base_url(api_key, settings, DEFAULT_BASE_URL.to_string())
    }

    pub fn with_base_url(api_key: String, settings: AiSettings, base_url: String) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .connect_timeout(Duration::from_secs(15))
            .build()
            .expect("http client");
        GeminiProvider { http, base_url, api_key, settings, timeout: None, sampling: None, schema_in_prompt: false }
    }

    /// Asks for this temperature and seed in every request of this provider, whatever the task says.
    #[allow(dead_code)]
    pub(crate) fn with_sampling(mut self, temperature: f32, seed: i64) -> Self {
        self.sampling = Some(Some((temperature, seed)));
        self
    }

    /// Describes the schema in the instructions instead of asking the service to enforce it.
    #[allow(dead_code)]
    pub(crate) fn with_schema_in_prompt(mut self) -> Self {
        self.schema_in_prompt = true;
        self
    }

    /// Sends no sampling parameter in any request, even for a task that asks for one.
    #[allow(dead_code)]
    pub(crate) fn without_sampling(mut self) -> Self {
        self.sampling = Some(None);
        self
    }

    #[cfg(test)]
    pub(crate) fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    fn model(&self, tier: ModelTier) -> String {
        self.settings.model_in_use(ProviderKind::Gemini, tier)
    }

    /// The JSON body for a request to the model a call of this tier goes to first.
    pub(crate) fn build_body(&self, req: &AiRequest) -> Value {
        self.body_for(req, &self.model(req.tier))
    }

    /// The JSON body for a request to `model`. Whether it carries a temperature depends on the
    /// model (`AiSettings::sampling_models`), not only on the task.
    pub(crate) fn body_for(&self, req: &AiRequest, model: &str) -> Value {
        let mut parts: Vec<Value> = req.context.iter().map(|c| json!({ "text": c })).collect();
        parts.push(json!({ "text": req.user }));

        // The model's own thinking counts against the output limit, so the limit gets a margin.
        let headroom = match req.tier {
            ModelTier::Strong => self.settings.thinking_headroom,
            ModelTier::Light => self.settings.light_headroom,
        };
        let mut generation = json!({
            "maxOutputTokens": req.max_output_tokens + headroom,
            "responseMimeType": "application/json",
        });
        let mut system = req.system.clone();
        if self.schema_in_prompt {
            system = format!(
                "{system}

# Esquema de la respuesta
Responde con un único objeto JSON que cumpla este JSON Schema, con todos los campos requeridos y solo los valores permitidos:
{}",
                req.output_schema
            );
        } else {
            generation["responseJsonSchema"] = without_additional_properties(&req.output_schema);
        }
        if req.tier == ModelTier::Strong && !self.settings.effort_strong.is_empty() {
            generation["thinkingConfig"] = json!({ "thinkingLevel": self.settings.effort_strong });
        }

        // Sampling only for the tasks that classify (`AiTask::sampling`); the rest keep the
        // provider's defaults (ADR-007). No safety overrides: the defaults are the tested ones.
        let measured_safe = self.settings.sampling_models.iter().any(|m| model.starts_with(m.as_str()));
        let wanted = self.sampling.unwrap_or(if measured_safe { req.task.sampling() } else { None });
        if let Some((temperature, seed)) = wanted {
            generation["temperature"] = json!(temperature);
            generation["seed"] = json!(seed);
        }

        json!({
            "systemInstruction": { "parts": [{ "text": system }] },
            "contents": [{ "role": "user", "parts": parts }],
            "generationConfig": generation,
        })
    }
}

/// Gemini accepts only part of JSON Schema. `additionalProperties: false` is the one keyword
/// we use that is not guaranteed, and dropping it loses nothing: `pipeline::validate` checks
/// the answer against the full schema afterwards.
fn without_additional_properties(schema: &Value) -> Value {
    match schema {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(k, _)| k.as_str() != "additionalProperties")
                .map(|(k, v)| (k.clone(), without_additional_properties(v)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(without_additional_properties).collect()),
        other => other.clone(),
    }
}

/// Turns a successful response body into an `AiResponse`. `model` is the model that was
/// asked: it is what the price table knows (the response's own version string may carry a suffix).
pub(crate) fn parse_response(body: &Value, model: &str) -> Result<AiResponse, AiError> {
    if body["promptFeedback"]["blockReason"].is_string() {
        return Err(AiError::Refused);
    }
    let candidate = body["candidates"].get(0).ok_or_else(|| AiError::BadOutput("no candidate in the response".into()))?;
    match candidate["finishReason"].as_str() {
        Some("MAX_TOKENS") => return Err(AiError::Truncated),
        Some("SAFETY" | "PROHIBITED_CONTENT" | "BLOCKLIST" | "SPII" | "RECITATION" | "IMAGE_SAFETY") => {
            return Err(AiError::Refused)
        }
        _ => {}
    }
    // Parts marked `thought` are the model's summary of its thinking, not the answer.
    let text: String = candidate["content"]["parts"]
        .as_array()
        .map(|parts| {
            parts
                .iter()
                .filter(|p| p["thought"] != json!(true))
                .filter_map(|p| p["text"].as_str())
                .collect()
        })
        .unwrap_or_default();
    if text.trim().is_empty() {
        return Err(AiError::BadOutput("no text in the response".into()));
    }
    let value: Value =
        serde_json::from_str(&text).map_err(|e| AiError::BadOutput(format!("not valid JSON: {e}")))?;

    let u = &body["usageMetadata"];
    let n = |k: &str| u[k].as_u64().unwrap_or(0);
    let (prompt, cached, answer, thoughts) =
        (n("promptTokenCount"), n("cachedContentTokenCount"), n("candidatesTokenCount"), n("thoughtsTokenCount"));
    Ok(AiResponse {
        value,
        model: model.to_string(),
        usage: Usage {
            // Gemini counts cached tokens inside the prompt; our cost formula prices them apart.
            input_tokens: prompt.saturating_sub(cached),
            // Thinking is billed as output.
            output_tokens: answer + thoughts,
            cache_read_tokens: cached,
            cache_write_tokens: 0,
            thought_tokens: thoughts,
        },
    })
}

/// Sorts a failed HTTP answer into the errors the rest of the app understands.
pub(crate) fn classify_error(status: u16, body: &Value) -> AiError {
    let message: String = body["error"]["message"].as_str().unwrap_or("").chars().take(300).collect();
    let everything = body["error"].to_string().to_lowercase();
    match status {
        401 | 403 => AiError::Auth,
        // Google answers 400 (not 401) for a key it does not recognise.
        400 if everything.contains("api_key_invalid") || everything.contains("api key not valid") => AiError::Auth,
        // The same 429 means "wait a minute" or "come back tomorrow"; only the second is final.
        429 if everything.contains("perday") || everything.contains("per day") => AiError::QuotaReached,
        429 => AiError::RateLimited,
        s => AiError::Http { status: s, message },
    }
}

fn transport_error(e: reqwest::Error) -> AiError {
    // a connection that cannot be made is "offline" (this includes the connect timeout); a
    // request that was sent and then took too long is a timeout
    if e.is_connect() {
        AiError::Offline
    } else if e.is_timeout() {
        AiError::Timeout
    } else if e.is_request() {
        AiError::Offline
    } else {
        AiError::Internal(e.to_string())
    }
}

#[async_trait]
impl AiProvider for GeminiProvider {
    fn name(&self) -> &str {
        "gemini"
    }

    fn model_name(&self, tier: ModelTier) -> String {
        self.model(tier)
    }

    fn model_chain(&self, tier: ModelTier) -> Vec<String> {
        self.settings.chain_of(ProviderKind::Gemini, tier)
    }

    async fn complete(&self, req: &AiRequest) -> Result<AiResponse, AiError> {
        self.complete_with_model(req, &self.model(req.tier)).await
    }

    async fn complete_with_model(&self, req: &AiRequest, model: &str) -> Result<AiResponse, AiError> {
        let model = model.to_string();
        let resp = self
            .http
            .post(format!("{}/v1beta/models/{}:generateContent", self.base_url, model))
            .header("x-goog-api-key", &self.api_key)
            .timeout(self.timeout.unwrap_or_else(|| Duration::from_secs(req.task.timeout_secs())))
            .json(&self.body_for(req, &model))
            .send()
            .await
            .map_err(transport_error)?;
        let status = resp.status().as_u16();
        let body: Value = resp.json().await.unwrap_or(Value::Null);
        match status {
            200 => parse_response(&body, &model),
            s => Err(classify_error(s, &body)),
        }
    }

    /// Asks Google about each configured model. It checks the key and the model names
    /// without generating anything, so it does not spend the daily allowance of generation calls.
    async fn check(&self) -> Result<Vec<ModelCheck>, AiError> {
        let mut seen: Vec<String> = Vec::new();
        let mut report = Vec::new();
        for tier in [ModelTier::Light, ModelTier::Strong] {
            let model = self.model(tier).to_string();
            if seen.contains(&model) {
                continue;
            }
            seen.push(model.clone());
            let resp = self
                .http
                .get(format!("{}/v1beta/models/{}", self.base_url, model))
                .header("x-goog-api-key", &self.api_key)
                .send()
                .await
                .map_err(transport_error)?;
            let status = resp.status().as_u16();
            let body: Value = resp.json().await.unwrap_or(Value::Null);
            match status {
                200 => {
                    let methods = body["supportedGenerationMethods"].as_array();
                    // A model that does not list its methods is assumed able; one that lists them must include this one.
                    let can_generate = methods.map_or(true, |m| m.iter().any(|x| x == "generateContent"));
                    report.push(ModelCheck { model, exists: true, can_generate });
                }
                404 => report.push(ModelCheck { model, exists: false, can_generate: false }),
                s => return Err(classify_error(s, &body)),
            }
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::AiTask;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn schema() -> Value {
        json!({"type":"object","properties":{"enough":{"type":"boolean"},
               "detail":{"type":"object","properties":{"n":{"anyOf":[{"type":"integer"},{"type":"null"}]}},
                         "required":["n"],"additionalProperties":false},
               "question":{"type":"string"}},
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
            "candidates": [{"content": {"role": "model", "parts": [{"text": text}]}, "finishReason": "STOP"}],
            "usageMetadata": {"promptTokenCount": 1200, "cachedContentTokenCount": 1000, "candidatesTokenCount": 30,
                              "thoughtsTokenCount": 70, "totalTokenCount": 1300},
            "modelVersion": "gemini-3.5-flash-lite-001"
        })
    }

    #[test]
    fn light_request_has_json_output_a_margin_and_no_thinking_setting() {
        let p = GeminiProvider::new("k".into(), AiSettings::default());
        let b = p.build_body(&request(ModelTier::Light));
        assert_eq!(b["systemInstruction"]["parts"][0]["text"], "SISTEMA FIJO");
        let parts = b["contents"][0]["parts"].as_array().unwrap();
        assert_eq!(parts.first().unwrap()["text"], "Contexto A");
        assert_eq!(parts.last().unwrap()["text"], "Pregunta");
        assert_eq!(b["contents"][0]["role"], "user");
        let g = &b["generationConfig"];
        assert_eq!(g["responseMimeType"], "application/json");
        assert_eq!(g["maxOutputTokens"], 150 + 200);
        assert!(g.get("thinkingConfig").is_none());
        for k in ["temperature", "topP", "topK", "safetySettings", "tools"] {
            assert!(b.get(k).is_none() && g.get(k).is_none(), "{k} must not be sent");
        }
    }

    #[test]
    fn sampling_is_sent_only_for_the_tasks_that_classify_and_only_to_models_measured_safe() {
        let plain = GeminiProvider::new("k".into(), AiSettings::default());
        // a task that writes for a person keeps the provider's defaults
        let g = &plain.body_for(&request(ModelTier::Light), "gemini-3.5-flash-lite")["generationConfig"];
        assert!(g.get("temperature").is_none() && g.get("seed").is_none(), "{g}");
        // the reading of a call is pinned on the model measured safe...
        let review = AiRequest { task: AiTask::CallCanonical, ..request(ModelTier::Strong) };
        let g = &plain.body_for(&review, "gemini-3.5-flash-lite")["generationConfig"];
        assert_eq!((g["temperature"].as_f64(), g["seed"].as_i64()), (Some(0.0), Some(7)));
        // ...and not on Preview, which looped until it filled its output with temperature 0
        let g = &plain.body_for(&review, "gemini-3-flash-preview")["generationConfig"];
        assert!(g.get("temperature").is_none() && g.get("seed").is_none(), "{g}");
        // the default model of the strong tier is Preview
        assert!(plain.build_body(&review)["generationConfig"].get("temperature").is_none());
        // the measurement tests can override it either way
        let none = GeminiProvider::new("k".into(), AiSettings::default()).without_sampling();
        assert!(none.body_for(&review, "gemini-3.5-flash-lite")["generationConfig"].get("temperature").is_none());
        let other = GeminiProvider::new("k".into(), AiSettings::default()).with_sampling(0.5, 3);
        let g = &other.build_body(&request(ModelTier::Strong))["generationConfig"];
        assert_eq!((g["temperature"].as_f64(), g["seed"].as_i64()), (Some(0.5), Some(3)));
    }

    #[test]
    fn a_schema_the_service_refuses_can_go_in_the_instructions_instead() {
        let p = GeminiProvider::new("k".into(), AiSettings::default()).with_schema_in_prompt();
        let b = p.build_body(&request(ModelTier::Light));
        let g = &b["generationConfig"];
        assert_eq!(g["responseMimeType"], "application/json");
        assert!(g.get("responseJsonSchema").is_none());
        let system = b["systemInstruction"]["parts"][0]["text"].as_str().unwrap();
        assert!(system.starts_with("SISTEMA FIJO") && system.contains("\"required\":[\"enough\",\"question\"]"), "{system}");
    }

    #[test]
    fn strong_request_sets_the_thinking_level_and_a_bigger_margin() {
        let p = GeminiProvider::new("k".into(), AiSettings::default());
        let b = p.build_body(&request(ModelTier::Strong));
        assert_eq!(b["generationConfig"]["thinkingConfig"]["thinkingLevel"], "medium");
        assert_eq!(b["generationConfig"]["maxOutputTokens"], 150 + 4000);
        // an empty effort means "provider default": nothing is sent
        let s = AiSettings { effort_strong: String::new(), ..Default::default() };
        let b = GeminiProvider::new("k".into(), s).build_body(&request(ModelTier::Strong));
        assert!(b["generationConfig"].get("thinkingConfig").is_none());
    }

    #[test]
    fn the_schema_is_sent_without_additional_properties_but_keeps_everything_else() {
        let p = GeminiProvider::new("k".into(), AiSettings::default());
        let sent = &p.build_body(&request(ModelTier::Light))["generationConfig"]["responseJsonSchema"];
        assert!(!sent.to_string().contains("additionalProperties"));
        assert_eq!(sent["required"], json!(["enough", "question"]));
        assert_eq!(sent["properties"]["detail"]["properties"]["n"]["anyOf"][1]["type"], "null");
        // the original is untouched, so local validation still rejects extras
        assert!(schema().to_string().contains("additionalProperties"));
    }

    #[test]
    fn model_names_come_from_settings() {
        let s = AiSettings { gemini_model_light: "modelo-x".into(), ..Default::default() };
        let p = GeminiProvider::new("k".into(), s);
        assert_eq!(p.model_name(ModelTier::Light), "modelo-x");
        assert_eq!(p.model_name(ModelTier::Strong), "gemini-3-flash-preview");
    }

    #[test]
    fn parses_the_answer_and_maps_usage_with_thinking_and_cache() {
        let r = parse_response(&ok_body("{\"enough\":true,\"question\":\"\"}"), "gemini-3.5-flash-lite").unwrap();
        assert_eq!(r.value["enough"], true);
        // the model asked is reported, not the versioned name Google answers with
        assert_eq!(r.model, "gemini-3.5-flash-lite");
        assert_eq!(
            r.usage,
            Usage { input_tokens: 200, output_tokens: 100, cache_read_tokens: 1000, cache_write_tokens: 0, thought_tokens: 70 }
        );
    }

    #[test]
    fn thought_parts_are_not_part_of_the_answer() {
        let mut b = ok_body("");
        b["candidates"][0]["content"]["parts"] = json!([
            {"text": "Estoy pensando en cómo preguntar...", "thought": true},
            {"text": "{\"enough\":false,"},
            {"text": "\"question\":\"¿Cuántas caídas?\"}"}
        ]);
        let r = parse_response(&b, "m").unwrap();
        assert_eq!(r.value["question"], "¿Cuántas caídas?");
    }

    #[test]
    fn truncation_blocks_and_bad_json_are_errors() {
        let mut b = ok_body("{}");
        b["candidates"][0]["finishReason"] = json!("MAX_TOKENS");
        assert!(matches!(parse_response(&b, "m"), Err(AiError::Truncated)));
        b["candidates"][0]["finishReason"] = json!("SAFETY");
        assert!(matches!(parse_response(&b, "m"), Err(AiError::Refused)));
        let blocked = json!({"promptFeedback": {"blockReason": "PROHIBITED_CONTENT"}});
        assert!(matches!(parse_response(&blocked, "m"), Err(AiError::Refused)));
        assert!(matches!(parse_response(&ok_body("no es json"), "m"), Err(AiError::BadOutput(_))));
        assert!(matches!(parse_response(&json!({"candidates": []}), "m"), Err(AiError::BadOutput(_))));
        // thinking used the whole budget: STOP with no text at all
        let mut empty = ok_body("");
        empty["candidates"][0]["content"] = json!({"role": "model"});
        assert!(matches!(parse_response(&empty, "m"), Err(AiError::BadOutput(_))));
    }

    #[test]
    fn http_failures_are_sorted_into_app_errors() {
        let key_bad = json!({"error": {"code": 400, "message": "API key not valid. Please pass a valid API key.", "status": "INVALID_ARGUMENT"}});
        assert!(matches!(classify_error(400, &key_bad), AiError::Auth));
        assert!(matches!(classify_error(403, &json!({})), AiError::Auth));
        let per_minute = json!({"error": {"code": 429, "status": "RESOURCE_EXHAUSTED",
            "message": "Quota exceeded for metric GenerateRequestsPerMinutePerProjectPerModel-FreeTier"}});
        assert!(matches!(classify_error(429, &per_minute), AiError::RateLimited));
        let per_day = json!({"error": {"code": 429, "status": "RESOURCE_EXHAUSTED",
            "details": [{"violations": [{"quotaId": "GenerateRequestsPerDayPerProjectPerModel-FreeTier"}]}]}});
        assert!(matches!(classify_error(429, &per_day), AiError::QuotaReached));
        assert!(matches!(classify_error(503, &json!({"error": {"message": "overloaded"}})), AiError::Http { status: 503, .. }));
        // a plain 400 about the request itself stays a plain error
        assert!(matches!(classify_error(400, &json!({"error": {"message": "Invalid JSON payload"}})), AiError::Http { status: 400, .. }));
    }

    #[tokio::test]
    async fn sends_the_key_in_the_header_to_the_model_path_and_returns_the_answer() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1beta/models/gemini-3.5-flash-lite:generateContent"))
            .and(header("x-goog-api-key", "clave-de-prueba"))
            .respond_with(ResponseTemplate::new(200).set_body_json(ok_body("{\"enough\":false,\"question\":\"¿Cuántas?\"}")))
            .expect(1)
            .mount(&server)
            .await;
        let p = GeminiProvider::with_base_url("clave-de-prueba".into(), AiSettings::default(), server.uri());
        let r = p.complete(&request(ModelTier::Light)).await.unwrap();
        assert_eq!(r.value["question"], "¿Cuántas?");
        // the key travels in a header, never in the URL
        let url = server.received_requests().await.unwrap()[0].url.to_string();
        assert!(!url.contains("clave-de-prueba"), "{url}");
    }

    #[tokio::test]
    async fn http_errors_are_classified_over_the_wire() {
        for (status, body, expect) in [
            (400u16, json!({"error": {"message": "API key not valid"}}), "auth"),
            (429, json!({"error": {"message": "Quota exceeded ... PerMinute"}}), "rate"),
            (429, json!({"error": {"message": "Quota exceeded ... PerDay"}}), "day"),
            (500, json!({"error": {"message": "x"}}), "http"),
        ] {
            let server = MockServer::start().await;
            Mock::given(method("POST")).respond_with(ResponseTemplate::new(status).set_body_json(body)).mount(&server).await;
            let p = GeminiProvider::with_base_url("k".into(), AiSettings::default(), server.uri());
            let got = match p.complete(&request(ModelTier::Light)).await.unwrap_err() {
                AiError::Auth => "auth",
                AiError::RateLimited => "rate",
                AiError::QuotaReached => "day",
                AiError::Http { .. } => "http",
                _ => "other",
            };
            assert_eq!(got, expect);
        }
    }

    #[tokio::test]
    async fn a_slow_answer_is_a_timeout_not_offline() {
        // found on a live run: a call that hung for 120 s was reported as "sin internet"
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(600)).set_body_json(ok_body("{\"enough\":true,\"question\":\"\"}")))
            .mount(&server)
            .await;
        let p = GeminiProvider::with_base_url("k".into(), AiSettings::default(), server.uri()).with_timeout(Duration::from_millis(100));
        assert!(matches!(p.complete(&request(ModelTier::Light)).await, Err(AiError::Timeout)));
        // an answer inside the limit is fine
        let p = GeminiProvider::with_base_url("k".into(), AiSettings::default(), server.uri()).with_timeout(Duration::from_secs(5));
        assert!(p.complete(&request(ModelTier::Light)).await.is_ok());
    }

    #[tokio::test]
    async fn unreachable_server_means_offline() {
        let p = GeminiProvider::with_base_url("k".into(), AiSettings::default(), "http://127.0.0.1:1".into());
        assert!(matches!(p.complete(&request(ModelTier::Light)).await, Err(AiError::Offline)));
    }

    #[tokio::test]
    async fn check_reports_each_model_without_a_generation_call() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1beta/models/gemini-3.5-flash-lite"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                json!({"name": "models/gemini-3.5-flash-lite", "supportedGenerationMethods": ["generateContent", "countTokens"]}),
            ))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/v1beta/models/gemini-3-flash-preview"))
            .respond_with(ResponseTemplate::new(404).set_body_json(json!({"error": {"message": "not found"}})))
            .mount(&server)
            .await;
        let p = GeminiProvider::with_base_url("k".into(), AiSettings::default(), server.uri());
        let r = p.check().await.unwrap();
        assert_eq!(r[0], ModelCheck { model: "gemini-3.5-flash-lite".into(), exists: true, can_generate: true });
        assert_eq!(r[1], ModelCheck { model: "gemini-3-flash-preview".into(), exists: false, can_generate: false });
        let calls = server.received_requests().await.unwrap();
        assert!(calls.iter().all(|c| c.method.as_str() == "GET"), "no POST: nothing was generated");
    }

    #[tokio::test]
    async fn check_with_a_wrong_key_says_so() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(400).set_body_json(json!({"error": {"message": "API key not valid"}})))
            .mount(&server)
            .await;
        let p = GeminiProvider::with_base_url("mala".into(), AiSettings::default(), server.uri());
        assert!(matches!(p.check().await, Err(AiError::Auth)));
    }
}
