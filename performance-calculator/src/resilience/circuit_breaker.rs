use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use async_trait::async_trait;
use tracing::{info, warn, error};
use thiserror::Error;

/// Circuit breaker error
#[derive(Error, Debug)]
pub enum CircuitBreakerError<E> {
    #[error("Circuit is open")]
    Open,
    
    #[error("Underlying service error: {0}")]
    ServiceError(E),
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests are allowed
    Closed,
    
    /// Circuit is open, requests are not allowed
    Open,
    
    /// Circuit is half-open, allowing a limited number of requests to test if the service is healthy
    HalfOpen,
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
    pub state: CircuitState,
    
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
            state: CircuitState::Closed,
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
#[async_trait]
pub trait CircuitBreaker<T, E> {
    /// Execute a function with circuit breaker protection
    async fn execute<F, Fut>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, E>> + Send;
    
    /// Get current circuit breaker metrics
    fn metrics(&self) -> CircuitBreakerMetrics;
    
    /// Reset the circuit breaker
    fn reset(&self);
}

/// Standard circuit breaker implementation
pub struct StandardCircuitBreaker {
    config: CircuitBreakerConfig,
    metrics: Arc<Mutex<CircuitBreakerMetrics>>,
}

impl StandardCircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(Mutex::new(CircuitBreakerMetrics::default())),
        }
    }
    
    /// Create a new circuit breaker with default configuration
    pub fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }
    
    /// Check if the circuit is closed
    fn is_closed(&self) -> bool {
        let metrics = self.metrics.lock().unwrap();
        metrics.state == CircuitState::Closed
    }
    
    /// Check if the circuit is open
    fn is_open(&self) -> bool {
        let metrics = self.metrics.lock().unwrap();
        
        if metrics.state == CircuitState::Open {
            // Check if reset timeout has elapsed
            let elapsed = metrics.last_state_change.elapsed().as_secs();
            
            if elapsed >= self.config.reset_timeout_seconds {
                // Transition to half-open state
                drop(metrics);
                self.transition_to_half_open();
                return false;
            }
            
            return true;
        }
        
        false
    }
    
    /// Check if the circuit is half-open
    fn is_half_open(&self) -> bool {
        let metrics = self.metrics.lock().unwrap();
        metrics.state == CircuitState::HalfOpen
    }
    
    /// Transition to open state
    fn transition_to_open(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        
        if metrics.state != CircuitState::Open {
            info!("Circuit breaker transitioning to OPEN state");
            metrics.state = CircuitState::Open;
            metrics.last_state_change = Instant::now();
        }
    }
    
    /// Transition to half-open state
    fn transition_to_half_open(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        
        if metrics.state != CircuitState::HalfOpen {
            info!("Circuit breaker transitioning to HALF-OPEN state");
            metrics.state = CircuitState::HalfOpen;
            metrics.half_open_success_count = 0;
            metrics.last_state_change = Instant::now();
        }
    }
    
    /// Transition to closed state
    fn transition_to_closed(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        
        if metrics.state != CircuitState::Closed {
            info!("Circuit breaker transitioning to CLOSED state");
            metrics.state = CircuitState::Closed;
            metrics.failure_count = 0;
            metrics.last_state_change = Instant::now();
        }
    }
    
    /// Record a successful request
    fn record_success(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.total_requests += 1;
        metrics.successful_requests += 1;
        
        match metrics.state {
            CircuitState::Closed => {
                // Reset failure count on success
                metrics.failure_count = 0;
            },
            CircuitState::HalfOpen => {
                // Increment success count in half-open state
                metrics.half_open_success_count += 1;
                
                // Check if we've reached the threshold to close the circuit
                if metrics.half_open_success_count >= self.config.half_open_request_threshold {
                    drop(metrics);
                    self.transition_to_closed();
                }
            },
            CircuitState::Open => {
                // This shouldn't happen, but just in case
                warn!("Successful request while circuit is open");
            },
        }
    }
    
    /// Record a failed request
    fn record_failure(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.total_requests += 1;
        metrics.failed_requests += 1;
        
        match metrics.state {
            CircuitState::Closed => {
                // Increment failure count
                metrics.failure_count += 1;
                
                // Check if we've reached the threshold to open the circuit
                if metrics.failure_count >= self.config.failure_threshold {
                    drop(metrics);
                    self.transition_to_open();
                }
            },
            CircuitState::HalfOpen => {
                // Any failure in half-open state opens the circuit again
                drop(metrics);
                self.transition_to_open();
            },
            CircuitState::Open => {
                // This shouldn't happen, but just in case
                warn!("Failed request while circuit is open");
            },
        }
    }
}

