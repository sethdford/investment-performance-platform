variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-east-1"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "dev"
  
  validation {
    condition     = contains(["dev", "test", "prod"], var.environment)
    error_message = "Environment must be one of: dev, test, prod."
  }
}

variable "dynamodb_read_capacity" {
  description = "DynamoDB read capacity units"
  type        = number
  default     = 5
}

variable "dynamodb_write_capacity" {
  description = "DynamoDB write capacity units"
  type        = number
  default     = 5
}

variable "timestream_database_name" {
  description = "Timestream database name"
  type        = string
  default     = "investment-performance"
}

variable "timestream_table_name" {
  description = "Timestream table name"
  type        = string
  default     = "performance-metrics"
}

variable "sqs_queue_name" {
  description = "SQS queue name"
  type        = string
  default     = "investment-performance-queue"
}

variable "lambda_memory_size" {
  description = "Lambda function memory size"
  type        = number
  default     = 512
}

variable "lambda_timeout" {
  description = "Lambda function timeout"
  type        = number
  default     = 30
}

variable "lambda_code_s3_bucket" {
  description = "S3 bucket for Lambda function code"
  type        = string
  default     = "investment-performance-lambda-code"
}

variable "api_handler_function_s3_key" {
  description = "S3 key for the API Handler Lambda function code"
  type        = string
  default     = "api-handler.zip"
}

variable "event_processor_function_s3_key" {
  description = "S3 key for the Event Processor Lambda function code"
  type        = string
  default     = "event-processor.zip"
}

variable "performance_calculator_function_s3_key" {
  description = "S3 key for the Performance Calculator Lambda function code"
  type        = string
  default     = "performance-calculator.zip"
}

variable "api_gateway_stage_name" {
  description = "API Gateway stage name"
  type        = string
  default     = "v1"
}

variable "alarm_actions" {
  description = "List of ARNs to notify when an alarm transitions to ALARM state"
  type        = list(string)
  default     = []
}

variable "ok_actions" {
  description = "List of ARNs to notify when an alarm transitions to OK state"
  type        = list(string)
  default     = []
} 