use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc, NaiveDate, NaiveDateTime, Duration, Datelike};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time;
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::calculations::{
    query_api::{QueryApi, PerformanceQueryParams, RiskQueryParams, AttributionQueryParams},
    audit::{AuditTrailManager, CalculationEventBuilder},
};

/// Schedule frequency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScheduleFrequency {
    /// Run once at a specific time
    Once(DateTime<Utc>),
    
    /// Run daily at a specific time
    Daily {
        /// Hour of the day (0-23)
        hour: u32,
        
        /// Minute of the hour (0-59)
        minute: u32,
    },
    
    /// Run weekly on a specific day and time
    Weekly {
        /// Day of the week (0 = Sunday, 6 = Saturday)
        day_of_week: u32,
        
        /// Hour of the day (0-23)
        hour: u32,
        
        /// Minute of the hour (0-59)
        minute: u32,
    },
    
    /// Run monthly on a specific day and time
    Monthly {
        /// Day of the month (1-31)
        day_of_month: u32,
        
        /// Hour of the day (0-23)
        hour: u32,
        
        /// Minute of the hour (0-59)
        minute: u32,
    },
    
    /// Run quarterly on a specific day and time
    Quarterly {
        /// Month of the quarter (1, 4, 7, 10)
        month: u32,
        
        /// Day of the month (1-31)
        day_of_month: u32,
        
        /// Hour of the day (0-23)
        hour: u32,
        
        /// Minute of the hour (0-59)
        minute: u32,
    },
}

impl ScheduleFrequency {
    /// Calculate the next run time based on the current time
    pub fn next_run_time(&self, current_time: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self {
            ScheduleFrequency::Once(time) => {
                if *time > current_time {
                    Some(*time)
                } else {
                    None // Already passed
                }
            },
            ScheduleFrequency::Daily { hour, minute } => {
                let naive_date = current_time.naive_utc().date();
                let target_time = naive_date.and_hms_opt(*hour, *minute, 0).unwrap();
                
                let target_datetime = DateTime::<Utc>::from_naive_utc_and_offset(target_time, Utc);
                
                if target_datetime > current_time {
                    // Today's run is still in the future
                    Some(target_datetime)
                } else {
                    // Schedule for tomorrow
                    let tomorrow = naive_date.checked_add_days(chrono::Days::new(1)).unwrap();
                    let tomorrow_time = tomorrow.and_hms_opt(*hour, *minute, 0).unwrap();
                    Some(DateTime::<Utc>::from_naive_utc_and_offset(tomorrow_time, Utc))
                }
            },
            ScheduleFrequency::Weekly { day_of_week, hour, minute } => {
                let current_day_of_week = current_time.weekday().num_days_from_sunday();
                let days_until_target = (*day_of_week + 7 - current_day_of_week) % 7;
                
                let target_date = current_time.naive_utc().date()
                    .checked_add_days(chrono::Days::new(days_until_target.into())).unwrap();
                let target_time = target_date.and_hms_opt(*hour, *minute, 0).unwrap();
                let target_datetime = DateTime::<Utc>::from_naive_utc_and_offset(target_time, Utc);
                
                if days_until_target == 0 && target_datetime <= current_time {
                    // Today is the target day but the time has passed, schedule for next week
                    let next_week = target_date.checked_add_days(chrono::Days::new(7)).unwrap();
                    let next_week_time = next_week.and_hms_opt(*hour, *minute, 0).unwrap();
                    Some(DateTime::<Utc>::from_naive_utc_and_offset(next_week_time, Utc))
                } else {
                    Some(target_datetime)
                }
            },
            ScheduleFrequency::Monthly { day_of_month, hour, minute } => {
                let current_date = current_time.naive_utc().date();
                let current_day = current_date.day();
                let current_month = current_date.month();
                let current_year = current_date.year();
                
                // Try this month
                if current_day <= *day_of_month {
                    // Target day is in this month
                    let target_date = NaiveDate::from_ymd_opt(current_year, current_month, *day_of_month)
                        .unwrap_or_else(|| {
                            // Handle invalid dates (e.g., Feb 30)
                            let last_day = get_last_day_of_month(current_year, current_month);
                            NaiveDate::from_ymd_opt(current_year, current_month, last_day).unwrap()
                        });
                    
                    let target_time = target_date.and_hms_opt(*hour, *minute, 0).unwrap();
                    let target_datetime = DateTime::<Utc>::from_naive_utc_and_offset(target_time, Utc);
                    
                    if target_datetime > current_time {
                        return Some(target_datetime);
                    }
                }
                
                // Target is in next month
                let (next_year, next_month) = if current_month == 12 {
                    (current_year + 1, 1)
                } else {
                    (current_year, current_month + 1)
                };
                
                let target_date = NaiveDate::from_ymd_opt(next_year, next_month, *day_of_month)
                    .unwrap_or_else(|| {
                        // Handle invalid dates
                        let last_day = get_last_day_of_month(next_year, next_month);
                        NaiveDate::from_ymd_opt(next_year, next_month, last_day).unwrap()
                    });
                
                let target_time = target_date.and_hms_opt(*hour, *minute, 0).unwrap();
                Some(DateTime::<Utc>::from_naive_utc_and_offset(target_time, Utc))
            },
            ScheduleFrequency::Quarterly { month, day_of_month, hour, minute } => {
                let current_date = current_time.naive_utc().date();
                let current_month = current_date.month();
                let current_year = current_date.year();
                
                // Calculate the next quarter month
                let quarter_months = [1, 4, 7, 10];
                let current_quarter = (current_month - 1) / 3;
                let next_quarter_month = quarter_months[((current_quarter + 1) % 4) as usize];
                
                let target_year = if next_quarter_month < current_month {
                    current_year + 1
                } else {
                    current_year
                };
                
                // If we're in the target month, check if the day has passed
                if current_month == *month {
                    let current_day = current_date.day();
                    
                    if current_day < *day_of_month {
                        // Target day is still in this month
                        let target_date = NaiveDate::from_ymd_opt(current_year, current_month, *day_of_month)
                            .unwrap_or_else(|| {
                                // Handle invalid dates
                                let last_day = get_last_day_of_month(current_year, current_month);
                                NaiveDate::from_ymd_opt(current_year, current_month, last_day).unwrap()
                            });
                        
                        let target_time = target_date.and_hms_opt(*hour, *minute, 0).unwrap();
                        let target_datetime = DateTime::<Utc>::from_naive_utc_and_offset(target_time, Utc);
                        
                        if target_datetime > current_time {
                            return Some(target_datetime);
                        }
                    }
                }
                
                // Target is in a future month
                let target_date = NaiveDate::from_ymd_opt(target_year, *month, *day_of_month)
                    .unwrap_or_else(|| {
                        // Handle invalid dates
                        let last_day = get_last_day_of_month(target_year, *month);
                        NaiveDate::from_ymd_opt(target_year, *month, last_day).unwrap()
                    });
                
                let target_time = target_date.and_hms_opt(*hour, *minute, 0).unwrap();
                Some(DateTime::<Utc>::from_naive_utc_and_offset(target_time, Utc))
            },
        }
    }
}