#[async_trait]
impl<T, E> CircuitBreaker<T, E> for StandardCircuitBreaker
where
    T: Send,
    E: std::error::Error + Send + Sync + 'static,
{
    async fn execute<F, Fut>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, E>> + Send,
    {
        // Check if circuit is open
        if self.is_open() {
            return Err(CircuitBreakerError::Open);
        }
        
        // Execute the function
        match f().await {
            Ok(result) => {
                // Record success
                self.record_success();
                Ok(result)
            },
            Err(err) => {
                // Record failure
                self.record_failure();
                Err(CircuitBreakerError::ServiceError(err))
            },
        }
    }
    
    fn metrics(&self) -> CircuitBreakerMetrics {
        self.metrics.lock().unwrap().clone()
    }
    
    fn reset(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        *metrics = CircuitBreakerMetrics::default();
        info!("Circuit breaker reset to initial state");
    }
}

/// Circuit breaker registry to manage multiple circuit breakers
pub struct CircuitBreakerRegistry {
    circuit_breakers: Arc<Mutex<HashMap<String, Arc<StandardCircuitBreaker>>>>,
}

impl CircuitBreakerRegistry {
    /// Create a new circuit breaker registry
    pub fn new() -> Self {
        Self {
            circuit_breakers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Get or create a circuit breaker
    pub fn get_or_create(&self, name: &str, config: Option<CircuitBreakerConfig>) -> Arc<StandardCircuitBreaker> {
        let mut circuit_breakers = self.circuit_breakers.lock().unwrap();
        
        if let Some(cb) = circuit_breakers.get(name) {
            cb.clone()
        } else {
            let cb = Arc::new(StandardCircuitBreaker::new(config.unwrap_or_default()));
            circuit_breakers.insert(name.to_string(), cb.clone());
            cb
        }
    }
    
    /// Get all circuit breakers
    pub fn get_all(&self) -> HashMap<String, Arc<StandardCircuitBreaker>> {
        let circuit_breakers = self.circuit_breakers.lock().unwrap();
        circuit_breakers.clone()
    }
    
    /// Reset all circuit breakers
    pub fn reset_all(&self) {
        let circuit_breakers = self.circuit_breakers.lock().unwrap();
        
        for (name, cb) in circuit_breakers.iter() {
            info!("Resetting circuit breaker: {}", name);
            cb.reset();
        }
    }
}

impl Default for CircuitBreakerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global circuit breaker registry
static mut CIRCUIT_BREAKER_REGISTRY: Option<CircuitBreakerRegistry> = None;

/// Get the global circuit breaker registry
pub fn get_circuit_breaker_registry() -> &'static CircuitBreakerRegistry {
    unsafe {
        if CIRCUIT_BREAKER_REGISTRY.is_none() {
            CIRCUIT_BREAKER_REGISTRY = Some(CircuitBreakerRegistry::new());
        }
        
        CIRCUIT_BREAKER_REGISTRY.as_ref().unwrap()
    }
}

/// Get a circuit breaker from the global registry
pub fn get_circuit_breaker(name: &str, config: Option<CircuitBreakerConfig>) -> Arc<StandardCircuitBreaker> {
    get_circuit_breaker_registry().get_or_create(name, config)
} 