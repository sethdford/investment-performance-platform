# Structured Logging in the Investment Platform

This document provides guidance on using the structured logging system implemented in the Investment Platform.

## Overview

The Investment Platform uses the `tracing` crate for structured, contextual logging. This provides several advantages over traditional logging:

1. **Structured Data**: Logs include structured data that can be easily parsed and analyzed.
2. **Context Propagation**: Context is automatically propagated through async code.
3. **Span-Based Tracing**: Operations can be grouped into spans for better visibility.
4. **Multiple Output Formats**: Logs can be output in human-readable or JSON format.
5. **Configurable Levels**: Log levels can be configured at runtime.

## Logging Configuration

The logging system is configured through the `LoggingConfig` struct:

```rust
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
}
```

### Default Configuration

The default configuration logs to stdout at the INFO level in human-readable format:

```rust
let config = LoggingConfig::default();
```

### Custom Configuration

You can customize the configuration to suit your needs:

```rust
let config = LoggingConfig {
    level: Level::DEBUG,
    log_to_stdout: true,
    log_to_file: true,
    log_dir: Some("logs".to_string()),
    log_file_prefix: Some("investment-platform".to_string()),
    json_format: true,
    log_spans: true,
};
```

## Initializing Logging

To initialize logging, call the `init_logging` function with your configuration:

```rust
let _guard = init_logging(config);
```

The returned guard must be kept alive for the duration of the program to ensure logs are flushed.

For convenience, you can also use the `init_default_logging` function:

```rust
let _guard = init_default_logging();
```

## Using Structured Logging

### Basic Logging

The `tracing` crate provides macros for logging at different levels:

```rust
use tracing::{trace, debug, info, warn, error};

trace!("This is a trace message");
debug!("This is a debug message");
info!("This is an info message");
warn!("This is a warning message");
error!("This is an error message");
```

### Structured Data

You can include structured data in your logs:

```rust
info!(
    account_id = %account.id,
    balance = account.balance,
    "Account created"
);
```

### Spans

Spans group related logs and provide context:

```rust
use tracing::{span, Level};

let span = span!(Level::INFO, "process_transaction", transaction_id = %id);
let _enter = span.enter();

// Logs within this scope will be associated with the span
info!("Processing transaction");
```

### Instrumenting Functions

You can use the `#[instrument]` attribute to automatically create spans for functions:

```rust
use tracing::instrument;

#[instrument(skip(password), fields(user_id = %user.id))]
fn authenticate(user: &User, password: &str) -> Result<(), AuthError> {
    // Function body
}
```

## Log Levels

The following log levels are available, in order of increasing severity:

1. **TRACE**: Very detailed information, typically only useful for debugging.
2. **DEBUG**: Detailed information that is useful for debugging.
3. **INFO**: Information messages that highlight the progress of the application.
4. **WARN**: Potentially harmful situations that might require attention.
5. **ERROR**: Error events that might still allow the application to continue running.

## Best Practices

1. **Use Appropriate Levels**: Use the appropriate log level for each message.
2. **Include Context**: Include relevant context in your logs, such as IDs and values.
3. **Instrument Key Functions**: Use the `#[instrument]` attribute on key functions.
4. **Create Spans for Operations**: Create spans for long-running or complex operations.
5. **Skip Sensitive Data**: Use the `skip` parameter to exclude sensitive data from logs.
6. **Use Structured Data**: Include structured data in your logs for better analysis.

## Example: AWS TLH Alert Service

The AWS TLH Alert Service uses structured logging extensively:

```rust
#[instrument(skip(self, account), fields(account_id = %account.id))]
pub async fn start_monitoring(&mut self, account: &UnifiedManagedAccount) -> ApiResult<()> {
    info!(
        account_id = %account.id,
        region = %self.config.region,
        alert_frequency = self.config.alert_frequency_seconds,
        "Starting AWS TLH Alert Service"
    );
    
    // ... function body ...
}
```

This creates a span for the `start_monitoring` function, skips the `self` and `account` parameters (which might be large), and includes the `account_id` field in the span. It then logs an info message with additional structured data.

## Log Aggregation and Analysis

The structured logs produced by the Investment Platform can be aggregated and analyzed using various tools:

1. **AWS CloudWatch Logs**: For AWS deployments, logs can be sent to CloudWatch Logs.
2. **Elasticsearch/Kibana**: For on-premises or cloud deployments, logs can be sent to Elasticsearch and visualized with Kibana.
3. **Grafana/Loki**: Logs can be sent to Loki and visualized with Grafana.
4. **Datadog**: Logs can be sent to Datadog for monitoring and analysis.

## Conclusion

The structured logging system in the Investment Platform provides comprehensive observability into the system's behavior. By following the guidelines in this document, you can ensure that your logs are informative, contextual, and easy to analyze. 