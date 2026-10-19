//! A provider and an embedder that never send more than `PER_MINUTE` requests in any minute, for the real batteries.
//! The pace of the pipeline counts in `ai_usage`, and a battery builds a new database for every case or round, so it
//! does not see the calls before; this one counts in the process. The text model and the vectors have a window each,
//! because the provider counts them per model.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::time::Instant;

use async_trait::async_trait;

use super::{AiError, AiProvider, AiRequest, AiResponse, ModelCheck, ModelTier};
use crate::documents::canonical::retrieve::Embedder;

/// One under the 15 per minute of the free plan, so a slow clock never makes it 16.
pub(crate) const PER_MINUTE: usize = 14;

struct Window(Mutex<VecDeque<Instant>>);

impl Window {
    const fn new() -> Self {
        Window(Mutex::new(VecDeque::new()))
    }

    /// Waits until a request fits in the last minute, and counts it.
    async fn wait(&self) {
        loop {
            let wait = {
                let mut sent = self.0.lock().unwrap();
                let now = Instant::now();
                while sent.front().is_some_and(|t| now.duration_since(*t) >= Duration::from_secs(60)) {
                    sent.pop_front();
                }
                if sent.len() < PER_MINUTE {
                    sent.push_back(now);
                    return;
                }
                Duration::from_secs(60).saturating_sub(now.duration_since(sent[0])) + Duration::from_millis(250)
            };
            tokio::time::sleep(wait).await;
        }
    }
}

static TEXT: Window = Window::new();
static VECTORS: Window = Window::new();

struct Paced(Box<dyn AiProvider>);

#[async_trait]
impl AiProvider for Paced {
    async fn complete(&self, req: &AiRequest) -> Result<AiResponse, AiError> {
        TEXT.wait().await;
        self.0.complete(req).await
    }
    fn name(&self) -> &str {
        self.0.name()
    }
    fn model_name(&self, tier: ModelTier) -> String {
        self.0.model_name(tier)
    }
    fn model_chain(&self, tier: ModelTier) -> Vec<String> {
        self.0.model_chain(tier)
    }
    async fn complete_with_model(&self, req: &AiRequest, model: &str) -> Result<AiResponse, AiError> {
        TEXT.wait().await;
        self.0.complete_with_model(req, model).await
    }
    async fn check(&self) -> Result<Vec<ModelCheck>, AiError> {
        self.0.check().await
    }
}

struct PacedEmbedder(Arc<dyn Embedder>);

#[async_trait]
impl Embedder for PacedEmbedder {
    async fn embed(&self, texts: &[String], query: bool) -> Result<Vec<Vec<f32>>, String> {
        VECTORS.wait().await;
        self.0.embed(texts, query).await
    }
    fn label(&self) -> String {
        self.0.label()
    }
    fn usage(&self) -> (usize, usize) {
        self.0.usage()
    }
}

/// The provider, at most `PER_MINUTE` calls in any minute.
pub(crate) fn provider(inner: Box<dyn AiProvider>) -> Box<dyn AiProvider> {
    Box::new(Paced(inner))
}

/// The embedder, at most `PER_MINUTE` requests in any minute.
pub(crate) fn embedder(inner: Arc<dyn Embedder>) -> Arc<dyn Embedder> {
    Arc::new(PacedEmbedder(inner))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn never_more_than_fourteen_in_a_minute() {
        let w = Window::new();
        let start = tokio::time::Instant::now();
        for _ in 0..PER_MINUTE {
            w.wait().await;
        }
        assert!(start.elapsed() < Duration::from_secs(1), "the first fourteen go at once");
        w.wait().await;
        assert!(start.elapsed() >= Duration::from_secs(60), "the fifteenth waits for the minute");
    }
}