/// Helper function to get the last day of a month
fn get_last_day_of_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                29 // Leap year
            } else {
                28
            }
        },
        _ => panic!("Invalid month: {}", month),
    }
}

/// Notification channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    /// Email notification
    Email {
        /// Recipient email addresses
        recipients: Vec<String>,
        
        /// Email subject template
        subject_template: String,
        
        /// Email body template
        body_template: String,
    },
    
    /// Webhook notification
    Webhook {
        /// Webhook URL
        url: String,
        
        /// HTTP method (POST, PUT, etc.)
        method: String,
        
        /// HTTP headers
        headers: HashMap<String, String>,
    },
    
    /// SNS notification
    SNS {
        /// SNS topic ARN
        topic_arn: String,
        
        /// Message subject
        subject: String,
    },
    
    /// SQS notification
    SQS {
        /// SQS queue URL
        queue_url: String,
    },
}

/// Scheduled calculation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduledCalculationType {
    /// Performance calculation
    Performance(PerformanceQueryParams),
    
    /// Risk calculation
    Risk(RiskQueryParams),
    
    /// Attribution calculation
    Attribution(AttributionQueryParams),
}

/// Scheduled calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledCalculation {
    /// Unique identifier
    pub id: String,
    
    /// Name of the scheduled calculation
    pub name: String,
    
    /// Description
    pub description: Option<String>,
    
    /// Calculation type
    pub calculation_type: ScheduledCalculationType,
    
    /// Schedule frequency
    pub frequency: ScheduleFrequency,
    
    /// Whether the schedule is enabled
    pub enabled: bool,
    
    /// Notification channels
    pub notification_channels: Vec<NotificationChannel>,
    
    /// Last run time
    pub last_run_time: Option<DateTime<Utc>>,
    
    /// Next run time
    pub next_run_time: Option<DateTime<Utc>>,
    
    /// Created by
    pub created_by: String,
    
    /// Created at
    pub created_at: DateTime<Utc>,
    
    /// Updated at
    pub updated_at: DateTime<Utc>,
}

/// Scheduled calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledCalculationResult {
    /// Schedule ID
    pub schedule_id: String,
    
    /// Run ID
    pub run_id: String,
    
    /// Run time
    pub run_time: DateTime<Utc>,
    
    /// Status
    pub status: ScheduledCalculationStatus,
    
    /// Result data
    pub result: Option<serde_json::Value>,
    
    /// Error message
    pub error_message: Option<String>,
    
    /// Duration in milliseconds
    pub duration_ms: u64,
    
    /// Notification status
    pub notification_status: HashMap<String, NotificationStatus>,
}

