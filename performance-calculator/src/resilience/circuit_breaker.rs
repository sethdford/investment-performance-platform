use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use async_trait::async_trait;
use tracing::{info, warn, error};
use thiserror::Error;
use std::collections::HashMap;
use std::fmt::Debug;
use std::error::Error;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use tokio::time::sleep;
use std::future::Future;
use std::pin::Pin;

/// Circuit breaker error
#[derive(Error, Debug)]
pub enum CircuitBreakerError<E> {
    #[error("Circuit is open")]
    Open,
    
    #[error("Underlying service error: {0}")]
    ServiceError(E),

    #[error("Internal circuit breaker error: {0}")]
    Internal(String),
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitBreakerState {
    Closed = 0,
    Open = 1,
    HalfOpen = 2,
}

impl From<u32> for CircuitBreakerState {
    fn from(value: u32) -> Self {
        match value {
            0 => CircuitBreakerState::Closed,
            1 => CircuitBreakerState::Open,
            2 => CircuitBreakerState::HalfOpen,
            _ => CircuitBreakerState::Open, // Default to Open for safety
        }
    }
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open the circuit
    pub failure_threshold: u32,
    
    /// Reset timeout in seconds
    pub reset_timeout_seconds: u64,
    
    /// Half-open request threshold
    pub half_open_request_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            reset_timeout_seconds: 60,
            half_open_request_threshold: 3,
        }
    }
}

/// Circuit breaker metrics
#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    /// Current state
    pub state: CircuitBreakerState,
    
    /// Failure count
    pub failure_count: u32,
    
    /// Success count in half-open state
    pub half_open_success_count: u32,
    
    /// Last state change timestamp
    pub last_state_change: Instant,
    
    /// Total number of requests
    pub total_requests: u64,
    
    /// Total number of successful requests
    pub successful_requests: u64,
    
    /// Total number of failed requests
    pub failed_requests: u64,
}

impl Default for CircuitBreakerMetrics {
    fn default() -> Self {
        Self {
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            half_open_success_count: 0,
            last_state_change: Instant::now(),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
        }
    }
}

/// Circuit breaker trait
pub trait CircuitBreaker<E>: Send + Sync {
    type Future: Future<Output = Result<(), CircuitBreakerError<E>>> + Send;
    
    fn check(&self) -> Self::Future;
    
    fn on_success(&self);
    fn on_error(&self, error: &E);
    fn get_state(&self) -> CircuitBreakerState;
}

/// Standard circuit breaker implementation
pub struct StandardCircuitBreaker {
    name: String,
    state: Arc<AtomicU32>,
    failure_threshold: u32,
    reset_timeout: Duration,
    last_failure_time: Arc<AtomicU32>,
    failure_count: Arc<AtomicU32>,
}

impl StandardCircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        Self {
            name,
            state: Arc::new(AtomicU32::new(CircuitBreakerState::Closed as u32)),
            failure_threshold: config.failure_threshold,
            reset_timeout: Duration::from_secs(config.reset_timeout_seconds),
            last_failure_time: Arc::new(AtomicU32::new(0)),
            failure_count: Arc::new(AtomicU32::new(0)),
        }
    }
    
    /// Create a new circuit breaker with default configuration
    pub fn default() -> Self {
        Self::new("default".to_string(), CircuitBreakerConfig::default())
    }
    
    /// Check if the circuit is closed
    fn is_closed(&self) -> Result<bool, String> {
        let state = CircuitBreakerState::from(self.state.load(Ordering::SeqCst));
        Ok(state == CircuitBreakerState::Closed)
    }
    
    /// Check if the circuit is open
    fn is_open(&self) -> Result<bool, String> {
        let state = CircuitBreakerState::from(self.state.load(Ordering::SeqCst));
        
        if state == CircuitBreakerState::Open {
            // Check if reset timeout has elapsed
            let last_failure_timestamp = self.last_failure_time.load(Ordering::SeqCst);
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
            let elapsed = now.saturating_sub(last_failure_timestamp);
            let config_secs = self.reset_timeout.as_secs() as u32;
            
            if elapsed >= config_secs {
                // Transition to half-open state
                self.state.store(CircuitBreakerState::HalfOpen as u32, Ordering::SeqCst);
                return Ok(false);
            }
            
            return Ok(true);
        }
        
        Ok(false)
    }
    
    /// Check if the circuit is half-open
    fn is_half_open(&self) -> Result<bool, String> {
        let state = CircuitBreakerState::from(self.state.load(Ordering::SeqCst));
        Ok(state == CircuitBreakerState::HalfOpen)
    }
    
    /// Transition to open state
    fn transition_to_open(&self) -> Result<(), String> {
        let state = self.state.load(Ordering::SeqCst);
        
        if state != CircuitBreakerState::Open as u32 {
            info!("Circuit breaker transitioning to OPEN state");
            self.state.store(CircuitBreakerState::Open as u32, Ordering::SeqCst);
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
            self.last_failure_time.store(now, Ordering::SeqCst);
        }
        Ok(())
    }
    
    /// Transition to half-open state
    fn transition_to_half_open(&self) -> Result<(), String> {
        let state = self.state.load(Ordering::SeqCst);
        
        if state != CircuitBreakerState::HalfOpen as u32 {
            info!("Circuit breaker transitioning to HALF-OPEN state");
            self.state.store(CircuitBreakerState::HalfOpen as u32, Ordering::SeqCst);
            self.failure_count.store(0, Ordering::SeqCst);
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
            self.last_failure_time.store(now, Ordering::SeqCst);
        }
        Ok(())
    }
    
    /// Transition to closed state
    fn transition_to_closed(&self) -> Result<(), String> {
        let state = self.state.load(Ordering::SeqCst);
        
        if state != CircuitBreakerState::Closed as u32 {
            info!("Circuit breaker transitioning to CLOSED state");
            self.state.store(CircuitBreakerState::Closed as u32, Ordering::SeqCst);
            self.failure_count.store(0, Ordering::SeqCst);
            self.last_failure_time.store(0, Ordering::SeqCst);
        }
        Ok(())
    }
    
    /// Record a successful request
    fn record_success(&self) -> Result<(), String> {
        self.failure_count.store(0, Ordering::SeqCst);
        self.state.store(CircuitBreakerState::Closed as u32, Ordering::SeqCst);
        Ok(())
    }
    
    /// Record a failed request
    fn record_failure(&self) -> Result<(), String> {
        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        if failures >= self.failure_threshold {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
            self.last_failure_time.store(now, Ordering::SeqCst);
            self.state.store(CircuitBreakerState::Open as u32, Ordering::SeqCst);
        }
        Ok(())
    }
}

