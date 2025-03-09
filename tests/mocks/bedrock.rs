use anyhow::Result;
use async_trait::async_trait;
use futures_util::{Stream, StreamExt};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;

/// Mock implementation of the AWS Bedrock client for testing
#[derive(Clone)]
pub struct MockBedrockClient {
    responses: Arc<Mutex<HashMap<String, String>>>,
    streaming_responses: Arc<Mutex<HashMap<String, Vec<String>>>>,
    delay: Duration,
    chunk_delay: Duration,
    should_fail: bool,
    failure_message: String,
}

impl Default for MockBedrockClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBedrockClient {
    /// Create a new mock Bedrock client
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
            streaming_responses: Arc::new(Mutex::new(HashMap::new())),
            delay: Duration::from_millis(100),
            chunk_delay: Duration::from_millis(50),
            should_fail: false,
            failure_message: "Mock failure".to_string(),
        }
    }

    /// Add a response for a specific prompt
    pub fn with_response(mut self, prompt: &str, response: &str) -> Self {
        self.responses
            .lock()
            .unwrap()
            .insert(prompt.to_string(), response.to_string());
        self
    }

    /// Add a streaming response for a specific prompt
    pub fn with_streaming_response(mut self, prompt: &str, chunks: Vec<&str>) -> Self {
        let chunks = chunks.iter().map(|s| s.to_string()).collect();
        self.streaming_responses
            .lock()
            .unwrap()
            .insert(prompt.to_string(), chunks);
        self
    }

    /// Set the delay before responding
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Set the delay between streaming chunks
    pub fn with_chunk_delay(mut self, delay: Duration) -> Self {
        self.chunk_delay = delay;
        self
    }

    /// Configure the client to fail with a specific message
    pub fn with_failure(mut self, message: &str) -> Self {
        self.should_fail = true;
        self.failure_message = message.to_string();
        self
    }

    /// Reset the client to its default state
    pub fn reset(&mut self) {
        self.responses.lock().unwrap().clear();
        self.streaming_responses.lock().unwrap().clear();
        self.delay = Duration::from_millis(100);
        self.chunk_delay = Duration::from_millis(50);
        self.should_fail = false;
        self.failure_message = "Mock failure".to_string();
    }

    /// Invoke the model with a prompt
    pub async fn invoke_model(&self, prompt: &str) -> Result<String> {
        // Simulate network delay
        sleep(self.delay).await;

        if self.should_fail {
            return Err(anyhow::anyhow!("{}", self.failure_message));
        }

        let responses = self.responses.lock().unwrap();
        if let Some(response) = responses.get(prompt) {
            Ok(response.clone())
        } else {
            // Default mock response if no specific response is configured
            Ok(format!("Mock response for: {}", prompt))
        }
    }

    /// Invoke the model with streaming response
    pub async fn invoke_model_with_streaming(
        &self,
        prompt: &str,
    ) -> Result<impl Stream<Item = Result<String>>> {
        // Simulate network delay
        sleep(self.delay).await;

        if self.should_fail {
            return Err(anyhow::anyhow!("{}", self.failure_message));
        }

        let streaming_responses = self.streaming_responses.lock().unwrap();
        let chunks = if let Some(chunks) = streaming_responses.get(prompt) {
            chunks.clone()
        } else {
            // Default mock streaming response if no specific response is configured
            vec![
                "Mock ".to_string(),
                "streaming ".to_string(),
                "response ".to_string(),
                format!("for: {}", prompt),
            ]
        };

        let chunk_delay = self.chunk_delay;
        let (tx, rx) = mpsc::channel(10);

        // Spawn a task to send chunks with delays
        tokio::spawn(async move {
            for chunk in chunks {
                if tx.send(Ok(chunk)).await.is_err() {
                    break;
                }
                sleep(chunk_delay).await;
            }
        });

        Ok(MockStream { receiver: rx })
    }
}

/// Mock implementation of a Stream for testing streaming responses
pub struct MockStream {
    receiver: mpsc::Receiver<Result<String>>,
}

impl Stream for MockStream {
    type Item = Result<String>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}

/// Trait for abstracting over real and mock Bedrock clients
#[async_trait]
pub trait BedrockClientTrait {
    async fn invoke_model(&self, prompt: &str) -> Result<String>;
    async fn invoke_model_with_streaming(
        &self,
        prompt: &str,
    ) -> Result<Box<dyn Stream<Item = Result<String>> + Unpin + Send>>;
}

#[async_trait]
impl BedrockClientTrait for MockBedrockClient {
    async fn invoke_model(&self, prompt: &str) -> Result<String> {
        self.invoke_model(prompt).await
    }

    async fn invoke_model_with_streaming(
        &self,
        prompt: &str,
    ) -> Result<Box<dyn Stream<Item = Result<String>> + Unpin + Send>> {
        let stream = self.invoke_model_with_streaming(prompt).await?;
        Ok(Box::new(stream))
    }
}

/// Example usage of the mock client
#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;

    #[tokio::test]
    async fn test_mock_invoke_model() {
        let client = MockBedrockClient::new()
            .with_response("test prompt", "test response")
            .with_delay(Duration::from_millis(10));

        let response = client.invoke_model("test prompt").await.unwrap();
        assert_eq!(response, "test response");

        // Test default response for unknown prompt
        let response = client.invoke_model("unknown prompt").await.unwrap();
        assert_eq!(response, "Mock response for: unknown prompt");
    }

    #[tokio::test]
    async fn test_mock_invoke_model_with_streaming() {
        let client = MockBedrockClient::new()
            .with_streaming_response(
                "test prompt",
                vec!["chunk1", "chunk2", "chunk3"],
            )
            .with_chunk_delay(Duration::from_millis(10));

        let stream = client.invoke_model_with_streaming("test prompt").await.unwrap();
        let chunks: Vec<String> = stream
            .map(|result| result.unwrap())
            .collect::<Vec<_>>()
            .await;

        assert_eq!(chunks, vec!["chunk1", "chunk2", "chunk3"]);
    }

    #[tokio::test]
    async fn test_mock_failure() {
        let client = MockBedrockClient::new().with_failure("Test failure message");

        let result = client.invoke_model("test prompt").await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Test failure message"
        );
    }
} 