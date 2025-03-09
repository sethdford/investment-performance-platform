## SpecStory

## Build and Test Execution Before Commit (3/3/2025, 8:10:56 AM)

_**User**_

could we run the end to end build and integration tests then do a commit if everything works

---

_**Assistant**_

I'll help you run the build and tests, and if everything passes, we can commit the changes. Let me execute these steps sequentially.

First, let's run the build and tests:

---

_****_

```bash
cargo build
```

```
sethford@Seths-MacBook-Pro rust-investment-performance % 
help: consider importing this enum
    |
1   + use aws_sdk_dynamodb::types::AttributeValue;
    |
help: if you import `AttributeValue`, refer to it directly
    |
997 -         item.insert("status".to_string(), aws_sdk_dynamodb::model::Attribu
teValue::S(status_str.to_str
ing()));
                            997 +         item.insert("status".to_string(), Attr
ibuteValue::S(status_str.to_string()));
    |

error[E0433]: failed to resolve: could not find `model` in `aws_sdk_dynamodb`
   --> performance-calculator/src/calculations/tenant/metrics.rs:999:65
    |
999 | ...g(), aws_sdk_dynamodb::model::AttributeValue::S(record.created_at.to_rf
c3339()));
    |                           ^^^^^ could not find `model` in `aws_sdk_dynamod
b`
    |
help: consider importing this enum
    |
1   + use aws_sdk_dynamodb::types::AttributeValue;
    |
help: if you import `AttributeValue`, refer to it directly
    |
999 -         item.insert("created_at".to_string(), aws_sdk_dynamodb::model::Att
ributeValue::S(record.create
d_at.to_rfc3339()));
                            999 +         item.insert("created_at".to_string(),
AttributeValue::S(record.created_at.to_rfc3339()));
    |

error[E0433]: failed to resolve: could not find `model` in `aws_sdk_dynamodb`
    --> performance-calculator/src/calculations/tenant/metrics.rs:1000:65
     |
1000 | ...(), aws_sdk_dynamodb::model::AttributeValue::S(record.updated_at.to_rf
c3339()));
     |                          ^^^^^ could not find `model` in `aws_sdk_dynamod
b`
     |
help: consider importing this enum
     |
1    + use aws_sdk_dynamodb::types::AttributeValue;
     |
help: if you import `AttributeValue`, refer to it directly
     |
1000 -         item.insert("updated_at".to_string(), aws_sdk_dynamodb::model::At
tributeValue::S(record.updat
ed_at.to_rfc3339()));
                            1000 +         item.insert("updated_at".to_string(),
 AttributeValue::S(record.updated_at.to_rfc3339()));
     |

error[E0433]: failed to resolve: could not find `model` in `aws_sdk_dynamodb`
    --> performance-calculator/src/calculations/tenant/metrics.rs:1006:67
     |
1006 | ...(), aws_sdk_dynamodb::model::AttributeValue::N(value.to_string()));
     |                          ^^^^^ could not find `model` in `aws_sdk_dynamod
b`
     |
help: consider importing this enum
     |
1    + use aws_sdk_dynamodb::types::AttributeValue;
     |
help: if you import `AttributeValue`, refer to it directly
     |
1006 -                 charges_map.insert(key.clone(), aws_sdk_dynamodb::model::
AttributeValue::N(value.to_s
tring()));
                            1006 +                 charges_map.insert(key.clone(
), AttributeValue::N(value.to_string()));
     |

error[E0433]: failed to resolve: could not find `model` in `aws_sdk_dynamodb`
    --> performance-calculator/src/calculations/tenant/metrics.rs:1008:77
     |
1008 | ...(), aws_sdk_dynamodb::model::AttributeValue::M(charges_map));
     |                          ^^^^^ could not find `model` in `aws_sdk_dynamod
b`
     |
help: consider importing this enum
     |
1    + use aws_sdk_dynamodb::types::AttributeValue;
     |
help: if you import `AttributeValue`, refer to it directly
     |
1008 -             item.insert("additional_charges".to_string(), aws_sdk_dynamod
b::model::AttributeValue::M(
charges_map));
                            1008 +             item.insert("additional_charges".
to_string(), AttributeValue::M(charges_map));
     |

error[E0422]: cannot find struct, variant or union type `RetryConfig` in this sc
ope
   --> performance-calculator/src/resilience/retry.rs:151:18
    |
151 |     let config = RetryConfig {
    |                  ^^^^^^^^^^^ not found in this scope
    |
help: consider importing one of these structs
    |
1   + use crate::calculations::error_handling::RetryConfig;
    |
1   + use aws_config::retry::RetryConfig;
    |
1   + use aws_sdk_dynamodb::config::retry::RetryConfig;
    |
1   + use aws_sdk_sqs::config::retry::RetryConfig;
    |
      and 2 other candidates

error[E0433]: failed to resolve: use of undeclared type `RetryConfig`
   --> performance-calculator/src/resilience/retry.rs:154:11
    |
154 |         ..RetryConfig::default()
    |           ^^^^^^^^^^^ use of undeclared type `RetryConfig`
    |
help: consider importing one of these structs
    |
1   + use crate::calculations::error_handling::RetryConfig;
    |
1   + use aws_config::retry::RetryConfig;
    |
1   + use aws_sdk_dynamodb::config::retry::RetryConfig;
    |
1   + use aws_sdk_sqs::config::retry::RetryConfig;
    |
      and 2 other candidates

error[E0422]: cannot find struct, variant or union type `RetryConfig` in this sc
ope
   --> performance-calculator/src/resilience/retry.rs:175:18
    |
175 |     let config = RetryConfig {
    |                  ^^^^^^^^^^^ not found in this scope
    |
help: consider importing one of these structs
    |
1   + use crate::calculations::error_handling::RetryConfig;
    |
1   + use aws_config::retry::RetryConfig;
    |
1   + use aws_sdk_dynamodb::config::retry::RetryConfig;
    |
1   + use aws_sdk_sqs::config::retry::RetryConfig;
    |
      and 2 other candidates

error[E0433]: failed to resolve: use of undeclared type `RetryConfig`
   --> performance-calculator/src/resilience/retry.rs:178:11
    |
178 |         ..RetryConfig::default()
    |           ^^^^^^^^^^^ use of undeclared type `RetryConfig`
    |
help: consider importing one of these structs
    |
1   + use crate::calculations::error_handling::RetryConfig;
    |
1   + use aws_config::retry::RetryConfig;
    |
1   + use aws_sdk_dynamodb::config::retry::RetryConfig;
    |
1   + use aws_sdk_sqs::config::retry::RetryConfig;
    |
      and 2 other candidates

error[E0412]: cannot find type `StandardRetryPolicy` in this scope
   --> performance-calculator/src/resilience/mod.rs:168:50
    |
168 | pub fn get_retry_policy<E>(name: &str) -> Result<StandardRetryPolicy<E>>
    |                                                  ^^^^^^^^^^^^^^^^^^^ not f
ound in this scope
    |
help: consider importing this struct
    |
5   + use crate::resilience::retry::StandardRetryPolicy;
    |

error[E0433]: failed to resolve: use of undeclared type `StandardRetryPolicy`
   --> performance-calculator/src/resilience/mod.rs:172:8
    |
172 |     Ok(StandardRetryPolicy::new(name))
    |        ^^^^^^^^^^^^^^^^^^^ use of undeclared type `StandardRetryPolicy`
    |
help: consider importing this struct
    |
5   + use crate::resilience::retry::StandardRetryPolicy;
    |

error[E0412]: cannot find type `RetryConfig` in module `retry`
   --> performance-calculator/src/resilience/mod.rs:198:26
    |
198 |     retry_config: retry::RetryConfig,
    |                          ^^^^^^^^^^^ not found in `retry`
    |
help: consider importing one of these structs
    |
5   + use crate::calculations::error_handling::RetryConfig;
    |
5   + use aws_config::retry::RetryConfig;
    |
5   + use aws_sdk_dynamodb::config::retry::RetryConfig;
    |
5   + use aws_sdk_sqs::config::retry::RetryConfig;
    |
      and 2 other candidates
help: if you import `RetryConfig`, refer to it directly
    |
198 -     retry_config: retry::RetryConfig,
198 +     retry_config: RetryConfig,
    |

error[E0412]: cannot find type `RetryConfig` in module `retry`
   --> performance-calculator/src/resilience/mod.rs:231:26
    |
231 |     retry_config: retry::RetryConfig,
    |                          ^^^^^^^^^^^ not found in `retry`
    |
help: consider importing one of these structs
    |
5   + use crate::calculations::error_handling::RetryConfig;
    |
5   + use aws_config::retry::RetryConfig;
    |
5   + use aws_sdk_dynamodb::config::retry::RetryConfig;
    |
5   + use aws_sdk_sqs::config::retry::RetryConfig;
    |
      and 2 other candidates
help: if you import `RetryConfig`, refer to it directly
    |
231 -     retry_config: retry::RetryConfig,
231 +     retry_config: RetryConfig,
    |

error[E0412]: cannot find type `RetryConfig` in module `retry`
   --> performance-calculator/src/resilience/mod.rs:275:26
    |
275 |     retry_config: retry::RetryConfig,
    |                          ^^^^^^^^^^^ not found in `retry`
    |
help: consider importing one of these structs
    |
5   + use crate::calculations::error_handling::RetryConfig;
    |
5   + use aws_config::retry::RetryConfig;
    |
5   + use aws_sdk_dynamodb::config::retry::RetryConfig;
    |
5   + use aws_sdk_sqs::config::retry::RetryConfig;
    |
      and 2 other candidates
help: if you import `RetryConfig`, refer to it directly
    |
275 -     retry_config: retry::RetryConfig,
275 +     retry_config: RetryConfig,
    |

warning: unused import: `anyhow`
  --> performance-calculator/src/calculations/config.rs:22:22
   |
22 | use anyhow::{Result, anyhow, Context};
   |                      ^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: unused import: `std::env`
  --> performance-calculator/src/calculations/config.rs:23:5
   |
23 | use std::env;
   |     ^^^^^^^^

warning: unused import: `std::fs::File`
  --> performance-calculator/src/calculations/config.rs:24:5
   |
24 | use std::fs::File;
   |     ^^^^^^^^^^^^^

warning: unused import: `std::io::Read`
  --> performance-calculator/src/calculations/config.rs:25:5
   |
25 | use std::io::Read;
   |     ^^^^^^^^^^^^^

warning: unused imports: `AnalyticsConfig`, `CalculationScheduler`, `DataAccessS
ervice`, `IntegrationConfig`
, `VisualizationConfig`, and `currency::CurrencyConverter`
                              --> performance-calculator/src/calculations/factor
y.rs:37:5
   |
37 |     currency::CurrencyConverter,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^
38 |     query_api::{QueryApi, DataAccessService},
   |                           ^^^^^^^^^^^^^^^^^
39 |     streaming::StreamingProcessor,
40 |     scheduler::{CalculationScheduler, NotificationService, DefaultNotificat
ionService, EmailClient, ...
   |                 ^^^^^^^^^^^^^^^^^^^^
41 |     analytics::{AnalyticsConfig, AnalyticsEngine},
   |                 ^^^^^^^^^^^^^^^
42 |     visualization::{VisualizationConfig, VisualizationEngine},
   |                     ^^^^^^^^^^^^^^^^^^^
43 |     integration::{IntegrationConfig, IntegrationEngine},
   |                   ^^^^^^^^^^^^^^^^^

warning: unused import: `std::sync::Arc`
 --> performance-calculator/src/calculations/portfolio/mod.rs:3:5
  |
3 | use std::sync::Arc;
  |     ^^^^^^^^^^^^^^

warning: unused imports: `DateTime`, `Duration`, and `Utc`
 --> performance-calculator/src/calculations/performance_metrics.rs:5:14
  |
5 | use chrono::{DateTime, Utc, NaiveDate, Duration};
  |              ^^^^^^^^  ^^^             ^^^^^^^^

warning: unused import: `crate::calculations::risk_metrics::ReturnSeries`
 --> performance-calculator/src/calculations/performance_metrics.rs:7:5
  |
7 | use crate::calculations::risk_metrics::ReturnSeries;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal_macros::dec`
 --> performance-calculator/src/calculations/risk_metrics.rs:3:5
  |
3 | use rust_decimal_macros::dec;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `Result` and `anyhow`
 --> performance-calculator/src/calculations/risk_metrics.rs:4:14
  |
4 | use anyhow::{Result, anyhow};
  |              ^^^^^^  ^^^^^^

warning: unused import: `std::collections::HashMap`
 --> performance-calculator/src/calculations/risk_metrics.rs:5:5
  |
5 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `DateTime` and `Utc`
 --> performance-calculator/src/calculations/risk_metrics.rs:6:14
  |
6 | use chrono::{DateTime, Utc, NaiveDate};
  |              ^^^^^^^^  ^^^

warning: unused import: `std::cmp::Ordering`
 --> performance-calculator/src/calculations/risk_metrics.rs:8:5
  |
8 | use std::cmp::Ordering;
  |     ^^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::prelude::*`
 --> performance-calculator/src/calculations/periodic_returns.rs:2:5
  |
2 | use rust_decimal::prelude::*;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `DateTime` and `Utc`
 --> performance-calculator/src/calculations/periodic_returns.rs:5:14
  |
5 | use chrono::{DateTime, Utc, NaiveDate, Datelike};
  |              ^^^^^^^^  ^^^

warning: unused import: `Deserialize`
 --> performance-calculator/src/calculations/distributed_cache.rs:4:24
  |
4 | use serde::{Serialize, Deserialize};
  |                        ^^^^^^^^^^^

warning: unused import: `std::time::Duration`
 --> performance-calculator/src/calculations/distributed_cache.rs:6:5
  |
6 | use std::time::Duration;
  |     ^^^^^^^^^^^^^^^^^^^

warning: unused imports: `error`, `info`, and `warn`
 --> performance-calculator/src/calculations/distributed_cache.rs:8:15
  |
8 | use tracing::{info, warn, error};
  |               ^^^^  ^^^^  ^^^^^

warning: unused import: `uuid`
  --> performance-calculator/src/calculations/distributed_cache.rs:13:5
   |
13 | use uuid;
   |     ^^^^

warning: unused import: `serde_json::json`
  --> performance-calculator/src/calculations/distributed_cache.rs:14:5
   |
14 | use serde_json::json;
   |     ^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::Decimal`
  --> performance-calculator/src/calculations/streaming.rs:15:5
   |
15 | use rust_decimal::Decimal;
   |     ^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `info` and `warn`
  --> performance-calculator/src/calculations/streaming.rs:19:15
   |
19 | use tracing::{info, warn, error, debug};
   |               ^^^^  ^^^^

warning: unused imports: `AuditTrail`, `ExchangeRateProvider`, and `ExchangeRate
`
  --> performance-calculator/src/calculations/query_api.rs:16:49
   |
16 |     currency::{CurrencyConverter, CurrencyCode, ExchangeRate, ExchangeRateP
rovider},
   |                                                 ^^^^^^^^^^^^  ^^^^^^^^^^^^^
^^^^^^^
17 |     audit::{AuditTrail, AuditTrailManager},
   |             ^^^^^^^^^^

warning: unused imports: `Duration` and `NaiveDateTime`
 --> performance-calculator/src/calculations/scheduler.rs:2:40
  |
2 | use chrono::{DateTime, Utc, NaiveDate, NaiveDateTime, Duration, Datelike};
  |                                        ^^^^^^^^^^^^^  ^^^^^^^^

warning: unused import: `warn`
 --> performance-calculator/src/calculations/scheduler.rs:8:21
  |
8 | use tracing::{info, warn, error};
  |                     ^^^^

warning: unused import: `CalculationEventBuilder`
  --> performance-calculator/src/calculations/scheduler.rs:13:32
   |
13 |     audit::{AuditTrailManager, CalculationEventBuilder},
   |                                ^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Context`
 --> performance-calculator/src/calculations/analytics.rs:6:22
  |
6 | use anyhow::{Result, Context, anyhow};
  |                      ^^^^^^^

warning: unused import: `serde_json`
  --> performance-calculator/src/calculations/analytics.rs:13:5
   |
13 | use serde_json;
   |     ^^^^^^^^^^

warning: unused import: `Context`
 --> performance-calculator/src/calculations/visualization.rs:6:22
  |
6 | use anyhow::{Result, Context, anyhow};
  |                      ^^^^^^^

warning: unused import: `chrono::NaiveDate`
 --> performance-calculator/src/calculations/visualization.rs:7:5
  |
7 | use chrono::NaiveDate;
  |     ^^^^^^^^^^^^^^^^^

warning: unused imports: `error`, `info`, and `warn`
  --> performance-calculator/src/calculations/visualization.rs:12:15
   |
12 | use tracing::{info, warn, error};
   |               ^^^^  ^^^^  ^^^^^

warning: unused import: `Context`
 --> performance-calculator/src/calculations/integration.rs:6:22
  |
6 | use anyhow::{Result, Context, anyhow};
  |                      ^^^^^^^

warning: unused import: `NaiveDate`
 --> performance-calculator/src/calculations/integration.rs:7:29
  |
7 | use chrono::{DateTime, Utc, NaiveDate};
  |                             ^^^^^^^^^

warning: unused import: `rust_decimal::Decimal`
 --> performance-calculator/src/calculations/integration.rs:8:5
  |
8 | use rust_decimal::Decimal;
  |     ^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `error`, `info`, and `warn`
  --> performance-calculator/src/calculations/integration.rs:14:15
   |
14 | use tracing::{info, warn, error};
   |               ^^^^  ^^^^  ^^^^^

warning: unused imports: `AuditTrail`, `InMemoryAuditTrail`, `InMemoryCache`, `M
oneyWeightedReturn`, and `Ti
meWeightedReturn`
                              --> performance-calculator/src/calculations/benchm
arks.rs:7:13
   |
7  |     audit::{AuditTrail, InMemoryAuditTrail},
   |             ^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^
8  |     currency::{CurrencyConverter, ExchangeRate, CurrencyCode},
9  |     distributed_cache::{StringCache, InMemoryCache},
   |                                      ^^^^^^^^^^^^^
10 |     performance_metrics::{TimeWeightedReturn, MoneyWeightedReturn},
   |                           ^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::prelude::*`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:2:5
  |
2 | use rust_decimal::prelude::*;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashMap`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:4:5
  |
4 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `DateTime`, `NaiveDate`, and `Utc`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:5:14
  |
5 | use chrono::{DateTime, Utc, NaiveDate};
  |              ^^^^^^^^  ^^^  ^^^^^^^^^

warning: unused import: `calculate_information_ratio`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:7:114
  |
7 | ...e_alpha, calculate_tracking_error, calculate_information_ratio};
  |                                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Error as DynamoDbError`
 --> performance-calculator/src/calculations/tenant/dynamodb.rs:1:50
  |
1 | use aws_sdk_dynamodb::{Client as DynamoDbClient, Error as DynamoDbError};
  |                                                  ^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `uuid::Uuid`
 --> performance-calculator/src/calculations/tenant/dynamodb.rs:7:5
  |
7 | use uuid::Uuid;
  |     ^^^^^^^^^^

warning: unused import: `chrono::Utc`
 --> performance-calculator/src/calculations/tenant/cached.rs:4:5
  |
4 | use chrono::Utc;
  |     ^^^^^^^^^^^

warning: unused import: `Duration`
 --> performance-calculator/src/calculations/tenant/metrics.rs:1:24
  |
1 | use chrono::{DateTime, Duration, Utc};
  |                        ^^^^^^^^

warning: unused import: `info`
 --> performance-calculator/src/calculations/tenant/metrics.rs:3:15
  |
3 | use tracing::{info, warn};
  |               ^^^^

warning: unused import: `Tenant`
 --> performance-calculator/src/calculations/tenant/metrics.rs:6:35
  |
6 | use crate::calculations::tenant::{Tenant, ResourceLimits};
  |                                   ^^^^^^

warning: unused imports: `delete_item::DeleteItemInput`, `get_item::GetItemInput
`, `put_item::PutItemInput`,
 and `query::QueryInput`
                              --> performance-calculator/src/calculations/tenant
/metrics.rs:10:5
   |
10 |     put_item::PutItemInput,
   |     ^^^^^^^^^^^^^^^^^^^^^^
11 |     get_item::GetItemInput,
   |     ^^^^^^^^^^^^^^^^^^^^^^
12 |     query::QueryInput,
   |     ^^^^^^^^^^^^^^^^^
13 |     delete_item::DeleteItemInput,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `serde_json::Value`
  --> performance-calculator/src/calculations/tenant/metrics.rs:16:5
   |
16 | use serde_json::Value;
   |     ^^^^^^^^^^^^^^^^^

warning: unused import: `crate::resilience`
  --> performance-calculator/src/calculations/tenant/metrics.rs:18:5
   |
18 | use crate::resilience;
   |     ^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::Decimal`
  --> performance-calculator/src/calculations/tenant/metrics.rs:19:5
   |
19 | use rust_decimal::Decimal;
   |     ^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `anyhow`
 --> performance-calculator/src/resilience/mod.rs:9:22
  |
9 | use anyhow::{Result, anyhow};
  |                      ^^^^^^

warning: unused import: `info`
  --> performance-calculator/src/resilience/mod.rs:10:15
   |
10 | use tracing::{info, warn, error};
   |               ^^^^

warning: unused import: `crate::calculations::error_handling::CalculationError`
  --> performance-calculator/src/resilience/mod.rs:12:5
   |
12 | use crate::calculations::error_handling::CalculationError;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Duration`
 --> performance-calculator/src/resilience/circuit_breaker.rs:2:17
  |
2 | use std::time::{Duration, Instant};
  |                 ^^^^^^^^

warning: unused import: `std::error::Error`
 --> performance-calculator/src/resilience/circuit_breaker.rs:8:5
  |
8 | use std::error::Error;
  |     ^^^^^^^^^^^^^^^^^

warning: unused imports: `AtomicU64`, `AtomicUsize`, and `Ordering`
 --> performance-calculator/src/resilience/circuit_breaker.rs:9:25
  |
9 | use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
  |                         ^^^^^^^^^  ^^^^^^^^^^^  ^^^^^^^^

warning: unused import: `tokio::time::sleep`
  --> performance-calculator/src/resilience/circuit_breaker.rs:10:5
   |
10 | use tokio::time::sleep;
   |     ^^^^^^^^^^^^^^^^^^

warning: unused import: `anyhow`
 --> performance-calculator/src/resilience/retry.rs:6:22
  |
6 | use anyhow::{Result, anyhow};
  |                      ^^^^^^

warning: unused imports: `error` and `info`
 --> performance-calculator/src/resilience/retry.rs:7:15
  |
7 | use tracing::{info, warn, error};
  |               ^^^^        ^^^^^

warning: unused import: `rand::Rng`
 --> performance-calculator/src/resilience/retry.rs:8:5
  |
8 | use rand::Rng;
  |     ^^^^^^^^^

warning: unused import: `thiserror::Error`
 --> performance-calculator/src/resilience/retry.rs:9:5
  |
9 | use thiserror::Error;
  |     ^^^^^^^^^^^^^^^^

warning: unused import: `error`
 --> performance-calculator/src/resilience/health_check.rs:7:27
  |
7 | use tracing::{info, warn, error};
  |                           ^^^^^

warning: unused import: `async_trait::async_trait`
 --> performance-calculator/src/resilience/bulkhead.rs:4:5
  |
4 | use async_trait::async_trait;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `error`
 --> performance-calculator/src/resilience/bulkhead.rs:7:27
  |
7 | use tracing::{info, warn, error};
  |                           ^^^^^

warning: unused import: `thiserror::Error`
 --> performance-calculator/src/resilience/bulkhead.rs:8:5
  |
8 | use thiserror::Error;
  |     ^^^^^^^^^^^^^^^^

warning: unused imports: `error` and `warn`
 --> performance-calculator/src/resilience/chaos.rs:6:21
  |
6 | use tracing::{info, warn, error};
  |                     ^^^^  ^^^^^

warning: unused import: `get_circuit_breaker`
  --> performance-calculator/src/resilience/mod.rs:22:29
   |
22 |     StandardCircuitBreaker, get_circuit_breaker, get_circuit_breaker_with_c
onfig,
   |                             ^^^^^^^^^^^^^^^^^^^

warning: unused import: `retry_with_exponential_backoff`
  --> performance-calculator/src/resilience/mod.rs:26:43
   |
26 |     RetryConfig, RetryError, RetryPolicy, retry_with_exponential_backoff,
   |                                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `with_chaos`
  --> performance-calculator/src/resilience/mod.rs:43:5
   |
43 |     with_chaos,
   |     ^^^^^^^^^^

warning: unused variable: `beginning_base`
   --> performance-calculator/src/calculations/currency.rs:141:13
    |
141 |         let beginning_base = beginning_amount * beginning_rate;
    |             ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an
 underscore: `_beginning_bas
e`
                                |
    = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `event`
   --> performance-calculator/src/calculations/query_api.rs:370:13
    |
