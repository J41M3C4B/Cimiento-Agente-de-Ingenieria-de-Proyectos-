//! A scripted provider for tests: no network, answers in the order given.

use super::{AiError, AiProvider, AiRequest, AiResponse, ModelTier};
use async_trait::async_trait;
use std::collections::VecDeque;
use std::sync::Mutex;

pub struct MockProvider {
    replies: Mutex<VecDeque<Result<AiResponse, AiError>>>,
    seen: Mutex<Vec<AiRequest>>,
    /// Models to try in order; empty = a single model called "mock".
    chain: Vec<String>,
    /// Models of the light tier; empty = the same as `chain`.
    light_chain: Vec<String>,
    models_used: Mutex<Vec<String>>,
}

impl MockProvider {
    pub fn new(replies: Vec<Result<AiResponse, AiError>>) -> Self {
        MockProvider { replies: Mutex::new(replies.into()), seen: Mutex::new(Vec::new()), chain: Vec::new(), light_chain: Vec::new(), models_used: Mutex::new(Vec::new()) }
    }

    /// A provider with backups: `chain[0]` is the main model.
    pub fn with_chain(replies: Vec<Result<AiResponse, AiError>>, chain: &[&str]) -> Self {
        MockProvider { chain: chain.iter().map(|m| m.to_string()).collect(), ..Self::new(replies) }
    }

    /// Gives the light tier its own models, so a task allowed to step down has somewhere to go.
    pub fn with_light(mut self, light: &[&str]) -> Self {
        self.light_chain = light.iter().map(|m| m.to_string()).collect();
        self
    }

    /// Every request received, in order.
    pub fn requests(&self) -> Vec<AiRequest> {
        self.seen.lock().unwrap().clone()
    }

    /// The model each request was sent to, in order.
    pub fn models_used(&self) -> Vec<String> {
        self.models_used.lock().unwrap().clone()
    }

    /// What the scanner still finds in the requests sent: the values of personal data that reached the provider
    /// (ADR-044: none should). The labels and marks of the vault are not data. `allowed` are the institution's own
    /// (its phone, its mail), which may go (ADR-040).
    pub fn personal_data_sent(&self, allowed: &[&str]) -> Vec<String> {
        use crate::scanner::{RegexScanner, ScannerConfig, SensitiveScanner};
        let scanner = RegexScanner::new(ScannerConfig::default());
        let mut out = Vec::new();
        for r in self.seen.lock().unwrap().iter() {
            for text in r.context.iter().chain([&r.user, &r.system]) {
                for f in scanner.scan(text).findings {
                    let found = &text[f.span.clone()];
                    if f.kind != crate::scanner::FindingKind::HealthContext && !allowed.iter().any(|a| found.contains(a) || a.contains(found)) {
                        out.push(format!("{}: {found}", f.kind.key()));
                    }
                }
            }
        }
        out
    }

    fn answer(&self, req: &AiRequest, model: &str) -> Result<AiResponse, AiError> {
        if std::env::var("CIMIENTO_LEAK_PROBE").is_ok() {
            let scanner = crate::scanner::RegexScanner::new(crate::scanner::ScannerConfig::default());
            for text in req.context.iter().chain([&req.user]) {
                let r = crate::scanner::SensitiveScanner::scan(&scanner, text);
                for f in r.findings.iter().filter(|f| f.kind != crate::scanner::FindingKind::HealthContext) {
                    eprintln!("LEAK_PROBE {} {}: {}", req.task.as_str(), f.kind.key(), &text[f.span.clone()]);
                }
            }
        }
        self.seen.lock().unwrap().push(req.clone());
        self.models_used.lock().unwrap().push(model.to_string());
        self.replies
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| Err(AiError::Internal("the mock has no more replies".into())))
    }
}

#[async_trait]
impl AiProvider for MockProvider {
    fn name(&self) -> &str {
        "mock"
    }

    fn model_name(&self, _tier: ModelTier) -> String {
        self.chain.first().cloned().unwrap_or_else(|| "mock".into())
    }

    fn model_chain(&self, tier: ModelTier) -> Vec<String> {
        if tier == ModelTier::Light && !self.light_chain.is_empty() {
            return self.light_chain.clone();
        }
        if self.chain.is_empty() { vec![self.model_name(tier)] } else { self.chain.clone() }
    }

    async fn complete(&self, req: &AiRequest) -> Result<AiResponse, AiError> {
        let model = self.model_name(req.tier);
        self.answer(req, &model)
    }

    async fn complete_with_model(&self, req: &AiRequest, model: &str) -> Result<AiResponse, AiError> {
        self.answer(req, model)
    }
}
