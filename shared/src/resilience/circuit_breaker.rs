//! Circuit breaker pattern implementation

use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use tracing::{info, warn, error};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed (allowing requests)
    Closed,
    /// Circuit is open (blocking requests)
    Open,
    /// Circuit is half-open (allowing a test request)
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold (percentage)
    pub failure_threshold: f64,
    /// Minimum number of requests before calculating failure rate
    pub minimum_requests: u32,
    /// Reset timeout in milliseconds
    pub reset_timeout_ms: u64,
    /// Window size for tracking failures
    pub window_size: usize,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 50.0,
            minimum_requests: 5,
            reset_timeout_ms: 5000,
            window_size: 10,
        }
    }
}

/// Circuit breaker
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    /// Circuit state
    state: Arc<Mutex<CircuitState>>,
    /// Last state change timestamp
    last_state_change: Arc<Mutex<Instant>>,
    /// Request results (true = success, false = failure)
    results: Arc<Mutex<VecDeque<bool>>>,
    /// Configuration
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            last_state_change: Arc::new(Mutex::new(Instant::now())),
            results: Arc::new(Mutex::new(VecDeque::with_capacity(config.window_size))),
            config,
        }
    }
    
    /// Get the current circuit state
    pub fn state(&self) -> CircuitState {
        *self.state.lock().unwrap()
    }
    
    /// Check if the circuit is allowing requests
    pub fn allow_request(&self) -> bool {
        let mut state = self.state.lock().unwrap();
        let last_state_change = *self.last_state_change.lock().unwrap();
        
        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if reset timeout has elapsed
                if last_state_change.elapsed() > Duration::from_millis(self.config.reset_timeout_ms) {
                    info!("Circuit breaker transitioning from Open to Half-Open");
                    *state = CircuitState::HalfOpen;
                    *self.last_state_change.lock().unwrap() = Instant::now();
                    true
                } else {
                    false
                }
            },
            CircuitState::HalfOpen => true,
        }
    }
    
    /// Record a success
    pub fn record_success(&self) {
        let mut state = self.state.lock().unwrap();
        
        match *state {
            CircuitState::Closed => {
                // Add success to results
                let mut results = self.results.lock().unwrap();
                if results.len() >= self.config.window_size {
                    results.pop_front();
                }
                results.push_back(true);
            },
            CircuitState::HalfOpen => {
                // Transition to closed on success
                info!("Circuit breaker transitioning from Half-Open to Closed");
                *state = CircuitState::Closed;
                *self.last_state_change.lock().unwrap() = Instant::now();
                
                // Reset results
                let mut results = self.results.lock().unwrap();
                results.clear();
                results.push_back(true);
            },
            CircuitState::Open => {
                // Should not happen
                warn!("Unexpected success recorded while circuit is Open");
            },
        }
    }
    
    /// Record a failure
    pub fn record_failure(&self) {
        let mut state = self.state.lock().unwrap();
        
        match *state {
            CircuitState::Closed => {
                // Add failure to results
                let mut results = self.results.lock().unwrap();
                if results.len() >= self.config.window_size {
                    results.pop_front();
                }
                results.push_back(false);
                
                // Check if failure threshold is exceeded
                let results_vec: Vec<bool> = results.iter().copied().collect();
                if self.should_trip(&results_vec) {
                    info!("Circuit breaker transitioning from Closed to Open");
                    *state = CircuitState::Open;
                    *self.last_state_change.lock().unwrap() = Instant::now();
                }
            },
            CircuitState::HalfOpen => {
                // Transition back to open on failure
                info!("Circuit breaker transitioning from Half-Open to Open");
                *state = CircuitState::Open;
                *self.last_state_change.lock().unwrap() = Instant::now();
            },
            CircuitState::Open => {
                // Should not happen
                warn!("Unexpected failure recorded while circuit is Open");
            },
        }
    }
    
    /// Check if the circuit should trip
    fn should_trip(&self, results: &[bool]) -> bool {
        if results.len() < self.config.minimum_requests as usize {
            return false;
        }
        
        let failure_count = results.iter().filter(|&&r| !r).count();
        let failure_rate = failure_count as f64 / results.len() as f64 * 100.0;
        
        failure_rate >= self.config.failure_threshold
    }
}

// Global circuit breakers
lazy_static::lazy_static! {
    static ref CIRCUIT_BREAKERS: Mutex<HashMap<String, CircuitBreaker>> = Mutex::new(HashMap::new());
}

/// Get or create a circuit breaker
pub fn get_circuit_breaker(name: &str, config: CircuitBreakerConfig) -> CircuitBreaker {
    let mut breakers = CIRCUIT_BREAKERS.lock().unwrap();
    
    if let Some(breaker) = breakers.get(name) {
        breaker.clone()
    } else {
        let breaker = CircuitBreaker::new(config);
        breakers.insert(name.to_string(), breaker.clone());
        breaker
    }
}

/// Execute a function with circuit breaker
pub async fn with_circuit_breaker<F, Fut, T, E>(
    operation_name: &str,
    config: CircuitBreakerConfig,
    f: F,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let breaker = get_circuit_breaker(operation_name, config);
    
    if !breaker.allow_request() {
        error!("Circuit breaker '{}' is open, rejecting request", operation_name);
        return Err(format!("Circuit breaker is open for operation '{}'", operation_name).into());
    }
    
    match f().await {
        Ok(result) => {
            breaker.record_success();
            Ok(result)
        },
        Err(e) => {
            breaker.record_failure();
            error!("Operation '{}' failed: {}", operation_name, e);
            Err(e)
        }
    }
} 