impl<E> CircuitBreaker<E> for StandardCircuitBreaker
where
    E: std::error::Error + Send + Sync + 'static,
{
    type Future = Pin<Box<dyn Future<Output = Result<(), CircuitBreakerError<E>>> + Send>>;

    fn check(&self) -> Self::Future {
        let state = CircuitBreakerState::from(self.state.load(Ordering::SeqCst));
        let last_failure_timestamp = self.last_failure_time.load(Ordering::SeqCst);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
        let elapsed = now.saturating_sub(last_failure_timestamp);
        let config_secs = self.reset_timeout.as_secs() as u32;

        Box::pin(async move {
            match state {
                CircuitBreakerState::Closed => Ok(()),
                CircuitBreakerState::Open => {
                    if elapsed > config_secs {
                        Ok(())
                    } else {
                        Err(CircuitBreakerError::Open)
                    }
                }
                CircuitBreakerState::HalfOpen => Ok(()),
            }
        })
    }

    fn on_success(&self) {
        self.failure_count.store(0, Ordering::SeqCst);
        self.state.store(CircuitBreakerState::Closed as u32, Ordering::SeqCst);
    }

    fn on_error(&self, error: &E) {
        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        if failures >= self.failure_threshold {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as u32;
            self.last_failure_time.store(now, Ordering::SeqCst);
            self.state.store(CircuitBreakerState::Open as u32, Ordering::SeqCst);
        }
    }

    fn get_state(&self) -> CircuitBreakerState {
        CircuitBreakerState::from(self.state.load(Ordering::SeqCst))
    }
}

/// Registry for circuit breakers
pub struct CircuitBreakerRegistry {
    circuit_breakers: Arc<Mutex<HashMap<String, Arc<StandardCircuitBreaker>>>>,
}

impl CircuitBreakerRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            circuit_breakers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Get or create a circuit breaker
    pub fn get_or_create(&self, name: &str, config: Option<CircuitBreakerConfig>) -> Result<Arc<StandardCircuitBreaker>, String> {
        let mut breakers = self.circuit_breakers.lock()
            .map_err(|e| format!("Failed to acquire registry lock: {}", e))?;
        
        if let Some(breaker) = breakers.get(name) {
            Ok(Arc::clone(breaker))
        } else {
            let breaker = Arc::new(StandardCircuitBreaker::new(name.to_string(), config.unwrap_or_default()));
            breakers.insert(name.to_string(), Arc::clone(&breaker));
            Ok(breaker)
        }
    }
    
    /// Get all circuit breakers
    pub fn get_all(&self) -> Result<HashMap<String, Arc<StandardCircuitBreaker>>, String> {
        self.circuit_breakers.lock()
            .map(|breakers| breakers.clone())
            .map_err(|e| format!("Failed to acquire registry lock: {}", e))
    }
    
    /// Reset all circuit breakers
    pub fn reset_all(&self) -> Result<(), String> {
        let breakers = self.circuit_breakers.lock()
            .map_err(|e| format!("Failed to acquire registry lock: {}", e))?;
        
        for breaker in breakers.values() {
            breaker.transition_to_closed()
                .map_err(|e| format!("Failed to reset circuit breaker: {:?}", e))?;
        }
        Ok(())
    }
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Global registry instance
lazy_static::lazy_static! {
    static ref CIRCUIT_BREAKER_REGISTRY: CircuitBreakerRegistry = CircuitBreakerRegistry::new();
}

/// Get the global circuit breaker registry
pub fn get_circuit_breaker_registry() -> &'static CircuitBreakerRegistry {
    &CIRCUIT_BREAKER_REGISTRY
}

/// Get a circuit breaker by name
pub fn get_circuit_breaker(name: &str) -> Result<Arc<StandardCircuitBreaker>, String> {
    // For now, just return a new circuit breaker with default config
    Ok(Arc::new(StandardCircuitBreaker::default()))
}

/// Get a circuit breaker by name with custom config
pub fn get_circuit_breaker_with_config(name: &str, config: CircuitBreakerConfig) -> Result<Arc<StandardCircuitBreaker>, String> {
    // For now, just return a new circuit breaker with the provided config
    Ok(Arc::new(StandardCircuitBreaker::new(name.to_string(), config)))
} 