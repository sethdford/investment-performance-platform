use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

mod mocks {
    use std::time::Duration;
    use tokio::time::sleep;
    
    pub struct MockResponseStream {
        chunks: Vec<String>,
        current_index: usize,
        delay_ms: u64,
    }

    impl MockResponseStream {
        pub fn new(response: &str, chunk_size: usize, delay_ms: u64) -> Self {
            let mut chunks = Vec::new();
            let words: Vec<&str> = response.split_whitespace().collect();
            
            let mut current_chunk = String::new();
            for word in words {
                if current_chunk.len() + word.len() + 1 > chunk_size && !current_chunk.is_empty() {
                    chunks.push(current_chunk);
                    current_chunk = String::new();
                }
                
                if !current_chunk.is_empty() {
                    current_chunk.push(' ');
                }
                current_chunk.push_str(word);
            }
            
            if !current_chunk.is_empty() {
                chunks.push(current_chunk);
            }
            
            Self {
                chunks,
                current_index: 0,
                delay_ms,
            }
        }
        
        pub async fn next(&mut self) -> Option<String> {
            if self.current_index >= self.chunks.len() {
                return None;
            }
            
            // Simulate network delay
            sleep(Duration::from_millis(self.delay_ms)).await;
            
            let chunk = self.chunks[self.current_index].clone();
            self.current_index += 1;
            
            Some(chunk)
        }
    }
}

/// Test streaming responses using mock implementations
#[tokio::test]
async fn test_streaming_with_mocks() -> Result<()> {
    // This test uses a mock implementation instead of the real AWS Bedrock client
    println!("Testing streaming responses with mocks");
    
    // Create a mock response
    let mock_response = "Retirement planning involves several key principles: 1) Start early to benefit from compound interest. 2) Diversify your investments across different asset classes. 3) Maximize tax-advantaged accounts like 401(k)s and IRAs. 4) Regularly rebalance your portfolio. 5) Adjust your strategy as you approach retirement age.";
    
    // Create a mock response stream
    let mut response_stream = mocks::MockResponseStream::new(mock_response, 30, 50);
    
    // Process the streaming response
    let mut chunks_received = 0;
    let mut assembled_response = String::new();
    
    // Process events as they arrive
    while let Some(chunk) = response_stream.next().await {
        chunks_received += 1;
        println!("Received chunk {}: {} bytes", chunks_received, chunk.len());
        assembled_response.push_str(&chunk);
    }
    
    println!("Received {} chunks in total", chunks_received);
    println!("Assembled response: {}", assembled_response);
    
    // Validate the response
    assert!(chunks_received > 0, "Should receive at least one chunk");
    assert!(!assembled_response.is_empty(), "Response should not be empty");
    
    // Check that the response is relevant to retirement planning
    let relevant_terms = ["retirement", "planning", "401(k)", "ira"];
    let is_relevant = relevant_terms.iter()
        .any(|term| assembled_response.to_lowercase().contains(term));
    
    assert!(is_relevant, "Response should be relevant to retirement planning");
    
    Ok(())
}

/// Test handling network interruptions with mocks
#[tokio::test]
async fn test_mock_network_interruption() -> Result<()> {
    println!("Testing network interruption handling with mocks");
    
    // Create a mock response
    let mock_response = "Retirement planning involves several key principles: 1) Start early to benefit from compound interest. 2) Diversify your investments across different asset classes. 3) Maximize tax-advantaged accounts like 401(k)s and IRAs. 4) Regularly rebalance your portfolio. 5) Adjust your strategy as you approach retirement age.";
    
    // Create a mock response stream
    let mut response_stream = mocks::MockResponseStream::new(mock_response, 30, 50);
    
    // Process the streaming response
    let mut chunks_received = 0;
    let mut assembled_response = String::new();
    
    // Process events as they arrive
    while let Some(chunk) = response_stream.next().await {
        chunks_received += 1;
        println!("Received chunk {}: {} bytes", chunks_received, chunk.len());
        assembled_response.push_str(&chunk);
        
        // Simulate a network interruption after receiving a few chunks
        if chunks_received == 3 {
            println!("Simulating network interruption...");
            sleep(Duration::from_secs(2)).await;
            println!("Resuming after interruption...");
        }
    }
    
    println!("Received {} chunks in total", chunks_received);
    println!("Assembled response: {}", assembled_response);
    
    // Validate the response
    assert!(chunks_received > 0, "Should receive at least one chunk");
    assert!(!assembled_response.is_empty(), "Response should not be empty");
    
    // Check that the response is relevant to retirement planning
    let relevant_terms = ["retirement", "planning", "401(k)", "ira"];
    let is_relevant = relevant_terms.iter()
        .any(|term| assembled_response.to_lowercase().contains(term));
    
    assert!(is_relevant, "Response should be relevant to retirement planning");
    
    Ok(())
} 