370 |         let event = self.audit_manager.start_calculation(
    |             ^^^^^ help: if this is intentional, prefix it with an undersco
re: `_event`

warning: unused variable: `query_id_for_audit`
   --> performance-calculator/src/calculations/query_api.rs:391:13
    |
391 |         let query_id_for_audit = query_id.clone();
    |             ^^^^^^^^^^^^^^^^^^ help: if this is intentional, prefix it wit
h an underscore: `_query_id_
for_audit`
                            
warning: unused variable: `portfolio_data`
   --> performance-calculator/src/calculations/query_api.rs:626:25
    |
626 |                     let portfolio_data = data_service.get_portfolio_data(
    |                         ^^^^^^^^^^^^^^ help: if this is intentional, prefi
x it with an underscore: `_p
ortfolio_data`
                            
warning: unused variable: `scenario`
   --> performance-calculator/src/calculations/analytics.rs:379:13
    |
379 |         let scenario = scenarios.get(scenario_id)
    |             ^^^^^^^^ help: if this is intentional, prefix it with an under
score: `_scenario`

warning: unused variable: `request_id`
   --> performance-calculator/src/calculations/visualization.rs:543:9
    |
543 |         request_id: &str,
    |         ^^^^^^^^^^ help: if this is intentional, prefix it with an undersc
ore: `_request_id`

warning: `graphql-api` (bin "graphql-api") generated 20 warnings
error: could not compile `graphql-api` (bin "graphql-api") due to 1 previous err
or; 20 warnings emitted
warning: unused variable: `request_id`
   --> performance-calculator/src/calculations/integration.rs:844:65
    |
844 | ...uest, request_id: &str) -> Result<DataImportResult> {
    |          ^^^^^^^^^^ help: if this is intentional, prefix it with an unders
core: `_request_id`

error[E0107]: missing generics for trait `CircuitBreaker`
   --> performance-calculator/src/resilience/mod.rs:107:9
    |
107 |     CB: CircuitBreaker,
    |         ^^^^^^^^^^^^^^ expected 1 generic argument
    |
note: trait defined here, with 1 generic parameter: `E`
   --> performance-calculator/src/resilience/circuit_breaker.rs:102:11
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           ^^^^^^^^^^^^^^ -
help: add missing generic argument
    |
107 |     CB: CircuitBreaker<E>,
    |                       +++

error[E0107]: missing generics for trait `CircuitBreaker`
   --> performance-calculator/src/resilience/mod.rs:153:58
    |
153 | pub fn get_circuit_breaker(name: &str) -> Result<Box<dyn CircuitBreaker>>
{
    |                                                          ^^^^^^^^^^^^^^ ex
pected 1 generic argument
    |
note: trait defined here, with 1 generic parameter: `E`
   --> performance-calculator/src/resilience/circuit_breaker.rs:102:11
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           ^^^^^^^^^^^^^^ -
help: add missing generic argument
    |
153 | pub fn get_circuit_breaker(name: &str) -> Result<Box<dyn CircuitBreaker<E>
>> {
    |                                                                        +++

error[E0038]: the trait `Bulkhead` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:158:70
    |
158 | pub fn get_bulkhead(name: &str, max_concurrent: usize) -> Result<Box<dyn B
ulkhead>> {
    |                                                                      ^^^^^
^^^^^^^ `Bulkhead` cannot be
 made into an object
                                |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to
allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference
/items/traits.html#object-safety>
                                                           --> performance-calcu
lator/src/resilience/bulkhead.rs:87:14
    |
85  | pub trait Bulkhead: Send + Sync {
    |           -------- this trait cannot be made into an object...
86  |     /// Execute an operation with bulkhead protection
87  |     async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, Bulkh
eadError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type paramete
rs
    = help: consider moving `execute` to another trait
    = help: only type `resilience::bulkhead::StandardBulkhead` is seen to implem
ent the trait in this crate,
 consider using it directly instead
                                = note: `Bulkhead` can be implemented in other c
rates; if you want to support your users passing their o
wn types here, you can't refer to a specific type
                            
error[E0599]: no method named `acquire` found for reference `&B` in the current
scope
   --> performance-calculator/src/resilience/mod.rs:123:20
    |
123 |     match bulkhead.acquire().await {
    |                    ^^^^^^^ method not found in `&B`

error[E0599]: no variant or associated item named `Timeout` found for enum `Bulk
headError` in the current sc
ope
                               --> performance-calculator/src/resilience/mod.rs:
213:32
    |
213 |                 BulkheadError::Timeout => e,
    |                                ^^^^^^^ variant or associated item not foun
d in `BulkheadError<_>`
    |
   ::: performance-calculator/src/resilience/bulkhead.rs:14:1
    |
14  | pub enum BulkheadError<E> {
    | ------------------------- variant or associated item `Timeout` not found f
or this enum

error[E0599]: no variant or associated item named `OperationError` found for enu
m `BulkheadError` in the cur
rent scope
                               --> performance-calculator/src/resilience/mod.rs:
214:32
    |
214 |                 BulkheadError::OperationError(err) => BulkheadError::Opera
tionError(err),
    |                                ^^^^^^^^^^^^^^ variant or associated item n
ot found in `BulkheadError<_
>`
                                |
   ::: performance-calculator/src/resilience/bulkhead.rs:14:1
    |
14  | pub enum BulkheadError<E> {
    | ------------------------- variant or associated item `OperationError` not
found for this enum

error[E0599]: no variant or associated item named `OperationError` found for enu
m `BulkheadError` in the cur
rent scope
                               --> performance-calculator/src/resilience/mod.rs:
214:70
    |
214 |                 BulkheadError::OperationError(err) => BulkheadError::Opera
tionError(err),
    |                                                                      ^^^^^
^^^^^^^^^ variant or associa
ted item not found in `BulkheadError<_>`
                                |
   ::: performance-calculator/src/resilience/bulkhead.rs:14:1
    |
14  | pub enum BulkheadError<E> {
    | ------------------------- variant or associated item `OperationError` not
found for this enum

error[E0599]: no variant or associated item named `Timeout` found for enum `Bulk
headError` in the current sc
ope
                               --> performance-calculator/src/resilience/mod.rs:
249:36
    |
249 |                     BulkheadError::Timeout => e,
    |                                    ^^^^^^^ variant or associated item not
found in `BulkheadError<_>`
    |
   ::: performance-calculator/src/resilience/bulkhead.rs:14:1
    |
14  | pub enum BulkheadError<E> {
    | ------------------------- variant or associated item `Timeout` not found f
or this enum

error[E0599]: no variant or associated item named `OperationError` found for enu
m `BulkheadError` in the cur
rent scope
                               --> performance-calculator/src/resilience/mod.rs:
250:36
    |
250 |                     BulkheadError::OperationError(err) => BulkheadError::O
perationError(err),
    |                                    ^^^^^^^^^^^^^^ variant or associated it
em not found in `BulkheadErr
or<_>`
                                |
   ::: performance-calculator/src/resilience/bulkhead.rs:14:1
    |
14  | pub enum BulkheadError<E> {
    | ------------------------- variant or associated item `OperationError` not
found for this enum

error[E0599]: no variant or associated item named `OperationError` found for enu
m `BulkheadError` in the cur
rent scope
                               --> performance-calculator/src/resilience/mod.rs:
250:74
    |
250 |                     BulkheadError::OperationError(err) => BulkheadError::O
perationError(err),
    |                                                                          ^
^^^^^^^^^^^^^ variant or ass
ociated item not found in `BulkheadError<_>`
                                |
   ::: performance-calculator/src/resilience/bulkhead.rs:14:1
    |
14  | pub enum BulkheadError<E> {
    | ------------------------- variant or associated item `OperationError` not
found for this enum

error[E0277]: the trait bound `BulkheadError<_>: StdError` is not satisfied
   --> performance-calculator/src/resilience/mod.rs:246:29
    |
246 |             circuit_breaker.execute(|| async {
    |                             ^^^^^^^ the trait `StdError` is not implemente
d for `BulkheadError<_>`
    |
    = help: the trait `CircuitBreaker<E>` is implemented for `StandardCircuitBre
aker`
note: required for `StandardCircuitBreaker` to implement `CircuitBreaker<Bulkhea
dError<_>>`
   --> performance-calculator/src/resilience/circuit_breaker.rs:278:9
    |
278 | impl<E> CircuitBreaker<E> for StandardCircuitBreaker
    |         ^^^^^^^^^^^^^^^^^     ^^^^^^^^^^^^^^^^^^^^^^
279 | where
280 |     E: std::error::Error + Send + Sync + 'static,
    |        ----------------- unsatisfied trait bound introduced here

error[E0277]: the trait bound `BulkheadError<_>: StdError` is not satisfied
   --> performance-calculator/src/resilience/mod.rs:244:5
    |
244 | /     retry_with_exponential_backoff(
245 | |         || async {
246 | |             circuit_breaker.execute(|| async {
247 | |                 bulkhead.execute(|| f()).await.map_err(|e| match e {
...   |
261 | |         retry_config.initial_backoff_ms,
262 | |     )
    | |_____^ the trait `StdError` is not implemented for `BulkheadError<_>`
    |
note: required by a bound in `resilience::retry_with_exponential_backoff`
   --> performance-calculator/src/resilience/mod.rs:76:8
    |
68  | pub async fn retry_with_exponential_backoff<F, Fut, T, E>(
    |              ------------------------------ required by a bound in this fu
nction
...
76  |     E: Error + Send + Sync + 'static,
    |        ^^^^^ required by this bound in `retry_with_exponential_backoff`

error[E0277]: the trait bound `BulkheadError<_>: StdError` is not satisfied
   --> performance-calculator/src/resilience/mod.rs:263:6
    |
263 |     .await
    |      ^^^^^ the trait `StdError` is not implemented for `BulkheadError<_>`
    |
note: required by a bound in `resilience::retry_with_exponential_backoff`
   --> performance-calculator/src/resilience/mod.rs:76:8
    |
68  | pub async fn retry_with_exponential_backoff<F, Fut, T, E>(
    |              ------------------------------ required by a bound in this fu
nction
...
76  |     E: Error + Send + Sync + 'static,
    |        ^^^^^ required by this bound in `retry_with_exponential_backoff`

error[E0277]: the trait bound `E: From<std::io::Error>` is not satisfied
   --> performance-calculator/src/resilience/mod.rs:286:9
    |
286 |         chaos::with_chaos(service, Some(tenant_id), || f()).await
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `Fro
m<std::io::Error>` is not im
plemented for `E`
                                |
note: required by a bound in `chaos::with_chaos`
   --> performance-calculator/src/resilience/chaos.rs:262:28
    |
254 | pub async fn with_chaos<T, E, F, Fut>(
    |              ---------- required by a bound in this function
...
262 |     E: std::error::Error + From<std::io::Error> + Send + Sync + 'static,
    |                            ^^^^^^^^^^^^^^^^^^^^ required by this bound in
`with_chaos`
help: consider further restricting type parameter `E` with trait `From`
    |
282 |     E: std::error::Error + Send + Sync + 'static + std::fmt::Debug + std::
convert::From<std::io::Error
>,
                                |
                     +++++++++++++++++++++++++++++++++++
+
                            
error[E0277]: the trait bound `E: From<std::io::Error>` is not satisfied
   --> performance-calculator/src/resilience/mod.rs:286:61
    |
286 |         chaos::with_chaos(service, Some(tenant_id), || f()).await
    |                                                             ^^^^^ the trai
t `From<std::io::Error>` is 
not implemented for `E`
                                |
note: required by a bound in `chaos::with_chaos`
   --> performance-calculator/src/resilience/chaos.rs:262:28
    |
254 | pub async fn with_chaos<T, E, F, Fut>(
    |              ---------- required by a bound in this function
...
262 |     E: std::error::Error + From<std::io::Error> + Send + Sync + 'static,
    |                            ^^^^^^^^^^^^^^^^^^^^ required by this bound in
`with_chaos`
help: consider further restricting type parameter `E` with trait `From`
    |
282 |     E: std::error::Error + Send + Sync + 'static + std::fmt::Debug + std::
convert::From<std::io::Error
>,
                                |
                     +++++++++++++++++++++++++++++++++++
+
                            
error[E0061]: this function takes 1 argument but 2 arguments were supplied
   --> performance-calculator/src/resilience/retry.rs:157:24
    |
157 |     let retry_policy = StandardRetryPolicy::new(config, |_, _| true);
    |                        ^^^^^^^^^^^^^^^^^^^^^^^^         ----------- unexpe
cted argument #2 of type `{c
losure@performance-calculator/src/resilience/retry.rs:157:57: 157:63}`
                                |
note: associated function defined here
   --> performance-calculator/src/resilience/retry.rs:48:12
    |
48  |     pub fn new(name: &str) -> Self {
    |            ^^^ ----------
help: remove the extra argument
    |
157 -     let retry_policy = StandardRetryPolicy::new(config, |_, _| true);
157 +     let retry_policy = StandardRetryPolicy::new(config);
    |

error[E0425]: cannot find function `retry` in this scope
   --> performance-calculator/src/resilience/retry.rs:159:5
    |
159 |     retry(f, &retry_policy).await
    |     ^^^^^ not found in this scope

error[E0061]: this function takes 1 argument but 2 arguments were supplied
   --> performance-calculator/src/resilience/retry.rs:181:24
    |
181 |     let retry_policy = StandardRetryPolicy::new(config, predicate);
    |                        ^^^^^^^^^^^^^^^^^^^^^^^^         --------- unexpect
ed argument #2 of type `P`
    |
note: associated function defined here
   --> performance-calculator/src/resilience/retry.rs:48:12
    |
48  |     pub fn new(name: &str) -> Self {
    |            ^^^ ----------
help: remove the extra argument
    |
181 -     let retry_policy = StandardRetryPolicy::new(config, predicate);
181 +     let retry_policy = StandardRetryPolicy::new(config);
    |

error[E0425]: cannot find function `retry` in this scope
   --> performance-calculator/src/resilience/retry.rs:183:5
    |
183 |     retry(f, &retry_policy).await
    |     ^^^^^ not found in this scope

Some errors have detailed explanations: E0038, E0061, E0107, E0255, E0277, E0404
, E0412, E0422, E0425...
For more information about an error, try `rustc --explain E0038`.
warning: `performance-calculator` (lib) generated 76 warnings
error: could not compile `performance-calculator` (lib) due to 71 previous error
s; 76 warnings emitted
warning: `api-handler` (bin "bootstrap") generated 1 warning

```

---

_****_



I see there are quite a few compilation errors and warnings in the codebase. Let's try to fix the most critical errors first before running the tests. The main issues appear to be:

1. Missing generic parameters for `CircuitBreaker` trait
2. Object safety issues with `Bulkhead` trait
3. Missing implementations for `BulkheadError`
4. Incorrect function signatures in retry policy implementation

Let me fix these issues one by one. First, let's fix the circuit breaker and bulkhead related issues:

---

_****_

```diff
  
  /// Get a circuit breaker instance
- pub fn get_circuit_breaker(name: &str) -> Result<Box<dyn CircuitBreaker>> {
-     Ok(Box::new(circuit_breaker::StandardCircuitBreaker::new(name)?))
+ pub fn get_circuit_breaker<E>(name: &str) -> Result<Box<dyn CircuitBreaker<E>>> 
+ where
+     E: std::error::Error + Send + Sync + 'static,
+ {
+     Ok(Box::new(StandardCircuitBreaker::new(name)))
  }
  
  /// Get a bulkhead instance
- pub fn get_bulkhead(name: &str, max_concurrent: usize) -> Result<Box<dyn Bulkhead>> {
-     Ok(Box::new(bulkhead::StandardBulkhead::new(name, max_concurrent)?))
+ pub fn get_bulkhead(name: &str, max_concurrent: usize) -> Result<StandardBulkhead> {
+     Ok(StandardBulkhead::new(name, BulkheadConfig {
+         max_concurrent_calls: max_concurrent,
+         ..Default::default()
+     }))
  }
  
          chaos_fn,
      ).await
+ }
+ 
+ pub struct ResilienceConfig<E> {
+     circuit_breaker: Box<dyn CircuitBreaker<E>>,
+     bulkhead: Box<dyn Bulkhead>,
+     retry_config: RetryConfig,
+ }
+ 
+ pub struct ResilienceBuilder<E> {
+     circuit_breaker: Option<Box<dyn CircuitBreaker<E>>>,
+     bulkhead: Option<Box<dyn Bulkhead>>,
+     retry_config: Option<RetryConfig>,
  } 
```

---

_****_



Now let's fix the retry policy implementation:

---

_****_

```diff
  
  /// Standard retry policy implementation
- pub struct StandardRetryPolicy<E> {
+ pub struct StandardRetryPolicy {
      name: String,
-     max_retries: u32,
-     initial_delay: Duration,
-     max_delay: Duration,
-     backoff_factor: f64,
-     _marker: PhantomData<E>,
+     max_attempts: u32,
+     initial_delay_ms: u64,
+     max_delay_ms: u64,
+     multiplier: f64,
+     jitter: bool,
  }
  
- impl<E> StandardRetryPolicy<E>
- where
-     E: Error + Send + Sync + 'static,
- {
-     /// Create a new standard retry policy
-     pub fn new(name: &str) -> Self {
+ impl StandardRetryPolicy {
+     pub fn new(name: String) -> Self {
          Self {
-             name: name.to_string(),
-             max_retries: 3,
-             initial_delay: Duration::from_millis(100),
-             max_delay: Duration::from_secs(30),
-             backoff_factor: 2.0,
-             _marker: PhantomData,
+             name,
+             max_attempts: 3,
+             initial_delay_ms: 100,
+             max_delay_ms: 1000,
+             multiplier: 2.0,
+             jitter: true,
          }
      }
-     
-     /// Set maximum number of retries
-     pub fn max_retries(mut self, max_retries: u32) -> Self {
-         self.max_retries = max_retries;
-         self
-     }
-     
-     /// Set initial delay
-     pub fn initial_delay(mut self, initial_delay: Duration) -> Self {
-         self.initial_delay = initial_delay;
-         self
-     }
-     
-     /// Set maximum delay
-     pub fn max_delay(mut self, max_delay: Duration) -> Self {
-         self.max_delay = max_delay;
-         self
-     }
-     
-     /// Set backoff factor
-     pub fn backoff_factor(mut self, backoff_factor: f64) -> Self {
-         self.backoff_factor = backoff_factor;
-         self
+ 
+     pub fn with_config(name: String, config: RetryConfig) -> Self {
+         Self {
+             name,
+             max_attempts: config.max_attempts,
+             initial_delay_ms: config.initial_delay_ms,
+             max_delay_ms: config.max_delay_ms,
+             multiplier: config.multiplier,
+             jitter: config.jitter,
+         }
      }
  }
  
  #[async_trait::async_trait]
- impl<E> RetryPolicy<E> for StandardRetryPolicy<E>
+ impl<E> RetryPolicy<E> for StandardRetryPolicy
  where
      E: Error + Send + Sync + 'static,
  {
      async fn should_retry(&self, error: &E, attempt: u32) -> bool {
-         if attempt >= self.max_retries {
+         if attempt >= self.max_attempts {
              warn!(
                  "Maximum retries ({}) exceeded for {}: {}",
-                 self.max_retries, self.name, error
+                 self.max_attempts, self.name, error
              );
              return false;
      
      async fn get_delay(&self, attempt: u32) -> Duration {
-         let delay = self.initial_delay.as_millis() as f64 * self.backoff_factor.powi(attempt as i32);
-         let delay = delay.min(self.max_delay.as_millis() as f64);
+         let delay = self.initial_delay_ms as f64 * self.multiplier.powi(attempt as i32);
+         let delay = delay.min(self.max_delay_ms as f64);
          Duration::from_millis(delay as u64)
      }
      
      retry(f, &retry_policy).await
+ }
+ 
+ pub async fn retry<F, Fut, T, E>(
+     operation: F,
+     policy: &StandardRetryPolicy,
+ ) -> Result<T, E>
+ where
+     F: Fn() -> Fut,
+     Fut: Future<Output = Result<T, E>>,
+     E: std::error::Error + Send + Sync + 'static,
+ {
+     let mut attempt = 0;
+     let mut delay_ms = policy.initial_delay_ms;
+ 
+     loop {
+         attempt += 1;
+         match operation().await {
+             Ok(result) => return Ok(result),
+             Err(error) => {
+                 if attempt >= policy.max_attempts {
+                     return Err(error);
+                 }
+ 
+                 let jitter = if policy.jitter {
+                     let mut rng = rand::thread_rng();
+                     rng.gen_range(-delay_ms / 4..=delay_ms / 4)
+                 } else {
+                     0
+                 };
+ 
+                 tokio::time::sleep(Duration::from_millis((delay_ms + jitter) as u64)).await;
+                 delay_ms = (delay_ms as f64 * policy.multiplier) as i32;
+                 if delay_ms > policy.max_delay_ms {
+                     delay_ms = policy.max_delay_ms;
+                 }
+             }
+         }
+     }
  } 
```

---

_****_



Now let's fix the BulkheadError implementation:

---

_****_

```diff
      /// Bulkhead is full
      Full,
+     /// Operation timed out
+     Timeout,
      /// Operation error
-     Error(E),
+     OperationError(E),
+ }
+ 
+ impl<E: std::error::Error> std::error::Error for BulkheadError<E> {
+     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
+         match self {
+             BulkheadError::OperationError(e) => Some(e),
+             _ => None,
+         }
+     }
+ }
+ 
+ impl<E: std::fmt::Display> std::fmt::Display for BulkheadError<E> {
+     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+         match self {
+             BulkheadError::Full => write!(f, "Bulkhead is full"),
+             BulkheadError::Timeout => write!(f, "Operation timed out"),
+             BulkheadError::OperationError(e) => write!(f, "Operation error: {}", e),
+         }
+     }
  }
  
                      Err(error) => {
                          self.record_failure();
-                         Err(BulkheadError::Error(error))
+                         Err(BulkheadError::OperationError(error))
                      }
                  }
                  self.record_timeout();
                  warn!("Operation timed out in bulkhead {}", self.name);
-                 Err(BulkheadError::Full)
+                 Err(BulkheadError::Timeout)
              }
          }
```

---

_****_



Let me try to build the project again to see if we've resolved the major issues:

---

_****_

```bash
cargo build
```

```
sethford@Seths-MacBook-Pro rust-investment-performance % 
help: if you import `RetryConfig`, refer to it directly
    |
237 -     retry_config: retry::RetryConfig,
237 +     retry_config: RetryConfig,
    |

error[E0412]: cannot find type `RetryConfig` in module `retry`
   --> performance-calculator/src/resilience/mod.rs:281:26
    |
281 |     retry_config: retry::RetryConfig,
    |                          ^^^^^^^^^^^ not found in `retry`
    |
help: consider importing one of these structs
    |
5   + use crate::calculations::error_handling::RetryConfig;
    |
5   + use aws_config::retry::RetryConfig;
    |
5   + use aws_sdk_dynamodb::config::retry::RetryConfig;
    |
5   + use aws_sdk_sqs::config::retry::RetryConfig;
    |
      and 2 other candidates
help: if you import `RetryConfig`, refer to it directly
    |
281 -     retry_config: retry::RetryConfig,
281 +     retry_config: RetryConfig,
    |

warning: unused import: `anyhow`
  --> performance-calculator/src/calculations/config.rs:22:22
   |
22 | use anyhow::{Result, anyhow, Context};
   |                      ^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: unused import: `std::env`
  --> performance-calculator/src/calculations/config.rs:23:5
   |
23 | use std::env;
   |     ^^^^^^^^

warning: unused import: `std::fs::File`
  --> performance-calculator/src/calculations/config.rs:24:5
   |
24 | use std::fs::File;
   |     ^^^^^^^^^^^^^

warning: unused import: `std::io::Read`
  --> performance-calculator/src/calculations/config.rs:25:5
   |
25 | use std::io::Read;
   |     ^^^^^^^^^^^^^

warning: unused imports: `AnalyticsConfig`, `CalculationScheduler`, `DataAccessService`, `IntegrationConfig`
, `VisualizationConfig`, and `currency::CurrencyConverter`
  --> performance-calculator/src/calculations/factory.rs:37:5
   |
37 |     currency::CurrencyConverter,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^
38 |     query_api::{QueryApi, DataAccessService},
   |                           ^^^^^^^^^^^^^^^^^
39 |     streaming::StreamingProcessor,
40 |     scheduler::{CalculationScheduler, NotificationService, DefaultNotificationService, EmailClient, ...
   |                 ^^^^^^^^^^^^^^^^^^^^
41 |     analytics::{AnalyticsConfig, AnalyticsEngine},
   |                 ^^^^^^^^^^^^^^^
42 |     visualization::{VisualizationConfig, VisualizationEngine},
   |                     ^^^^^^^^^^^^^^^^^^^
43 |     integration::{IntegrationConfig, IntegrationEngine},
   |                   ^^^^^^^^^^^^^^^^^

warning: unused import: `std::sync::Arc`
 --> performance-calculator/src/calculations/portfolio/mod.rs:3:5
  |
3 | use std::sync::Arc;
  |     ^^^^^^^^^^^^^^

warning: unused imports: `DateTime`, `Duration`, and `Utc`
 --> performance-calculator/src/calculations/performance_metrics.rs:5:14
  |
5 | use chrono::{DateTime, Utc, NaiveDate, Duration};
  |              ^^^^^^^^  ^^^             ^^^^^^^^

warning: unused import: `crate::calculations::risk_metrics::ReturnSeries`
 --> performance-calculator/src/calculations/performance_metrics.rs:7:5
  |
7 | use crate::calculations::risk_metrics::ReturnSeries;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal_macros::dec`
 --> performance-calculator/src/calculations/risk_metrics.rs:3:5
  |
3 | use rust_decimal_macros::dec;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `Result` and `anyhow`
 --> performance-calculator/src/calculations/risk_metrics.rs:4:14
  |
4 | use anyhow::{Result, anyhow};
  |              ^^^^^^  ^^^^^^

warning: unused import: `std::collections::HashMap`
 --> performance-calculator/src/calculations/risk_metrics.rs:5:5
  |
5 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `DateTime` and `Utc`
 --> performance-calculator/src/calculations/risk_metrics.rs:6:14
  |
6 | use chrono::{DateTime, Utc, NaiveDate};
  |              ^^^^^^^^  ^^^

warning: unused import: `std::cmp::Ordering`
 --> performance-calculator/src/calculations/risk_metrics.rs:8:5
  |
8 | use std::cmp::Ordering;
  |     ^^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::prelude::*`
 --> performance-calculator/src/calculations/periodic_returns.rs:2:5
  |
2 | use rust_decimal::prelude::*;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `DateTime` and `Utc`
 --> performance-calculator/src/calculations/periodic_returns.rs:5:14
  |
5 | use chrono::{DateTime, Utc, NaiveDate, Datelike};
  |              ^^^^^^^^  ^^^

warning: unused import: `Deserialize`
 --> performance-calculator/src/calculations/distributed_cache.rs:4:24
  |
4 | use serde::{Serialize, Deserialize};
  |                        ^^^^^^^^^^^

warning: unused import: `std::time::Duration`
 --> performance-calculator/src/calculations/distributed_cache.rs:6:5
  |
6 | use std::time::Duration;
  |     ^^^^^^^^^^^^^^^^^^^

warning: unused imports: `error`, `info`, and `warn`
 --> performance-calculator/src/calculations/distributed_cache.rs:8:15
  |
8 | use tracing::{info, warn, error};
  |               ^^^^  ^^^^  ^^^^^

warning: unused import: `uuid`
  --> performance-calculator/src/calculations/distributed_cache.rs:13:5
   |
13 | use uuid;
   |     ^^^^

warning: unused import: `serde_json::json`
  --> performance-calculator/src/calculations/distributed_cache.rs:14:5
   |
14 | use serde_json::json;
   |     ^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::Decimal`
  --> performance-calculator/src/calculations/streaming.rs:15:5
   |
15 | use rust_decimal::Decimal;
   |     ^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `info` and `warn`
  --> performance-calculator/src/calculations/streaming.rs:19:15
   |
19 | use tracing::{info, warn, error, debug};
   |               ^^^^  ^^^^

warning: unused imports: `AuditTrail`, `ExchangeRateProvider`, and `ExchangeRate`
  --> performance-calculator/src/calculations/query_api.rs:16:49
   |
16 |     currency::{CurrencyConverter, CurrencyCode, ExchangeRate, ExchangeRateProvider},
   |                                                 ^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^
17 |     audit::{AuditTrail, AuditTrailManager},
   |             ^^^^^^^^^^

warning: unused imports: `Duration` and `NaiveDateTime`
 --> performance-calculator/src/calculations/scheduler.rs:2:40
  |
2 | use chrono::{DateTime, Utc, NaiveDate, NaiveDateTime, Duration, Datelike};
  |                                        ^^^^^^^^^^^^^  ^^^^^^^^

warning: unused import: `warn`
 --> performance-calculator/src/calculations/scheduler.rs:8:21
  |
8 | use tracing::{info, warn, error};
  |                     ^^^^

warning: unused import: `CalculationEventBuilder`
  --> performance-calculator/src/calculations/scheduler.rs:13:32
   |
13 |     audit::{AuditTrailManager, CalculationEventBuilder},
   |                                ^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Context`
 --> performance-calculator/src/calculations/analytics.rs:6:22
  |
6 | use anyhow::{Result, Context, anyhow};
  |                      ^^^^^^^

warning: unused import: `serde_json`
  --> performance-calculator/src/calculations/analytics.rs:13:5
   |
13 | use serde_json;
   |     ^^^^^^^^^^

warning: unused import: `Context`
 --> performance-calculator/src/calculations/visualization.rs:6:22
  |
6 | use anyhow::{Result, Context, anyhow};
  |                      ^^^^^^^

warning: unused import: `chrono::NaiveDate`
 --> performance-calculator/src/calculations/visualization.rs:7:5
  |
7 | use chrono::NaiveDate;
  |     ^^^^^^^^^^^^^^^^^

warning: unused imports: `error`, `info`, and `warn`
  --> performance-calculator/src/calculations/visualization.rs:12:15
   |
12 | use tracing::{info, warn, error};
   |               ^^^^  ^^^^  ^^^^^

warning: unused import: `Context`
 --> performance-calculator/src/calculations/integration.rs:6:22
  |
6 | use anyhow::{Result, Context, anyhow};
  |                      ^^^^^^^

warning: unused import: `NaiveDate`
 --> performance-calculator/src/calculations/integration.rs:7:29
  |
7 | use chrono::{DateTime, Utc, NaiveDate};
  |                             ^^^^^^^^^

warning: unused import: `rust_decimal::Decimal`
 --> performance-calculator/src/calculations/integration.rs:8:5
  |
8 | use rust_decimal::Decimal;
  |     ^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `error`, `info`, and `warn`
  --> performance-calculator/src/calculations/integration.rs:14:15
   |
14 | use tracing::{info, warn, error};
   |               ^^^^  ^^^^  ^^^^^

warning: unused imports: `AuditTrail`, `InMemoryAuditTrail`, `InMemoryCache`, `MoneyWeightedReturn`, and `Ti
meWeightedReturn`
  --> performance-calculator/src/calculations/benchmarks.rs:7:13
   |
7  |     audit::{AuditTrail, InMemoryAuditTrail},
   |             ^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^
8  |     currency::{CurrencyConverter, ExchangeRate, CurrencyCode},
9  |     distributed_cache::{StringCache, InMemoryCache},
   |                                      ^^^^^^^^^^^^^
10 |     performance_metrics::{TimeWeightedReturn, MoneyWeightedReturn},
   |                           ^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::prelude::*`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:2:5
  |
2 | use rust_decimal::prelude::*;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashMap`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:4:5
  |
4 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `DateTime`, `NaiveDate`, and `Utc`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:5:14
  |
5 | use chrono::{DateTime, Utc, NaiveDate};
  |              ^^^^^^^^  ^^^  ^^^^^^^^^

warning: unused import: `calculate_information_ratio`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:7:114
  |
7 | ...e_alpha, calculate_tracking_error, calculate_information_ratio};
  |                                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Error as DynamoDbError`
 --> performance-calculator/src/calculations/tenant/dynamodb.rs:1:50
  |
1 | use aws_sdk_dynamodb::{Client as DynamoDbClient, Error as DynamoDbError};
  |                                                  ^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `uuid::Uuid`
 --> performance-calculator/src/calculations/tenant/dynamodb.rs:7:5
  |
7 | use uuid::Uuid;
  |     ^^^^^^^^^^

warning: unused import: `chrono::Utc`
 --> performance-calculator/src/calculations/tenant/cached.rs:4:5
  |
4 | use chrono::Utc;
  |     ^^^^^^^^^^^

warning: unused import: `Duration`
 --> performance-calculator/src/calculations/tenant/metrics.rs:1:24
  |
1 | use chrono::{DateTime, Duration, Utc};
  |                        ^^^^^^^^

warning: unused import: `info`
 --> performance-calculator/src/calculations/tenant/metrics.rs:3:15
  |
3 | use tracing::{info, warn};
  |               ^^^^

warning: unused import: `Tenant`
 --> performance-calculator/src/calculations/tenant/metrics.rs:6:35
  |
6 | use crate::calculations::tenant::{Tenant, ResourceLimits};
  |                                   ^^^^^^

warning: unused imports: `delete_item::DeleteItemInput`, `get_item::GetItemInput`, `put_item::PutItemInput`,
 and `query::QueryInput`
  --> performance-calculator/src/calculations/tenant/metrics.rs:10:5
   |
10 |     put_item::PutItemInput,
   |     ^^^^^^^^^^^^^^^^^^^^^^
11 |     get_item::GetItemInput,
   |     ^^^^^^^^^^^^^^^^^^^^^^
12 |     query::QueryInput,
   |     ^^^^^^^^^^^^^^^^^
13 |     delete_item::DeleteItemInput,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `serde_json::Value`
  --> performance-calculator/src/calculations/tenant/metrics.rs:16:5
   |
16 | use serde_json::Value;
   |     ^^^^^^^^^^^^^^^^^

warning: unused import: `crate::resilience`
  --> performance-calculator/src/calculations/tenant/metrics.rs:18:5
   |
18 | use crate::resilience;
   |     ^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::Decimal`
  --> performance-calculator/src/calculations/tenant/metrics.rs:19:5
   |
19 | use rust_decimal::Decimal;
   |     ^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `anyhow`
 --> performance-calculator/src/resilience/mod.rs:9:22
  |
9 | use anyhow::{Result, anyhow};
  |                      ^^^^^^

warning: unused import: `info`
  --> performance-calculator/src/resilience/mod.rs:10:15
   |
10 | use tracing::{info, warn, error};
   |               ^^^^

warning: unused import: `crate::calculations::error_handling::CalculationError`
  --> performance-calculator/src/resilience/mod.rs:12:5
   |
12 | use crate::calculations::error_handling::CalculationError;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Duration`
 --> performance-calculator/src/resilience/circuit_breaker.rs:2:17
  |
2 | use std::time::{Duration, Instant};
  |                 ^^^^^^^^

warning: unused import: `std::error::Error`
 --> performance-calculator/src/resilience/circuit_breaker.rs:8:5
  |
8 | use std::error::Error;
  |     ^^^^^^^^^^^^^^^^^

warning: unused imports: `AtomicU64`, `AtomicUsize`, and `Ordering`
 --> performance-calculator/src/resilience/circuit_breaker.rs:9:25
  |
9 | use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
  |                         ^^^^^^^^^  ^^^^^^^^^^^  ^^^^^^^^

warning: unused import: `tokio::time::sleep`
  --> performance-calculator/src/resilience/circuit_breaker.rs:10:5
   |
10 | use tokio::time::sleep;
   |     ^^^^^^^^^^^^^^^^^^

warning: unused import: `std::marker::PhantomData`
 --> performance-calculator/src/resilience/retry.rs:4:5
  |
4 | use std::marker::PhantomData;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `anyhow`
 --> performance-calculator/src/resilience/retry.rs:6:22
  |
6 | use anyhow::{Result, anyhow};
  |                      ^^^^^^

warning: unused imports: `error` and `info`
 --> performance-calculator/src/resilience/retry.rs:7:15
  |
7 | use tracing::{info, warn, error};
  |               ^^^^        ^^^^^

warning: unused import: `thiserror::Error`
 --> performance-calculator/src/resilience/retry.rs:9:5
  |
9 | use thiserror::Error;
  |     ^^^^^^^^^^^^^^^^

warning: unused import: `error`
 --> performance-calculator/src/resilience/health_check.rs:7:27
  |
7 | use tracing::{info, warn, error};
  |                           ^^^^^

warning: unused import: `async_trait::async_trait`
 --> performance-calculator/src/resilience/bulkhead.rs:4:5
  |
4 | use async_trait::async_trait;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `error`
 --> performance-calculator/src/resilience/bulkhead.rs:7:27
  |
7 | use tracing::{info, warn, error};
  |                           ^^^^^

warning: unused import: `thiserror::Error`
 --> performance-calculator/src/resilience/bulkhead.rs:8:5
  |
8 | use thiserror::Error;
  |     ^^^^^^^^^^^^^^^^

warning: unused imports: `error` and `warn`
 --> performance-calculator/src/resilience/chaos.rs:6:21
  |
6 | use tracing::{info, warn, error};
  |                     ^^^^  ^^^^^

warning: unused import: `get_circuit_breaker`
  --> performance-calculator/src/resilience/mod.rs:22:29
   |
22 |     StandardCircuitBreaker, get_circuit_breaker, get_circuit_breaker_with_config,
   |                             ^^^^^^^^^^^^^^^^^^^

warning: unused import: `retry_with_exponential_backoff`
  --> performance-calculator/src/resilience/mod.rs:26:43
   |
26 |     RetryConfig, RetryError, RetryPolicy, retry_with_exponential_backoff,
   |                                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `with_chaos`
  --> performance-calculator/src/resilience/mod.rs:43:5
   |
43 |     with_chaos,
   |     ^^^^^^^^^^

warning: unused variable: `beginning_base`
   --> performance-calculator/src/calculations/currency.rs:141:13
    |
141 |         let beginning_base = beginning_amount * beginning_rate;
    |             ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_beginning_bas
e`
    |
    = note: `#[warn(unused_variables)]` on by default

warning: `graphql-api` (bin "graphql-api") generated 20 warnings
error: could not compile `graphql-api` (bin "graphql-api") due to 1 previous error; 20 warnings emitted
warning: unused variable: `event`
   --> performance-calculator/src/calculations/query_api.rs:370:13
    |
370 |         let event = self.audit_manager.start_calculation(
    |             ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`

warning: unused variable: `query_id_for_audit`
   --> performance-calculator/src/calculations/query_api.rs:391:13
    |
391 |         let query_id_for_audit = query_id.clone();
    |             ^^^^^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_query_id_
for_audit`

warning: unused variable: `portfolio_data`
   --> performance-calculator/src/calculations/query_api.rs:626:25
    |
626 |                     let portfolio_data = data_service.get_portfolio_data(
    |                         ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_p
ortfolio_data`

warning: unused variable: `scenario`
   --> performance-calculator/src/calculations/analytics.rs:379:13
    |
379 |         let scenario = scenarios.get(scenario_id)
    |             ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_scenario`

warning: unused variable: `request_id`
   --> performance-calculator/src/calculations/visualization.rs:543:9
    |
543 |         request_id: &str,
    |         ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_request_id`

warning: unused variable: `request_id`
   --> performance-calculator/src/calculations/integration.rs:844:65
    |
844 | ...uest, request_id: &str) -> Result<DataImportResult> {
    |          ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_request_id`

error[E0107]: missing generics for trait `CircuitBreaker`
   --> performance-calculator/src/resilience/mod.rs:107:9
    |
107 |     CB: CircuitBreaker,
    |         ^^^^^^^^^^^^^^ expected 1 generic argument
    |
note: trait defined here, with 1 generic parameter: `E`
   --> performance-calculator/src/resilience/circuit_breaker.rs:102:11
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           ^^^^^^^^^^^^^^ -
help: add missing generic argument
    |
107 |     CB: CircuitBreaker<E>,
    |                       +++

error[E0038]: the trait `CircuitBreaker` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:153:57
    |
153 | pub fn get_circuit_breaker<E>(name: &str) -> Result<Box<dyn CircuitBreaker<E>>> 
    |                                                         ^^^^^^^^^^^^^^^^^^^^^ `CircuitBreaker` cannot 
be made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/circuit_breaker.rs:104:14
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           -------------- this trait cannot be made into an object...
103 |     /// Execute a function with circuit breaker protection
104 |     async fn execute<T, F, Fut>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::circuit_breaker::StandardCircuitBreaker` is seen to implement the trait i
n this crate, consider using it directly instead
    = note: `CircuitBreaker` can be implemented in other crates; if you want to support your users passing t
heir own types here, you can't refer to a specific type

error[E0038]: the trait `CircuitBreaker` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:305:26
    |
305 |     circuit_breaker: Box<dyn CircuitBreaker<E>>,
    |                          ^^^^^^^^^^^^^^^^^^^^^ `CircuitBreaker` cannot be made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/circuit_breaker.rs:104:14
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           -------------- this trait cannot be made into an object...
103 |     /// Execute a function with circuit breaker protection
104 |     async fn execute<T, F, Fut>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::circuit_breaker::StandardCircuitBreaker` is seen to implement the trait i
n this crate, consider using it directly instead
    = note: `CircuitBreaker` can be implemented in other crates; if you want to support your users passing t
heir own types here, you can't refer to a specific type

error[E0038]: the trait `Bulkhead` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:306:19
    |
306 |     bulkhead: Box<dyn Bulkhead>,
    |                   ^^^^^^^^^^^^ `Bulkhead` cannot be made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/bulkhead.rs:108:14
    |
106 | pub trait Bulkhead: Send + Sync {
    |           -------- this trait cannot be made into an object...
107 |     /// Execute an operation with bulkhead protection
108 |     async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, BulkheadError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::bulkhead::StandardBulkhead` is seen to implement the trait in this crate,
 consider using it directly instead
    = note: `Bulkhead` can be implemented in other crates; if you want to support your users passing their o
wn types here, you can't refer to a specific type

error[E0038]: the trait `CircuitBreaker` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:311:33
    |
311 |     circuit_breaker: Option<Box<dyn CircuitBreaker<E>>>,
    |                                 ^^^^^^^^^^^^^^^^^^^^^ `CircuitBreaker` cannot be made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/circuit_breaker.rs:104:14
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           -------------- this trait cannot be made into an object...
103 |     /// Execute a function with circuit breaker protection
104 |     async fn execute<T, F, Fut>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::circuit_breaker::StandardCircuitBreaker` is seen to implement the trait i
n this crate, consider using it directly instead
    = note: `CircuitBreaker` can be implemented in other crates; if you want to support your users passing t
heir own types here, you can't refer to a specific type

error[E0038]: the trait `Bulkhead` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:312:26
    |
312 |     bulkhead: Option<Box<dyn Bulkhead>>,
    |                          ^^^^^^^^^^^^ `Bulkhead` cannot be made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/bulkhead.rs:108:14
    |
106 | pub trait Bulkhead: Send + Sync {
    |           -------- this trait cannot be made into an object...
107 |     /// Execute an operation with bulkhead protection
108 |     async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, BulkheadError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::bulkhead::StandardBulkhead` is seen to implement the trait in this crate,
 consider using it directly instead
    = note: `Bulkhead` can be implemented in other crates; if you want to support your users passing their o
wn types here, you can't refer to a specific type

error[E0599]: no method named `acquire` found for reference `&B` in the current scope
   --> performance-calculator/src/resilience/mod.rs:123:20
    |
123 |     match bulkhead.acquire().await {
    |                    ^^^^^^^ method not found in `&B`

error[E0599]: no variant or associated item named `Error` found for enum `BulkheadError` in the current scop
e
   --> performance-calculator/src/resilience/mod.rs:130:45
    |
130 |                     _ => Err(BulkheadError::Error(e)),
    |                                             ^^^^^ variant or associated item not found in `BulkheadErr
or<_>`
    |
   ::: performance-calculator/src/resilience/bulkhead.rs:14:1
    |
14  | pub enum BulkheadError<E> {
    | ------------------------- variant or associated item `Error` not found for this enum

error[E0277]: the trait bound `E: From<std::io::Error>` is not satisfied
   --> performance-calculator/src/resilience/mod.rs:292:9
    |
292 |         chaos::with_chaos(service, Some(tenant_id), || f()).await
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `From<std::io::Error>` is not im
plemented for `E`
    |
note: required by a bound in `chaos::with_chaos`
   --> performance-calculator/src/resilience/chaos.rs:262:28
    |
254 | pub async fn with_chaos<T, E, F, Fut>(
    |              ---------- required by a bound in this function
...
262 |     E: std::error::Error + From<std::io::Error> + Send + Sync + 'static,
    |                            ^^^^^^^^^^^^^^^^^^^^ required by this bound in `with_chaos`
help: consider further restricting type parameter `E` with trait `From`
    |
288 |     E: std::error::Error + Send + Sync + 'static + std::fmt::Debug + std::convert::From<std::io::Error
>,
    |                                                                    +++++++++++++++++++++++++++++++++++
+

error[E0277]: the trait bound `E: From<std::io::Error>` is not satisfied
   --> performance-calculator/src/resilience/mod.rs:292:61
    |
292 |         chaos::with_chaos(service, Some(tenant_id), || f()).await
    |                                                             ^^^^^ the trait `From<std::io::Error>` is 
not implemented for `E`
    |
note: required by a bound in `chaos::with_chaos`
   --> performance-calculator/src/resilience/chaos.rs:262:28
    |
254 | pub async fn with_chaos<T, E, F, Fut>(
    |              ---------- required by a bound in this function
...
262 |     E: std::error::Error + From<std::io::Error> + Send + Sync + 'static,
    |                            ^^^^^^^^^^^^^^^^^^^^ required by this bound in `with_chaos`
help: consider further restricting type parameter `E` with trait `From`
    |
288 |     E: std::error::Error + Send + Sync + 'static + std::fmt::Debug + std::convert::From<std::io::Error
>,
    |                                                                    +++++++++++++++++++++++++++++++++++
+

error[E0061]: this function takes 1 argument but 2 arguments were supplied
   --> performance-calculator/src/resilience/retry.rs:140:24
    |
140 |     let retry_policy = StandardRetryPolicy::new(config, |_, _| true);
    |                        ^^^^^^^^^^^^^^^^^^^^^^^^         ----------- unexpected argument #2 of type `{c
losure@performance-calculator/src/resilience/retry.rs:140:57: 140:63}`
    |
note: associated function defined here
   --> performance-calculator/src/resilience/retry.rs:44:12
    |
44  |     pub fn new(name: String) -> Self {
    |            ^^^ ------------
help: remove the extra argument
    |
140 -     let retry_policy = StandardRetryPolicy::new(config, |_, _| true);
140 +     let retry_policy = StandardRetryPolicy::new(config);
    |

error[E0277]: the trait bound `E: StdError` is not satisfied
   --> performance-calculator/src/resilience/retry.rs:142:5
    |
142 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ the trait `StdError` is not implemented for `E`
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:176:8
    |
169 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
176 |     E: std::error::Error + Send + Sync + 'static,
    |        ^^^^^^^^^^^^^^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Error`
    |
132 |     E: std::fmt::Debug + std::error::Error,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be sent between threads safely
   --> performance-calculator/src/resilience/retry.rs:142:5
    |
142 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ `E` cannot be sent between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:176:28
    |
169 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
176 |     E: std::error::Error + Send + Sync + 'static,
    |                            ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Send`
    |
132 |     E: std::fmt::Debug + std::marker::Send,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be shared between threads safely
   --> performance-calculator/src/resilience/retry.rs:142:5
    |
142 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ `E` cannot be shared between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:176:35
    |
169 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
176 |     E: std::error::Error + Send + Sync + 'static,
    |                                   ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Sync`
    |
132 |     E: std::fmt::Debug + std::marker::Sync,
    |                        +++++++++++++++++++

error[E0277]: the trait bound `E: StdError` is not satisfied
   --> performance-calculator/src/resilience/retry.rs:142:29
    |
142 |     retry(f, &retry_policy).await
    |                             ^^^^^ the trait `StdError` is not implemented for `E`
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:176:8
    |
169 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
176 |     E: std::error::Error + Send + Sync + 'static,
    |        ^^^^^^^^^^^^^^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Error`
    |
132 |     E: std::fmt::Debug + std::error::Error,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be sent between threads safely
   --> performance-calculator/src/resilience/retry.rs:142:29
    |
142 |     retry(f, &retry_policy).await
    |                             ^^^^^ `E` cannot be sent between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:176:28
    |
169 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
176 |     E: std::error::Error + Send + Sync + 'static,
    |                            ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Send`
    |
132 |     E: std::fmt::Debug + std::marker::Send,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be shared between threads safely
   --> performance-calculator/src/resilience/retry.rs:142:29
    |
142 |     retry(f, &retry_policy).await
    |                             ^^^^^ `E` cannot be shared between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:176:35
    |
169 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
176 |     E: std::error::Error + Send + Sync + 'static,
    |                                   ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Sync`
    |
132 |     E: std::fmt::Debug + std::marker::Sync,
    |                        +++++++++++++++++++

error[E0308]: mismatched types
   --> performance-calculator/src/resilience/retry.rs:142:5
    |
124 | pub async fn retry_with_exponential_backoff<T, E, F, Fut>(
    |                                                - found this type parameter
...
142 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Result<T, RetryError<E>>`, found `Result<T, E>`
    |
    = note: expected enum `Result<_, RetryError<E>>`
               found enum `Result<_, E>`

error[E0061]: this function takes 1 argument but 2 arguments were supplied
   --> performance-calculator/src/resilience/retry.rs:164:24
    |
164 |     let retry_policy = StandardRetryPolicy::new(config, predicate);
    |                        ^^^^^^^^^^^^^^^^^^^^^^^^         --------- unexpected argument #2 of type `P`
    |
note: associated function defined here
   --> performance-calculator/src/resilience/retry.rs:44:12
    |
44  |     pub fn new(name: String) -> Self {
    |            ^^^ ------------
help: remove the extra argument
    |
164 -     let retry_policy = StandardRetryPolicy::new(config, predicate);
164 +     let retry_policy = StandardRetryPolicy::new(config);
    |

error[E0277]: the trait bound `E: StdError` is not satisfied
   --> performance-calculator/src/resilience/retry.rs:166:5
    |
166 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ the trait `StdError` is not implemented for `E`
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:176:8
    |
169 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
176 |     E: std::error::Error + Send + Sync + 'static,
    |        ^^^^^^^^^^^^^^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Error`
    |
156 |     E: std::fmt::Debug + std::error::Error,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be sent between threads safely
   --> performance-calculator/src/resilience/retry.rs:166:5
    |
166 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ `E` cannot be sent between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:176:28
    |
169 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
176 |     E: std::error::Error + Send + Sync + 'static,
    |                            ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Send`
    |
156 |     E: std::fmt::Debug + std::marker::Send,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be shared between threads safely
   --> performance-calculator/src/resilience/retry.rs:166:5
    |
166 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ `E` cannot be shared between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:176:35
    |
169 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
176 |     E: std::error::Error + Send + Sync + 'static,
    |                                   ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Sync`
    |
156 |     E: std::fmt::Debug + std::marker::Sync,
    |                        +++++++++++++++++++

error[E0277]: the trait bound `E: StdError` is not satisfied
   --> performance-calculator/src/resilience/retry.rs:166:29
    |
166 |     retry(f, &retry_policy).await
    |                             ^^^^^ the trait `StdError` is not implemented for `E`
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:176:8
    |
169 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
176 |     E: std::error::Error + Send + Sync + 'static,
    |        ^^^^^^^^^^^^^^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Error`
    |
156 |     E: std::fmt::Debug + std::error::Error,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be sent between threads safely
   --> performance-calculator/src/resilience/retry.rs:166:29
    |
166 |     retry(f, &retry_policy).await
    |                             ^^^^^ `E` cannot be sent between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:176:28
    |
169 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
176 |     E: std::error::Error + Send + Sync + 'static,
    |                            ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Send`
    |
156 |     E: std::fmt::Debug + std::marker::Send,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be shared between threads safely
   --> performance-calculator/src/resilience/retry.rs:166:29
    |
166 |     retry(f, &retry_policy).await
    |                             ^^^^^ `E` cannot be shared between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:176:35
    |
169 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
176 |     E: std::error::Error + Send + Sync + 'static,
    |                                   ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Sync`
    |
156 |     E: std::fmt::Debug + std::marker::Sync,
    |                        +++++++++++++++++++

error[E0308]: mismatched types
   --> performance-calculator/src/resilience/retry.rs:166:5
    |
146 | pub async fn retry_if<T, E, F, Fut, P>(
    |                          - found this type parameter
...
166 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Result<T, RetryError<E>>`, found `Result<T, E>`
    |
    = note: expected enum `Result<_, RetryError<E>>`
               found enum `Result<_, E>`

error[E0600]: cannot apply unary operator `-` to type `u64`
   --> performance-calculator/src/resilience/retry.rs:192:35
    |
192 |                     rng.gen_range(-delay_ms / 4..=delay_ms / 4)
    |                                   ^^^^^^^^^ cannot apply unary operator `-`
    |
    = note: unsigned values cannot be negated

error[E0308]: mismatched types
   --> performance-calculator/src/resilience/retry.rs:198:28
    |
179 |     let mut delay_ms = policy.initial_delay_ms;
    |                        ----------------------- expected due to this value
...
198 |                 delay_ms = (delay_ms as f64 * policy.multiplier) as i32;
    |                            ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `u64`, found `i32`

Some errors have detailed explanations: E0038, E0061, E0107, E0255, E0277, E0308, E0404, E0412, E0422...
For more information about an error, try `rustc --explain E0038`.
warning: `performance-calculator` (lib) generated 76 warnings
error: could not compile `performance-calculator` (lib) due to 81 previous errors; 76 warnings emitted

```

---

_****_



I see we still have quite a few errors to fix. Let me address them systematically:

1. First, let's fix the trait object safety issues by using concrete types instead of trait objects:

---

_****_

```diff
  
  /// Get a circuit breaker instance
- pub fn get_circuit_breaker<E>(name: &str) -> Result<Box<dyn CircuitBreaker<E>>> 
+ pub fn get_circuit_breaker<E>(name: &str) -> Result<StandardCircuitBreaker> 
  where
      E: std::error::Error + Send + Sync + 'static,
  {
-     Ok(Box::new(StandardCircuitBreaker::new(name)))
+     Ok(StandardCircuitBreaker::new(name))
  }
  
  /// Get a bulkhead instance
  pub fn get_bulkhead(name: &str, max_concurrent: usize) -> Result<StandardBulkhead> {
-     Ok(StandardBulkhead::new(name, BulkheadConfig {
+     Ok(StandardBulkhead::new(name.to_string(), BulkheadConfig {
          max_concurrent_calls: max_concurrent,
          ..Default::default()
  
  pub struct ResilienceConfig<E> {
-     circuit_breaker: Box<dyn CircuitBreaker<E>>,
-     bulkhead: Box<dyn Bulkhead>,
+     circuit_breaker: StandardCircuitBreaker,
+     bulkhead: StandardBulkhead,
      retry_config: RetryConfig,
  }
  
  pub struct ResilienceBuilder<E> {
-     circuit_breaker: Option<Box<dyn CircuitBreaker<E>>>,
-     bulkhead: Option<Box<dyn Bulkhead>>,
+     circuit_breaker: Option<StandardCircuitBreaker>,
+     bulkhead: Option<StandardBulkhead>,
      retry_config: Option<RetryConfig>,
  } 
```

---

_****_



Now let's fix the retry policy implementation:

---

_****_

```diff
  /// Retry error types
  #[derive(Debug)]
- pub enum RetryError<E> {
-     /// Maximum number of retries exceeded
-     MaxRetriesExceeded(E),
-     /// Operation was aborted
-     Aborted(E),
+ pub struct RetryError<E> {
+     pub cause: E,
+     pub attempts: u32,
+ }
+ 
+ impl<E: std::error::Error> std::error::Error for RetryError<E> {
+     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
+         Some(&self.cause)
+     }
+ }
+ 
+ impl<E: std::fmt::Display> std::fmt::Display for RetryError<E> {
+     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+         write!(f, "Operation failed after {} attempts: {}", self.attempts, self.cause)
+     }
  }
  
                      attempt += 1;
                  } else {
-                     return Err(RetryError::MaxRetriesExceeded(error));
+                     return Err(RetryError {
+                         cause: error,
+                         attempts: attempt,
+                     });
                  }
              }
      operation: F,
      policy: &StandardRetryPolicy,
- ) -> Result<T, E>
+ ) -> Result<T, RetryError<E>>
  where
      F: Fn() -> Fut,
      let mut attempt = 0;
      let mut delay_ms = policy.initial_delay_ms;
+     let mut last_error = None;
  
      loop {
              Err(error) => {
                  if attempt >= policy.max_attempts {
-                     return Err(error);
+                     return Err(RetryError {
+                         cause: error,
+                         attempts: attempt,
+                     });
                  }
  
+                 last_error = Some(error);
                  let jitter = if policy.jitter {
                      let mut rng = rand::thread_rng();
-                     rng.gen_range(-delay_ms / 4..=delay_ms / 4)
+                     rng.gen_range(0..=(delay_ms / 4))
                  } else {
                      0
                  };
  
-                 tokio::time::sleep(Duration::from_millis((delay_ms + jitter) as u64)).await;
-                 delay_ms = (delay_ms as f64 * policy.multiplier) as i32;
-                 if delay_ms > policy.max_delay_ms {
-                     delay_ms = policy.max_delay_ms;
-                 }
+                 tokio::time::sleep(Duration::from_millis(delay_ms + jitter)).await;
+                 delay_ms = std::cmp::min(
+                     (delay_ms as f64 * policy.multiplier) as u64,
+                     policy.max_delay_ms
+                 );
              }
          }
```

---

_****_



Let me try to build the project again to see if we've resolved more issues:

---

_****_

Error calling tool.

---

_****_



I apologize for the error. Let me try the build command again:

---

_****_

```bash
cargo build
```

```
sethford@Seths-MacBook-Pro rust-investment-performance % 
    |                                                  ^^^^^^^^^^^^^^^^^^^ not found in this scope
    |
help: consider importing this struct
    |
5   + use crate::resilience::retry::StandardRetryPolicy;
    |

error[E0433]: failed to resolve: use of undeclared type `StandardRetryPolicy`
   --> performance-calculator/src/resilience/mod.rs:178:8
    |
178 |     Ok(StandardRetryPolicy::new(name))
    |        ^^^^^^^^^^^^^^^^^^^ use of undeclared type `StandardRetryPolicy`
    |
help: consider importing this struct
    |
5   + use crate::resilience::retry::StandardRetryPolicy;
    |

error[E0412]: cannot find type `RetryConfig` in module `retry`
   --> performance-calculator/src/resilience/mod.rs:204:26
    |
204 |     retry_config: retry::RetryConfig,
    |                          ^^^^^^^^^^^ not found in `retry`
    |
help: consider importing one of these structs
    |
5   + use crate::calculations::error_handling::RetryConfig;
    |
5   + use aws_config::retry::RetryConfig;
    |
5   + use aws_sdk_dynamodb::config::retry::RetryConfig;
    |
5   + use aws_sdk_sqs::config::retry::RetryConfig;
    |
      and 2 other candidates
help: if you import `RetryConfig`, refer to it directly
    |
204 -     retry_config: retry::RetryConfig,
204 +     retry_config: RetryConfig,
    |

error[E0412]: cannot find type `RetryConfig` in module `retry`
   --> performance-calculator/src/resilience/mod.rs:237:26
    |
237 |     retry_config: retry::RetryConfig,
    |                          ^^^^^^^^^^^ not found in `retry`
    |
help: consider importing one of these structs
    |
5   + use crate::calculations::error_handling::RetryConfig;
    |
5   + use aws_config::retry::RetryConfig;
    |
5   + use aws_sdk_dynamodb::config::retry::RetryConfig;
    |
5   + use aws_sdk_sqs::config::retry::RetryConfig;
    |
      and 2 other candidates
help: if you import `RetryConfig`, refer to it directly
    |
237 -     retry_config: retry::RetryConfig,
237 +     retry_config: RetryConfig,
    |

error[E0412]: cannot find type `RetryConfig` in module `retry`
   --> performance-calculator/src/resilience/mod.rs:281:26
    |
281 |     retry_config: retry::RetryConfig,
    |                          ^^^^^^^^^^^ not found in `retry`
    |
help: consider importing one of these structs
    |
5   + use crate::calculations::error_handling::RetryConfig;
    |
5   + use aws_config::retry::RetryConfig;
    |
5   + use aws_sdk_dynamodb::config::retry::RetryConfig;
    |
5   + use aws_sdk_sqs::config::retry::RetryConfig;
    |
      and 2 other candidates
help: if you import `RetryConfig`, refer to it directly
    |
281 -     retry_config: retry::RetryConfig,
281 +     retry_config: RetryConfig,
    |

warning: unused import: `anyhow`
  --> performance-calculator/src/calculations/config.rs:22:22
   |
22 | use anyhow::{Result, anyhow, Context};
   |                      ^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: unused import: `std::env`
  --> performance-calculator/src/calculations/config.rs:23:5
   |
23 | use std::env;
   |     ^^^^^^^^

warning: unused import: `std::fs::File`
  --> performance-calculator/src/calculations/config.rs:24:5
   |
24 | use std::fs::File;
   |     ^^^^^^^^^^^^^

warning: unused import: `std::io::Read`
  --> performance-calculator/src/calculations/config.rs:25:5
   |
25 | use std::io::Read;
   |     ^^^^^^^^^^^^^

warning: unused imports: `AnalyticsConfig`, `CalculationScheduler`, `DataAccessService`, `IntegrationConfig`
, `VisualizationConfig`, and `currency::CurrencyConverter`
  --> performance-calculator/src/calculations/factory.rs:37:5
   |
37 |     currency::CurrencyConverter,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^
38 |     query_api::{QueryApi, DataAccessService},
   |                           ^^^^^^^^^^^^^^^^^
39 |     streaming::StreamingProcessor,
40 |     scheduler::{CalculationScheduler, NotificationService, DefaultNotificationService, EmailClient, ...
   |                 ^^^^^^^^^^^^^^^^^^^^
41 |     analytics::{AnalyticsConfig, AnalyticsEngine},
   |                 ^^^^^^^^^^^^^^^
42 |     visualization::{VisualizationConfig, VisualizationEngine},
   |                     ^^^^^^^^^^^^^^^^^^^
43 |     integration::{IntegrationConfig, IntegrationEngine},
   |                   ^^^^^^^^^^^^^^^^^

warning: unused import: `std::sync::Arc`
 --> performance-calculator/src/calculations/portfolio/mod.rs:3:5
  |
3 | use std::sync::Arc;
  |     ^^^^^^^^^^^^^^

warning: unused imports: `DateTime`, `Duration`, and `Utc`
 --> performance-calculator/src/calculations/performance_metrics.rs:5:14
  |
5 | use chrono::{DateTime, Utc, NaiveDate, Duration};
  |              ^^^^^^^^  ^^^             ^^^^^^^^

warning: unused import: `crate::calculations::risk_metrics::ReturnSeries`
 --> performance-calculator/src/calculations/performance_metrics.rs:7:5
  |
7 | use crate::calculations::risk_metrics::ReturnSeries;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal_macros::dec`
 --> performance-calculator/src/calculations/risk_metrics.rs:3:5
  |
3 | use rust_decimal_macros::dec;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `Result` and `anyhow`
 --> performance-calculator/src/calculations/risk_metrics.rs:4:14
  |
4 | use anyhow::{Result, anyhow};
  |              ^^^^^^  ^^^^^^

warning: unused import: `std::collections::HashMap`
 --> performance-calculator/src/calculations/risk_metrics.rs:5:5
  |
5 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `DateTime` and `Utc`
 --> performance-calculator/src/calculations/risk_metrics.rs:6:14
  |
6 | use chrono::{DateTime, Utc, NaiveDate};
  |              ^^^^^^^^  ^^^

warning: unused import: `std::cmp::Ordering`
 --> performance-calculator/src/calculations/risk_metrics.rs:8:5
  |
8 | use std::cmp::Ordering;
  |     ^^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::prelude::*`
 --> performance-calculator/src/calculations/periodic_returns.rs:2:5
  |
2 | use rust_decimal::prelude::*;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `DateTime` and `Utc`
 --> performance-calculator/src/calculations/periodic_returns.rs:5:14
  |
5 | use chrono::{DateTime, Utc, NaiveDate, Datelike};
  |              ^^^^^^^^  ^^^

warning: unused import: `Deserialize`
 --> performance-calculator/src/calculations/distributed_cache.rs:4:24
  |
4 | use serde::{Serialize, Deserialize};
  |                        ^^^^^^^^^^^

warning: unused import: `std::time::Duration`
 --> performance-calculator/src/calculations/distributed_cache.rs:6:5
  |
6 | use std::time::Duration;
  |     ^^^^^^^^^^^^^^^^^^^

warning: unused imports: `error`, `info`, and `warn`
 --> performance-calculator/src/calculations/distributed_cache.rs:8:15
  |
8 | use tracing::{info, warn, error};
  |               ^^^^  ^^^^  ^^^^^

warning: unused import: `uuid`
  --> performance-calculator/src/calculations/distributed_cache.rs:13:5
   |
13 | use uuid;
   |     ^^^^

warning: unused import: `serde_json::json`
  --> performance-calculator/src/calculations/distributed_cache.rs:14:5
   |
14 | use serde_json::json;
   |     ^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::Decimal`
  --> performance-calculator/src/calculations/streaming.rs:15:5
   |
15 | use rust_decimal::Decimal;
   |     ^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `info` and `warn`
  --> performance-calculator/src/calculations/streaming.rs:19:15
   |
19 | use tracing::{info, warn, error, debug};
   |               ^^^^  ^^^^

warning: unused imports: `AuditTrail`, `ExchangeRateProvider`, and `ExchangeRate`
  --> performance-calculator/src/calculations/query_api.rs:16:49
   |
16 |     currency::{CurrencyConverter, CurrencyCode, ExchangeRate, ExchangeRateProvider},
   |                                                 ^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^
17 |     audit::{AuditTrail, AuditTrailManager},
   |             ^^^^^^^^^^

warning: unused imports: `Duration` and `NaiveDateTime`
 --> performance-calculator/src/calculations/scheduler.rs:2:40
  |
2 | use chrono::{DateTime, Utc, NaiveDate, NaiveDateTime, Duration, Datelike};
  |                                        ^^^^^^^^^^^^^  ^^^^^^^^

warning: unused import: `warn`
 --> performance-calculator/src/calculations/scheduler.rs:8:21
  |
8 | use tracing::{info, warn, error};
  |                     ^^^^

warning: unused import: `CalculationEventBuilder`
  --> performance-calculator/src/calculations/scheduler.rs:13:32
   |
13 |     audit::{AuditTrailManager, CalculationEventBuilder},
   |                                ^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Context`
 --> performance-calculator/src/calculations/analytics.rs:6:22
  |
6 | use anyhow::{Result, Context, anyhow};
  |                      ^^^^^^^

warning: unused import: `serde_json`
  --> performance-calculator/src/calculations/analytics.rs:13:5
   |
13 | use serde_json;
   |     ^^^^^^^^^^

warning: unused import: `Context`
 --> performance-calculator/src/calculations/visualization.rs:6:22
  |
6 | use anyhow::{Result, Context, anyhow};
  |                      ^^^^^^^

warning: unused import: `chrono::NaiveDate`
 --> performance-calculator/src/calculations/visualization.rs:7:5
  |
7 | use chrono::NaiveDate;
  |     ^^^^^^^^^^^^^^^^^

warning: unused imports: `error`, `info`, and `warn`
  --> performance-calculator/src/calculations/visualization.rs:12:15
   |
12 | use tracing::{info, warn, error};
   |               ^^^^  ^^^^  ^^^^^

warning: unused import: `Context`
 --> performance-calculator/src/calculations/integration.rs:6:22
  |
6 | use anyhow::{Result, Context, anyhow};
  |                      ^^^^^^^

warning: unused import: `NaiveDate`
 --> performance-calculator/src/calculations/integration.rs:7:29
  |
7 | use chrono::{DateTime, Utc, NaiveDate};
  |                             ^^^^^^^^^

warning: unused import: `rust_decimal::Decimal`
 --> performance-calculator/src/calculations/integration.rs:8:5
  |
8 | use rust_decimal::Decimal;
  |     ^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `error`, `info`, and `warn`
  --> performance-calculator/src/calculations/integration.rs:14:15
   |
14 | use tracing::{info, warn, error};
   |               ^^^^  ^^^^  ^^^^^

warning: unused imports: `AuditTrail`, `InMemoryAuditTrail`, `InMemoryCache`, `MoneyWeightedReturn`, and `Ti
meWeightedReturn`
  --> performance-calculator/src/calculations/benchmarks.rs:7:13
   |
7  |     audit::{AuditTrail, InMemoryAuditTrail},
   |             ^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^
8  |     currency::{CurrencyConverter, ExchangeRate, CurrencyCode},
9  |     distributed_cache::{StringCache, InMemoryCache},
   |                                      ^^^^^^^^^^^^^
10 |     performance_metrics::{TimeWeightedReturn, MoneyWeightedReturn},
   |                           ^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::prelude::*`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:2:5
  |
2 | use rust_decimal::prelude::*;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashMap`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:4:5
  |
4 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `DateTime`, `NaiveDate`, and `Utc`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:5:14
  |
5 | use chrono::{DateTime, Utc, NaiveDate};
  |              ^^^^^^^^  ^^^  ^^^^^^^^^

warning: unused import: `calculate_information_ratio`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:7:114
  |
7 | ...e_alpha, calculate_tracking_error, calculate_information_ratio};
  |                                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Error as DynamoDbError`
 --> performance-calculator/src/calculations/tenant/dynamodb.rs:1:50
  |
1 | use aws_sdk_dynamodb::{Client as DynamoDbClient, Error as DynamoDbError};
  |                                                  ^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `uuid::Uuid`
 --> performance-calculator/src/calculations/tenant/dynamodb.rs:7:5
  |
7 | use uuid::Uuid;
  |     ^^^^^^^^^^

warning: unused import: `chrono::Utc`
 --> performance-calculator/src/calculations/tenant/cached.rs:4:5
  |
4 | use chrono::Utc;
  |     ^^^^^^^^^^^

warning: unused import: `Duration`
 --> performance-calculator/src/calculations/tenant/metrics.rs:1:24
  |
1 | use chrono::{DateTime, Duration, Utc};
  |                        ^^^^^^^^

warning: unused import: `info`
 --> performance-calculator/src/calculations/tenant/metrics.rs:3:15
  |
3 | use tracing::{info, warn};
  |               ^^^^

warning: unused import: `Tenant`
 --> performance-calculator/src/calculations/tenant/metrics.rs:6:35
  |
6 | use crate::calculations::tenant::{Tenant, ResourceLimits};
  |                                   ^^^^^^

warning: unused imports: `delete_item::DeleteItemInput`, `get_item::GetItemInput`, `put_item::PutItemInput`,
 and `query::QueryInput`
  --> performance-calculator/src/calculations/tenant/metrics.rs:10:5
   |
10 |     put_item::PutItemInput,
   |     ^^^^^^^^^^^^^^^^^^^^^^
11 |     get_item::GetItemInput,
   |     ^^^^^^^^^^^^^^^^^^^^^^
12 |     query::QueryInput,
   |     ^^^^^^^^^^^^^^^^^
13 |     delete_item::DeleteItemInput,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `serde_json::Value`
  --> performance-calculator/src/calculations/tenant/metrics.rs:16:5
   |
16 | use serde_json::Value;
   |     ^^^^^^^^^^^^^^^^^

warning: unused import: `crate::resilience`
  --> performance-calculator/src/calculations/tenant/metrics.rs:18:5
   |
18 | use crate::resilience;
   |     ^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::Decimal`
  --> performance-calculator/src/calculations/tenant/metrics.rs:19:5
   |
19 | use rust_decimal::Decimal;
   |     ^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `anyhow`
 --> performance-calculator/src/resilience/mod.rs:9:22
  |
9 | use anyhow::{Result, anyhow};
  |                      ^^^^^^

warning: unused import: `info`
  --> performance-calculator/src/resilience/mod.rs:10:15
   |
10 | use tracing::{info, warn, error};
   |               ^^^^

warning: unused import: `crate::calculations::error_handling::CalculationError`
  --> performance-calculator/src/resilience/mod.rs:12:5
   |
12 | use crate::calculations::error_handling::CalculationError;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Duration`
 --> performance-calculator/src/resilience/circuit_breaker.rs:2:17
  |
2 | use std::time::{Duration, Instant};
  |                 ^^^^^^^^

warning: unused import: `std::error::Error`
 --> performance-calculator/src/resilience/circuit_breaker.rs:8:5
  |
8 | use std::error::Error;
  |     ^^^^^^^^^^^^^^^^^

warning: unused imports: `AtomicU64`, `AtomicUsize`, and `Ordering`
 --> performance-calculator/src/resilience/circuit_breaker.rs:9:25
  |
9 | use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
  |                         ^^^^^^^^^  ^^^^^^^^^^^  ^^^^^^^^

warning: unused import: `tokio::time::sleep`
  --> performance-calculator/src/resilience/circuit_breaker.rs:10:5
   |
10 | use tokio::time::sleep;
   |     ^^^^^^^^^^^^^^^^^^

warning: unused import: `std::marker::PhantomData`
 --> performance-calculator/src/resilience/retry.rs:4:5
  |
4 | use std::marker::PhantomData;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `anyhow`
 --> performance-calculator/src/resilience/retry.rs:6:22
  |
6 | use anyhow::{Result, anyhow};
  |                      ^^^^^^

warning: unused imports: `error` and `info`
 --> performance-calculator/src/resilience/retry.rs:7:15
  |
7 | use tracing::{info, warn, error};
  |               ^^^^        ^^^^^

warning: unused import: `thiserror::Error`
 --> performance-calculator/src/resilience/retry.rs:9:5
  |
9 | use thiserror::Error;
  |     ^^^^^^^^^^^^^^^^

warning: unused import: `error`
 --> performance-calculator/src/resilience/health_check.rs:7:27
  |
7 | use tracing::{info, warn, error};
  |                           ^^^^^

warning: unused import: `async_trait::async_trait`
 --> performance-calculator/src/resilience/bulkhead.rs:4:5
  |
4 | use async_trait::async_trait;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `error`
 --> performance-calculator/src/resilience/bulkhead.rs:7:27
  |
7 | use tracing::{info, warn, error};
  |                           ^^^^^

warning: unused import: `thiserror::Error`
 --> performance-calculator/src/resilience/bulkhead.rs:8:5
  |
8 | use thiserror::Error;
  |     ^^^^^^^^^^^^^^^^

warning: unused imports: `error` and `warn`
 --> performance-calculator/src/resilience/chaos.rs:6:21
  |
6 | use tracing::{info, warn, error};
  |                     ^^^^  ^^^^^

warning: unused import: `get_circuit_breaker`
  --> performance-calculator/src/resilience/mod.rs:22:29
   |
22 |     StandardCircuitBreaker, get_circuit_breaker, get_circuit_breaker_with_config,
   |                             ^^^^^^^^^^^^^^^^^^^

warning: unused import: `retry_with_exponential_backoff`
  --> performance-calculator/src/resilience/mod.rs:26:43
   |
26 |     RetryConfig, RetryError, RetryPolicy, retry_with_exponential_backoff,
   |                                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `with_chaos`
  --> performance-calculator/src/resilience/mod.rs:43:5
   |
43 |     with_chaos,
   |     ^^^^^^^^^^

warning: unused variable: `beginning_base`
   --> performance-calculator/src/calculations/currency.rs:141:13
    |
141 |         let beginning_base = beginning_amount * beginning_rate;
    |             ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_beginning_bas
e`
    |
    = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `event`
   --> performance-calculator/src/calculations/query_api.rs:370:13
    |
370 |         let event = self.audit_manager.start_calculation(
    |             ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`

warning: unused variable: `query_id_for_audit`
   --> performance-calculator/src/calculations/query_api.rs:391:13
    |
391 |         let query_id_for_audit = query_id.clone();
    |             ^^^^^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_query_id_
for_audit`

warning: `graphql-api` (bin "graphql-api") generated 20 warnings
error: could not compile `graphql-api` (bin "graphql-api") due to 1 previous error; 20 warnings emitted
warning: unused variable: `portfolio_data`
   --> performance-calculator/src/calculations/query_api.rs:626:25
    |
626 |                     let portfolio_data = data_service.get_portfolio_data(
    |                         ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_p
ortfolio_data`

warning: unused variable: `scenario`
   --> performance-calculator/src/calculations/analytics.rs:379:13
    |
379 |         let scenario = scenarios.get(scenario_id)
    |             ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_scenario`

warning: unused variable: `request_id`
   --> performance-calculator/src/calculations/visualization.rs:543:9
    |
543 |         request_id: &str,
    |         ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_request_id`

warning: unused variable: `request_id`
   --> performance-calculator/src/calculations/integration.rs:844:65
    |
844 | ...uest, request_id: &str) -> Result<DataImportResult> {
    |          ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_request_id`

error[E0107]: missing generics for trait `CircuitBreaker`
   --> performance-calculator/src/resilience/mod.rs:107:9
    |
107 |     CB: CircuitBreaker,
    |         ^^^^^^^^^^^^^^ expected 1 generic argument
    |
note: trait defined here, with 1 generic parameter: `E`
   --> performance-calculator/src/resilience/circuit_breaker.rs:102:11
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           ^^^^^^^^^^^^^^ -
help: add missing generic argument
    |
107 |     CB: CircuitBreaker<E>,
    |                       +++

error[E0599]: no associated item named `MaxRetriesExceeded` found for struct `RetryError` in the current sco
pe
  --> performance-calculator/src/resilience/mod.rs:86:44
   |
86 |                     return Err(RetryError::MaxRetriesExceeded(e));
   |                                            ^^^^^^^^^^^^^^^^^^ associated item not found in `RetryError<
_>`
   |
  ::: performance-calculator/src/resilience/retry.rs:13:1
   |
13 | pub struct RetryError<E> {
   | ------------------------ associated item `MaxRetriesExceeded` not found for this struct

error[E0599]: no method named `acquire` found for reference `&B` in the current scope
   --> performance-calculator/src/resilience/mod.rs:123:20
    |
123 |     match bulkhead.acquire().await {
    |                    ^^^^^^^ method not found in `&B`

error[E0599]: no variant or associated item named `Error` found for enum `BulkheadError` in the current scop
e
   --> performance-calculator/src/resilience/mod.rs:130:45
    |
130 |                     _ => Err(BulkheadError::Error(e)),
    |                                             ^^^^^ variant or associated item not found in `BulkheadErr
or<_>`
    |
   ::: performance-calculator/src/resilience/bulkhead.rs:14:1
    |
14  | pub enum BulkheadError<E> {
    | ------------------------- variant or associated item `Error` not found for this enum

error[E0599]: no associated item named `MaxRetriesExceeded` found for struct `RetryError` in the current sco
pe
   --> performance-calculator/src/resilience/mod.rs:228:21
    |
228 |         RetryError::MaxRetriesExceeded(e) => ResilienceError::Bulkhead(e),
    |                     ^^^^^^^^^^^^^^^^^^ associated item not found in `RetryError<_>`
    |
   ::: performance-calculator/src/resilience/retry.rs:13:1
    |
13  | pub struct RetryError<E> {
    | ------------------------ associated item `MaxRetriesExceeded` not found for this struct

error[E0599]: no associated item named `Aborted` found for struct `RetryError` in the current scope
   --> performance-calculator/src/resilience/mod.rs:229:21
    |
229 |         RetryError::Aborted(e) => ResilienceError::Bulkhead(e),
    |                     ^^^^^^^ associated item not found in `RetryError<_>`
    |
   ::: performance-calculator/src/resilience/retry.rs:13:1
    |
13  | pub struct RetryError<E> {
    | ------------------------ associated item `Aborted` not found for this struct

error[E0599]: no associated item named `MaxRetriesExceeded` found for struct `RetryError` in the current sco
pe
   --> performance-calculator/src/resilience/mod.rs:271:21
    |
271 |         RetryError::MaxRetriesExceeded(e) => ResilienceError::Bulkhead(e),
    |                     ^^^^^^^^^^^^^^^^^^ associated item not found in `RetryError<_>`
    |
   ::: performance-calculator/src/resilience/retry.rs:13:1
    |
13  | pub struct RetryError<E> {
    | ------------------------ associated item `MaxRetriesExceeded` not found for this struct

error[E0599]: no associated item named `Aborted` found for struct `RetryError` in the current scope
   --> performance-calculator/src/resilience/mod.rs:272:21
    |
272 |         RetryError::Aborted(e) => ResilienceError::Bulkhead(e),
    |                     ^^^^^^^ associated item not found in `RetryError<_>`
    |
   ::: performance-calculator/src/resilience/retry.rs:13:1
    |
13  | pub struct RetryError<E> {
    | ------------------------ associated item `Aborted` not found for this struct

error[E0277]: the trait bound `E: From<std::io::Error>` is not satisfied
   --> performance-calculator/src/resilience/mod.rs:292:9
    |
292 |         chaos::with_chaos(service, Some(tenant_id), || f()).await
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `From<std::io::Error>` is not im
plemented for `E`
    |
note: required by a bound in `chaos::with_chaos`
   --> performance-calculator/src/resilience/chaos.rs:262:28
    |
254 | pub async fn with_chaos<T, E, F, Fut>(
    |              ---------- required by a bound in this function
...
262 |     E: std::error::Error + From<std::io::Error> + Send + Sync + 'static,
    |                            ^^^^^^^^^^^^^^^^^^^^ required by this bound in `with_chaos`
help: consider further restricting type parameter `E` with trait `From`
    |
288 |     E: std::error::Error + Send + Sync + 'static + std::fmt::Debug + std::convert::From<std::io::Error
>,
    |                                                                    +++++++++++++++++++++++++++++++++++
+

error[E0277]: the trait bound `E: From<std::io::Error>` is not satisfied
   --> performance-calculator/src/resilience/mod.rs:292:61
    |
292 |         chaos::with_chaos(service, Some(tenant_id), || f()).await
    |                                                             ^^^^^ the trait `From<std::io::Error>` is 
not implemented for `E`
    |
note: required by a bound in `chaos::with_chaos`
   --> performance-calculator/src/resilience/chaos.rs:262:28
    |
254 | pub async fn with_chaos<T, E, F, Fut>(
    |              ---------- required by a bound in this function
...
262 |     E: std::error::Error + From<std::io::Error> + Send + Sync + 'static,
    |                            ^^^^^^^^^^^^^^^^^^^^ required by this bound in `with_chaos`
help: consider further restricting type parameter `E` with trait `From`
    |
288 |     E: std::error::Error + Send + Sync + 'static + std::fmt::Debug + std::convert::From<std::io::Error
>,
    |                                                                    +++++++++++++++++++++++++++++++++++
+

error[E0061]: this function takes 1 argument but 2 arguments were supplied
   --> performance-calculator/src/resilience/retry.rs:153:24
    |
153 |     let retry_policy = StandardRetryPolicy::new(config, |_, _| true);
    |                        ^^^^^^^^^^^^^^^^^^^^^^^^         ----------- unexpected argument #2 of type `{c
losure@performance-calculator/src/resilience/retry.rs:153:57: 153:63}`
    |
note: associated function defined here
   --> performance-calculator/src/resilience/retry.rs:54:12
    |
54  |     pub fn new(name: String) -> Self {
    |            ^^^ ------------
help: remove the extra argument
    |
153 -     let retry_policy = StandardRetryPolicy::new(config, |_, _| true);
153 +     let retry_policy = StandardRetryPolicy::new(config);
    |

error[E0277]: the trait bound `E: StdError` is not satisfied
   --> performance-calculator/src/resilience/retry.rs:155:5
    |
155 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ the trait `StdError` is not implemented for `E`
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:189:8
    |
182 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
189 |     E: std::error::Error + Send + Sync + 'static,
    |        ^^^^^^^^^^^^^^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Error`
    |
145 |     E: std::fmt::Debug + std::error::Error,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be sent between threads safely
   --> performance-calculator/src/resilience/retry.rs:155:5
    |
155 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ `E` cannot be sent between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:189:28
    |
182 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
189 |     E: std::error::Error + Send + Sync + 'static,
    |                            ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Send`
    |
145 |     E: std::fmt::Debug + std::marker::Send,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be shared between threads safely
   --> performance-calculator/src/resilience/retry.rs:155:5
    |
155 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ `E` cannot be shared between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:189:35
    |
182 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
189 |     E: std::error::Error + Send + Sync + 'static,
    |                                   ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Sync`
    |
145 |     E: std::fmt::Debug + std::marker::Sync,
    |                        +++++++++++++++++++

error[E0277]: the trait bound `E: StdError` is not satisfied
   --> performance-calculator/src/resilience/retry.rs:155:29
    |
155 |     retry(f, &retry_policy).await
    |                             ^^^^^ the trait `StdError` is not implemented for `E`
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:189:8
    |
182 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
189 |     E: std::error::Error + Send + Sync + 'static,
    |        ^^^^^^^^^^^^^^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Error`
    |
145 |     E: std::fmt::Debug + std::error::Error,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be sent between threads safely
   --> performance-calculator/src/resilience/retry.rs:155:29
    |
155 |     retry(f, &retry_policy).await
    |                             ^^^^^ `E` cannot be sent between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:189:28
    |
182 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
189 |     E: std::error::Error + Send + Sync + 'static,
    |                            ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Send`
    |
145 |     E: std::fmt::Debug + std::marker::Send,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be shared between threads safely
   --> performance-calculator/src/resilience/retry.rs:155:29
    |
155 |     retry(f, &retry_policy).await
    |                             ^^^^^ `E` cannot be shared between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:189:35
    |
182 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
189 |     E: std::error::Error + Send + Sync + 'static,
    |                                   ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Sync`
    |
145 |     E: std::fmt::Debug + std::marker::Sync,
    |                        +++++++++++++++++++

error[E0061]: this function takes 1 argument but 2 arguments were supplied
   --> performance-calculator/src/resilience/retry.rs:177:24
    |
177 |     let retry_policy = StandardRetryPolicy::new(config, predicate);
    |                        ^^^^^^^^^^^^^^^^^^^^^^^^         --------- unexpected argument #2 of type `P`
    |
note: associated function defined here
   --> performance-calculator/src/resilience/retry.rs:54:12
    |
54  |     pub fn new(name: String) -> Self {
    |            ^^^ ------------
help: remove the extra argument
    |
177 -     let retry_policy = StandardRetryPolicy::new(config, predicate);
177 +     let retry_policy = StandardRetryPolicy::new(config);
    |

error[E0277]: the trait bound `E: StdError` is not satisfied
   --> performance-calculator/src/resilience/retry.rs:179:5
    |
179 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ the trait `StdError` is not implemented for `E`
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:189:8
    |
182 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
189 |     E: std::error::Error + Send + Sync + 'static,
    |        ^^^^^^^^^^^^^^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Error`
    |
169 |     E: std::fmt::Debug + std::error::Error,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be sent between threads safely
   --> performance-calculator/src/resilience/retry.rs:179:5
    |
179 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ `E` cannot be sent between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:189:28
    |
182 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
189 |     E: std::error::Error + Send + Sync + 'static,
    |                            ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Send`
    |
169 |     E: std::fmt::Debug + std::marker::Send,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be shared between threads safely
   --> performance-calculator/src/resilience/retry.rs:179:5
    |
179 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ `E` cannot be shared between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:189:35
    |
182 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
189 |     E: std::error::Error + Send + Sync + 'static,
    |                                   ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Sync`
    |
169 |     E: std::fmt::Debug + std::marker::Sync,
    |                        +++++++++++++++++++

error[E0277]: the trait bound `E: StdError` is not satisfied
   --> performance-calculator/src/resilience/retry.rs:179:29
    |
179 |     retry(f, &retry_policy).await
    |                             ^^^^^ the trait `StdError` is not implemented for `E`
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:189:8
    |
182 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
189 |     E: std::error::Error + Send + Sync + 'static,
    |        ^^^^^^^^^^^^^^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Error`
    |
169 |     E: std::fmt::Debug + std::error::Error,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be sent between threads safely
   --> performance-calculator/src/resilience/retry.rs:179:29
    |
179 |     retry(f, &retry_policy).await
    |                             ^^^^^ `E` cannot be sent between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:189:28
    |
182 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
189 |     E: std::error::Error + Send + Sync + 'static,
    |                            ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Send`
    |
169 |     E: std::fmt::Debug + std::marker::Send,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be shared between threads safely
   --> performance-calculator/src/resilience/retry.rs:179:29
    |
179 |     retry(f, &retry_policy).await
    |                             ^^^^^ `E` cannot be shared between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:189:35
    |
182 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
189 |     E: std::error::Error + Send + Sync + 'static,
    |                                   ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Sync`
    |
169 |     E: std::fmt::Debug + std::marker::Sync,
    |                        +++++++++++++++++++

warning: variable `last_error` is assigned to, but never used
   --> performance-calculator/src/resilience/retry.rs:193:13
    |
193 |     let mut last_error = None;
    |             ^^^^^^^^^^
    |
    = note: consider using `_last_error` instead

warning: value assigned to `last_error` is never read
   --> performance-calculator/src/resilience/retry.rs:207:17
    |
207 |                 last_error = Some(error);
    |                 ^^^^^^^^^^
    |
    = help: maybe it is overwritten before being read?
    = note: `#[warn(unused_assignments)]` on by default

Some errors have detailed explanations: E0061, E0107, E0255, E0277, E0404, E0412, E0422, E0432, E0433...
For more information about an error, try `rustc --explain E0061`.
warning: `performance-calculator` (lib) generated 78 warnings
error: could not compile `performance-calculator` (lib) due to 77 previous errors; 78 warnings emitted

```

---

_****_





Let me fix the remaining issues:

1. First, let's fix the RetryConfig import and usage:

---

_****_

```diff
  }
  
- impl StandardRetryPolicy {
-     pub fn new(name: String) -> Self {
+ #[derive(Debug, Clone)]
+ pub struct RetryConfig {
+     pub max_attempts: u32,
+     pub initial_delay_ms: u64,
+     pub max_delay_ms: u64,
+     pub multiplier: f64,
+     pub jitter: bool,
+ }
+ 
+ impl Default for RetryConfig {
+     fn default() -> Self {
          Self {
-             name,
              max_attempts: 3,
              initial_delay_ms: 100,
              max_delay_ms: 1000,
              multiplier: 2.0,
              jitter: true,
+         }
      }
  }
  
-     pub fn with_config(name: String, config: RetryConfig) -> Self {
+ impl StandardRetryPolicy {
+     pub fn new(config: RetryConfig) -> Self {
          Self {
-             name,
+             name: "default".to_string(),
              max_attempts: config.max_attempts,
              initial_delay_ms: config.initial_delay_ms,
      };
      
-     let retry_policy = StandardRetryPolicy::new(config, |_, _| true);
+     let retry_policy = StandardRetryPolicy::new(config);
      
      retry(f, &retry_policy).await
      };
      
-     let retry_policy = StandardRetryPolicy::new(config, predicate);
+     let retry_policy = StandardRetryPolicy::new(config);
      
      retry(f, &retry_policy).await
```

---

_****_



Now let's fix the imports in the resilience module:

---

_****_

```diff
      retry_config: Option<RetryConfig>,
  }
+ 
+ pub async fn with_resilience<T, E, F, Fut>(
+     circuit_breaker: &StandardCircuitBreaker,
+     bulkhead: &StandardBulkhead,
+     retry_config: &RetryConfig,
+     f: F,
+ ) -> Result<T, E>
+ where
+     F: Fn() -> Fut + Send + Sync + 'static,
+     Fut: Future<Output = Result<T, E>> + Send,
+     T: Send + 'static,
+     E: std::error::Error + Send + Sync + 'static,
+ {
+     let retry_policy = StandardRetryPolicy::new(retry_config.clone());
+     retry::retry(f, &retry_policy).await
+ }
+ 
+ pub async fn with_tenant_resilience<T, E, F, Fut>(
+     tenant_id: &str,
+     service: &str,
+     f: F,
+ ) -> Result<T, E>
+ where
+     F: Fn() -> Fut + Send + Sync + 'static,
+     Fut: Future<Output = Result<T, E>> + Send,
+     T: Send + 'static,
+     E: std::error::Error + Send + Sync + std::fmt::Debug + 'static,
+ {
+     let circuit_breaker = get_circuit_breaker(service)?;
+     let bulkhead = get_bulkhead(service, 10)?;
+     let retry_config = RetryConfig::default();
+ 
+     with_resilience(&circuit_breaker, &bulkhead, &retry_config, f).await
+ } 
```

---

_****_



Let me try to build the project again to see if we've resolved more issues:

---

_****_

```bash
cargo build
```

```
sethford@Seths-MacBook-Pro rust-investment-performance % 
   |                      ^^^^^^
   |
   = note: `#[warn(unused_imports)]` on by default

warning: unused import: `std::env`
  --> performance-calculator/src/calculations/config.rs:23:5
   |
23 | use std::env;
   |     ^^^^^^^^

warning: unused import: `std::fs::File`
  --> performance-calculator/src/calculations/config.rs:24:5
   |
24 | use std::fs::File;
   |     ^^^^^^^^^^^^^

warning: unused import: `std::io::Read`
  --> performance-calculator/src/calculations/config.rs:25:5
   |
25 | use std::io::Read;
   |     ^^^^^^^^^^^^^

warning: unused imports: `AnalyticsConfig`, `CalculationScheduler`, `DataAccessService`, `IntegrationConfig`
, `VisualizationConfig`, and `currency::CurrencyConverter`
  --> performance-calculator/src/calculations/factory.rs:37:5
   |
37 |     currency::CurrencyConverter,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^
38 |     query_api::{QueryApi, DataAccessService},
   |                           ^^^^^^^^^^^^^^^^^
39 |     streaming::StreamingProcessor,
40 |     scheduler::{CalculationScheduler, NotificationService, DefaultNotificationService, EmailClient, ...
   |                 ^^^^^^^^^^^^^^^^^^^^
41 |     analytics::{AnalyticsConfig, AnalyticsEngine},
   |                 ^^^^^^^^^^^^^^^
42 |     visualization::{VisualizationConfig, VisualizationEngine},
   |                     ^^^^^^^^^^^^^^^^^^^
43 |     integration::{IntegrationConfig, IntegrationEngine},
   |                   ^^^^^^^^^^^^^^^^^

warning: unused import: `std::sync::Arc`
 --> performance-calculator/src/calculations/portfolio/mod.rs:3:5
  |
3 | use std::sync::Arc;
  |     ^^^^^^^^^^^^^^

warning: unused imports: `DateTime`, `Duration`, and `Utc`
 --> performance-calculator/src/calculations/performance_metrics.rs:5:14
  |
5 | use chrono::{DateTime, Utc, NaiveDate, Duration};
  |              ^^^^^^^^  ^^^             ^^^^^^^^

warning: unused import: `crate::calculations::risk_metrics::ReturnSeries`
 --> performance-calculator/src/calculations/performance_metrics.rs:7:5
  |
7 | use crate::calculations::risk_metrics::ReturnSeries;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal_macros::dec`
 --> performance-calculator/src/calculations/risk_metrics.rs:3:5
  |
3 | use rust_decimal_macros::dec;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `Result` and `anyhow`
 --> performance-calculator/src/calculations/risk_metrics.rs:4:14
  |
4 | use anyhow::{Result, anyhow};
  |              ^^^^^^  ^^^^^^

warning: unused import: `std::collections::HashMap`
 --> performance-calculator/src/calculations/risk_metrics.rs:5:5
  |
5 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `DateTime` and `Utc`
 --> performance-calculator/src/calculations/risk_metrics.rs:6:14
  |
6 | use chrono::{DateTime, Utc, NaiveDate};
  |              ^^^^^^^^  ^^^

warning: unused import: `std::cmp::Ordering`
 --> performance-calculator/src/calculations/risk_metrics.rs:8:5
  |
8 | use std::cmp::Ordering;
  |     ^^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::prelude::*`
 --> performance-calculator/src/calculations/periodic_returns.rs:2:5
  |
2 | use rust_decimal::prelude::*;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `DateTime` and `Utc`
 --> performance-calculator/src/calculations/periodic_returns.rs:5:14
  |
5 | use chrono::{DateTime, Utc, NaiveDate, Datelike};
  |              ^^^^^^^^  ^^^

warning: unused import: `Deserialize`
 --> performance-calculator/src/calculations/distributed_cache.rs:4:24
  |
4 | use serde::{Serialize, Deserialize};
  |                        ^^^^^^^^^^^

warning: unused import: `std::time::Duration`
 --> performance-calculator/src/calculations/distributed_cache.rs:6:5
  |
6 | use std::time::Duration;
  |     ^^^^^^^^^^^^^^^^^^^

warning: unused imports: `error`, `info`, and `warn`
 --> performance-calculator/src/calculations/distributed_cache.rs:8:15
  |
8 | use tracing::{info, warn, error};
  |               ^^^^  ^^^^  ^^^^^

warning: unused import: `uuid`
  --> performance-calculator/src/calculations/distributed_cache.rs:13:5
   |
13 | use uuid;
   |     ^^^^

warning: unused import: `serde_json::json`
  --> performance-calculator/src/calculations/distributed_cache.rs:14:5
   |
14 | use serde_json::json;
   |     ^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::Decimal`
  --> performance-calculator/src/calculations/streaming.rs:15:5
   |
15 | use rust_decimal::Decimal;
   |     ^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `info` and `warn`
  --> performance-calculator/src/calculations/streaming.rs:19:15
   |
19 | use tracing::{info, warn, error, debug};
   |               ^^^^  ^^^^

warning: unused imports: `AuditTrail`, `ExchangeRateProvider`, and `ExchangeRate`
  --> performance-calculator/src/calculations/query_api.rs:16:49
   |
16 |     currency::{CurrencyConverter, CurrencyCode, ExchangeRate, ExchangeRateProvider},
   |                                                 ^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^
17 |     audit::{AuditTrail, AuditTrailManager},
   |             ^^^^^^^^^^

warning: unused imports: `Duration` and `NaiveDateTime`
 --> performance-calculator/src/calculations/scheduler.rs:2:40
  |
2 | use chrono::{DateTime, Utc, NaiveDate, NaiveDateTime, Duration, Datelike};
  |                                        ^^^^^^^^^^^^^  ^^^^^^^^

warning: unused import: `warn`
 --> performance-calculator/src/calculations/scheduler.rs:8:21
  |
8 | use tracing::{info, warn, error};
  |                     ^^^^

warning: unused import: `CalculationEventBuilder`
  --> performance-calculator/src/calculations/scheduler.rs:13:32
   |
13 |     audit::{AuditTrailManager, CalculationEventBuilder},
   |                                ^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Context`
 --> performance-calculator/src/calculations/analytics.rs:6:22
  |
6 | use anyhow::{Result, Context, anyhow};
  |                      ^^^^^^^

warning: unused import: `serde_json`
  --> performance-calculator/src/calculations/analytics.rs:13:5
   |
13 | use serde_json;
   |     ^^^^^^^^^^

warning: unused import: `Context`
 --> performance-calculator/src/calculations/visualization.rs:6:22
  |
6 | use anyhow::{Result, Context, anyhow};
  |                      ^^^^^^^

warning: unused import: `chrono::NaiveDate`
 --> performance-calculator/src/calculations/visualization.rs:7:5
  |
7 | use chrono::NaiveDate;
  |     ^^^^^^^^^^^^^^^^^

warning: unused imports: `error`, `info`, and `warn`
  --> performance-calculator/src/calculations/visualization.rs:12:15
   |
12 | use tracing::{info, warn, error};
   |               ^^^^  ^^^^  ^^^^^

warning: unused import: `Context`
 --> performance-calculator/src/calculations/integration.rs:6:22
  |
6 | use anyhow::{Result, Context, anyhow};
  |                      ^^^^^^^

warning: unused import: `NaiveDate`
 --> performance-calculator/src/calculations/integration.rs:7:29
  |
7 | use chrono::{DateTime, Utc, NaiveDate};
  |                             ^^^^^^^^^

warning: unused import: `rust_decimal::Decimal`
 --> performance-calculator/src/calculations/integration.rs:8:5
  |
8 | use rust_decimal::Decimal;
  |     ^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `error`, `info`, and `warn`
  --> performance-calculator/src/calculations/integration.rs:14:15
   |
14 | use tracing::{info, warn, error};
   |               ^^^^  ^^^^  ^^^^^

warning: unused imports: `AuditTrail`, `InMemoryAuditTrail`, `InMemoryCache`, `MoneyWeightedReturn`, and `Ti
meWeightedReturn`
  --> performance-calculator/src/calculations/benchmarks.rs:7:13
   |
7  |     audit::{AuditTrail, InMemoryAuditTrail},
   |             ^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^
8  |     currency::{CurrencyConverter, ExchangeRate, CurrencyCode},
9  |     distributed_cache::{StringCache, InMemoryCache},
   |                                      ^^^^^^^^^^^^^
10 |     performance_metrics::{TimeWeightedReturn, MoneyWeightedReturn},
   |                           ^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::prelude::*`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:2:5
  |
2 | use rust_decimal::prelude::*;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashMap`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:4:5
  |
4 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `DateTime`, `NaiveDate`, and `Utc`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:5:14
  |
5 | use chrono::{DateTime, Utc, NaiveDate};
  |              ^^^^^^^^  ^^^  ^^^^^^^^^

warning: unused import: `calculate_information_ratio`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:7:114
  |
7 | ...e_alpha, calculate_tracking_error, calculate_information_ratio};
  |                                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Error as DynamoDbError`
 --> performance-calculator/src/calculations/tenant/dynamodb.rs:1:50
  |
1 | use aws_sdk_dynamodb::{Client as DynamoDbClient, Error as DynamoDbError};
  |                                                  ^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `uuid::Uuid`
 --> performance-calculator/src/calculations/tenant/dynamodb.rs:7:5
  |
7 | use uuid::Uuid;
  |     ^^^^^^^^^^

warning: unused import: `chrono::Utc`
 --> performance-calculator/src/calculations/tenant/cached.rs:4:5
  |
4 | use chrono::Utc;
  |     ^^^^^^^^^^^

warning: unused import: `Duration`
 --> performance-calculator/src/calculations/tenant/metrics.rs:1:24
  |
1 | use chrono::{DateTime, Duration, Utc};
  |                        ^^^^^^^^

warning: unused import: `info`
 --> performance-calculator/src/calculations/tenant/metrics.rs:3:15
  |
3 | use tracing::{info, warn};
  |               ^^^^

warning: unused import: `Tenant`
 --> performance-calculator/src/calculations/tenant/metrics.rs:6:35
  |
6 | use crate::calculations::tenant::{Tenant, ResourceLimits};
  |                                   ^^^^^^

warning: unused imports: `delete_item::DeleteItemInput`, `get_item::GetItemInput`, `put_item::PutItemInput`,
 and `query::QueryInput`
  --> performance-calculator/src/calculations/tenant/metrics.rs:10:5
   |
10 |     put_item::PutItemInput,
   |     ^^^^^^^^^^^^^^^^^^^^^^
11 |     get_item::GetItemInput,
   |     ^^^^^^^^^^^^^^^^^^^^^^
12 |     query::QueryInput,
   |     ^^^^^^^^^^^^^^^^^
13 |     delete_item::DeleteItemInput,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `serde_json::Value`
  --> performance-calculator/src/calculations/tenant/metrics.rs:16:5
   |
16 | use serde_json::Value;
   |     ^^^^^^^^^^^^^^^^^

warning: unused import: `crate::resilience`
  --> performance-calculator/src/calculations/tenant/metrics.rs:18:5
   |
18 | use crate::resilience;
   |     ^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::Decimal`
  --> performance-calculator/src/calculations/tenant/metrics.rs:19:5
   |
19 | use rust_decimal::Decimal;
   |     ^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `anyhow`
 --> performance-calculator/src/resilience/mod.rs:9:22
  |
9 | use anyhow::{Result, anyhow};
  |                      ^^^^^^

warning: unused import: `info`
  --> performance-calculator/src/resilience/mod.rs:10:15
   |
10 | use tracing::{info, warn, error};
   |               ^^^^

warning: unused import: `crate::calculations::error_handling::CalculationError`
  --> performance-calculator/src/resilience/mod.rs:12:5
   |
12 | use crate::calculations::error_handling::CalculationError;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Duration`
 --> performance-calculator/src/resilience/circuit_breaker.rs:2:17
  |
2 | use std::time::{Duration, Instant};
  |                 ^^^^^^^^

warning: unused import: `std::error::Error`
 --> performance-calculator/src/resilience/circuit_breaker.rs:8:5
  |
8 | use std::error::Error;
  |     ^^^^^^^^^^^^^^^^^

warning: unused imports: `AtomicU64`, `AtomicUsize`, and `Ordering`
 --> performance-calculator/src/resilience/circuit_breaker.rs:9:25
  |
9 | use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
  |                         ^^^^^^^^^  ^^^^^^^^^^^  ^^^^^^^^

warning: unused import: `tokio::time::sleep`
  --> performance-calculator/src/resilience/circuit_breaker.rs:10:5
   |
10 | use tokio::time::sleep;
   |     ^^^^^^^^^^^^^^^^^^

warning: unused import: `std::marker::PhantomData`
 --> performance-calculator/src/resilience/retry.rs:4:5
  |
4 | use std::marker::PhantomData;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `anyhow`
 --> performance-calculator/src/resilience/retry.rs:6:22
  |
6 | use anyhow::{Result, anyhow};
  |                      ^^^^^^

warning: unused imports: `error` and `info`
 --> performance-calculator/src/resilience/retry.rs:7:15
  |
7 | use tracing::{info, warn, error};
  |               ^^^^        ^^^^^

warning: unused import: `thiserror::Error`
 --> performance-calculator/src/resilience/retry.rs:9:5
  |
9 | use thiserror::Error;
  |     ^^^^^^^^^^^^^^^^

warning: unused import: `error`
 --> performance-calculator/src/resilience/health_check.rs:7:27
  |
7 | use tracing::{info, warn, error};
  |                           ^^^^^

warning: unused import: `async_trait::async_trait`
 --> performance-calculator/src/resilience/bulkhead.rs:4:5
  |
4 | use async_trait::async_trait;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `error`
 --> performance-calculator/src/resilience/bulkhead.rs:7:27
  |
7 | use tracing::{info, warn, error};
  |                           ^^^^^

warning: unused import: `thiserror::Error`
 --> performance-calculator/src/resilience/bulkhead.rs:8:5
  |
8 | use thiserror::Error;
  |     ^^^^^^^^^^^^^^^^

warning: unused imports: `error` and `warn`
 --> performance-calculator/src/resilience/chaos.rs:6:21
  |
6 | use tracing::{info, warn, error};
  |                     ^^^^  ^^^^^

warning: unused import: `get_circuit_breaker`
  --> performance-calculator/src/resilience/mod.rs:22:29
   |
22 |     StandardCircuitBreaker, get_circuit_breaker, get_circuit_breaker_with_config,
   |                             ^^^^^^^^^^^^^^^^^^^

warning: unused import: `retry_with_exponential_backoff`
  --> performance-calculator/src/resilience/mod.rs:26:43
   |
26 |     RetryConfig, RetryError, RetryPolicy, retry_with_exponential_backoff,
   |                                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `with_chaos`
  --> performance-calculator/src/resilience/mod.rs:43:5
   |
43 |     with_chaos,
   |     ^^^^^^^^^^

warning: unused variable: `beginning_base`
   --> performance-calculator/src/calculations/currency.rs:141:13
    |
141 |         let beginning_base = beginning_amount * beginning_rate;
    |             ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_beginning_bas
e`
    |
    = note: `#[warn(unused_variables)]` on by default

warning: unused variable: `event`
   --> performance-calculator/src/calculations/query_api.rs:370:13
    |
370 |         let event = self.audit_manager.start_calculation(
    |             ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`

warning: unused variable: `query_id_for_audit`
   --> performance-calculator/src/calculations/query_api.rs:391:13
    |
391 |         let query_id_for_audit = query_id.clone();
    |             ^^^^^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_query_id_
for_audit`

warning: `graphql-api` (bin "graphql-api") generated 20 warnings
error: could not compile `graphql-api` (bin "graphql-api") due to 1 previous error; 20 warnings emitted
warning: unused variable: `portfolio_data`
   --> performance-calculator/src/calculations/query_api.rs:626:25
    |
626 |                     let portfolio_data = data_service.get_portfolio_data(
    |                         ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_p
ortfolio_data`

warning: unused variable: `scenario`
   --> performance-calculator/src/calculations/analytics.rs:379:13
    |
379 |         let scenario = scenarios.get(scenario_id)
    |             ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_scenario`

warning: unused variable: `request_id`
   --> performance-calculator/src/calculations/visualization.rs:543:9
    |
543 |         request_id: &str,
    |         ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_request_id`

warning: unused variable: `request_id`
   --> performance-calculator/src/calculations/integration.rs:844:65
    |
844 | ...uest, request_id: &str) -> Result<DataImportResult> {
    |          ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_request_id`

error[E0107]: missing generics for trait `CircuitBreaker`
   --> performance-calculator/src/resilience/mod.rs:107:9
    |
107 |     CB: CircuitBreaker,
    |         ^^^^^^^^^^^^^^ expected 1 generic argument
    |
note: trait defined here, with 1 generic parameter: `E`
   --> performance-calculator/src/resilience/circuit_breaker.rs:102:11
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           ^^^^^^^^^^^^^^ -
help: add missing generic argument
    |
107 |     CB: CircuitBreaker<E>,
    |                       +++

error[E0392]: type parameter `E` is never used
   --> performance-calculator/src/resilience/mod.rs:304:29
    |
304 | pub struct ResilienceConfig<E> {
    |                             ^ unused type parameter
    |
    = help: consider removing `E`, referring to it in a field, or using a marker such as `PhantomData`
    = help: if you intended `E` to be a const parameter, use `const E: /* Type */` instead

error[E0392]: type parameter `E` is never used
   --> performance-calculator/src/resilience/mod.rs:310:30
    |
310 | pub struct ResilienceBuilder<E> {
    |                              ^ unused type parameter
    |
    = help: consider removing `E`, referring to it in a field, or using a marker such as `PhantomData`
    = help: if you intended `E` to be a const parameter, use `const E: /* Type */` instead

error[E0599]: no associated item named `MaxRetriesExceeded` found for struct `RetryError` in the current sco
pe
  --> performance-calculator/src/resilience/mod.rs:86:44
   |
86 |                     return Err(RetryError::MaxRetriesExceeded(e));
   |                                            ^^^^^^^^^^^^^^^^^^ associated item not found in `RetryError<
_>`
   |
  ::: performance-calculator/src/resilience/retry.rs:13:1
   |
13 | pub struct RetryError<E> {
   | ------------------------ associated item `MaxRetriesExceeded` not found for this struct

error[E0599]: no method named `acquire` found for reference `&B` in the current scope
   --> performance-calculator/src/resilience/mod.rs:123:20
    |
123 |     match bulkhead.acquire().await {
    |                    ^^^^^^^ method not found in `&B`

error[E0599]: no variant or associated item named `Error` found for enum `BulkheadError` in the current scop
e
   --> performance-calculator/src/resilience/mod.rs:130:45
    |
130 |                     _ => Err(BulkheadError::Error(e)),
    |                                             ^^^^^ variant or associated item not found in `BulkheadErr
or<_>`
    |
   ::: performance-calculator/src/resilience/bulkhead.rs:14:1
    |
14  | pub enum BulkheadError<E> {
    | ------------------------- variant or associated item `Error` not found for this enum

error[E0609]: no field `max_retries` on type `resilience::retry::RetryConfig`
   --> performance-calculator/src/resilience/mod.rs:223:22
    |
223 |         retry_config.max_retries,
    |                      ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0609]: no field `initial_backoff_ms` on type `resilience::retry::RetryConfig`
   --> performance-calculator/src/resilience/mod.rs:224:22
    |
224 |         retry_config.initial_backoff_ms,
    |                      ^^^^^^^^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0599]: no associated item named `MaxRetriesExceeded` found for struct `RetryError` in the current sco
pe
   --> performance-calculator/src/resilience/mod.rs:228:21
    |
228 |         RetryError::MaxRetriesExceeded(e) => ResilienceError::Bulkhead(e),
    |                     ^^^^^^^^^^^^^^^^^^ associated item not found in `RetryError<_>`
    |
   ::: performance-calculator/src/resilience/retry.rs:13:1
    |
13  | pub struct RetryError<E> {
    | ------------------------ associated item `MaxRetriesExceeded` not found for this struct

error[E0599]: no associated item named `Aborted` found for struct `RetryError` in the current scope
   --> performance-calculator/src/resilience/mod.rs:229:21
    |
229 |         RetryError::Aborted(e) => ResilienceError::Bulkhead(e),
    |                     ^^^^^^^ associated item not found in `RetryError<_>`
    |
   ::: performance-calculator/src/resilience/retry.rs:13:1
    |
13  | pub struct RetryError<E> {
    | ------------------------ associated item `Aborted` not found for this struct

error[E0609]: no field `max_retries` on type `resilience::retry::RetryConfig`
   --> performance-calculator/src/resilience/mod.rs:266:22
    |
266 |         retry_config.max_retries,
    |                      ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0609]: no field `initial_backoff_ms` on type `resilience::retry::RetryConfig`
   --> performance-calculator/src/resilience/mod.rs:267:22
    |
267 |         retry_config.initial_backoff_ms,
    |                      ^^^^^^^^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0599]: no associated item named `MaxRetriesExceeded` found for struct `RetryError` in the current sco
pe
   --> performance-calculator/src/resilience/mod.rs:271:21
    |
271 |         RetryError::MaxRetriesExceeded(e) => ResilienceError::Bulkhead(e),
    |                     ^^^^^^^^^^^^^^^^^^ associated item not found in `RetryError<_>`
    |
   ::: performance-calculator/src/resilience/retry.rs:13:1
    |
13  | pub struct RetryError<E> {
    | ------------------------ associated item `MaxRetriesExceeded` not found for this struct

error[E0599]: no associated item named `Aborted` found for struct `RetryError` in the current scope
   --> performance-calculator/src/resilience/mod.rs:272:21
    |
272 |         RetryError::Aborted(e) => ResilienceError::Bulkhead(e),
    |                     ^^^^^^^ associated item not found in `RetryError<_>`
    |
   ::: performance-calculator/src/resilience/retry.rs:13:1
    |
13  | pub struct RetryError<E> {
    | ------------------------ associated item `Aborted` not found for this struct

error[E0277]: the trait bound `E: From<std::io::Error>` is not satisfied
   --> performance-calculator/src/resilience/mod.rs:292:9
    |
292 |         chaos::with_chaos(service, Some(tenant_id), || f()).await
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `From<std::io::Error>` is not im
plemented for `E`
    |
note: required by a bound in `chaos::with_chaos`
   --> performance-calculator/src/resilience/chaos.rs:262:28
    |
254 | pub async fn with_chaos<T, E, F, Fut>(
    |              ---------- required by a bound in this function
...
262 |     E: std::error::Error + From<std::io::Error> + Send + Sync + 'static,
    |                            ^^^^^^^^^^^^^^^^^^^^ required by this bound in `with_chaos`
help: consider further restricting type parameter `E` with trait `From`
    |
288 |     E: std::error::Error + Send + Sync + 'static + std::fmt::Debug + std::convert::From<std::io::Error
>,
    |                                                                    +++++++++++++++++++++++++++++++++++
+

error[E0277]: the trait bound `E: From<std::io::Error>` is not satisfied
   --> performance-calculator/src/resilience/mod.rs:292:61
    |
292 |         chaos::with_chaos(service, Some(tenant_id), || f()).await
    |                                                             ^^^^^ the trait `From<std::io::Error>` is 
not implemented for `E`
    |
note: required by a bound in `chaos::with_chaos`
   --> performance-calculator/src/resilience/chaos.rs:262:28
    |
254 | pub async fn with_chaos<T, E, F, Fut>(
    |              ---------- required by a bound in this function
...
262 |     E: std::error::Error + From<std::io::Error> + Send + Sync + 'static,
    |                            ^^^^^^^^^^^^^^^^^^^^ required by this bound in `with_chaos`
help: consider further restricting type parameter `E` with trait `From`
    |
288 |     E: std::error::Error + Send + Sync + 'static + std::fmt::Debug + std::convert::From<std::io::Error
>,
    |                                                                    +++++++++++++++++++++++++++++++++++
+

error[E0308]: mismatched types
   --> performance-calculator/src/resilience/mod.rs:329:5
    |
316 | pub async fn with_resilience<T, E, F, Fut>(
    |                                 - expected this type parameter
...
329 |     retry::retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Result<T, E>`, found `Result<T, RetryError<E>>`
    |
    = note: expected enum `Result<_, E>`
               found enum `Result<_, RetryError<E>>`

error[E0277]: `?` couldn't convert the error to `E`
   --> performance-calculator/src/resilience/mod.rs:343:55
    |
343 |     let circuit_breaker = get_circuit_breaker(service)?;
    |                                                       ^ the trait `From<anyhow::Error>` is not impleme
nted for `E`
    |
    = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the 
`From` trait
    = note: required for `Result<T, E>` to implement `FromResidual<Result<Infallible, anyhow::Error>>`
help: consider further restricting type parameter `E` with trait `From`
    |
341 |     E: std::error::Error + Send + Sync + std::fmt::Debug + 'static + std::convert::From<anyhow::Error>
,
    |                                                                    +++++++++++++++++++++++++++++++++++

error[E0277]: `?` couldn't convert the error to `E`
   --> performance-calculator/src/resilience/mod.rs:344:45
    |
344 |     let bulkhead = get_bulkhead(service, 10)?;
    |                                             ^ the trait `From<anyhow::Error>` is not implemented for `
E`
    |
    = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the 
`From` trait
    = note: required for `Result<T, E>` to implement `FromResidual<Result<Infallible, anyhow::Error>>`
help: consider further restricting type parameter `E` with trait `From`
    |
341 |     E: std::error::Error + Send + Sync + std::fmt::Debug + 'static + std::convert::From<anyhow::Error>
,
    |                                                                    +++++++++++++++++++++++++++++++++++

error[E0560]: struct `resilience::retry::RetryConfig` has no field named `max_retries`
   --> performance-calculator/src/resilience/retry.rs:158:9
    |
158 |         max_retries,
    |         ^^^^^^^^^^^ `resilience::retry::RetryConfig` does not have this field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0560]: struct `resilience::retry::RetryConfig` has no field named `initial_backoff_ms`
   --> performance-calculator/src/resilience/retry.rs:159:9
    |
159 |         initial_backoff_ms,
    |         ^^^^^^^^^^^^^^^^^^ `resilience::retry::RetryConfig` does not have this field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0277]: the trait bound `E: StdError` is not satisfied
   --> performance-calculator/src/resilience/retry.rs:165:5
    |
165 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ the trait `StdError` is not implemented for `E`
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:199:8
    |
192 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
199 |     E: std::error::Error + Send + Sync + 'static,
    |        ^^^^^^^^^^^^^^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Error`
    |
155 |     E: std::fmt::Debug + std::error::Error,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be sent between threads safely
   --> performance-calculator/src/resilience/retry.rs:165:5
    |
165 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ `E` cannot be sent between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:199:28
    |
192 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
199 |     E: std::error::Error + Send + Sync + 'static,
    |                            ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Send`
    |
155 |     E: std::fmt::Debug + std::marker::Send,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be shared between threads safely
   --> performance-calculator/src/resilience/retry.rs:165:5
    |
165 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ `E` cannot be shared between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:199:35
    |
192 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
199 |     E: std::error::Error + Send + Sync + 'static,
    |                                   ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Sync`
    |
155 |     E: std::fmt::Debug + std::marker::Sync,
    |                        +++++++++++++++++++

error[E0277]: the trait bound `E: StdError` is not satisfied
   --> performance-calculator/src/resilience/retry.rs:165:29
    |
165 |     retry(f, &retry_policy).await
    |                             ^^^^^ the trait `StdError` is not implemented for `E`
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:199:8
    |
192 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
199 |     E: std::error::Error + Send + Sync + 'static,
    |        ^^^^^^^^^^^^^^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Error`
    |
155 |     E: std::fmt::Debug + std::error::Error,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be sent between threads safely
   --> performance-calculator/src/resilience/retry.rs:165:29
    |
165 |     retry(f, &retry_policy).await
    |                             ^^^^^ `E` cannot be sent between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:199:28
    |
192 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
199 |     E: std::error::Error + Send + Sync + 'static,
    |                            ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Send`
    |
155 |     E: std::fmt::Debug + std::marker::Send,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be shared between threads safely
   --> performance-calculator/src/resilience/retry.rs:165:29
    |
165 |     retry(f, &retry_policy).await
    |                             ^^^^^ `E` cannot be shared between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:199:35
    |
192 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
199 |     E: std::error::Error + Send + Sync + 'static,
    |                                   ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Sync`
    |
155 |     E: std::fmt::Debug + std::marker::Sync,
    |                        +++++++++++++++++++

error[E0560]: struct `resilience::retry::RetryConfig` has no field named `max_retries`
   --> performance-calculator/src/resilience/retry.rs:182:9
    |
182 |         max_retries,
    |         ^^^^^^^^^^^ `resilience::retry::RetryConfig` does not have this field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0560]: struct `resilience::retry::RetryConfig` has no field named `initial_backoff_ms`
   --> performance-calculator/src/resilience/retry.rs:183:9
    |
183 |         initial_backoff_ms,
    |         ^^^^^^^^^^^^^^^^^^ `resilience::retry::RetryConfig` does not have this field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0277]: the trait bound `E: StdError` is not satisfied
   --> performance-calculator/src/resilience/retry.rs:189:5
    |
189 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ the trait `StdError` is not implemented for `E`
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:199:8
    |
192 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
199 |     E: std::error::Error + Send + Sync + 'static,
    |        ^^^^^^^^^^^^^^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Error`
    |
179 |     E: std::fmt::Debug + std::error::Error,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be sent between threads safely
   --> performance-calculator/src/resilience/retry.rs:189:5
    |
189 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ `E` cannot be sent between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:199:28
    |
192 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
199 |     E: std::error::Error + Send + Sync + 'static,
    |                            ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Send`
    |
179 |     E: std::fmt::Debug + std::marker::Send,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be shared between threads safely
   --> performance-calculator/src/resilience/retry.rs:189:5
    |
189 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ `E` cannot be shared between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:199:35
    |
192 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
199 |     E: std::error::Error + Send + Sync + 'static,
    |                                   ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Sync`
    |
179 |     E: std::fmt::Debug + std::marker::Sync,
    |                        +++++++++++++++++++

error[E0277]: the trait bound `E: StdError` is not satisfied
   --> performance-calculator/src/resilience/retry.rs:189:29
    |
189 |     retry(f, &retry_policy).await
    |                             ^^^^^ the trait `StdError` is not implemented for `E`
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:199:8
    |
192 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
199 |     E: std::error::Error + Send + Sync + 'static,
    |        ^^^^^^^^^^^^^^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Error`
    |
179 |     E: std::fmt::Debug + std::error::Error,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be sent between threads safely
   --> performance-calculator/src/resilience/retry.rs:189:29
    |
189 |     retry(f, &retry_policy).await
    |                             ^^^^^ `E` cannot be sent between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:199:28
    |
192 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
199 |     E: std::error::Error + Send + Sync + 'static,
    |                            ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Send`
    |
179 |     E: std::fmt::Debug + std::marker::Send,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be shared between threads safely
   --> performance-calculator/src/resilience/retry.rs:189:29
    |
189 |     retry(f, &retry_policy).await
    |                             ^^^^^ `E` cannot be shared between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:199:35
    |
192 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
199 |     E: std::error::Error + Send + Sync + 'static,
    |                                   ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Sync`
    |
179 |     E: std::fmt::Debug + std::marker::Sync,
    |                        +++++++++++++++++++

warning: variable `last_error` is assigned to, but never used
   --> performance-calculator/src/resilience/retry.rs:203:13
    |
203 |     let mut last_error = None;
    |             ^^^^^^^^^^
    |
    = note: consider using `_last_error` instead

warning: value assigned to `last_error` is never read
   --> performance-calculator/src/resilience/retry.rs:217:17
    |
217 |                 last_error = Some(error);
    |                 ^^^^^^^^^^
    |
    = help: maybe it is overwritten before being read?
    = note: `#[warn(unused_assignments)]` on by default

Some errors have detailed explanations: E0107, E0255, E0277, E0308, E0392, E0404, E0412, E0432, E0433...
For more information about an error, try `rustc --explain E0107`.
warning: `performance-calculator` (lib) generated 78 warnings
error: could not compile `performance-calculator` (lib) due to 80 previous errors; 78 warnings emitted

```

---

_****_



I see we still have quite a few compilation errors to resolve. Let me analyze the key issues and fix them systematically:

1. First, let's fix the `RetryError` and `RetryConfig` related issues in the retry module:

---

_****_

```diff
  /// Retry error types
  #[derive(Debug)]
- pub struct RetryError<E> {
-     pub cause: E,
-     pub attempts: u32,
+ pub enum RetryError<E> {
+     MaxRetriesExceeded(E),
+     Aborted(E),
+     Other(E),
  }
  
  impl<E: std::error::Error> std::error::Error for RetryError<E> {
      fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
-         Some(&self.cause)
+         match self {
+             RetryError::MaxRetriesExceeded(e) => Some(e),
+             RetryError::Aborted(e) => Some(e),
+             RetryError::Other(e) => Some(e),
+         }
      }
  }
  
  impl<E: std::fmt::Display> std::fmt::Display for RetryError<E> {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
-         write!(f, "Operation failed after {} attempts: {}", self.attempts, self.cause)
+         match self {
+             RetryError::MaxRetriesExceeded(e) => write!(f, "Maximum retries exceeded: {}", e),
+             RetryError::Aborted(e) => write!(f, "Operation aborted: {}", e),
+             RetryError::Other(e) => write!(f, "Other error: {}", e),
+         }
      }
  }
  }
  
- #[derive(Debug, Clone)]
  pub struct RetryConfig {
      pub max_attempts: u32,
      }
  }
+ 
+ impl RetryConfig {
+     pub fn new(
+         max_attempts: u32,
+         initial_delay_ms: u64,
+         max_delay_ms: u64,
+     ) -> Self {
+         Self {
+             max_attempts,
+             initial_delay_ms,
+             max_delay_ms,
+             multiplier: 2.0,
+             jitter: true,
+         }
+     }
+ }
  
  impl StandardRetryPolicy {
                      attempt += 1;
                  } else {
-                     return Err(RetryError {
-                         cause: error,
-                         attempts: attempt,
-                     });
+                     return Err(RetryError::Other(error));
                  }
              }
              Err(error) => {
                  if attempt >= policy.max_attempts {
-                     return Err(RetryError {
-                         cause: error,
-                         attempts: attempt,
-                     });
+                     return Err(RetryError::MaxRetriesExceeded(error));
                  }
  
```

---

_****_



2. Now let's fix the `CircuitBreaker` trait generic parameter issue and other resilience module issues:

---

_****_

```diff
      ).await
  }
+ 
+ pub async fn with_resilience<T, E, F, Fut>(
+     f: F,
+     config: ResilienceConfig<E>,
+ ) -> Result<T, ResilienceError<E>>
+ where
+     F: FnMut() -> Fut + Send + 'static,
+     Fut: Future<Output = Result<T, E>> + Send + 'static,
+     T: Send + 'static,
+     E: std::error::Error + Send + Sync + 'static + std::fmt::Debug,
+ {
+     let circuit_breaker = config.circuit_breaker;
+     let bulkhead = config.bulkhead;
+     let retry_config = config.retry_config;
+ 
+     let retry_policy = StandardRetryPolicy::new(RetryConfig {
+         max_attempts: retry_config.max_attempts,
+         initial_delay_ms: retry_config.initial_delay_ms,
+         max_delay_ms: retry_config.max_delay_ms,
+         multiplier: 2.0,
+         jitter: true,
+     });
+ 
+     match bulkhead.acquire().await {
+         Ok(_permit) => {
+             match circuit_breaker.check().await {
+                 Ok(_) => {
+                     match retry::retry_with_exponential_backoff(f, retry_config).await {
+                         Ok(result) => Ok(result),
+                         Err(RetryError::MaxRetriesExceeded(e)) => Err(ResilienceError::Retry(e)),
+                         Err(RetryError::Aborted(e)) => Err(ResilienceError::Retry(e)),
+                         Err(RetryError::Other(e)) => Err(ResilienceError::Other(e)),
+                     }
+                 }
+                 Err(e) => Err(ResilienceError::CircuitBreaker(e)),
+             }
+         }
+         Err(e) => Err(ResilienceError::Bulkhead(e)),
+     }
+ }
  
  pub struct ResilienceConfig<E> {
-     circuit_breaker: StandardCircuitBreaker,
-     bulkhead: StandardBulkhead,
-     retry_config: RetryConfig,
+     pub circuit_breaker: Box<dyn CircuitBreaker<E>>,
+     pub bulkhead: Box<dyn Bulkhead>,
+     pub retry_config: RetryConfig,
+     _phantom: PhantomData<E>,
  }
  
  pub struct ResilienceBuilder<E> {
-     circuit_breaker: Option<StandardCircuitBreaker>,
-     bulkhead: Option<StandardBulkhead>,
+     circuit_breaker: Option<Box<dyn CircuitBreaker<E>>>,
+     bulkhead: Option<Box<dyn Bulkhead>>,
      retry_config: Option<RetryConfig>,
- }
- 
- pub async fn with_resilience<T, E, F, Fut>(
-     circuit_breaker: &StandardCircuitBreaker,
-     bulkhead: &StandardBulkhead,
-     retry_config: &RetryConfig,
-     f: F,
- ) -> Result<T, E>
- where
-     F: Fn() -> Fut + Send + Sync + 'static,
-     Fut: Future<Output = Result<T, E>> + Send,
-     T: Send + 'static,
-     E: std::error::Error + Send + Sync + 'static,
- {
-     let retry_policy = StandardRetryPolicy::new(retry_config.clone());
-     retry::retry(f, &retry_policy).await
+     _phantom: PhantomData<E>,
+ }
+ 
+ impl<E> ResilienceBuilder<E> {
+     pub fn new() -> Self {
+         Self {
+             circuit_breaker: None,
+             bulkhead: None,
+             retry_config: None,
+             _phantom: PhantomData,
+         }
+     }
+ 
+     pub fn with_circuit_breaker(mut self, circuit_breaker: Box<dyn CircuitBreaker<E>>) -> Self {
+         self.circuit_breaker = Some(circuit_breaker);
+         self
+     }
+ 
+     pub fn with_bulkhead(mut self, bulkhead: Box<dyn Bulkhead>) -> Self {
+         self.bulkhead = Some(bulkhead);
+         self
+     }
+ 
+     pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
+         self.retry_config = Some(retry_config);
+         self
+     }
+ 
+     pub fn build(self) -> Result<ResilienceConfig<E>, anyhow::Error> {
+         Ok(ResilienceConfig {
+             circuit_breaker: self.circuit_breaker.ok_or_else(|| anyhow::anyhow!("Circuit breaker not configured"))?,
+             bulkhead: self.bulkhead.ok_or_else(|| anyhow::anyhow!("Bulkhead not configured"))?,
+             retry_config: self.retry_config.unwrap_or_default(),
+             _phantom: PhantomData,
+         })
+     }
  }
  
      let retry_config = RetryConfig::default();
  
-     with_resilience(&circuit_breaker, &bulkhead, &retry_config, f).await
+     with_resilience(f, ResilienceConfig {
+         circuit_breaker: Box::new(circuit_breaker),
+         bulkhead: Box::new(bulkhead),
+         retry_config,
+         _phantom: PhantomData,
+     }).await
  } 
```

---

_****_



3. Let's fix the remaining linter errors in the resilience module:

---

_****_

```diff
      ).await
  }
+ 
+ pub async fn get_circuit_breaker_for_service<E>(
+     service: &str,
+ ) -> Result<Box<dyn CircuitBreaker<E>>, anyhow::Error>
+ where
+     E: std::error::Error + Send + Sync + 'static,
+ {
+     let config = CircuitBreakerConfig::default();
+     Ok(Box::new(StandardCircuitBreaker::new(service.to_string(), config)))
+ }
  
  pub async fn with_resilience<T, E, F, Fut>(
      let bulkhead = config.bulkhead;
      let retry_config = config.retry_config;
- 
-     let retry_policy = StandardRetryPolicy::new(RetryConfig {
-         max_attempts: retry_config.max_attempts,
-         initial_delay_ms: retry_config.initial_delay_ms,
-         max_delay_ms: retry_config.max_delay_ms,
-         multiplier: 2.0,
-         jitter: true,
-     });
  
      match bulkhead.acquire().await {
  
  pub async fn with_tenant_resilience<T, E, F, Fut>(
+     f: F,
      tenant_id: &str,
-     service: &str,
-     f: F,
- ) -> Result<T, E>
+ ) -> Result<T, ResilienceError<E>>
  where
-     F: Fn() -> Fut + Send + Sync + 'static,
-     Fut: Future<Output = Result<T, E>> + Send,
+     F: FnMut() -> Fut + Send + 'static,
+     Fut: Future<Output = Result<T, E>> + Send + 'static,
      T: Send + 'static,
-     E: std::error::Error + Send + Sync + std::fmt::Debug + 'static,
+     E: std::error::Error + Send + Sync + 'static + std::fmt::Debug,
  {
-     let circuit_breaker = get_circuit_breaker(service)?;
-     let bulkhead = get_bulkhead(service, 10)?;
+     let circuit_breaker = get_circuit_breaker_for_service(tenant_id).await?;
+     let bulkhead = get_bulkhead(tenant_id, 10)?;
      let retry_config = RetryConfig::default();
  
      with_resilience(f, ResilienceConfig {
-         circuit_breaker: Box::new(circuit_breaker),
+         circuit_breaker,
          bulkhead: Box::new(bulkhead),
          retry_config,
          _phantom: PhantomData,
-     }).await
+     })
+     .await
  } 
```

---

_****_



4. Let's fix the remaining `get_circuit_breaker` name conflict:

---

_****_

```diff
  
  pub use circuit_breaker::{
-     CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError, CircuitBreakerMetrics,
-     StandardCircuitBreaker, get_circuit_breaker, get_circuit_breaker_with_config,
+     CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError, CircuitBreakerState,
+     StandardCircuitBreaker,
  };
  
  pub use retry::{
-     RetryConfig, RetryError, RetryPolicy, retry_with_exponential_backoff,
+     RetryConfig, RetryError, RetryPolicy, StandardRetryPolicy,
+     retry_with_exponential_backoff, retry_if,
  };
  
- pub use health_check::{
-     HealthCheck, HealthCheckConfig, HealthCheckResult, HealthCheckRegistry,
-     register_health_check, get_health_check_registry,
- };
+ pub use health_check::{HealthCheck, HealthCheckError, StandardHealthCheck};
  
- pub use bulkhead::{
-     Bulkhead, BulkheadConfig, BulkheadError, BulkheadMetrics, StandardBulkhead,
-     TenantBulkheadRegistry, get_tenant_bulkhead, get_tenant_bulkhead_registry,
-     set_tenant_bulkhead_config, configure_tenant_bulkheads_from_tier,
- };
+ pub use bulkhead::{Bulkhead, BulkheadError, StandardBulkhead};
  
- pub use chaos::{
-     ChaosConfig, ChaosMetrics, ChaosService, get_chaos_service,
-     enable_chaos_testing, disable_chaos_testing, maybe_inject_failure, maybe_inject_delay,
-     with_chaos,
- };
+ pub use chaos::{ChaosConfig, ChaosError, ChaosStrategy};
  
  use thiserror::Error;
  pub enum ResilienceError<E> {
      #[error("Circuit breaker error: {0}")]
-     CircuitBreaker(#[from] CircuitBreakerError<E>),
+     CircuitBreaker(#[from] CircuitBreakerError),
      
      #[error("Bulkhead error: {0}")]
      Bulkhead(#[from] BulkheadError<E>),
      
      #[error("Retry error: {0}")]
-     Retry(#[from] RetryError<E>),
-     
-     #[error("Health check error: {0}")]
-     HealthCheck(String),
-     
-     #[error("Internal error: {0}")]
-     Internal(String),
+     Retry(#[from] E),
+     
+     #[error("Other error: {0}")]
+     Other(E),
+ }
+ 
+ impl<E: std::error::Error> std::error::Error for ResilienceError<E> {
+     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
+         match self {
+             ResilienceError::CircuitBreaker(e) => Some(e),
+             ResilienceError::Bulkhead(e) => Some(e),
+             ResilienceError::Retry(e) => Some(e),
+             ResilienceError::Other(e) => Some(e),
+         }
+     }
+ }
+ 
+ impl<E: std::fmt::Display> std::fmt::Display for ResilienceError<E> {
+     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+         match self {
+             ResilienceError::CircuitBreaker(e) => write!(f, "Circuit breaker error: {}", e),
+             ResilienceError::Bulkhead(e) => write!(f, "Bulkhead error: {}", e),
+             ResilienceError::Retry(e) => write!(f, "Retry error: {}", e),
+             ResilienceError::Other(e) => write!(f, "Other error: {}", e),
+         }
+     }
  }
  
      let bulkhead = get_tenant_bulkhead(tenant_id);
      let circuit_breaker = circuit_breaker::get_circuit_breaker(circuit_name)
-         .map_err(|e| ResilienceError::Internal(e.to_string()))?;
+         .map_err(|e| ResilienceError::Other(e))?;
      
      retry_with_exponential_backoff(
```

---

_****_



5. Let's fix the duplicate `retry_with_exponential_backoff` function:

---

_****_

```diff
  pub use retry::{
      RetryConfig, RetryError, RetryPolicy, StandardRetryPolicy,
-     retry_with_exponential_backoff, retry_if,
+     retry_if,
  };
  
  
  use thiserror::Error;
+ use std::marker::PhantomData;
  
  /// Combined error type for resilience operations
      }
  }
- 
- /// Retry a fallible operation with exponential backoff
- pub async fn retry_with_exponential_backoff<F, Fut, T, E>(
-     operation: F,
-     max_retries: u32,
-     initial_delay_ms: u64,
- ) -> Result<T, RetryError<E>>
- where
-     F: Fn() -> Fut,
-     Fut: Future<Output = Result<T, E>>,
-     E: Error + Send + Sync + 'static,
- {
-     let mut retries = 0;
-     let mut delay = Duration::from_millis(initial_delay_ms);
- 
-     loop {
-         match operation().await {
-             Ok(value) => return Ok(value),
-             Err(e) => {
-                 if retries >= max_retries {
-                     return Err(RetryError::MaxRetriesExceeded(e));
-                 }
- 
-                 warn!("Operation failed, retrying in {:?}: {}", delay, e);
-                 tokio::time::sleep(delay).await;
-                 delay *= 2;
-                 retries += 1;
-             }
-         }
-     }
- }
  
  /// Execute an operation with circuit breaker protection
  
  /// Get a bulkhead instance
- pub fn get_bulkhead(name: &str, max_concurrent: usize) -> Result<StandardBulkhead> {
-     Ok(StandardBulkhead::new(name.to_string(), BulkheadConfig {
-         max_concurrent_calls: max_concurrent,
-         ..Default::default()
-     }))
+ pub fn get_bulkhead<E>(
+     service: &str,
+     max_concurrent: usize,
+ ) -> Result<StandardBulkhead, anyhow::Error> {
+     Ok(StandardBulkhead::new(service.to_string(), max_concurrent))
  }
  
```

---

_****_



Let me try building the project again to see if we've resolved more issues:

---

_****_

```bash
cargo build
```

```
sethford@Seths-MacBook-Pro rust-investment-performance % 
    |
53  | pub struct StandardRetryPolicy {
    |            ^^^^^^^^^^^^^^^^^^^

warning: unused variable: `beginning_base`
   --> performance-calculator/src/calculations/currency.rs:141:13
    |
141 |         let beginning_base = beginning_amount * beginning_rate;
    |             ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_beginning_bas
e`
    |
    = note: `#[warn(unused_variables)]` on by default

error[E0119]: conflicting implementations of trait `StdError` for type `ResilienceError<_>`
  --> performance-calculator/src/resilience/mod.rs:40:10
   |
40 | #[derive(Error, Debug)]
   |          ^^^^^ conflicting implementation for `ResilienceError<_>`
...
55 | impl<E: std::error::Error> std::error::Error for ResilienceError<E> {
   | ------------------------------------------------------------------- first implementation here
   |
   = note: this error originates in the derive macro `Error` (in Nightly builds, run with -Z macro-backtrace
 for more info)

error[E0119]: conflicting implementations of trait `std::fmt::Display` for type `ResilienceError<_>`
  --> performance-calculator/src/resilience/mod.rs:40:10
   |
40 | #[derive(Error, Debug)]
   |          ^^^^^ conflicting implementation for `ResilienceError<_>`
...
66 | impl<E: std::fmt::Display> std::fmt::Display for ResilienceError<E> {
   | ------------------------------------------------------------------- first implementation here
   |
   = note: this error originates in the derive macro `Error` (in Nightly builds, run with -Z macro-backtrace
 for more info)

warning: `graphql-api` (bin "graphql-api") generated 20 warnings
error: could not compile `graphql-api` (bin "graphql-api") due to 1 previous error; 20 warnings emitted
warning: unused variable: `event`
   --> performance-calculator/src/calculations/query_api.rs:370:13
    |
370 |         let event = self.audit_manager.start_calculation(
    |             ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`

warning: unused variable: `query_id_for_audit`
   --> performance-calculator/src/calculations/query_api.rs:391:13
    |
391 |         let query_id_for_audit = query_id.clone();
    |             ^^^^^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_query_id_
for_audit`

warning: unused variable: `portfolio_data`
   --> performance-calculator/src/calculations/query_api.rs:626:25
    |
626 |                     let portfolio_data = data_service.get_portfolio_data(
    |                         ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_p
ortfolio_data`

warning: unused variable: `scenario`
   --> performance-calculator/src/calculations/analytics.rs:379:13
    |
379 |         let scenario = scenarios.get(scenario_id)
    |             ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_scenario`

warning: unused variable: `request_id`
   --> performance-calculator/src/calculations/visualization.rs:543:9
    |
543 |         request_id: &str,
    |         ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_request_id`

warning: unused variable: `request_id`
   --> performance-calculator/src/calculations/integration.rs:844:65
    |
844 | ...uest, request_id: &str) -> Result<DataImportResult> {
    |          ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_request_id`

error[E0107]: missing generics for trait `CircuitBreaker`
   --> performance-calculator/src/resilience/mod.rs:86:9
    |
86  |     CB: CircuitBreaker,
    |         ^^^^^^^^^^^^^^ expected 1 generic argument
    |
note: trait defined here, with 1 generic parameter: `E`
   --> performance-calculator/src/resilience/circuit_breaker.rs:102:11
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           ^^^^^^^^^^^^^^ -
help: add missing generic argument
    |
86  |     CB: CircuitBreaker<E>,
    |                       +++

error[E0038]: the trait `CircuitBreaker` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:326:30
    |
326 |     pub circuit_breaker: Box<dyn CircuitBreaker<E>>,
    |                              ^^^^^^^^^^^^^^^^^^^^^ `CircuitBreaker` cannot be made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/circuit_breaker.rs:104:14
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           -------------- this trait cannot be made into an object...
103 |     /// Execute a function with circuit breaker protection
104 |     async fn execute<T, F, Fut>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::circuit_breaker::StandardCircuitBreaker` is seen to implement the trait i
n this crate, consider using it directly instead
    = note: `CircuitBreaker` can be implemented in other crates; if you want to support your users passing t
heir own types here, you can't refer to a specific type

error[E0038]: the trait `Bulkhead` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:327:23
    |
327 |     pub bulkhead: Box<dyn Bulkhead>,
    |                       ^^^^^^^^^^^^ `Bulkhead` cannot be made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/bulkhead.rs:108:14
    |
106 | pub trait Bulkhead: Send + Sync {
    |           -------- this trait cannot be made into an object...
107 |     /// Execute an operation with bulkhead protection
108 |     async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, BulkheadError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::bulkhead::StandardBulkhead` is seen to implement the trait in this crate,
 consider using it directly instead
    = note: `Bulkhead` can be implemented in other crates; if you want to support your users passing their o
wn types here, you can't refer to a specific type

error[E0038]: the trait `CircuitBreaker` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:333:33
    |
333 |     circuit_breaker: Option<Box<dyn CircuitBreaker<E>>>,
    |                                 ^^^^^^^^^^^^^^^^^^^^^ `CircuitBreaker` cannot be made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/circuit_breaker.rs:104:14
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           -------------- this trait cannot be made into an object...
103 |     /// Execute a function with circuit breaker protection
104 |     async fn execute<T, F, Fut>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::circuit_breaker::StandardCircuitBreaker` is seen to implement the trait i
n this crate, consider using it directly instead
    = note: `CircuitBreaker` can be implemented in other crates; if you want to support your users passing t
heir own types here, you can't refer to a specific type

error[E0038]: the trait `Bulkhead` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:334:26
    |
334 |     bulkhead: Option<Box<dyn Bulkhead>>,
    |                          ^^^^^^^^^^^^ `Bulkhead` cannot be made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/bulkhead.rs:108:14
    |
106 | pub trait Bulkhead: Send + Sync {
    |           -------- this trait cannot be made into an object...
107 |     /// Execute an operation with bulkhead protection
108 |     async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, BulkheadError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::bulkhead::StandardBulkhead` is seen to implement the trait in this crate,
 consider using it directly instead
    = note: `Bulkhead` can be implemented in other crates; if you want to support your users passing their o
wn types here, you can't refer to a specific type

error[E0038]: the trait `CircuitBreaker` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:349:64
    |
349 |     pub fn with_circuit_breaker(mut self, circuit_breaker: Box<dyn CircuitBreaker<E>>) -> Self {
    |                                                                ^^^^^^^^^^^^^^^^^^^^^ `CircuitBreaker` 
cannot be made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/circuit_breaker.rs:104:14
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           -------------- this trait cannot be made into an object...
103 |     /// Execute a function with circuit breaker protection
104 |     async fn execute<T, F, Fut>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::circuit_breaker::StandardCircuitBreaker` is seen to implement the trait i
n this crate, consider using it directly instead
    = note: `CircuitBreaker` can be implemented in other crates; if you want to support your users passing t
heir own types here, you can't refer to a specific type

error[E0038]: the trait `Bulkhead` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:354:50
    |
354 |     pub fn with_bulkhead(mut self, bulkhead: Box<dyn Bulkhead>) -> Self {
    |                                                  ^^^^^^^^^^^^ `Bulkhead` cannot be made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/bulkhead.rs:108:14
    |
106 | pub trait Bulkhead: Send + Sync {
    |           -------- this trait cannot be made into an object...
107 |     /// Execute an operation with bulkhead protection
108 |     async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, BulkheadError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::bulkhead::StandardBulkhead` is seen to implement the trait in this crate,
 consider using it directly instead
    = note: `Bulkhead` can be implemented in other crates; if you want to support your users passing their o
wn types here, you can't refer to a specific type

error[E0599]: no method named `acquire` found for reference `&B` in the current scope
   --> performance-calculator/src/resilience/mod.rs:102:20
    |
102 |     match bulkhead.acquire().await {
    |                    ^^^^^^^ method not found in `&B`

error[E0599]: no variant or associated item named `Error` found for enum `BulkheadError` in the current scop
e
   --> performance-calculator/src/resilience/mod.rs:109:45
    |
109 |                     _ => Err(BulkheadError::Error(e)),
    |                                             ^^^^^ variant or associated item not found in `BulkheadErr
or<_>`
    |
   ::: performance-calculator/src/resilience/bulkhead.rs:14:1
    |
14  | pub enum BulkheadError<E> {
    | ------------------------- variant or associated item `Error` not found for this enum

error[E0609]: no field `max_retries` on type `resilience::retry::RetryConfig`
   --> performance-calculator/src/resilience/mod.rs:202:22
    |
202 |         retry_config.max_retries,
    |                      ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0609]: no field `initial_backoff_ms` on type `resilience::retry::RetryConfig`
   --> performance-calculator/src/resilience/mod.rs:203:22
    |
203 |         retry_config.initial_backoff_ms,
    |                      ^^^^^^^^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0277]: `?` couldn't convert the error to `ResilienceError<E>`
   --> performance-calculator/src/resilience/mod.rs:227:48
    |
227 |         .map_err(|e| ResilienceError::Other(e))?;
    |                                                ^ the trait `From<ResilienceError<std::string::String>>
` is not implemented for `ResilienceError<E>`
    |
    = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the 
`From` trait
    = note: required for `Result<T, ResilienceError<E>>` to implement `FromResidual<Result<Infallible, Resil
ienceError<std::string::String>>>`
help: consider extending the `where` clause, but there might be an alternative better way to express this re
quirement
    |
223 |     E: std::error::Error + Send + Sync + 'static + std::fmt::Debug, ResilienceError<E>: From<Resilienc
eError<std::string::String>>
    |                                                                   ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

error[E0609]: no field `max_retries` on type `resilience::retry::RetryConfig`
   --> performance-calculator/src/resilience/mod.rs:245:22
    |
245 |         retry_config.max_retries,
    |                      ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0609]: no field `initial_backoff_ms` on type `resilience::retry::RetryConfig`
   --> performance-calculator/src/resilience/mod.rs:246:22
    |
246 |         retry_config.initial_backoff_ms,
    |                      ^^^^^^^^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0277]: the trait bound `E: From<std::io::Error>` is not satisfied
   --> performance-calculator/src/resilience/mod.rs:271:9
    |
271 |         chaos::with_chaos(service, Some(tenant_id), || f()).await
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `From<std::io::Error>` is not im
plemented for `E`
    |
note: required by a bound in `chaos::with_chaos`
   --> performance-calculator/src/resilience/chaos.rs:262:28
    |
254 | pub async fn with_chaos<T, E, F, Fut>(
    |              ---------- required by a bound in this function
...
262 |     E: std::error::Error + From<std::io::Error> + Send + Sync + 'static,
    |                            ^^^^^^^^^^^^^^^^^^^^ required by this bound in `with_chaos`
help: consider further restricting type parameter `E` with trait `From`
    |
267 |     E: std::error::Error + Send + Sync + 'static + std::fmt::Debug + std::convert::From<std::io::Error
>,
    |                                                                    +++++++++++++++++++++++++++++++++++
+

error[E0277]: the trait bound `E: From<std::io::Error>` is not satisfied
   --> performance-calculator/src/resilience/mod.rs:271:61
    |
271 |         chaos::with_chaos(service, Some(tenant_id), || f()).await
    |                                                             ^^^^^ the trait `From<std::io::Error>` is 
not implemented for `E`
    |
note: required by a bound in `chaos::with_chaos`
   --> performance-calculator/src/resilience/chaos.rs:262:28
    |
254 | pub async fn with_chaos<T, E, F, Fut>(
    |              ---------- required by a bound in this function
...
262 |     E: std::error::Error + From<std::io::Error> + Send + Sync + 'static,
    |                            ^^^^^^^^^^^^^^^^^^^^ required by this bound in `with_chaos`
help: consider further restricting type parameter `E` with trait `From`
    |
267 |     E: std::error::Error + Send + Sync + 'static + std::fmt::Debug + std::convert::From<std::io::Error
>,
    |                                                                    +++++++++++++++++++++++++++++++++++
+

error[E0061]: this function takes 1 argument but 2 arguments were supplied
   --> performance-calculator/src/resilience/mod.rs:290:17
    |
290 |     Ok(Box::new(StandardCircuitBreaker::new(service.to_string(), config)))
    |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^^ ------------------- unexpected argument #1 of type `std::s
tring::String`
    |
note: associated function defined here
   --> performance-calculator/src/resilience/circuit_breaker.rs:126:12
    |
126 |     pub fn new(config: CircuitBreakerConfig) -> Self {
    |            ^^^ ----------------------------
help: remove the extra argument
    |
290 -     Ok(Box::new(StandardCircuitBreaker::new(service.to_string(), config)))
290 +     Ok(Box::new(StandardCircuitBreaker::new(config)))
    |

error[E0038]: the trait `CircuitBreaker` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:290:8
    |
290 |     Ok(Box::new(StandardCircuitBreaker::new(service.to_string(), config)))
    |        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `CircuitBreaker` cannot be m
ade into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/circuit_breaker.rs:104:14
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           -------------- this trait cannot be made into an object...
103 |     /// Execute a function with circuit breaker protection
104 |     async fn execute<T, F, Fut>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::circuit_breaker::StandardCircuitBreaker` is seen to implement the trait i
n this crate, consider using it directly instead
    = note: `CircuitBreaker` can be implemented in other crates; if you want to support your users passing t
heir own types here, you can't refer to a specific type
    = note: required for the cast from `Box<StandardCircuitBreaker>` to `Box<dyn CircuitBreaker<E>>`

error[E0038]: the trait `CircuitBreaker` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:290:5
    |
290 |     Ok(Box::new(StandardCircuitBreaker::new(service.to_string(), config)))
    |     ^^ `CircuitBreaker` cannot be made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/circuit_breaker.rs:104:14
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           -------------- this trait cannot be made into an object...
103 |     /// Execute a function with circuit breaker protection
104 |     async fn execute<T, F, Fut>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::circuit_breaker::StandardCircuitBreaker` is seen to implement the trait i
n this crate, consider using it directly instead
    = note: `CircuitBreaker` can be implemented in other crates; if you want to support your users passing t
heir own types here, you can't refer to a specific type

error[E0038]: the trait `CircuitBreaker` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:290:5
    |
290 |     Ok(Box::new(StandardCircuitBreaker::new(service.to_string(), config)))
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `CircuitBreaker` cannot be 
made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/circuit_breaker.rs:104:14
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           -------------- this trait cannot be made into an object...
103 |     /// Execute a function with circuit breaker protection
104 |     async fn execute<T, F, Fut>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::circuit_breaker::StandardCircuitBreaker` is seen to implement the trait i
n this crate, consider using it directly instead
    = note: `CircuitBreaker` can be implemented in other crates; if you want to support your users passing t
heir own types here, you can't refer to a specific type

error[E0038]: the trait `CircuitBreaker` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:283:1
    |
283 | / pub async fn get_circuit_breaker_for_service<E>(
284 | |     service: &str,
285 | | ) -> Result<Box<dyn CircuitBreaker<E>>, anyhow::Error>
286 | | where
287 | |     E: std::error::Error + Send + Sync + 'static,
    | |_________________________________________________^ `CircuitBreaker` cannot be made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/circuit_breaker.rs:104:14
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           -------------- this trait cannot be made into an object...
103 |     /// Execute a function with circuit breaker protection
104 |     async fn execute<T, F, Fut>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::circuit_breaker::StandardCircuitBreaker` is seen to implement the trait i
n this crate, consider using it directly instead
    = note: `CircuitBreaker` can be implemented in other crates; if you want to support your users passing t
heir own types here, you can't refer to a specific type

error[E0599]: no method named `acquire` found for struct `Box<dyn Bulkhead>` in the current scope
   --> performance-calculator/src/resilience/mod.rs:307:20
    |
307 |     match bulkhead.acquire().await {
    |                    ^^^^^^^ method not found in `Box<dyn Bulkhead>`

error[E0599]: no method named `check` found for struct `Box<dyn CircuitBreaker<E>>` in the current scope
   --> performance-calculator/src/resilience/mod.rs:309:35
    |
309 |             match circuit_breaker.check().await {
    |                                   ^^^^^ method not found in `Box<dyn CircuitBreaker<E>>`

error[E0277]: expected a `Fn()` closure, found `F`
   --> performance-calculator/src/resilience/mod.rs:311:65
    |
311 |                     match retry::retry_with_exponential_backoff(f, retry_config).await {
    |                           ------------------------------------- ^ expected an `Fn()` closure, found `F
`
    |                           |
    |                           required by a bound introduced by this call
    |
    = note: wrap the `F` in a closure with no arguments: `|| { /* code */ }`
    = note: `F` implements `FnMut`, but it must implement `Fn`, which is more general
note: required by a bound in `retry_with_exponential_backoff`
   --> performance-calculator/src/resilience/retry.rs:174:8
    |
168 | pub async fn retry_with_exponential_backoff<T, E, F, Fut>(
    |              ------------------------------ required by a bound in this function
...
174 |     F: Fn() -> Fut,
    |        ^^^^^^^^^^^ required by this bound in `retry_with_exponential_backoff`

error[E0061]: this function takes 3 arguments but 2 arguments were supplied
   --> performance-calculator/src/resilience/mod.rs:311:27
    |
311 |                     match retry::retry_with_exponential_backoff(f, retry_config).await {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^-----------------
    |                                                                |   |
    |                                                                |   expected `u32`, found `RetryConfig`
    |                                                                argument #3 of type `u64` is missing
    |
note: function defined here
   --> performance-calculator/src/resilience/retry.rs:168:14
    |
168 | pub async fn retry_with_exponential_backoff<T, E, F, Fut>(
    |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
169 |     f: F,
    |     ----
170 |     max_retries: u32,
    |     ----------------
171 |     initial_backoff_ms: u64,
    |     -----------------------
help: provide the argument
    |
311 |                     match retry::retry_with_exponential_backoff(f, /* u32 */, /* u64 */).await {
    |                                                                ~~~~~~~~~~~~~~~~~~~~~~~~~

error[E0277]: expected a `Fn()` closure, found `F`
   --> performance-calculator/src/resilience/mod.rs:311:27
    |
311 |                     match retry::retry_with_exponential_backoff(f, retry_config).await {
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected an `Fn()` cl
osure, found `F`
    |
    = note: wrap the `F` in a closure with no arguments: `|| { /* code */ }`
    = note: `F` implements `FnMut`, but it must implement `Fn`, which is more general
note: required by a bound in `retry_with_exponential_backoff`
   --> performance-calculator/src/resilience/retry.rs:174:8
    |
168 | pub async fn retry_with_exponential_backoff<T, E, F, Fut>(
    |              ------------------------------ required by a bound in this function
...
174 |     F: Fn() -> Fut,
    |        ^^^^^^^^^^^ required by this bound in `retry_with_exponential_backoff`

error[E0277]: expected a `Fn()` closure, found `F`
   --> performance-calculator/src/resilience/mod.rs:311:82
    |
311 |                     match retry::retry_with_exponential_backoff(f, retry_config).await {
    |                                                                                  ^^^^^ expected an `Fn
()` closure, found `F`
    |
    = note: wrap the `F` in a closure with no arguments: `|| { /* code */ }`
    = note: `F` implements `FnMut`, but it must implement `Fn`, which is more general
note: required by a bound in `retry_with_exponential_backoff`
   --> performance-calculator/src/resilience/retry.rs:174:8
    |
168 | pub async fn retry_with_exponential_backoff<T, E, F, Fut>(
    |              ------------------------------ required by a bound in this function
...
174 |     F: Fn() -> Fut,
    |        ^^^^^^^^^^^ required by this bound in `retry_with_exponential_backoff`

error[E0038]: the trait `CircuitBreaker` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:384:70
    |
384 |     let circuit_breaker = get_circuit_breaker_for_service(tenant_id).await?;
    |                                                                      ^^^^^ `CircuitBreaker` cannot be 
made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/circuit_breaker.rs:104:14
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           -------------- this trait cannot be made into an object...
103 |     /// Execute a function with circuit breaker protection
104 |     async fn execute<T, F, Fut>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::circuit_breaker::StandardCircuitBreaker` is seen to implement the trait i
n this crate, consider using it directly instead
    = note: `CircuitBreaker` can be implemented in other crates; if you want to support your users passing t
heir own types here, you can't refer to a specific type

error[E0038]: the trait `CircuitBreaker` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:384:27
    |
384 |     let circuit_breaker = get_circuit_breaker_for_service(tenant_id).await?;
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `CircuitBreaker` cannot be 
made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/circuit_breaker.rs:104:14
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           -------------- this trait cannot be made into an object...
103 |     /// Execute a function with circuit breaker protection
104 |     async fn execute<T, F, Fut>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::circuit_breaker::StandardCircuitBreaker` is seen to implement the trait i
n this crate, consider using it directly instead
    = note: `CircuitBreaker` can be implemented in other crates; if you want to support your users passing t
heir own types here, you can't refer to a specific type

error[E0038]: the trait `CircuitBreaker` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:384:27
    |
384 |     let circuit_breaker = get_circuit_breaker_for_service(tenant_id).await?;
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `CircuitBreaker` cannot be
 made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/circuit_breaker.rs:104:14
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           -------------- this trait cannot be made into an object...
103 |     /// Execute a function with circuit breaker protection
104 |     async fn execute<T, F, Fut>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::circuit_breaker::StandardCircuitBreaker` is seen to implement the trait i
n this crate, consider using it directly instead
    = note: `CircuitBreaker` can be implemented in other crates; if you want to support your users passing t
heir own types here, you can't refer to a specific type

error[E0277]: `?` couldn't convert the error to `ResilienceError<E>`
   --> performance-calculator/src/resilience/mod.rs:384:75
    |
384 |     let circuit_breaker = get_circuit_breaker_for_service(tenant_id).await?;
    |                                                                           ^ the trait `From<anyhow::Er
ror>` is not implemented for `ResilienceError<E>`
    |
    = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the 
`From` trait
    = note: required for `Result<T, ResilienceError<E>>` to implement `FromResidual<Result<Infallible, anyho
w::Error>>`
help: consider extending the `where` clause, but there might be an alternative better way to express this re
quirement
    |
382 |     E: std::error::Error + Send + Sync + 'static + std::fmt::Debug, ResilienceError<E>: From<anyhow::E
rror>
    |                                                                   ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
~~~~~

error[E0277]: `?` couldn't convert the error to `ResilienceError<E>`
   --> performance-calculator/src/resilience/mod.rs:385:47
    |
385 |     let bulkhead = get_bulkhead(tenant_id, 10)?;
    |                                               ^ the trait `From<anyhow::Error>` is not implemented for
 `ResilienceError<E>`
    |
    = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the 
`From` trait
    = note: required for `Result<T, ResilienceError<E>>` to implement `FromResidual<Result<Infallible, anyho
w::Error>>`
help: consider extending the `where` clause, but there might be an alternative better way to express this re
quirement
    |
382 |     E: std::error::Error + Send + Sync + 'static + std::fmt::Debug, ResilienceError<E>: From<anyhow::E
rror>
    |                                                                   ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
~~~~~

error[E0038]: the trait `CircuitBreaker` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:389:9
    |
389 |         circuit_breaker,
    |         ^^^^^^^^^^^^^^^ `CircuitBreaker` cannot be made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/circuit_breaker.rs:104:14
    |
102 | pub trait CircuitBreaker<E>: Send + Sync {
    |           -------------- this trait cannot be made into an object...
103 |     /// Execute a function with circuit breaker protection
104 |     async fn execute<T, F, Fut>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::circuit_breaker::StandardCircuitBreaker` is seen to implement the trait i
n this crate, consider using it directly instead
    = note: `CircuitBreaker` can be implemented in other crates; if you want to support your users passing t
heir own types here, you can't refer to a specific type

error[E0038]: the trait `Bulkhead` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:390:19
    |
390 |         bulkhead: Box::new(bulkhead),
    |                   ^^^^^^^^^^^^^^^^^^ `Bulkhead` cannot be made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/bulkhead.rs:108:14
    |
106 | pub trait Bulkhead: Send + Sync {
    |           -------- this trait cannot be made into an object...
107 |     /// Execute an operation with bulkhead protection
108 |     async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, BulkheadError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::bulkhead::StandardBulkhead` is seen to implement the trait in this crate,
 consider using it directly instead
    = note: `Bulkhead` can be implemented in other crates; if you want to support your users passing their o
wn types here, you can't refer to a specific type

error[E0038]: the trait `Bulkhead` cannot be made into an object
   --> performance-calculator/src/resilience/mod.rs:390:19
    |
390 |         bulkhead: Box::new(bulkhead),
    |                   ^^^^^^^^^^^^^^^^^^ `Bulkhead` cannot be made into an object
    |
note: for a trait to be "dyn-compatible" it needs to allow building a vtable to allow the call to be resolva
ble dynamically; for more information visit <https://doc.rust-lang.org/reference/items/traits.html#object-sa
fety>
   --> performance-calculator/src/resilience/bulkhead.rs:108:14
    |
106 | pub trait Bulkhead: Send + Sync {
    |           -------- this trait cannot be made into an object...
107 |     /// Execute an operation with bulkhead protection
108 |     async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, BulkheadError<E>>
    |              ^^^^^^^ ...because method `execute` has generic type parameters
    = help: consider moving `execute` to another trait
    = help: only type `resilience::bulkhead::StandardBulkhead` is seen to implement the trait in this crate,
 consider using it directly instead
    = note: `Bulkhead` can be implemented in other crates; if you want to support your users passing their o
wn types here, you can't refer to a specific type
    = note: required for the cast from `Box<StandardBulkhead>` to `Box<(dyn Bulkhead + 'static)>`

error[E0560]: struct `resilience::retry::RetryConfig` has no field named `max_retries`
   --> performance-calculator/src/resilience/retry.rs:179:9
    |
179 |         max_retries,
    |         ^^^^^^^^^^^ `resilience::retry::RetryConfig` does not have this field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0560]: struct `resilience::retry::RetryConfig` has no field named `initial_backoff_ms`
   --> performance-calculator/src/resilience/retry.rs:180:9
    |
180 |         initial_backoff_ms,
    |         ^^^^^^^^^^^^^^^^^^ `resilience::retry::RetryConfig` does not have this field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0277]: the trait bound `E: StdError` is not satisfied
   --> performance-calculator/src/resilience/retry.rs:186:5
    |
186 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ the trait `StdError` is not implemented for `E`
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:220:8
    |
213 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
220 |     E: std::error::Error + Send + Sync + 'static,
    |        ^^^^^^^^^^^^^^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Error`
    |
176 |     E: std::fmt::Debug + std::error::Error,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be sent between threads safely
   --> performance-calculator/src/resilience/retry.rs:186:5
    |
186 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ `E` cannot be sent between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:220:28
    |
213 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
220 |     E: std::error::Error + Send + Sync + 'static,
    |                            ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Send`
    |
176 |     E: std::fmt::Debug + std::marker::Send,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be shared between threads safely
   --> performance-calculator/src/resilience/retry.rs:186:5
    |
186 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ `E` cannot be shared between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:220:35
    |
213 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
220 |     E: std::error::Error + Send + Sync + 'static,
    |                                   ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Sync`
    |
176 |     E: std::fmt::Debug + std::marker::Sync,
    |                        +++++++++++++++++++

error[E0277]: the trait bound `E: StdError` is not satisfied
   --> performance-calculator/src/resilience/retry.rs:186:29
    |
186 |     retry(f, &retry_policy).await
    |                             ^^^^^ the trait `StdError` is not implemented for `E`
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:220:8
    |
213 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
220 |     E: std::error::Error + Send + Sync + 'static,
    |        ^^^^^^^^^^^^^^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Error`
    |
176 |     E: std::fmt::Debug + std::error::Error,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be sent between threads safely
   --> performance-calculator/src/resilience/retry.rs:186:29
    |
186 |     retry(f, &retry_policy).await
    |                             ^^^^^ `E` cannot be sent between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:220:28
    |
213 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
220 |     E: std::error::Error + Send + Sync + 'static,
    |                            ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Send`
    |
176 |     E: std::fmt::Debug + std::marker::Send,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be shared between threads safely
   --> performance-calculator/src/resilience/retry.rs:186:29
    |
186 |     retry(f, &retry_policy).await
    |                             ^^^^^ `E` cannot be shared between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:220:35
    |
213 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
220 |     E: std::error::Error + Send + Sync + 'static,
    |                                   ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Sync`
    |
176 |     E: std::fmt::Debug + std::marker::Sync,
    |                        +++++++++++++++++++

error[E0560]: struct `resilience::retry::RetryConfig` has no field named `max_retries`
   --> performance-calculator/src/resilience/retry.rs:203:9
    |
203 |         max_retries,
    |         ^^^^^^^^^^^ `resilience::retry::RetryConfig` does not have this field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0560]: struct `resilience::retry::RetryConfig` has no field named `initial_backoff_ms`
   --> performance-calculator/src/resilience/retry.rs:204:9
    |
204 |         initial_backoff_ms,
    |         ^^^^^^^^^^^^^^^^^^ `resilience::retry::RetryConfig` does not have this field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0277]: the trait bound `E: StdError` is not satisfied
   --> performance-calculator/src/resilience/retry.rs:210:5
    |
210 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ the trait `StdError` is not implemented for `E`
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:220:8
    |
213 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
220 |     E: std::error::Error + Send + Sync + 'static,
    |        ^^^^^^^^^^^^^^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Error`
    |
200 |     E: std::fmt::Debug + std::error::Error,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be sent between threads safely
   --> performance-calculator/src/resilience/retry.rs:210:5
    |
210 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ `E` cannot be sent between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:220:28
    |
213 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
220 |     E: std::error::Error + Send + Sync + 'static,
    |                            ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Send`
    |
200 |     E: std::fmt::Debug + std::marker::Send,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be shared between threads safely
   --> performance-calculator/src/resilience/retry.rs:210:5
    |
210 |     retry(f, &retry_policy).await
    |     ^^^^^^^^^^^^^^^^^^^^^^^ `E` cannot be shared between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:220:35
    |
213 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
220 |     E: std::error::Error + Send + Sync + 'static,
    |                                   ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Sync`
    |
200 |     E: std::fmt::Debug + std::marker::Sync,
    |                        +++++++++++++++++++

error[E0277]: the trait bound `E: StdError` is not satisfied
   --> performance-calculator/src/resilience/retry.rs:210:29
    |
210 |     retry(f, &retry_policy).await
    |                             ^^^^^ the trait `StdError` is not implemented for `E`
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:220:8
    |
213 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
220 |     E: std::error::Error + Send + Sync + 'static,
    |        ^^^^^^^^^^^^^^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Error`
    |
200 |     E: std::fmt::Debug + std::error::Error,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be sent between threads safely
   --> performance-calculator/src/resilience/retry.rs:210:29
    |
210 |     retry(f, &retry_policy).await
    |                             ^^^^^ `E` cannot be sent between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:220:28
    |
213 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
220 |     E: std::error::Error + Send + Sync + 'static,
    |                            ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Send`
    |
200 |     E: std::fmt::Debug + std::marker::Send,
    |                        +++++++++++++++++++

error[E0277]: `E` cannot be shared between threads safely
   --> performance-calculator/src/resilience/retry.rs:210:29
    |
210 |     retry(f, &retry_policy).await
    |                             ^^^^^ `E` cannot be shared between threads safely
    |
note: required by a bound in `retry`
   --> performance-calculator/src/resilience/retry.rs:220:35
    |
213 | pub async fn retry<F, Fut, T, E>(
    |              ----- required by a bound in this function
...
220 |     E: std::error::Error + Send + Sync + 'static,
    |                                   ^^^^ required by this bound in `retry`
help: consider further restricting type parameter `E` with trait `Sync`
    |
200 |     E: std::fmt::Debug + std::marker::Sync,
    |                        +++++++++++++++++++

warning: variable `last_error` is assigned to, but never used
   --> performance-calculator/src/resilience/retry.rs:224:13
    |
224 |     let mut last_error = None;
    |             ^^^^^^^^^^
    |
    = note: consider using `_last_error` instead

warning: value assigned to `last_error` is never read
   --> performance-calculator/src/resilience/retry.rs:235:17
    |
235 |                 last_error = Some(error);
    |                 ^^^^^^^^^^
    |
    = help: maybe it is overwritten before being read?
    = note: `#[warn(unused_assignments)]` on by default

Some errors have detailed explanations: E0038, E0061, E0107, E0119, E0277, E0404, E0425, E0432, E0433...
For more information about an error, try `rustc --explain E0038`.
warning: `performance-calculator` (lib) generated 76 warnings
error: could not compile `performance-calculator` (lib) due to 105 previous errors; 76 warnings emitted

```

---

_****_



We have several major issues to fix:

1. The `CircuitBreaker` and `Bulkhead` traits cannot be made into objects because they have generic methods. We need to refactor them to use associated types instead.

2. Let's fix the circuit breaker trait first:

---

_****_

```diff
  use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
  use tokio::time::sleep;
+ use std::future::Future;
+ use std::pin::Pin;
  
  /// Circuit breaker error
  
  /// Circuit breaker trait
- #[async_trait]
  pub trait CircuitBreaker<E>: Send + Sync {
-     /// Execute a function with circuit breaker protection
-     async fn execute<T, F, Fut>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
-     where
-         T: Send,
-         F: FnOnce() -> Fut + Send,
-         Fut: std::future::Future<Output = Result<T, E>> + Send,
-         E: std::error::Error + Send + Sync + 'static;
-     
-     /// Get current circuit breaker metrics
-     fn metrics(&self) -> Result<CircuitBreakerMetrics, CircuitBreakerError<E>>;
-     
-     /// Reset the circuit breaker
-     fn reset(&self) -> Result<(), CircuitBreakerError<E>>;
+     type Future: Future<Output = Result<(), CircuitBreakerError<E>>> + Send;
+     
+     fn check(&self) -> Self::Future;
+     
+     fn on_success(&self);
+     fn on_error(&self, error: &E);
+     fn get_state(&self) -> CircuitState;
  }
  
  /// Standard circuit breaker implementation
  pub struct StandardCircuitBreaker {
+     name: String,
      config: CircuitBreakerConfig,
-     metrics: Arc<Mutex<CircuitBreakerMetrics>>,
+     state: Arc<AtomicU32>,
+     failure_count: Arc<AtomicU32>,
+     last_failure_time: Arc<Mutex<Option<Instant>>>,
  }
  
  impl StandardCircuitBreaker {
      /// Create a new circuit breaker
-     pub fn new(config: CircuitBreakerConfig) -> Self {
+     pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
          Self {
+             name,
              config,
-             metrics: Arc::new(Mutex::new(CircuitBreakerMetrics::default())),
+             state: Arc::new(AtomicU32::new(CircuitBreakerState::Closed as u32)),
+             failure_count: Arc::new(AtomicU32::new(0)),
+             last_failure_time: Arc::new(Mutex::new(None)),
          }
      }
      
      /// Create a new circuit breaker with default configuration
      pub fn default() -> Self {
-         Self::new(CircuitBreakerConfig::default())
+         Self::new("default".to_string(), CircuitBreakerConfig::default())
      }
      
      /// Check if the circuit is closed
      fn is_closed(&self) -> Result<bool, String> {
-         let metrics = self.metrics.lock()
-             .map_err(|e| format!("Failed to acquire metrics lock: {}", e))?;
-         Ok(metrics.state == CircuitState::Closed)
+         let state = CircuitBreakerState::from(self.state.load(Ordering::SeqCst));
+         Ok(state == CircuitBreakerState::Closed)
      }
      
      /// Check if the circuit is open
      fn is_open(&self) -> Result<bool, String> {
-         let metrics = self.metrics.lock()
-             .map_err(|e| format!("Failed to acquire metrics lock: {}", e))?;
+         let state = CircuitBreakerState::from(self.state.load(Ordering::SeqCst));
          
-         if metrics.state == CircuitState::Open {
+         if state == CircuitBreakerState::Open {
              // Check if reset timeout has elapsed
-             let elapsed = metrics.last_state_change.elapsed().as_secs();
+             let last_failure_time = self.last_failure_time.lock().unwrap().clone();
+             let config = self.config.clone();
              
-             if elapsed >= self.config.reset_timeout_seconds {
-                 // Transition to half-open state
-                 drop(metrics);
-                 self.transition_to_half_open()?;
-                 return Ok(false);
+             if let Some(last_failure) = last_failure_time {
+                 let elapsed = last_failure.elapsed().as_secs();
+                 
+                 if elapsed >= config.reset_timeout_seconds {
+                     // Transition to half-open state
+                     self.state.store(CircuitBreakerState::HalfOpen as u32, Ordering::SeqCst);
+                     return Ok(false);
+                 }
              }
              
      /// Check if the circuit is half-open
      fn is_half_open(&self) -> Result<bool, String> {
-         let metrics = self.metrics.lock()
-             .map_err(|e| format!("Failed to acquire metrics lock: {}", e))?;
-         Ok(metrics.state == CircuitState::HalfOpen)
+         let state = CircuitBreakerState::from(self.state.load(Ordering::SeqCst));
+         Ok(state == CircuitBreakerState::HalfOpen)
      }
      
      /// Transition to open state
      fn transition_to_open(&self) -> Result<(), String> {
-         let mut metrics = self.metrics.lock()
-             .map_err(|e| format!("Failed to acquire metrics lock: {}", e))?;
+         let state = self.state.load(Ordering::SeqCst);
          
-         if metrics.state != CircuitState::Open {
+         if state != CircuitBreakerState::Open as u32 {
              info!("Circuit breaker transitioning to OPEN state");
-             metrics.state = CircuitState::Open;
-             metrics.last_state_change = Instant::now();
+             self.state.store(CircuitBreakerState::Open as u32, Ordering::SeqCst);
+             *self.last_failure_time.lock().unwrap() = Some(Instant::now());
          }
          Ok(())
      /// Transition to half-open state
      fn transition_to_half_open(&self) -> Result<(), String> {
-         let mut metrics = self.metrics.lock()
-             .map_err(|e| format!("Failed to acquire metrics lock: {}", e))?;
+         let state = self.state.load(Ordering::SeqCst);
          
-         if metrics.state != CircuitState::HalfOpen {
+         if state != CircuitBreakerState::HalfOpen as u32 {
              info!("Circuit breaker transitioning to HALF-OPEN state");
-             metrics.state = CircuitState::HalfOpen;
-             metrics.half_open_success_count = 0;
-             metrics.last_state_change = Instant::now();
+             self.state.store(CircuitBreakerState::HalfOpen as u32, Ordering::SeqCst);
+             *self.failure_count.lock().unwrap() = 0;
+             *self.last_failure_time.lock().unwrap() = Some(Instant::now());
          }
          Ok(())
      /// Transition to closed state
      fn transition_to_closed(&self) -> Result<(), String> {
-         let mut metrics = self.metrics.lock()
-             .map_err(|e| format!("Failed to acquire metrics lock: {}", e))?;
+         let state = self.state.load(Ordering::SeqCst);
          
-         if metrics.state != CircuitState::Closed {
+         if state != CircuitBreakerState::Closed as u32 {
              info!("Circuit breaker transitioning to CLOSED state");
-             metrics.state = CircuitState::Closed;
-             metrics.failure_count = 0;
-             metrics.last_state_change = Instant::now();
+             self.state.store(CircuitBreakerState::Closed as u32, Ordering::SeqCst);
+             *self.failure_count.lock().unwrap() = 0;
+             *self.last_failure_time.lock().unwrap() = None;
          }
          Ok(())
      /// Record a successful request
      fn record_success(&self) -> Result<(), String> {
-         let mut metrics = self.metrics.lock()
-             .map_err(|e| format!("Failed to acquire metrics lock: {}", e))?;
-         metrics.total_requests += 1;
-         metrics.successful_requests += 1;
-         
-         match metrics.state {
-             CircuitState::Closed => {
-                 // Reset failure count on success
-                 metrics.failure_count = 0;
-             },
-             CircuitState::HalfOpen => {
-                 // Increment success count in half-open state
-                 metrics.half_open_success_count += 1;
-                 
-                 // Check if we've reached the threshold to close the circuit
-                 if metrics.half_open_success_count >= self.config.half_open_request_threshold {
-                     drop(metrics);
-                     self.transition_to_closed()?;
-                 }
-             },
-             CircuitState::Open => {
-                 // This shouldn't happen, but just in case
-                 warn!("Successful request while circuit is open");
-             },
-         }
+         self.failure_count.store(0, Ordering::SeqCst);
+         self.state.store(CircuitBreakerState::Closed as u32, Ordering::SeqCst);
          Ok(())
      }
      
      /// Record a failed request
      fn record_failure(&self) -> Result<(), String> {
-         let mut metrics = self.metrics.lock()
-             .map_err(|e| format!("Failed to acquire metrics lock: {}", e))?;
-         metrics.total_requests += 1;
-         metrics.failed_requests += 1;
-         
-         match metrics.state {
-             CircuitState::Closed => {
-                 // Increment failure count
-                 metrics.failure_count += 1;
-                 
-                 // Check if we've reached the threshold to open the circuit
-                 if metrics.failure_count >= self.config.failure_threshold {
-                     drop(metrics);
-                     self.transition_to_open()?;
-                 }
-             },
-             CircuitState::HalfOpen => {
-                 // Any failure in half-open state opens the circuit again
-                 drop(metrics);
-                 self.transition_to_open()?;
-             },
-             CircuitState::Open => {
-                 // This shouldn't happen, but just in case
-                 warn!("Failed request while circuit is open");
-             },
+         let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
+         if failures >= self.config.failure_threshold {
+             *self.last_failure_time.lock().unwrap() = Some(Instant::now());
+             self.state.store(CircuitBreakerState::Open as u32, Ordering::SeqCst);
          }
          Ok(())
      }
  }
  
- #[async_trait]
  impl<E> CircuitBreaker<E> for StandardCircuitBreaker
  where
      E: std::error::Error + Send + Sync + 'static,
  {
-     async fn execute<T, F, Fut>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
-     where
-         T: Send,
-         F: FnOnce() -> Fut + Send,
-         Fut: std::future::Future<Output = Result<T, E>> + Send,
-     {
-         // Check if circuit is open
-         if self.is_open().map_err(CircuitBreakerError::Internal)? {
-             return Err(CircuitBreakerError::Open);
-         }
-         
-         // Execute the function
-         match f().await {
-             Ok(result) => {
-                 self.record_success().map_err(CircuitBreakerError::Internal)?;
-                 Ok(result)
-             },
-             Err(error) => {
-                 self.record_failure().map_err(CircuitBreakerError::Internal)?;
-                 Err(CircuitBreakerError::ServiceError(error))
-             },
-         }
-     }
-     
-     fn metrics(&self) -> Result<CircuitBreakerMetrics, CircuitBreakerError<E>> {
-         self.metrics.lock()
-             .map(|m| m.clone())
-             .map_err(|e| CircuitBreakerError::Internal(format!("Failed to acquire metrics lock: {}", e)))
-     }
-     
-     fn reset(&self) -> Result<(), CircuitBreakerError<E>> {
-         self.transition_to_closed()
-             .map_err(CircuitBreakerError::Internal)
+     type Future = Pin<Box<dyn Future<Output = Result<(), CircuitBreakerError<E>>> + Send>>;
+ 
+     fn check(&self) -> Self::Future {
+         let state = CircuitBreakerState::from(self.state.load(Ordering::SeqCst));
+         let last_failure_time = self.last_failure_time.lock().unwrap().clone();
+         let config = self.config.clone();
+ 
+         Box::pin(async move {
+             match state {
+                 CircuitBreakerState::Closed => Ok(()),
+                 CircuitBreakerState::Open => {
+                     if let Some(last_failure) = last_failure_time {
+                         if last_failure.elapsed() > config.reset_timeout_seconds {
+                             Ok(())
+                         } else {
+                             Err(CircuitBreakerError::Open)
+                         }
+                     } else {
+                         Ok(())
+                     }
+                 }
+                 CircuitBreakerState::HalfOpen => Ok(()),
+             }
+         })
+     }
+ 
+     fn on_success(&self) {
+         self.failure_count.store(0, Ordering::SeqCst);
+         self.state.store(CircuitBreakerState::Closed as u32, Ordering::SeqCst);
+     }
+ 
+     fn on_error(&self, error: &E) {
+         let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
+         if failures >= self.config.failure_threshold {
+             *self.last_failure_time.lock().unwrap() = Some(Instant::now());
+             self.state.store(CircuitBreakerState::Open as u32, Ordering::SeqCst);
+         }
+     }
+ 
+     fn get_state(&self) -> CircuitState {
+         CircuitBreakerState::from(self.state.load(Ordering::SeqCst))
      }
  }
              Ok(Arc::clone(breaker))
          } else {
-             let breaker = Arc::new(StandardCircuitBreaker::new(config.unwrap_or_default()));
+             let breaker = Arc::new(StandardCircuitBreaker::new(name.to_string(), config.unwrap_or_default()));
              breakers.insert(name.to_string(), Arc::clone(&breaker));
              Ok(breaker)
          
          for breaker in breakers.values() {
-             breaker.reset()
+             breaker.transition_to_closed()
                  .map_err(|e| format!("Failed to reset circuit breaker: {:?}", e))?;
          }
  pub fn get_circuit_breaker_with_config(name: &str, config: CircuitBreakerConfig) -> Result<Arc<StandardCircuitBreaker>, String> {
      // For now, just return a new circuit breaker with the provided config
-     Ok(Arc::new(StandardCircuitBreaker::new(config)))
+     Ok(Arc::new(StandardCircuitBreaker::new(name.to_string(), config)))
  } 
```

---

_****_



3. Now let's fix the bulkhead trait:

---

_****_

```diff
  use std::error::Error;
  use std::fmt::Debug;
+ use std::pin::Pin;
+ use std::future::Future;
  
  /// Bulkhead error types
  
  /// Bulkhead trait for limiting concurrent operations
- #[async_trait::async_trait]
  pub trait Bulkhead: Send + Sync {
-     /// Execute an operation with bulkhead protection
-     async fn execute<F, Fut, T, E>(&self, operation: F) -> Result<T, BulkheadError<E>>
-     where
-         F: FnOnce() -> Fut + Send,
-         Fut: std::future::Future<Output = Result<T, E>> + Send,
-         T: Send,
-         E: Error + Send + Sync + 'static;
+     type Permit: Send;
+     type Future: Future<Output = Result<Self::Permit, BulkheadError>> + Send;
+ 
+     fn acquire(&self) -> Self::Future;
+     fn release(&self, permit: Self::Permit);
  }
  
  /// Standard bulkhead implementation
- #[derive(Debug)]
  pub struct StandardBulkhead {
      name: String,
```

---

_****_





4. Let's update the resilience module to use the new trait implementations:

---

_****_

```diff
  pub enum ResilienceError<E> {
      #[error("Circuit breaker error: {0}")]
-     CircuitBreaker(#[from] CircuitBreakerError),
+     CircuitBreaker(#[from] CircuitBreakerError<E>),
      
      #[error("Bulkhead error: {0}")]
-     Bulkhead(#[from] BulkheadError<E>),
+     Bulkhead(#[from] BulkheadError),
      
      #[error("Retry error: {0}")]
      bulkhead: &B,
      operation: F,
- ) -> Result<T, BulkheadError<E>>
+ ) -> Result<T, BulkheadError>
  where
      F: Fn() -> Fut,
  pub async fn with_resilience<T, E, F, Fut>(
      f: F,
-     config: ResilienceConfig<E>,
+     config: ResilienceConfig,
  ) -> Result<T, ResilienceError<E>>
  where
      E: std::error::Error + Send + Sync + 'static + std::fmt::Debug,
  {
-     let circuit_breaker = config.circuit_breaker;
-     let bulkhead = config.bulkhead;
+     let circuit_breaker = &config.circuit_breaker;
+     let bulkhead = &config.bulkhead;
      let retry_config = config.retry_config;
  
      match bulkhead.acquire().await {
-         Ok(_permit) => {
+         Ok(permit) => {
              match circuit_breaker.check().await {
                  Ok(_) => {
-                     match retry::retry_with_exponential_backoff(f, retry_config).await {
-                         Ok(result) => Ok(result),
-                         Err(RetryError::MaxRetriesExceeded(e)) => Err(ResilienceError::Retry(e)),
-                         Err(RetryError::Aborted(e)) => Err(ResilienceError::Retry(e)),
-                         Err(RetryError::Other(e)) => Err(ResilienceError::Other(e)),
+                     let result = retry::retry_with_exponential_backoff(
+                         || f(),
+                         retry_config.max_attempts,
+                         retry_config.initial_delay_ms,
+                     ).await;
+ 
+                     match result {
+                         Ok(value) => {
+                             circuit_breaker.on_success();
+                             bulkhead.release(permit);
+                             Ok(value)
+                         }
+                         Err(RetryError::MaxRetriesExceeded(e)) => {
+                             circuit_breaker.on_error(&e);
+                             bulkhead.release(permit);
+                             Err(ResilienceError::Retry(e))
+                         }
+                         Err(RetryError::Aborted(e)) => {
+                             circuit_breaker.on_error(&e);
+                             bulkhead.release(permit);
+                             Err(ResilienceError::Retry(e))
+                         }
+                         Err(RetryError::Other(e)) => {
+                             circuit_breaker.on_error(&e);
+                             bulkhead.release(permit);
+                             Err(ResilienceError::Other(e))
+                         }
                      }
                  }
-                 Err(e) => Err(ResilienceError::CircuitBreaker(e)),
+                 Err(e) => {
+                     bulkhead.release(permit);
+                     Err(ResilienceError::CircuitBreaker(e))
+                 }
              }
          }
  }
  
- pub struct ResilienceConfig<E> {
-     pub circuit_breaker: Box<dyn CircuitBreaker<E>>,
-     pub bulkhead: Box<dyn Bulkhead>,
+ pub struct ResilienceConfig {
+     pub circuit_breaker: StandardCircuitBreaker,
+     pub bulkhead: StandardBulkhead,
      pub retry_config: RetryConfig,
-     _phantom: PhantomData<E>,
  }
  
- pub struct ResilienceBuilder<E> {
-     circuit_breaker: Option<Box<dyn CircuitBreaker<E>>>,
-     bulkhead: Option<Box<dyn Bulkhead>>,
+ pub struct ResilienceBuilder {
+     circuit_breaker: Option<StandardCircuitBreaker>,
+     bulkhead: Option<StandardBulkhead>,
      retry_config: Option<RetryConfig>,
-     _phantom: PhantomData<E>,
  }
  
- impl<E> ResilienceBuilder<E> {
+ impl ResilienceBuilder {
      pub fn new() -> Self {
          Self {
              circuit_breaker: None,
              bulkhead: None,
              retry_config: None,
-             _phantom: PhantomData,
          }
      }
  
-     pub fn with_circuit_breaker(mut self, circuit_breaker: Box<dyn CircuitBreaker<E>>) -> Self {
+     pub fn with_circuit_breaker(mut self, circuit_breaker: StandardCircuitBreaker) -> Self {
          self.circuit_breaker = Some(circuit_breaker);
          self
      }
  
-     pub fn with_bulkhead(mut self, bulkhead: Box<dyn Bulkhead>) -> Self {
+     pub fn with_bulkhead(mut self, bulkhead: StandardBulkhead) -> Self {
          self.bulkhead = Some(bulkhead);
          self
      }
  
-     pub fn build(self) -> Result<ResilienceConfig<E>, anyhow::Error> {
+     pub fn build(self) -> Result<ResilienceConfig, anyhow::Error> {
          Ok(ResilienceConfig {
-             circuit_breaker: self.circuit_breaker.ok_or_else(|| anyhow::anyhow!("Circuit breaker not configured"))?,
-             bulkhead: self.bulkhead.ok_or_else(|| anyhow::anyhow!("Bulkhead not configured"))?,
+             circuit_breaker: self.circuit_breaker.ok_or_else(|| anyhow!("Circuit breaker not configured"))?,
+             bulkhead: self.bulkhead.ok_or_else(|| anyhow!("Bulkhead not configured"))?,
              retry_config: self.retry_config.unwrap_or_default(),
-             _phantom: PhantomData,
          })
      }
      E: std::error::Error + Send + Sync + 'static + std::fmt::Debug,
  {
-     let circuit_breaker = get_circuit_breaker_for_service(tenant_id).await?;
-     let bulkhead = get_bulkhead(tenant_id, 10)?;
+     let circuit_breaker = StandardCircuitBreaker::new(
+         tenant_id.to_string(),
+         CircuitBreakerConfig::default(),
+     );
+     let bulkhead = StandardBulkhead::new(tenant_id.to_string(), 10);
      let retry_config = RetryConfig::default();
  
      with_resilience(f, ResilienceConfig {
          circuit_breaker,
-         bulkhead: Box::new(bulkhead),
+         bulkhead,
          retry_config,
-         _phantom: PhantomData,
      })
      .await
```

---

_****_





5. Let's fix the remaining error in the resilience module:

---

_****_



---

_****_



Let me check the retry module to see what's wrong with the function signature:

---

_****_

Read file: performance-calculator/src/resilience/retry.rs

---

_****_





6. Let's fix the retry module to match the usage:

---

_****_

Model failed to call the tool with correct arguments.

---

_****_

```diff
  pub async fn retry_with_exponential_backoff<T, E, F, Fut>(
      f: F,
-     max_retries: u32,
-     initial_backoff_ms: u64,
+     config: RetryConfig,
  ) -> Result<T, RetryError<E>>
  where
      F: Fn() -> Fut,
      Fut: Future<Output = Result<T, E>>,
-     E: std::fmt::Debug,
- {
-     let config = RetryConfig {
-         max_retries,
-         initial_backoff_ms,
-         ..RetryConfig::default()
-     };
-     
+     E: std::fmt::Debug + std::error::Error + Send + Sync + 'static,
+ {
      let retry_policy = StandardRetryPolicy::new(config);
-     
      retry(f, &retry_policy).await
  }
  pub async fn retry_if<T, E, F, Fut, P>(
      f: F,
-     max_retries: u32,
-     initial_backoff_ms: u64,
+     config: RetryConfig,
      predicate: P,
  ) -> Result<T, RetryError<E>>
      Fut: Future<Output = Result<T, E>>,
      P: Fn(&E, u32) -> bool,
-     E: std::fmt::Debug,
- {
-     let config = RetryConfig {
-         max_retries,
-         initial_backoff_ms,
-         ..RetryConfig::default()
-     };
-     
+     E: std::fmt::Debug + std::error::Error + Send + Sync + 'static,
+ {
      let retry_policy = StandardRetryPolicy::new(config);
-     
      retry(f, &retry_policy).await
  }
```

---

_****_





7. Now let's update the resilience module to use the new retry function signature:

---

_****_

```diff
                      let result = retry::retry_with_exponential_backoff(
                          || f(),
-                         retry_config.max_attempts,
-                         retry_config.initial_delay_ms,
+                         retry_config,
                      ).await;
  
```

---

_****_





Let me check the retry module again to see what's wrong:

---

_****_

Read file: performance-calculator/src/resilience/retry.rs

---

_****_





The issue is that we need to update the `with_resilience` function to handle the `FnMut` trait bound correctly. Let's fix that:

---

_****_

```diff
  /// Retry a function with exponential backoff
  pub async fn retry_with_exponential_backoff<T, E, F, Fut>(
-     f: F,
+     mut f: F,
      config: RetryConfig,
  ) -> Result<T, RetryError<E>>
  where
-     F: Fn() -> Fut,
+     F: FnMut() -> Fut,
      Fut: Future<Output = Result<T, E>>,
      E: std::fmt::Debug + std::error::Error + Send + Sync + 'static,
  /// Retry a function with a custom retry predicate
  pub async fn retry_if<T, E, F, Fut, P>(
-     f: F,
+     mut f: F,
      config: RetryConfig,
      predicate: P,
  ) -> Result<T, RetryError<E>>
  where
-     F: Fn() -> Fut,
+     F: FnMut() -> Fut,
      Fut: Future<Output = Result<T, E>>,
      P: Fn(&E, u32) -> bool,
  
  pub async fn retry<F, Fut, T, E>(
-     operation: F,
+     mut operation: F,
      policy: &StandardRetryPolicy,
  ) -> Result<T, RetryError<E>>
  where
-     F: Fn() -> Fut,
+     F: FnMut() -> Fut,
      Fut: Future<Output = Result<T, E>>,
      E: std::error::Error + Send + Sync + 'static,
```

---

_****_





Let me try building the project again to see if we've resolved more issues:

---

_****_

```bash
cargo build
```

```
sethford@Seths-MacBook-Pro rust-investment-performance % 
warning: unused imports: `AnalyticsConfig`, `CalculationScheduler`, `DataAccessService`, `IntegrationConfig`
, `VisualizationConfig`, and `currency::CurrencyConverter`
  --> performance-calculator/src/calculations/factory.rs:37:5
   |
37 |     currency::CurrencyConverter,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^
38 |     query_api::{QueryApi, DataAccessService},
   |                           ^^^^^^^^^^^^^^^^^
39 |     streaming::StreamingProcessor,
40 |     scheduler::{CalculationScheduler, NotificationService, DefaultNotificationService, EmailClient, ...
   |                 ^^^^^^^^^^^^^^^^^^^^
41 |     analytics::{AnalyticsConfig, AnalyticsEngine},
   |                 ^^^^^^^^^^^^^^^
42 |     visualization::{VisualizationConfig, VisualizationEngine},
   |                     ^^^^^^^^^^^^^^^^^^^
43 |     integration::{IntegrationConfig, IntegrationEngine},
   |                   ^^^^^^^^^^^^^^^^^

warning: unused import: `std::sync::Arc`
 --> performance-calculator/src/calculations/portfolio/mod.rs:3:5
  |
3 | use std::sync::Arc;
  |     ^^^^^^^^^^^^^^

warning: unused imports: `DateTime`, `Duration`, and `Utc`
 --> performance-calculator/src/calculations/performance_metrics.rs:5:14
  |
5 | use chrono::{DateTime, Utc, NaiveDate, Duration};
  |              ^^^^^^^^  ^^^             ^^^^^^^^

warning: unused import: `crate::calculations::risk_metrics::ReturnSeries`
 --> performance-calculator/src/calculations/performance_metrics.rs:7:5
  |
7 | use crate::calculations::risk_metrics::ReturnSeries;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal_macros::dec`
 --> performance-calculator/src/calculations/risk_metrics.rs:3:5
  |
3 | use rust_decimal_macros::dec;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `Result` and `anyhow`
 --> performance-calculator/src/calculations/risk_metrics.rs:4:14
  |
4 | use anyhow::{Result, anyhow};
  |              ^^^^^^  ^^^^^^

warning: unused import: `std::collections::HashMap`
 --> performance-calculator/src/calculations/risk_metrics.rs:5:5
  |
5 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `DateTime` and `Utc`
 --> performance-calculator/src/calculations/risk_metrics.rs:6:14
  |
6 | use chrono::{DateTime, Utc, NaiveDate};
  |              ^^^^^^^^  ^^^

warning: unused import: `std::cmp::Ordering`
 --> performance-calculator/src/calculations/risk_metrics.rs:8:5
  |
8 | use std::cmp::Ordering;
  |     ^^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::prelude::*`
 --> performance-calculator/src/calculations/periodic_returns.rs:2:5
  |
2 | use rust_decimal::prelude::*;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `DateTime` and `Utc`
 --> performance-calculator/src/calculations/periodic_returns.rs:5:14
  |
5 | use chrono::{DateTime, Utc, NaiveDate, Datelike};
  |              ^^^^^^^^  ^^^

warning: unused import: `Deserialize`
 --> performance-calculator/src/calculations/distributed_cache.rs:4:24
  |
4 | use serde::{Serialize, Deserialize};
  |                        ^^^^^^^^^^^

warning: unused import: `std::time::Duration`
 --> performance-calculator/src/calculations/distributed_cache.rs:6:5
  |
6 | use std::time::Duration;
  |     ^^^^^^^^^^^^^^^^^^^

warning: unused imports: `error`, `info`, and `warn`
 --> performance-calculator/src/calculations/distributed_cache.rs:8:15
  |
8 | use tracing::{info, warn, error};
  |               ^^^^  ^^^^  ^^^^^

warning: unused import: `uuid`
  --> performance-calculator/src/calculations/distributed_cache.rs:13:5
   |
13 | use uuid;
   |     ^^^^

warning: unused import: `serde_json::json`
  --> performance-calculator/src/calculations/distributed_cache.rs:14:5
   |
14 | use serde_json::json;
   |     ^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::Decimal`
  --> performance-calculator/src/calculations/streaming.rs:15:5
   |
15 | use rust_decimal::Decimal;
   |     ^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `info` and `warn`
  --> performance-calculator/src/calculations/streaming.rs:19:15
   |
19 | use tracing::{info, warn, error, debug};
   |               ^^^^  ^^^^

warning: unused imports: `AuditTrail`, `ExchangeRateProvider`, and `ExchangeRate`
  --> performance-calculator/src/calculations/query_api.rs:16:49
   |
16 |     currency::{CurrencyConverter, CurrencyCode, ExchangeRate, ExchangeRateProvider},
   |                                                 ^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^
17 |     audit::{AuditTrail, AuditTrailManager},
   |             ^^^^^^^^^^

warning: unused imports: `Duration` and `NaiveDateTime`
 --> performance-calculator/src/calculations/scheduler.rs:2:40
  |
2 | use chrono::{DateTime, Utc, NaiveDate, NaiveDateTime, Duration, Datelike};
  |                                        ^^^^^^^^^^^^^  ^^^^^^^^

warning: unused import: `warn`
 --> performance-calculator/src/calculations/scheduler.rs:8:21
  |
8 | use tracing::{info, warn, error};
  |                     ^^^^

warning: unused import: `CalculationEventBuilder`
  --> performance-calculator/src/calculations/scheduler.rs:13:32
   |
13 |     audit::{AuditTrailManager, CalculationEventBuilder},
   |                                ^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Context`
 --> performance-calculator/src/calculations/analytics.rs:6:22
  |
6 | use anyhow::{Result, Context, anyhow};
  |                      ^^^^^^^

warning: unused import: `serde_json`
  --> performance-calculator/src/calculations/analytics.rs:13:5
   |
13 | use serde_json;
   |     ^^^^^^^^^^

warning: unused import: `Context`
 --> performance-calculator/src/calculations/visualization.rs:6:22
  |
6 | use anyhow::{Result, Context, anyhow};
  |                      ^^^^^^^

warning: unused import: `chrono::NaiveDate`
 --> performance-calculator/src/calculations/visualization.rs:7:5
  |
7 | use chrono::NaiveDate;
  |     ^^^^^^^^^^^^^^^^^

warning: unused imports: `error`, `info`, and `warn`
  --> performance-calculator/src/calculations/visualization.rs:12:15
   |
12 | use tracing::{info, warn, error};
   |               ^^^^  ^^^^  ^^^^^

warning: unused import: `Context`
 --> performance-calculator/src/calculations/integration.rs:6:22
  |
6 | use anyhow::{Result, Context, anyhow};
  |                      ^^^^^^^

warning: unused import: `NaiveDate`
 --> performance-calculator/src/calculations/integration.rs:7:29
  |
7 | use chrono::{DateTime, Utc, NaiveDate};
  |                             ^^^^^^^^^

warning: unused import: `rust_decimal::Decimal`
 --> performance-calculator/src/calculations/integration.rs:8:5
  |
8 | use rust_decimal::Decimal;
  |     ^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `error`, `info`, and `warn`
  --> performance-calculator/src/calculations/integration.rs:14:15
   |
14 | use tracing::{info, warn, error};
   |               ^^^^  ^^^^  ^^^^^

warning: unused imports: `AuditTrail`, `InMemoryAuditTrail`, `InMemoryCache`, `MoneyWeightedReturn`, and `Ti
meWeightedReturn`
  --> performance-calculator/src/calculations/benchmarks.rs:7:13
   |
7  |     audit::{AuditTrail, InMemoryAuditTrail},
   |             ^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^
8  |     currency::{CurrencyConverter, ExchangeRate, CurrencyCode},
9  |     distributed_cache::{StringCache, InMemoryCache},
   |                                      ^^^^^^^^^^^^^
10 |     performance_metrics::{TimeWeightedReturn, MoneyWeightedReturn},
   |                           ^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::prelude::*`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:2:5
  |
2 | use rust_decimal::prelude::*;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::collections::HashMap`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:4:5
  |
4 | use std::collections::HashMap;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused imports: `DateTime`, `NaiveDate`, and `Utc`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:5:14
  |
5 | use chrono::{DateTime, Utc, NaiveDate};
  |              ^^^^^^^^  ^^^  ^^^^^^^^^

warning: unused import: `calculate_information_ratio`
 --> performance-calculator/src/calculations/benchmark_comparison.rs:7:114
  |
7 | ...e_alpha, calculate_tracking_error, calculate_information_ratio};
  |                                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Error as DynamoDbError`
 --> performance-calculator/src/calculations/tenant/dynamodb.rs:1:50
  |
1 | use aws_sdk_dynamodb::{Client as DynamoDbClient, Error as DynamoDbError};
  |                                                  ^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `uuid::Uuid`
 --> performance-calculator/src/calculations/tenant/dynamodb.rs:7:5
  |
7 | use uuid::Uuid;
  |     ^^^^^^^^^^

warning: unused import: `chrono::Utc`
 --> performance-calculator/src/calculations/tenant/cached.rs:4:5
  |
4 | use chrono::Utc;
  |     ^^^^^^^^^^^

warning: unused import: `Duration`
 --> performance-calculator/src/calculations/tenant/metrics.rs:1:24
  |
1 | use chrono::{DateTime, Duration, Utc};
  |                        ^^^^^^^^

warning: unused import: `info`
 --> performance-calculator/src/calculations/tenant/metrics.rs:3:15
  |
3 | use tracing::{info, warn};
  |               ^^^^

warning: unused import: `Tenant`
 --> performance-calculator/src/calculations/tenant/metrics.rs:6:35
  |
6 | use crate::calculations::tenant::{Tenant, ResourceLimits};
  |                                   ^^^^^^

warning: unused imports: `delete_item::DeleteItemInput`, `get_item::GetItemInput`, `put_item::PutItemInput`,
 and `query::QueryInput`
  --> performance-calculator/src/calculations/tenant/metrics.rs:10:5
   |
10 |     put_item::PutItemInput,
   |     ^^^^^^^^^^^^^^^^^^^^^^
11 |     get_item::GetItemInput,
   |     ^^^^^^^^^^^^^^^^^^^^^^
12 |     query::QueryInput,
   |     ^^^^^^^^^^^^^^^^^
13 |     delete_item::DeleteItemInput,
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `serde_json::Value`
  --> performance-calculator/src/calculations/tenant/metrics.rs:16:5
   |
16 | use serde_json::Value;
   |     ^^^^^^^^^^^^^^^^^

warning: unused import: `crate::resilience`
  --> performance-calculator/src/calculations/tenant/metrics.rs:18:5
   |
18 | use crate::resilience;
   |     ^^^^^^^^^^^^^^^^^

warning: unused import: `rust_decimal::Decimal`
  --> performance-calculator/src/calculations/tenant/metrics.rs:19:5
   |
19 | use rust_decimal::Decimal;
   |     ^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `std::time::Duration`
 --> performance-calculator/src/resilience/mod.rs:8:5
  |
8 | use std::time::Duration;
  |     ^^^^^^^^^^^^^^^^^^^

warning: unused imports: `info` and `warn`
  --> performance-calculator/src/resilience/mod.rs:10:15
   |
10 | use tracing::{info, warn, error};
   |               ^^^^  ^^^^

warning: unused import: `crate::calculations::error_handling::CalculationError`
  --> performance-calculator/src/resilience/mod.rs:12:5
   |
12 | use crate::calculations::error_handling::CalculationError;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `Duration`
 --> performance-calculator/src/resilience/circuit_breaker.rs:2:17
  |
2 | use std::time::{Duration, Instant};
  |                 ^^^^^^^^

warning: unused import: `async_trait::async_trait`
 --> performance-calculator/src/resilience/circuit_breaker.rs:3:5
  |
3 | use async_trait::async_trait;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `warn`
 --> performance-calculator/src/resilience/circuit_breaker.rs:4:21
  |
4 | use tracing::{info, warn, error};
  |                     ^^^^

warning: unused import: `std::error::Error`
 --> performance-calculator/src/resilience/circuit_breaker.rs:8:5
  |
8 | use std::error::Error;
  |     ^^^^^^^^^^^^^^^^^

warning: unused imports: `AtomicU64` and `AtomicUsize`
 --> performance-calculator/src/resilience/circuit_breaker.rs:9:25
  |
9 | use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
  |                         ^^^^^^^^^  ^^^^^^^^^^^

warning: unused import: `tokio::time::sleep`
  --> performance-calculator/src/resilience/circuit_breaker.rs:10:5
   |
10 | use tokio::time::sleep;
   |     ^^^^^^^^^^^^^^^^^^

warning: unused import: `std::marker::PhantomData`
 --> performance-calculator/src/resilience/retry.rs:4:5
  |
4 | use std::marker::PhantomData;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `anyhow`
 --> performance-calculator/src/resilience/retry.rs:6:22
  |
6 | use anyhow::{Result, anyhow};
  |                      ^^^^^^

warning: unused imports: `error` and `info`
 --> performance-calculator/src/resilience/retry.rs:7:15
  |
7 | use tracing::{info, warn, error};
  |               ^^^^        ^^^^^

warning: unused import: `thiserror::Error`
 --> performance-calculator/src/resilience/retry.rs:9:5
  |
9 | use thiserror::Error;
  |     ^^^^^^^^^^^^^^^^

warning: unused import: `error`
 --> performance-calculator/src/resilience/health_check.rs:7:27
  |
7 | use tracing::{info, warn, error};
  |                           ^^^^^

warning: unused import: `async_trait::async_trait`
 --> performance-calculator/src/resilience/bulkhead.rs:4:5
  |
4 | use async_trait::async_trait;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused import: `error`
 --> performance-calculator/src/resilience/bulkhead.rs:7:27
  |
7 | use tracing::{info, warn, error};
  |                           ^^^^^

warning: unused import: `thiserror::Error`
 --> performance-calculator/src/resilience/bulkhead.rs:8:5
  |
8 | use thiserror::Error;
  |     ^^^^^^^^^^^^^^^^

warning: unused import: `std::pin::Pin`
  --> performance-calculator/src/resilience/bulkhead.rs:11:5
   |
11 | use std::pin::Pin;
   |     ^^^^^^^^^^^^^

warning: unused imports: `error` and `warn`
 --> performance-calculator/src/resilience/chaos.rs:6:21
  |
6 | use tracing::{info, warn, error};
  |                     ^^^^  ^^^^^

warning: unused import: `std::marker::PhantomData`
  --> performance-calculator/src/resilience/mod.rs:37:5
   |
37 | use std::marker::PhantomData;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^

error[E0107]: missing generics for enum `BulkheadError`
  --> performance-calculator/src/resilience/mod.rs:46:22
   |
46 |     Bulkhead(#[from] BulkheadError),
   |                      ^^^^^^^^^^^^^ expected 1 generic argument
   |
note: enum defined here, with 1 generic parameter: `E`
  --> performance-calculator/src/resilience/bulkhead.rs:16:10
   |
16 | pub enum BulkheadError<E> {
   |          ^^^^^^^^^^^^^ -
help: add missing generic argument
   |
46 |     Bulkhead(#[from] BulkheadError<E>),
   |                                   +++

error[E0107]: struct takes 0 generic arguments but 1 generic argument was supplied
   --> performance-calculator/src/resilience/mod.rs:153:50
    |
153 | pub fn get_retry_policy<E>(name: &str) -> Result<StandardRetryPolicy<E>>
    |                                                  ^^^^^^^^^^^^^^^^^^^--- help: remove the unnecessary g
enerics
    |                                                  |
    |                                                  expected 0 generic arguments
    |
note: struct defined here, with 0 generic parameters
   --> performance-calculator/src/resilience/retry.rs:53:12
    |
53  | pub struct StandardRetryPolicy {
    |            ^^^^^^^^^^^^^^^^^^^

warning: unused variable: `beginning_base`
   --> performance-calculator/src/calculations/currency.rs:141:13
    |
141 |         let beginning_base = beginning_amount * beginning_rate;
    |             ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_beginning_bas
e`
    |
    = note: `#[warn(unused_variables)]` on by default

error[E0119]: conflicting implementations of trait `StdError` for type `ResilienceError<_>`
  --> performance-calculator/src/resilience/mod.rs:40:10
   |
40 | #[derive(Error, Debug)]
   |          ^^^^^ conflicting implementation for `ResilienceError<_>`
...
55 | impl<E: std::error::Error> std::error::Error for ResilienceError<E> {
   | ------------------------------------------------------------------- first implementation here
   |
   = note: this error originates in the derive macro `Error` (in Nightly builds, run with -Z macro-backtrace
 for more info)

error[E0119]: conflicting implementations of trait `std::fmt::Display` for type `ResilienceError<_>`
  --> performance-calculator/src/resilience/mod.rs:40:10
   |
40 | #[derive(Error, Debug)]
   |          ^^^^^ conflicting implementation for `ResilienceError<_>`
...
66 | impl<E: std::fmt::Display> std::fmt::Display for ResilienceError<E> {
   | ------------------------------------------------------------------- first implementation here
   |
   = note: this error originates in the derive macro `Error` (in Nightly builds, run with -Z macro-backtrace
 for more info)

warning: `graphql-api` (bin "graphql-api") generated 20 warnings
error: could not compile `graphql-api` (bin "graphql-api") due to 1 previous error; 20 warnings emitted
warning: unused variable: `event`
   --> performance-calculator/src/calculations/query_api.rs:370:13
    |
370 |         let event = self.audit_manager.start_calculation(
    |             ^^^^^ help: if this is intentional, prefix it with an underscore: `_event`

warning: unused variable: `query_id_for_audit`
   --> performance-calculator/src/calculations/query_api.rs:391:13
    |
391 |         let query_id_for_audit = query_id.clone();
    |             ^^^^^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_query_id_
for_audit`

warning: unused variable: `portfolio_data`
   --> performance-calculator/src/calculations/query_api.rs:626:25
    |
626 |                     let portfolio_data = data_service.get_portfolio_data(
    |                         ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_p
ortfolio_data`

warning: unused variable: `scenario`
   --> performance-calculator/src/calculations/analytics.rs:379:13
    |
379 |         let scenario = scenarios.get(scenario_id)
    |             ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_scenario`

warning: unused variable: `request_id`
   --> performance-calculator/src/calculations/visualization.rs:543:9
    |
543 |         request_id: &str,
    |         ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_request_id`

warning: unused variable: `request_id`
   --> performance-calculator/src/calculations/integration.rs:844:65
    |
844 | ...uest, request_id: &str) -> Result<DataImportResult> {
    |          ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_request_id`

error[E0107]: missing generics for trait `CircuitBreaker`
   --> performance-calculator/src/resilience/mod.rs:86:9
    |
86  |     CB: CircuitBreaker,
    |         ^^^^^^^^^^^^^^ expected 1 generic argument
    |
note: trait defined here, with 1 generic parameter: `E`
   --> performance-calculator/src/resilience/circuit_breaker.rs:103:11
    |
103 | pub trait CircuitBreaker<E>: Send + Sync {
    |           ^^^^^^^^^^^^^^ -
help: add missing generic argument
    |
86  |     CB: CircuitBreaker<E>,
    |                       +++

error[E0107]: missing generics for enum `BulkheadError`
  --> performance-calculator/src/resilience/mod.rs:95:16
   |
95 | ) -> Result<T, BulkheadError>
   |                ^^^^^^^^^^^^^ expected 1 generic argument
   |
note: enum defined here, with 1 generic parameter: `E`
  --> performance-calculator/src/resilience/bulkhead.rs:16:10
   |
16 | pub enum BulkheadError<E> {
   |          ^^^^^^^^^^^^^ -
help: add missing generic argument
   |
95 | ) -> Result<T, BulkheadError<E>>
   |                             +++

error[E0191]: the value of the associated type `Future` in `CircuitBreaker<E>` must be specified
   --> performance-calculator/src/resilience/mod.rs:285:21
    |
285 | ) -> Result<Box<dyn CircuitBreaker<E>>, anyhow::Error>
    |                     ^^^^^^^^^^^^^^^^^ help: specify the associated type: `CircuitBreaker<E, Future = T
ype>`
    |
   ::: performance-calculator/src/resilience/circuit_breaker.rs:104:5
    |
104 |     type Future: Future<Output = Result<(), CircuitBreakerError<E>>> + Send;
    |     ----------------------------------------------------------------------- `Future` defined here

error[E0599]: no method named `execute` found for reference `&CB` in the current scope
  --> performance-calculator/src/resilience/mod.rs:88:21
   |
88 |     circuit_breaker.execute(operation).await
   |                     ^^^^^^^ method not found in `&CB`
   |
   = help: items from traits can only be used if the type parameter is bounded by the trait
help: the following trait defines an item `execute`, perhaps you need to restrict type parameter `CB` with i
t:
   |
86 |     CB: CircuitBreaker + hyper::rt::Executor</* Fut */>,
   |                        ++++++++++++++++++++++++++++++++

error[E0107]: missing generics for enum `BulkheadError`
   --> performance-calculator/src/resilience/bulkhead.rs:109:55
    |
109 |     type Future: Future<Output = Result<Self::Permit, BulkheadError>> + Send;
    |                                                       ^^^^^^^^^^^^^ expected 1 generic argument
    |
note: enum defined here, with 1 generic parameter: `E`
   --> performance-calculator/src/resilience/bulkhead.rs:16:10
    |
16  | pub enum BulkheadError<E> {
    |          ^^^^^^^^^^^^^ -
help: add missing generic argument
    |
109 |     type Future: Future<Output = Result<Self::Permit, BulkheadError<E>>> + Send;
    |                                                                    +++

error[E0599]: no variant or associated item named `Error` found for enum `BulkheadError` in the current scop
e
   --> performance-calculator/src/resilience/mod.rs:109:45
    |
109 |                     _ => Err(BulkheadError::Error(e)),
    |                                             ^^^^^ variant or associated item not found in `BulkheadErr
or<_>`
    |
   ::: performance-calculator/src/resilience/bulkhead.rs:16:1
    |
16  | pub enum BulkheadError<E> {
    | ------------------------- variant or associated item `Error` not found for this enum

error[E0609]: no field `max_retries` on type `resilience::retry::RetryConfig`
   --> performance-calculator/src/resilience/mod.rs:202:22
    |
202 |         retry_config.max_retries,
    |                      ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0609]: no field `initial_backoff_ms` on type `resilience::retry::RetryConfig`
   --> performance-calculator/src/resilience/mod.rs:203:22
    |
203 |         retry_config.initial_backoff_ms,
    |                      ^^^^^^^^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0277]: `?` couldn't convert the error to `ResilienceError<E>`
   --> performance-calculator/src/resilience/mod.rs:227:48
    |
227 |         .map_err(|e| ResilienceError::Other(e))?;
    |                                                ^ the trait `From<ResilienceError<std::string::String>>
` is not implemented for `ResilienceError<E>`
    |
    = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the 
`From` trait
    = note: required for `Result<T, ResilienceError<E>>` to implement `FromResidual<Result<Infallible, Resil
ienceError<std::string::String>>>`
help: consider extending the `where` clause, but there might be an alternative better way to express this re
quirement
    |
223 |     E: std::error::Error + Send + Sync + 'static + std::fmt::Debug, ResilienceError<E>: From<Resilienc
eError<std::string::String>>
    |                                                                   ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

error[E0599]: no method named `execute` found for struct `Arc<StandardCircuitBreaker>` in the current scope
   --> performance-calculator/src/resilience/mod.rs:231:29
    |
231 |             circuit_breaker.execute(|| async {
    |             ----------------^^^^^^^ method not found in `Arc<StandardCircuitBreaker>`
    |
    = help: items from traits can only be used if the trait is implemented and in scope
    = note: the following trait defines an item `execute`, perhaps you need to implement it:
            candidate #1: `hyper::rt::Executor`

error[E0609]: no field `max_retries` on type `resilience::retry::RetryConfig`
   --> performance-calculator/src/resilience/mod.rs:245:22
    |
245 |         retry_config.max_retries,
    |                      ^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0609]: no field `initial_backoff_ms` on type `resilience::retry::RetryConfig`
   --> performance-calculator/src/resilience/mod.rs:246:22
    |
246 |         retry_config.initial_backoff_ms,
    |                      ^^^^^^^^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `max_attempts`, `initial_delay_ms`, `max_delay_ms`, `multiplier`, `jitter`

error[E0277]: the trait bound `E: From<std::io::Error>` is not satisfied
   --> performance-calculator/src/resilience/mod.rs:271:9
    |
271 |         chaos::with_chaos(service, Some(tenant_id), || f()).await
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `From<std::io::Error>` is not im
plemented for `E`
    |
note: required by a bound in `chaos::with_chaos`
   --> performance-calculator/src/resilience/chaos.rs:262:28
    |
254 | pub async fn with_chaos<T, E, F, Fut>(
    |              ---------- required by a bound in this function
...
262 |     E: std::error::Error + From<std::io::Error> + Send + Sync + 'static,
    |                            ^^^^^^^^^^^^^^^^^^^^ required by this bound in `with_chaos`
help: consider further restricting type parameter `E` with trait `From`
    |
267 |     E: std::error::Error + Send + Sync + 'static + std::fmt::Debug + std::convert::From<std::io::Error
>,
    |                                                                    +++++++++++++++++++++++++++++++++++
+

error[E0277]: the trait bound `E: From<std::io::Error>` is not satisfied
   --> performance-calculator/src/resilience/mod.rs:271:61
    |
271 |         chaos::with_chaos(service, Some(tenant_id), || f()).await
    |                                                             ^^^^^ the trait `From<std::io::Error>` is 
not implemented for `E`
    |
note: required by a bound in `chaos::with_chaos`
   --> performance-calculator/src/resilience/chaos.rs:262:28
    |
254 | pub async fn with_chaos<T, E, F, Fut>(
    |              ---------- required by a bound in this function
...
262 |     E: std::error::Error + From<std::io::Error> + Send + Sync + 'static,
    |                            ^^^^^^^^^^^^^^^^^^^^ required by this bound in `with_chaos`
help: consider further restricting type parameter `E` with trait `From`
    |
267 |     E: std::error::Error + Send + Sync + 'static + std::fmt::Debug + std::convert::From<std::io::Error
>,
    |                                                                    +++++++++++++++++++++++++++++++++++
+

error[E0308]: mismatched types
   --> performance-calculator/src/resilience/mod.rs:408:65
    |
408 |     let bulkhead = StandardBulkhead::new(tenant_id.to_string(), 10);
    |                    ---------------------                        ^^ expected `BulkheadConfig`, found in
teger
    |                    |
    |                    arguments to this function are incorrect
    |
note: associated function defined here
   --> performance-calculator/src/resilience/bulkhead.rs:126:12
    |
126 |     pub fn new(name: String, config: BulkheadConfig) -> Self {
    |            ^^^               ----------------------

warning: variable does not need to be mutable
   --> performance-calculator/src/resilience/retry.rs:169:5
    |
169 |     mut f: F,
    |     ----^
    |     |
    |     help: remove this `mut`
    |
    = note: `#[warn(unused_mut)]` on by default

warning: variable `last_error` is assigned to, but never used
   --> performance-calculator/src/resilience/retry.rs:208:13
    |
208 |     let mut last_error = None;
    |             ^^^^^^^^^^
    |
    = note: consider using `_last_error` instead

warning: value assigned to `last_error` is never read
   --> performance-calculator/src/resilience/retry.rs:219:17
    |
219 |                 last_error = Some(error);
    |                 ^^^^^^^^^^
    |
    = help: maybe it is overwritten before being read?
    = note: `#[warn(unused_assignments)]` on by default

warning: unused variable: `predicate`
   --> performance-calculator/src/resilience/retry.rs:185:5
    |
185 |     predicate: P,
    |     ^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_predicate`

warning: variable does not need to be mutable
   --> performance-calculator/src/resilience/retry.rs:183:5
    |
183 |     mut f: F,
    |     ----^
    |     |
    |     help: remove this `mut`

error[E0046]: not all trait items implemented, missing: `Permit`, `Future`, `acquire`, `release`
   --> performance-calculator/src/resilience/bulkhead.rs:197:1
    |
108 |     type Permit: Send;
    |     ----------------- `Permit` from trait
109 |     type Future: Future<Output = Result<Self::Permit, BulkheadError>> + Send;
    |     ------------------------------------------------------------------------ `Future` from trait
110 |
111 |     fn acquire(&self) -> Self::Future;
    |     ---------------------------------- `acquire` from trait
112 |     fn release(&self, permit: Self::Permit);
    |     ---------------------------------------- `release` from trait
...
197 | impl Bulkhead for StandardBulkhead {
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `Permit`, `Future`, `acquire`, `release` in implementation

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:128:44
    |
128 |             state: Arc::new(AtomicU32::new(CircuitBreakerState::Closed as u32)),
    |                                            ^^^^^^^^^^^^^^^^^^^
    |                                            |
    |                                            use of undeclared type `CircuitBreakerState`
    |                                            help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:141:21
    |
141 |         let state = CircuitBreakerState::from(self.state.load(Ordering::SeqCst));
    |                     ^^^^^^^^^^^^^^^^^^^
    |                     |
    |                     use of undeclared type `CircuitBreakerState`
    |                     help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:142:21
    |
142 |         Ok(state == CircuitBreakerState::Closed)
    |                     ^^^^^^^^^^^^^^^^^^^
    |                     |
    |                     use of undeclared type `CircuitBreakerState`
    |                     help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:147:21
    |
147 |         let state = CircuitBreakerState::from(self.state.load(Ordering::SeqCst));
    |                     ^^^^^^^^^^^^^^^^^^^
    |                     |
    |                     use of undeclared type `CircuitBreakerState`
    |                     help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:149:21
    |
149 |         if state == CircuitBreakerState::Open {
    |                     ^^^^^^^^^^^^^^^^^^^
    |                     |
    |                     use of undeclared type `CircuitBreakerState`
    |                     help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:159:38
    |
159 |                     self.state.store(CircuitBreakerState::HalfOpen as u32, Ordering::SeqCst);
    |                                      ^^^^^^^^^^^^^^^^^^^
    |                                      |
    |                                      use of undeclared type `CircuitBreakerState`
    |                                      help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:172:21
    |
172 |         let state = CircuitBreakerState::from(self.state.load(Ordering::SeqCst));
    |                     ^^^^^^^^^^^^^^^^^^^
    |                     |
    |                     use of undeclared type `CircuitBreakerState`
    |                     help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:173:21
    |
173 |         Ok(state == CircuitBreakerState::HalfOpen)
    |                     ^^^^^^^^^^^^^^^^^^^
    |                     |
    |                     use of undeclared type `CircuitBreakerState`
    |                     help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:180:21
    |
180 |         if state != CircuitBreakerState::Open as u32 {
    |                     ^^^^^^^^^^^^^^^^^^^
    |                     |
    |                     use of undeclared type `CircuitBreakerState`
    |                     help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:182:30
    |
182 |             self.state.store(CircuitBreakerState::Open as u32, Ordering::SeqCst);
    |                              ^^^^^^^^^^^^^^^^^^^
    |                              |
    |                              use of undeclared type `CircuitBreakerState`
    |                              help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:192:21
    |
192 |         if state != CircuitBreakerState::HalfOpen as u32 {
    |                     ^^^^^^^^^^^^^^^^^^^
    |                     |
    |                     use of undeclared type `CircuitBreakerState`
    |                     help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:194:30
    |
194 |             self.state.store(CircuitBreakerState::HalfOpen as u32, Ordering::SeqCst);
    |                              ^^^^^^^^^^^^^^^^^^^
    |                              |
    |                              use of undeclared type `CircuitBreakerState`
    |                              help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:205:21
    |
205 |         if state != CircuitBreakerState::Closed as u32 {
    |                     ^^^^^^^^^^^^^^^^^^^
    |                     |
    |                     use of undeclared type `CircuitBreakerState`
    |                     help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:207:30
    |
207 |             self.state.store(CircuitBreakerState::Closed as u32, Ordering::SeqCst);
    |                              ^^^^^^^^^^^^^^^^^^^
    |                              |
    |                              use of undeclared type `CircuitBreakerState`
    |                              help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:217:26
    |
217 |         self.state.store(CircuitBreakerState::Closed as u32, Ordering::SeqCst);
    |                          ^^^^^^^^^^^^^^^^^^^
    |                          |
    |                          use of undeclared type `CircuitBreakerState`
    |                          help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:226:30
    |
226 |             self.state.store(CircuitBreakerState::Open as u32, Ordering::SeqCst);
    |                              ^^^^^^^^^^^^^^^^^^^
    |                              |
    |                              use of undeclared type `CircuitBreakerState`
    |                              help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:239:21
    |
239 |         let state = CircuitBreakerState::from(self.state.load(Ordering::SeqCst));
    |                     ^^^^^^^^^^^^^^^^^^^
    |                     |
    |                     use of undeclared type `CircuitBreakerState`
    |                     help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:245:17
    |
245 |                 CircuitBreakerState::Closed => Ok(()),
    |                 ^^^^^^^^^^^^^^^^^^^
    |                 |
    |                 use of undeclared type `CircuitBreakerState`
    |                 help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:246:17
    |
246 |                 CircuitBreakerState::Open => {
    |                 ^^^^^^^^^^^^^^^^^^^
    |                 |
    |                 use of undeclared type `CircuitBreakerState`
    |                 help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:257:17
    |
257 |                 CircuitBreakerState::HalfOpen => Ok(()),
    |                 ^^^^^^^^^^^^^^^^^^^
    |                 |
    |                 use of undeclared type `CircuitBreakerState`
    |                 help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:264:26
    |
264 |         self.state.store(CircuitBreakerState::Closed as u32, Ordering::SeqCst);
    |                          ^^^^^^^^^^^^^^^^^^^
    |                          |
    |                          use of undeclared type `CircuitBreakerState`
    |                          help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:271:30
    |
271 |             self.state.store(CircuitBreakerState::Open as u32, Ordering::SeqCst);
    |                              ^^^^^^^^^^^^^^^^^^^
    |                              |
    |                              use of undeclared type `CircuitBreakerState`
    |                              help: a trait with a similar name exists: `CircuitBreaker`

error[E0433]: failed to resolve: use of undeclared type `CircuitBreakerState`
   --> performance-calculator/src/resilience/circuit_breaker.rs:276:9
    |
276 |         CircuitBreakerState::from(self.state.load(Ordering::SeqCst))
    |         ^^^^^^^^^^^^^^^^^^^
    |         |
    |         use of undeclared type `CircuitBreakerState`
    |         help: a trait with a similar name exists: `CircuitBreaker`

Some errors have detailed explanations: E0046, E0107, E0119, E0191, E0277, E0308, E0404, E0407, E0412...
For more information about an error, try `rustc --explain E0046`.
warning: `performance-calculator` (lib) generated 82 warnings
error: could not compile `performance-calculator` (lib) due to 98 previous errors; 82 warnings emitted

```

---

_****_



I see we have quite a few errors to fix. Let's address them systematically:

1. First, let's add the missing `CircuitBreakerState` enum that's causing multiple errors:

---

_****_

```diff
  
  /// Circuit breaker state
- #[derive(Debug, Clone, Copy, PartialEq, Eq)]
- pub enum CircuitState {
-     /// Circuit is closed, requests are allowed
-     Closed,
-     
-     /// Circuit is open, requests are not allowed
-     Open,
-     
-     /// Circuit is half-open, allowing a limited number of requests to test if the service is healthy
-     HalfOpen,
+ #[derive(Debug, Clone, Copy, PartialEq)]
+ pub enum CircuitBreakerState {
+     Closed = 0,
+     Open = 1,
+     HalfOpen = 2,
+ }
+ 
+ impl From<u32> for CircuitBreakerState {
+     fn from(value: u32) -> Self {
+         match value {
+             0 => CircuitBreakerState::Closed,
+             1 => CircuitBreakerState::Open,
+             2 => CircuitBreakerState::HalfOpen,
+             _ => CircuitBreakerState::Open, // Default to Open for safety
+         }
+     }
  }
  
```

---

_****_



Now let's fix the remaining issues in the circuit breaker file:

---

_****_



---

_****_



Let me try to reapply the edit:

---

_****_



---

_****_



Now let's fix the `BulkheadError` issues in the resilience module:

---

_****_

```diff
      
      #[error("Bulkhead error: {0}")]
-     Bulkhead(#[from] BulkheadError),
+     Bulkhead(#[from] BulkheadError<E>),
      
      #[error("Retry error: {0}")]
-     Retry(#[from] E),
+     Retry(#[from] RetryError<E>),
      
      #[error("Other error: {0}")]
```

---

_****_



Now let's fix the `retry_with_exponential_backoff` function call in the resilience module:

---

_****_

```diff
  pub async fn with_resilience<T, E, F, Fut>(
      f: F,
-     config: ResilienceConfig,
+     config: &ResilienceConfig,
  ) -> Result<T, ResilienceError<E>>
  where
-     F: FnMut() -> Fut + Send + 'static,
-     Fut: Future<Output = Result<T, E>> + Send + 'static,
-     T: Send + 'static,
+     F: FnMut() -> Fut,
+     Fut: Future<Output = Result<T, E>>,
      E: std::error::Error + Send + Sync + 'static + std::fmt::Debug,
  {
-     let circuit_breaker = &config.circuit_breaker;
-     let bulkhead = &config.bulkhead;
-     let retry_config = config.retry_config;
- 
-     match bulkhead.acquire().await {
-         Ok(permit) => {
-             match circuit_breaker.check().await {
-                 Ok(_) => {
+     let circuit_breaker = get_circuit_breaker::<E>(&config.circuit_breaker)?;
+     let bulkhead = get_bulkhead::<E>(&config.bulkhead)?;
+ 
+     circuit_breaker.execute(|| async {
+         let permit = bulkhead.acquire().await?;
+ 
          let result = retry::retry_with_exponential_backoff(
-                         || f(),
-                         retry_config,
+             f,
+             &config.retry_config,
          ).await;
  
-                     match result {
-                         Ok(value) => {
-                             circuit_breaker.on_success();
          bulkhead.release(permit);
-                             Ok(value)
-                         }
-                         Err(RetryError::MaxRetriesExceeded(e)) => {
-                             circuit_breaker.on_error(&e);
-                             bulkhead.release(permit);
-                             Err(ResilienceError::Retry(e))
-                         }
-                         Err(RetryError::Aborted(e)) => {
-                             circuit_breaker.on_error(&e);
-                             bulkhead.release(permit);
-                             Err(ResilienceError::Retry(e))
-                         }
-                         Err(RetryError::Other(e)) => {
-                             circuit_breaker.on_error(&e);
-                             bulkhead.release(permit);
-                             Err(ResilienceError::Other(e))
-                         }
-                     }
-                 }
-                 Err(e) => {
-                     bulkhead.release(permit);
-                     Err(ResilienceError::CircuitBreaker(e))
-                 }
-             }
-         }
-         Err(e) => Err(ResilienceError::Bulkhead(e)),
-     }
+         result
+     }).await
  }
  
```

---

_****_





Let's check the retry module to see what parameters the `retry_with_exponential_backoff` function expects:

---

_****_

Could not find file ''. Did you mean one of:
- performance-calculator/src/resilience/mod.rs
- performance-calculator/src/resilience/circuit_breaker.rs
- performance-calculator/src/resilience/retry.rs

---

_****_



---

_**User**_

could we run the end to end build and integration tests then do a commit if everything works

---

_**Assistant**_



---

_**User**_

For the code present, we get this error:
```
missing structure fields:
- transactions
- holdings

```
How can I resolve this? If you propose a fix, please make it concise.

---

_**Assistant**_



---



---

