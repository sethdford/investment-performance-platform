use std::io;
use tracing::Level;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry, Layer as SubscriberLayer};
use std::collections::HashMap;

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log level
    pub level: Level,
    /// Whether to log to stdout
    pub log_to_stdout: bool,
    /// Whether to log to file
    pub log_to_file: bool,
    /// Log file directory
    pub log_dir: Option<String>,
    /// Log file prefix
    pub log_file_prefix: Option<String>,
    /// Whether to use JSON format
    pub json_format: bool,
    /// Whether to log spans
    pub log_spans: bool,
    /// Module-specific log levels
    pub module_levels: HashMap<String, Level>,
    /// Whether to include source code information
    pub include_source_code: bool,
    /// Whether to include line numbers
    pub include_line_numbers: bool,
    /// Whether to include thread IDs
    pub include_thread_ids: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            log_to_stdout: true,
            log_to_file: false,
            log_dir: None,
            log_file_prefix: None,
            json_format: false,
            log_spans: true,
            module_levels: HashMap::new(),
            include_source_code: true,
            include_line_numbers: true,
            include_thread_ids: true,
        }
    }
}

/// Initialize logging with the given configuration
pub fn init_logging(config: LoggingConfig) -> Option<WorkerGuard> {
    // Start with the default filter based on the config level
    let mut env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{}", config.level)));

    // Add module-specific log levels
    for (module, level) in &config.module_levels {
        let directive = format!("{}={}", module, level);
        if let Ok(parsed) = directive.parse() {
            env_filter = env_filter.add_directive(parsed);
        }
    }

    let mut layers = Vec::new();
    let mut guard = None;

    // Add stdout layer if enabled
    if config.log_to_stdout {
        let stdout_layer = Layer::new()
            .with_writer(io::stdout)
            .with_ansi(true)
            .with_file(config.include_source_code)
            .with_line_number(config.include_line_numbers)
            .with_thread_ids(config.include_thread_ids);

        let stdout_layer = if config.json_format {
            SubscriberLayer::boxed(stdout_layer.json().with_span_events(get_span_events(config.log_spans)))
        } else {
            SubscriberLayer::boxed(stdout_layer.with_span_events(get_span_events(config.log_spans)))
        };

        layers.push(stdout_layer);
    }

    // Add file layer if enabled
    if config.log_to_file {
        if let (Some(log_dir), Some(log_file_prefix)) = (config.log_dir.as_ref(), config.log_file_prefix.as_ref()) {
            let file_appender = RollingFileAppender::new(
                Rotation::DAILY,
                log_dir,
                log_file_prefix,
            );
            let (non_blocking, worker_guard) = NonBlocking::new(file_appender);
            guard = Some(worker_guard);

            let file_layer = Layer::new()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_file(config.include_source_code)
                .with_line_number(config.include_line_numbers)
                .with_thread_ids(config.include_thread_ids);

            let file_layer = if config.json_format {
                SubscriberLayer::boxed(file_layer.json().with_span_events(get_span_events(config.log_spans)))
            } else {
                SubscriberLayer::boxed(file_layer.with_span_events(get_span_events(config.log_spans)))
            };

            layers.push(file_layer);
        }
    }

    // Initialize the subscriber
    Registry::default()
        .with(env_filter)
        .with(layers)
        .init();

    guard
}

/// Get span events based on configuration
fn get_span_events(log_spans: bool) -> FmtSpan {
    if log_spans {
        FmtSpan::NEW | FmtSpan::CLOSE | FmtSpan::ENTER | FmtSpan::EXIT
    } else {
        FmtSpan::NONE
    }
}

/// Initialize default logging
pub fn init_default_logging() -> Option<WorkerGuard> {
    init_logging(LoggingConfig::default())
}

/// Initialize logging for tests
#[cfg(test)]
pub fn init_test_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_test_writer()
        .try_init();
}

/// Create a logging configuration with common settings for development
pub fn development_config() -> LoggingConfig {
    let mut config = LoggingConfig::default();
    config.level = Level::DEBUG;
    config.json_format = false;
    config.log_spans = true;
    config.include_source_code = true;
    config.include_line_numbers = true;
    config.include_thread_ids = true;
    
    // Add module-specific log levels
    let mut module_levels = HashMap::new();
    module_levels.insert("investment_management".to_string(), Level::DEBUG);
    module_levels.insert("investment_management::api".to_string(), Level::DEBUG);
    module_levels.insert("investment_management::portfolio".to_string(), Level::DEBUG);
    module_levels.insert("investment_management::tax".to_string(), Level::DEBUG);
    module_levels.insert("investment_management::analytics".to_string(), Level::DEBUG);
    module_levels.insert("hyper".to_string(), Level::INFO);
    module_levels.insert("aws_sdk".to_string(), Level::INFO);
    config.module_levels = module_levels;
    
    config
}

/// Create a logging configuration with common settings for production
pub fn production_config() -> LoggingConfig {
    let mut config = LoggingConfig::default();
    config.level = Level::INFO;
    config.json_format = true;
    config.log_spans = true;
    config.include_source_code = false;
    config.include_line_numbers = true;
    config.include_thread_ids = true;
    config.log_to_file = true;
    config.log_dir = Some("logs".to_string());
    config.log_file_prefix = Some("investment-management".to_string());
    
    // Add module-specific log levels
    let mut module_levels = HashMap::new();
    module_levels.insert("investment_management".to_string(), Level::INFO);
    module_levels.insert("investment_management::api".to_string(), Level::INFO);
    module_levels.insert("investment_management::portfolio".to_string(), Level::INFO);
    module_levels.insert("investment_management::tax".to_string(), Level::INFO);
    module_levels.insert("investment_management::analytics".to_string(), Level::INFO);
    module_levels.insert("hyper".to_string(), Level::WARN);
    module_levels.insert("aws_sdk".to_string(), Level::WARN);
    config.module_levels = module_levels;
    
    config
}

/// Macro to log a method entry with parameters
#[macro_export]
macro_rules! log_method_entry {
    ($method:expr, $($param:expr),*) => {
        tracing::debug!(method = $method, $($param),*, "Method entry");
    };
}

/// Macro to log a method exit with result
#[macro_export]
macro_rules! log_method_exit {
    ($method:expr, $result:expr) => {
        match $result {
            Ok(ref value) => tracing::debug!(method = $method, "Method exit: success"),
            Err(ref err) => tracing::warn!(method = $method, error = ?err, "Method exit: error"),
        }
    };
}

/// Macro to log an error
#[macro_export]
macro_rules! log_error {
    ($method:expr, $err:expr, $($param:expr),*) => {
        tracing::error!(method = $method, error = ?$err, $($param),*, "Error occurred");
    };
}

/// Macro to log a warning
#[macro_export]
macro_rules! log_warning {
    ($method:expr, $message:expr, $($param:expr),*) => {
        tracing::warn!(method = $method, message = $message, $($param),*, "Warning");
    };
}

/// Macro to log an info message
#[macro_export]
macro_rules! log_info {
    ($method:expr, $message:expr, $($param:expr),*) => {
        tracing::info!(method = $method, message = $message, $($param),*, "Info");
    };
} 