/// Scheduled calculation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScheduledCalculationStatus {
    /// Scheduled
    Scheduled,
    
    /// Running
    Running,
    
    /// Completed successfully
    Completed,
    
    /// Failed
    Failed,
    
    /// Cancelled
    Cancelled,
}

/// Notification status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationStatus {
    /// Pending
    Pending,
    
    /// Sent
    Sent,
    
    /// Failed
    Failed,
}

/// Scheduler for performance calculations
pub struct CalculationScheduler {
    /// Query API
    query_api: Arc<QueryApi>,
    
    /// Audit trail manager
    audit_manager: Arc<AuditTrailManager>,
    
    /// Notification service
    notification_service: Arc<dyn NotificationService>,
    
    /// Scheduled calculations
    schedules: Arc<Mutex<Vec<ScheduledCalculation>>>,
    
    /// Scheduled calculation results
    results: Arc<Mutex<HashMap<String, Vec<ScheduledCalculationResult>>>>,
    
    /// Whether the scheduler is running
    running: Arc<Mutex<bool>>,
}

impl CalculationScheduler {
    /// Create a new calculation scheduler
    pub fn new(
        query_api: Arc<QueryApi>,
        audit_manager: Arc<AuditTrailManager>,
        notification_service: Arc<dyn NotificationService>,
    ) -> Self {
        Self {
            query_api,
            audit_manager,
            notification_service,
            schedules: Arc::new(Mutex::new(Vec::new())),
            results: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// Add a scheduled calculation
    pub async fn add_schedule(&self, schedule: ScheduledCalculation) -> Result<()> {
        let mut schedules = self.schedules.lock().await;
        
        // Check if a schedule with this ID already exists
        if schedules.iter().any(|s| s.id == schedule.id) {
            return Err(anyhow!("Schedule with ID {} already exists", schedule.id));
        }
        
        // Add the schedule
        schedules.push(schedule);
        
        Ok(())
    }
    
    /// Update a scheduled calculation
    pub async fn update_schedule(&self, schedule: ScheduledCalculation) -> Result<()> {
        let mut schedules = self.schedules.lock().await;
        
        // Find the schedule
        let index = schedules.iter().position(|s| s.id == schedule.id)
            .ok_or_else(|| anyhow!("Schedule with ID {} not found", schedule.id))?;
        
        // Update the schedule
        schedules[index] = schedule;
        
        Ok(())
    }
    
    /// Delete a scheduled calculation
    pub async fn delete_schedule(&self, schedule_id: &str) -> Result<()> {
        let mut schedules = self.schedules.lock().await;
        
        // Find the schedule
        let index = schedules.iter().position(|s| s.id == schedule_id)
            .ok_or_else(|| anyhow!("Schedule with ID {} not found", schedule_id))?;
        
        // Remove the schedule
        schedules.remove(index);
        
        Ok(())
    }
    
    /// Get a scheduled calculation
    pub async fn get_schedule(&self, schedule_id: &str) -> Result<ScheduledCalculation> {
        let schedules = self.schedules.lock().await;
        
        // Find the schedule
        let schedule = schedules.iter().find(|s| s.id == schedule_id)
            .ok_or_else(|| anyhow!("Schedule with ID {} not found", schedule_id))?;
        
        Ok(schedule.clone())
    }
    
    /// Get all scheduled calculations
    pub async fn get_all_schedules(&self) -> Result<Vec<ScheduledCalculation>> {
        let schedules = self.schedules.lock().await;
        Ok(schedules.clone())
    }
    
    /// Get scheduled calculation results
    pub async fn get_schedule_results(&self, schedule_id: &str) -> Result<Vec<ScheduledCalculationResult>> {
        let results = self.results.lock().await;
        
        // Get results for this schedule
        let schedule_results = results.get(schedule_id)
            .cloned()
            .unwrap_or_default();
        
        Ok(schedule_results)
    }
    
    /// Start the scheduler
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        
        if *running {
            return Err(anyhow!("Scheduler is already running"));
        }
        
        *running = true;
        
        // Start the scheduler loop
        let schedules = self.schedules.clone();
        let results = self.results.clone();
        let query_api = self.query_api.clone();
        let audit_manager = self.audit_manager.clone();
        let notification_service = self.notification_service.clone();
        let running_flag = self.running.clone();
        
        tokio::spawn(async move {
            info!("Starting calculation scheduler");
            
            while *running_flag.lock().await {
                // Get current time
                let now = Utc::now();
                
                // Check for schedules that need to run
                let mut schedules_to_run = Vec::new();
                
                {
                    let mut schedules_guard = schedules.lock().await;
                    
                    for schedule in schedules_guard.iter_mut() {
                        if !schedule.enabled {
                            continue;
                        }
                        
                        // Calculate next run time if not set
                        if schedule.next_run_time.is_none() {
                            schedule.next_run_time = schedule.frequency.next_run_time(now);
                        }
                        
                        // Check if it's time to run
                        if let Some(next_run) = schedule.next_run_time {
                            if next_run <= now {
                                // Time to run this schedule
                                schedules_to_run.push(schedule.clone());
                                
                                // Update last run time and calculate next run time
                                schedule.last_run_time = Some(now);
                                schedule.next_run_time = schedule.frequency.next_run_time(now);
                                schedule.updated_at = now;
                            }
                        }
                    }
                }
                
                // Run the scheduled calculations
                for schedule in schedules_to_run {
                    let run_id = Uuid::new_v4().to_string();
                    let request_id = format!("schedule:{}:run:{}", schedule.id, run_id);
                    
                    // Create a result entry
                    let mut result = ScheduledCalculationResult {
                        schedule_id: schedule.id.clone(),
                        run_id: run_id.clone(),
                        run_time: now,
                        status: ScheduledCalculationStatus::Running,
                        result: None,
                        error_message: None,
                        duration_ms: 0,
                        notification_status: HashMap::new(),
                    };
                    
                    // Initialize notification status
                    for (i, _) in schedule.notification_channels.iter().enumerate() {
                        result.notification_status.insert(i.to_string(), NotificationStatus::Pending);
                    }
                    
                    // Add to results
                    {
                        let mut results_guard = results.lock().await;
                        results_guard.entry(schedule.id.clone())
                            .or_insert_with(Vec::new)
                            .push(result.clone());
                    }
                    
                    // Start audit trail
                    let mut input_params = HashMap::new();
                    input_params.insert("schedule_id".to_string(), serde_json::json!(schedule.id));
                    input_params.insert("run_id".to_string(), serde_json::json!(run_id));
                    
                    let event = match audit_manager.start_calculation(
                        "scheduled_calculation",
                        &request_id,
                        "scheduler",
                        input_params,
                        vec![format!("schedule:{}", schedule.id)],
                    ).await {
                        Ok(e) => e,
                        Err(e) => {
                            error!(
                                schedule_id = %schedule.id,
                                run_id = %run_id,
                                error = %e,
                                "Failed to start audit trail for scheduled calculation"
                            );
                            continue;
                        }
                    };
                    
                    // Execute the calculation
                    let start_time = Utc::now();
                    let calculation_result = match schedule.calculation_type {
                        ScheduledCalculationType::Performance(ref params) => {
                            match query_api.calculate_performance(params.clone()).await {
                                Ok(r) => Ok(serde_json::to_value(r).unwrap_or_default()),
                                Err(e) => Err(e),
                            }
                        },
                        ScheduledCalculationType::Risk(ref params) => {
                            match query_api.calculate_risk(params.clone()).await {
                                Ok(r) => Ok(serde_json::to_value(r).unwrap_or_default()),
                                Err(e) => Err(e),
                            }
                        },
                        ScheduledCalculationType::Attribution(ref params) => {
                            match query_api.calculate_attribution(params.clone()).await {
                                Ok(r) => Ok(serde_json::to_value(r).unwrap_or_default()),
                                Err(e) => Err(e),
                            }
                        },
                    };
                    
                    let end_time = Utc::now();
                    let duration_ms = (end_time - start_time).num_milliseconds() as u64;
                    
                    // Update result
                    {
                        let mut results_guard = results.lock().await;
                        if let Some(results_vec) = results_guard.get_mut(&schedule.id) {
                            if let Some(result) = results_vec.iter_mut().find(|r| r.run_id == run_id) {
                                result.duration_ms = duration_ms;
                                
                                match calculation_result {
                                    Ok(r) => {
                                        result.status = ScheduledCalculationStatus::Completed;
                                        result.result = Some(r);
                                        
                                        // Complete audit trail
                                        if let Err(e) = audit_manager.complete_calculation(
                                            &event.event_id,
                                            vec![format!("calculation_result:{}", run_id)],
                                        ).await {
                                            error!(
                                                schedule_id = %schedule.id,
                                                run_id = %run_id,
                                                error = %e,
                                                "Failed to complete audit trail for scheduled calculation"
                                            );
                                        }
                                    },
                                    Err(e) => {
                                        result.status = ScheduledCalculationStatus::Failed;
                                        result.error_message = Some(e.to_string());
                                        
                                        // Fail audit trail
                                        if let Err(audit_err) = audit_manager.fail_calculation(
                                            &event.event_id,
                                            &e.to_string(),
                                        ).await {
                                            error!(
                                                schedule_id = %schedule.id,
                                                run_id = %run_id,
                                                error = %audit_err,
                                                "Failed to update audit trail for failed calculation"
                                            );
                                        }
                                    },
                                }
                            }
                        }
                    }
                    
                    // Send notifications
                    let result_clone = {
                        let results_guard = results.lock().await;
                        results_guard.get(&schedule.id)
                            .and_then(|results_vec| results_vec.iter().find(|r| r.run_id == run_id))
                            .cloned()
                    };
                    
                    if let Some(result) = result_clone {
                        for (i, channel) in schedule.notification_channels.iter().enumerate() {
                            let channel_id = i.to_string();
                            
                            // Send notification
                            match notification_service.send_notification(
                                channel,
                                &schedule,
                                &result,
                            ).await {
                                Ok(_) => {
                                    // Update notification status
                                    let mut results_guard = results.lock().await;
                                    if let Some(results_vec) = results_guard.get_mut(&schedule.id) {
                                        if let Some(result) = results_vec.iter_mut().find(|r| r.run_id == run_id) {
                                            result.notification_status.insert(channel_id, NotificationStatus::Sent);
                                        }
                                    }
                                },
                                Err(e) => {
                                    error!(
                                        schedule_id = %schedule.id,
                                        run_id = %run_id,
                                        channel = ?channel,
                                        error = %e,
                                        "Failed to send notification for scheduled calculation"
                                    );
                                    
                                    // Update notification status
                                    let mut results_guard = results.lock().await;
                                    if let Some(results_vec) = results_guard.get_mut(&schedule.id) {
                                        if let Some(result) = results_vec.iter_mut().find(|r| r.run_id == run_id) {
                                            result.notification_status.insert(channel_id, NotificationStatus::Failed);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Sleep for a short time before checking again
                time::sleep(time::Duration::from_secs(10)).await;
            }
            
            info!("Calculation scheduler stopped");
        });
        
        Ok(())
    }
    
    /// Stop the scheduler
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        
        if !*running {
            return Err(anyhow!("Scheduler is not running"));
        }
        
        *running = false;
        
        info!("Stopping calculation scheduler");
        
        Ok(())
    }
    
    /// Run a scheduled calculation immediately
    pub async fn run_now(&self, schedule_id: &str) -> Result<String> {
        // Get the schedule
        let schedule = self.get_schedule(schedule_id).await?;
        
        // Generate a run ID
        let run_id = Uuid::new_v4().to_string();
        let request_id = format!("schedule:{}:run:{}", schedule.id, run_id);
        let now = Utc::now();
        
        // Create a result entry
        let mut result = ScheduledCalculationResult {
            schedule_id: schedule.id.clone(),
            run_id: run_id.clone(),
            run_time: now,
            status: ScheduledCalculationStatus::Running,
            result: None,
            error_message: None,
            duration_ms: 0,
            notification_status: HashMap::new(),
        };
        
        // Initialize notification status
        for (i, _) in schedule.notification_channels.iter().enumerate() {
            result.notification_status.insert(i.to_string(), NotificationStatus::Pending);
        }
        
        // Add to results
        {
            let mut results_guard = self.results.lock().await;
            results_guard.entry(schedule.id.clone())
                .or_insert_with(Vec::new)
                .push(result.clone());
        }
        
        // Start audit trail
        let mut input_params = HashMap::new();
        input_params.insert("schedule_id".to_string(), serde_json::json!(schedule.id));
        input_params.insert("run_id".to_string(), serde_json::json!(run_id));
        input_params.insert("manual_run".to_string(), serde_json::json!(true));
        
        let event = self.audit_manager.start_calculation(
            "scheduled_calculation",
            &request_id,
            "scheduler",
            input_params,
            vec![format!("schedule:{}", schedule.id)],
        ).await?;
        
        // Execute the calculation
        let start_time = Utc::now();
        let calculation_result = match schedule.calculation_type {
            ScheduledCalculationType::Performance(ref params) => {
                match self.query_api.calculate_performance(params.clone()).await {
                    Ok(r) => Ok(serde_json::to_value(r).unwrap_or_default()),
                    Err(e) => Err(e),
                }
            },
            ScheduledCalculationType::Risk(ref params) => {
                match self.query_api.calculate_risk(params.clone()).await {
                    Ok(r) => Ok(serde_json::to_value(r).unwrap_or_default()),
                    Err(e) => Err(e),
                }
            },
            ScheduledCalculationType::Attribution(ref params) => {
                match self.query_api.calculate_attribution(params.clone()).await {
                    Ok(r) => Ok(serde_json::to_value(r).unwrap_or_default()),
                    Err(e) => Err(e),
                }
            },
        };
        
        let end_time = Utc::now();
        let duration_ms = (end_time - start_time).num_milliseconds() as u64;
        
        // Update result
        {
            let mut results_guard = self.results.lock().await;
            if let Some(results_vec) = results_guard.get_mut(&schedule.id) {
                if let Some(result) = results_vec.iter_mut().find(|r| r.run_id == run_id) {
                    result.duration_ms = duration_ms;
                    
                    match calculation_result {
                        Ok(r) => {
                            result.status = ScheduledCalculationStatus::Completed;
                            result.result = Some(r);
                            
                            // Complete audit trail
                            self.audit_manager.complete_calculation(
                                &event.event_id,
                                vec![format!("calculation_result:{}", run_id)],
                            ).await?;
                        },
                        Err(e) => {
                            result.status = ScheduledCalculationStatus::Failed;
                            result.error_message = Some(e.to_string());
                            
                            // Fail audit trail
                            self.audit_manager.fail_calculation(
                                &event.event_id,
                                &e.to_string(),
                            ).await?;
                            
                            return Err(e);
                        },
                    }
                }
            }
        }
        
        // Send notifications
        let result_clone = {
            let results_guard = self.results.lock().await;
            results_guard.get(&schedule.id)
                .and_then(|results_vec| results_vec.iter().find(|r| r.run_id == run_id))
                .cloned()
        };
        
        if let Some(result) = result_clone {
            for (i, channel) in schedule.notification_channels.iter().enumerate() {
                let channel_id = i.to_string();
                
                // Send notification
                match self.notification_service.send_notification(
                    channel,
                    &schedule,
                    &result,
                ).await {
                    Ok(_) => {
                        // Update notification status
                        let mut results_guard = self.results.lock().await;
                        if let Some(results_vec) = results_guard.get_mut(&schedule.id) {
                            if let Some(result) = results_vec.iter_mut().find(|r| r.run_id == run_id) {
                                result.notification_status.insert(channel_id, NotificationStatus::Sent);
                            }
                        }
                    },
                    Err(e) => {
                        error!(
                            schedule_id = %schedule.id,
                            run_id = %run_id,
                            channel = ?channel,
                            error = %e,
                            "Failed to send notification for scheduled calculation"
                        );
                        
                        // Update notification status
                        let mut results_guard = self.results.lock().await;
                        if let Some(results_vec) = results_guard.get_mut(&schedule.id) {
                            if let Some(result) = results_vec.iter_mut().find(|r| r.run_id == run_id) {
                                result.notification_status.insert(channel_id, NotificationStatus::Failed);
                            }
                        }
                    }
                }
            }
        }
        
        // Update last run time for the schedule
        {
            let mut schedules = self.schedules.lock().await;
            if let Some(schedule) = schedules.iter_mut().find(|s| s.id == schedule_id) {
                schedule.last_run_time = Some(now);
                schedule.updated_at = now;
            }
        }
        
        Ok(run_id)
    }
}

/// Notification service interface
#[async_trait::async_trait]
pub trait NotificationService: Send + Sync {
    /// Send a notification
    async fn send_notification(
        &self,
        channel: &NotificationChannel,
        schedule: &ScheduledCalculation,
        result: &ScheduledCalculationResult,
    ) -> Result<()>;
}

/// Default notification service implementation
pub struct DefaultNotificationService {
    /// Email client
    email_client: Option<Arc<dyn EmailClient>>,
    
    /// HTTP client
    http_client: reqwest::Client,
    
    /// AWS client
    aws_client: Option<Arc<dyn AwsClient>>,
}

impl DefaultNotificationService {
    /// Create a new default notification service
    pub fn new(
        email_client: Option<Arc<dyn EmailClient>>,
        aws_client: Option<Arc<dyn AwsClient>>,
    ) -> Self {
        Self {
            email_client,
            http_client: reqwest::Client::new(),
            aws_client,
        }
    }
}

#[async_trait::async_trait]
impl NotificationService for DefaultNotificationService {
    async fn send_notification(
        &self,
        channel: &NotificationChannel,
        schedule: &ScheduledCalculation,
        result: &ScheduledCalculationResult,
    ) -> Result<()> {
        match channel {
            NotificationChannel::Email { recipients, subject_template, body_template } => {
                // Check if email client is available
                let email_client = self.email_client.as_ref()
                    .ok_or_else(|| anyhow!("Email client not configured"))?;
                
                // Render templates
                let subject = render_template(subject_template, schedule, result)?;
                let body = render_template(body_template, schedule, result)?;
                
                // Send email
                email_client.send_email(recipients, &subject, &body).await?;
            },
            NotificationChannel::Webhook { url, method, headers } => {
                // Prepare request
                let request_builder = match method.to_uppercase().as_str() {
                    "POST" => self.http_client.post(url),
                    "PUT" => self.http_client.put(url),
                    _ => return Err(anyhow!("Unsupported HTTP method: {}", method)),
                };
                
                // Add headers
                let mut request_builder = request_builder;
                for (key, value) in headers {
                    request_builder = request_builder.header(key, value);
                }
                
                // Add body
                let body = serde_json::json!({
                    "schedule": schedule,
                    "result": result,
                });
                
                // Send request
                let response = request_builder
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| anyhow!("Failed to send webhook: {}", e))?;
                
                // Check response
                if !response.status().is_success() {
                    return Err(anyhow!("Webhook returned error: {}", response.status()));
                }
            },
            NotificationChannel::SNS { topic_arn, subject } => {
                // Check if AWS client is available
                let aws_client = self.aws_client.as_ref()
                    .ok_or_else(|| anyhow!("AWS client not configured"))?;
                
                // Prepare message
                let message = serde_json::to_string(&serde_json::json!({
                    "schedule": schedule,
                    "result": result,
                }))?;
                
                // Send SNS message
                aws_client.send_sns_message(topic_arn, subject, &message).await?;
            },
            NotificationChannel::SQS { queue_url } => {
                // Check if AWS client is available
                let aws_client = self.aws_client.as_ref()
                    .ok_or_else(|| anyhow!("AWS client not configured"))?;
                
                // Prepare message
                let message = serde_json::to_string(&serde_json::json!({
                    "schedule": schedule,
                    "result": result,
                }))?;
                
                // Send SQS message
                aws_client.send_sqs_message(queue_url, &message).await?;
            },
        }
        
        Ok(())
    }
}

/// Render a template with schedule and result data
fn render_template(
    template: &str,
    schedule: &ScheduledCalculation,
    result: &ScheduledCalculationResult,
) -> Result<String> {
    // This is a simplified implementation
    // In a real application, you would use a template engine like Handlebars or Tera
    
    let mut rendered = template.to_string();
    
    // Replace schedule variables
    rendered = rendered.replace("{{schedule.id}}", &schedule.id);
    rendered = rendered.replace("{{schedule.name}}", &schedule.name);
    
    if let Some(desc) = &schedule.description {
        rendered = rendered.replace("{{schedule.description}}", desc);
    }
    
    // Replace result variables
    rendered = rendered.replace("{{result.run_id}}", &result.run_id);
    rendered = rendered.replace("{{result.run_time}}", &result.run_time.to_rfc3339());
    rendered = rendered.replace("{{result.status}}", &format!("{:?}", result.status));
    rendered = rendered.replace("{{result.duration_ms}}", &result.duration_ms.to_string());
    
    if let Some(error) = &result.error_message {
        rendered = rendered.replace("{{result.error_message}}", error);
    }
    
    Ok(rendered)
}

/// Email client interface
#[async_trait::async_trait]
pub trait EmailClient: Send + Sync {
    /// Send an email
    async fn send_email(
        &self,
        recipients: &[String],
        subject: &str,
        body: &str,
    ) -> Result<()>;
}

/// AWS client interface
#[async_trait::async_trait]
pub trait AwsClient: Send + Sync {
    /// Send an SNS message
    async fn send_sns_message(
        &self,
        topic_arn: &str,
        subject: &str,
        message: &str,
    ) -> Result<()>;
    
    /// Send an SQS message
    async fn send_sqs_message(
        &self,
        queue_url: &str,
        message: &str,
    ) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculations::audit::InMemoryAuditTrailStorage;
    use crate::calculations::currency::{ExchangeRateProvider, ExchangeRate, CurrencyCode};
    use crate::calculations::query_api::{DataAccessService, PortfolioData, PortfolioHoldingsWithReturns, BenchmarkHoldingsWithReturns, HypotheticalTransaction};
    use crate::calculations::risk_metrics::ReturnSeries;
    use std::collections::HashMap;
    use chrono::NaiveDate;
    use mockall::predicate::*;
    use mockall::mock;
    
    // Mock exchange rate provider for testing
    mock! {
        ExchangeRateProviderMock {}
        
        #[async_trait::async_trait]
        impl ExchangeRateProvider for ExchangeRateProviderMock {
            async fn get_exchange_rate(
                &self,
                base_currency: &CurrencyCode,
                quote_currency: &CurrencyCode,
                date: NaiveDate,
                request_id: &str,
            ) -> anyhow::Result<ExchangeRate>;
        }
    }
    
    // Mock data access service for testing
    mock! {
        DataAccessServiceMock {}
        
        #[async_trait::async_trait]
        impl DataAccessService for DataAccessServiceMock {
            async fn get_portfolio_data(
                &self,
                portfolio_id: &str,
                start_date: NaiveDate,
                end_date: NaiveDate,
            ) -> anyhow::Result<PortfolioData>;
            
            async fn get_portfolio_returns(
                &self,
                portfolio_id: &str,
                start_date: NaiveDate,
                end_date: NaiveDate,
                frequency: &str,
            ) -> anyhow::Result<HashMap<NaiveDate, f64>>;
            
            async fn get_benchmark_returns(
                &self,
                benchmark_id: &str,
                start_date: NaiveDate,
                end_date: NaiveDate,
            ) -> anyhow::Result<ReturnSeries>;
            
            async fn get_benchmark_returns_by_frequency(
                &self,
                benchmark_id: &str,
                start_date: NaiveDate,
                end_date: NaiveDate,
                frequency: &str,
            ) -> anyhow::Result<ReturnSeries>;
            
            async fn get_portfolio_holdings_with_returns(
                &self,
                portfolio_id: &str,
                start_date: NaiveDate,
                end_date: NaiveDate,
            ) -> anyhow::Result<PortfolioHoldingsWithReturns>;
            
            async fn get_benchmark_holdings_with_returns(
                &self,
                benchmark_id: &str,
                start_date: NaiveDate,
                end_date: NaiveDate,
            ) -> anyhow::Result<BenchmarkHoldingsWithReturns>;
            
            async fn clone_portfolio_data(
                &self,
                source_portfolio_id: &str,
                target_portfolio_id: &str,
                start_date: NaiveDate,
                end_date: NaiveDate,
            ) -> anyhow::Result<()>;
            
            async fn apply_hypothetical_transaction(
                &self,
                portfolio_id: &str,
                transaction: &HypotheticalTransaction,
            ) -> anyhow::Result<()>;
            
            async fn delete_portfolio_data(&self, portfolio_id: &str) -> anyhow::Result<()>;
        }
    }
    
    // Mock notification service for testing
    mock! {
        NotificationServiceMock {}
        
        #[async_trait::async_trait]
        impl NotificationService for NotificationServiceMock {
            async fn send_notification(
                &self,
                channel: &NotificationChannel,
                schedule: &ScheduledCalculation,
                result: &ScheduledCalculationResult,
            ) -> Result<()>;
        }
    }
    
    #[tokio::test]
    async fn test_schedule_frequency_next_run_time() {
        // Test Once frequency
        let now = Utc::now();
        let future = now + Duration::hours(1);
        let past = now - Duration::hours(1);
        
        let once_future = ScheduleFrequency::Once(future);
        let once_past = ScheduleFrequency::Once(past);
        
        assert_eq!(once_future.next_run_time(now), Some(future));
        assert_eq!(once_past.next_run_time(now), None);
        
        // Test Daily frequency
        let daily = ScheduleFrequency::Daily {
            hour: 10,
            minute: 30,
        };
        
        let next_daily = daily.next_run_time(now);
        assert!(next_daily.is_some());
        
        // Test Weekly frequency
        let weekly = ScheduleFrequency::Weekly {
            day_of_week: 1, // Monday
            hour: 10,
            minute: 30,
        };
        
        let next_weekly = weekly.next_run_time(now);
        assert!(next_weekly.is_some());
        
        // Test Monthly frequency
        let monthly = ScheduleFrequency::Monthly {
            day_of_month: 15,
            hour: 10,
            minute: 30,
        };
        
        let next_monthly = monthly.next_run_time(now);
        assert!(next_monthly.is_some());
        
        // Test Quarterly frequency
        let quarterly = ScheduleFrequency::Quarterly {
            month: 1, // January
            day_of_month: 15,
            hour: 10,
            minute: 30,
        };
        
        let next_quarterly = quarterly.next_run_time(now);
        assert!(next_quarterly.is_some());
    }
    
    #[tokio::test]
    async fn test_add_and_get_schedule() {
        // Create dependencies
        let storage = Arc::new(InMemoryAuditTrailStorage::new());
        let audit_manager = Arc::new(AuditTrailManager::new(storage.clone()));
        
        // Create mock notification service
        let notification_service = Arc::new(MockNotificationServiceMock::new());
        
        // Create mock query API
        let query_api = Arc::new(crate::calculations::query_api::QueryApi::new(
            audit_manager.clone(),
            Arc::new(crate::calculations::distributed_cache::CacheFactory::create_in_memory_cache()),
            Arc::new(crate::calculations::currency::CurrencyConverter::new(
                Arc::new(MockExchangeRateProviderMock::new()),
                "USD".to_string(),
            )),
            Arc::new(MockDataAccessServiceMock::new()),
        ));
        
        // Create scheduler
        let scheduler = CalculationScheduler::new(
            query_api,
            audit_manager,
            notification_service,
        );
        
        // Create a schedule
        let schedule = ScheduledCalculation {
            id: "test-schedule".to_string(),
            name: "Test Schedule".to_string(),
            description: Some("Test description".to_string()),
            calculation_type: ScheduledCalculationType::Performance(
                crate::calculations::query_api::PerformanceQueryParams {
                    portfolio_id: "test-portfolio".to_string(),
                    start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
                    end_date: NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
                    twr_method: Some("daily".to_string()),
                    include_risk_metrics: Some(true),
                    include_periodic_returns: Some(false),
                    benchmark_id: None,
                    currency: None,
                    annualize: Some(false),
                    custom_params: None,
                }
            ),
            frequency: ScheduleFrequency::Daily {
                hour: 10,
                minute: 0,
            },
            enabled: true,
            notification_channels: vec![
                NotificationChannel::Email {
                    recipients: vec!["test@example.com".to_string()],
                    subject_template: "Performance Report for {{schedule.name}}".to_string(),
                    body_template: "Performance report is ready. Status: {{result.status}}".to_string(),
                },
            ],
            last_run_time: None,
            next_run_time: None,
            created_by: "test-user".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Add the schedule
        scheduler.add_schedule(schedule.clone()).await.unwrap();
        
        // Get the schedule
        let retrieved = scheduler.get_schedule(&schedule.id).await.unwrap();
        
        assert_eq!(retrieved.id, schedule.id);
        assert_eq!(retrieved.name, schedule.name);
        
        // Get all schedules
        let all_schedules = scheduler.get_all_schedules().await.unwrap();
        assert_eq!(all_schedules.len(), 1);
        assert_eq!(all_schedules[0].id, schedule.id);
    }
} 