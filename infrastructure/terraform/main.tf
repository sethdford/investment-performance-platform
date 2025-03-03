provider "aws" {
  region = var.aws_region
}

# DynamoDB Table
resource "aws_dynamodb_table" "main" {
  name         = "investment-performance-${var.environment}"
  billing_mode = "PROVISIONED"
  
  read_capacity  = var.dynamodb_read_capacity
  write_capacity = var.dynamodb_write_capacity
  
  hash_key  = "PK"
  range_key = "SK"
  
  attribute {
    name = "PK"
    type = "S"
  }
  
  attribute {
    name = "SK"
    type = "S"
  }
  
  attribute {
    name = "GSI1PK"
    type = "S"
  }
  
  attribute {
    name = "GSI1SK"
    type = "S"
  }
  
  attribute {
    name = "GSI2PK"
    type = "S"
  }
  
  attribute {
    name = "GSI2SK"
    type = "S"
  }
  
  global_secondary_index {
    name            = "GSI1"
    hash_key        = "GSI1PK"
    range_key       = "GSI1SK"
    projection_type = "ALL"
    read_capacity   = var.dynamodb_read_capacity
    write_capacity  = var.dynamodb_write_capacity
  }
  
  global_secondary_index {
    name            = "GSI2"
    hash_key        = "GSI2PK"
    range_key       = "GSI2SK"
    projection_type = "ALL"
    read_capacity   = var.dynamodb_read_capacity
    write_capacity  = var.dynamodb_write_capacity
  }
  
  point_in_time_recovery {
    enabled = true
  }
  
  server_side_encryption {
    enabled = true
  }
  
  tags = {
    Environment = var.environment
    Project     = "InvestmentPerformanceCalculator"
  }
}

# Timestream Database
resource "aws_timestreamwrite_database" "main" {
  database_name = "${var.timestream_database_name}-${var.environment}"
  
  tags = {
    Environment = var.environment
    Project     = "InvestmentPerformanceCalculator"
  }
}

# Timestream Table
resource "aws_timestreamwrite_table" "main" {
  database_name = aws_timestreamwrite_database.main.database_name
  table_name    = "${var.timestream_table_name}-${var.environment}"
  
  retention_properties {
    memory_store_retention_period_in_hours = 24
    magnetic_store_retention_period_in_days = 365
  }
  
  tags = {
    Environment = var.environment
    Project     = "InvestmentPerformanceCalculator"
  }
}

# SQS Queue
resource "aws_sqs_queue" "main" {
  name                      = "${var.sqs_queue_name}-${var.environment}"
  visibility_timeout_seconds = 300
  message_retention_seconds = 1209600
  
  redrive_policy = jsonencode({
    deadLetterTargetArn = aws_sqs_queue.dlq.arn
    maxReceiveCount     = 5
  })
  
  tags = {
    Environment = var.environment
    Project     = "InvestmentPerformanceCalculator"
  }
}

# SQS Dead Letter Queue
resource "aws_sqs_queue" "dlq" {
  name                      = "${var.sqs_queue_name}-${var.environment}-dlq"
  message_retention_seconds = 1209600
  
  tags = {
    Environment = var.environment
    Project     = "InvestmentPerformanceCalculator"
  }
}

# Lambda Execution Role
resource "aws_iam_role" "lambda_execution_role" {
  name = "investment-performance-lambda-role-${var.environment}"
  
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
      }
    ]
  })
  
  tags = {
    Environment = var.environment
    Project     = "InvestmentPerformanceCalculator"
  }
}

# Lambda Execution Role Policies
resource "aws_iam_role_policy_attachment" "lambda_basic_execution" {
  role       = aws_iam_role.lambda_execution_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy_attachment" "lambda_xray" {
  role       = aws_iam_role.lambda_execution_role.name
  policy_arn = "arn:aws:iam::aws:policy/AWSXrayWriteOnlyAccess"
}

resource "aws_iam_policy" "dynamodb_access" {
  name        = "investment-performance-dynamodb-access-${var.environment}"
  description = "Policy for DynamoDB access"
  
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = [
          "dynamodb:GetItem",
          "dynamodb:PutItem",
          "dynamodb:UpdateItem",
          "dynamodb:DeleteItem",
          "dynamodb:Query",
          "dynamodb:Scan",
          "dynamodb:BatchGetItem",
          "dynamodb:BatchWriteItem"
        ]
        Effect = "Allow"
        Resource = [
          aws_dynamodb_table.main.arn,
          "${aws_dynamodb_table.main.arn}/index/*"
        ]
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "dynamodb_access" {
  role       = aws_iam_role.lambda_execution_role.name
  policy_arn = aws_iam_policy.dynamodb_access.arn
}

resource "aws_iam_policy" "timestream_access" {
  name        = "investment-performance-timestream-access-${var.environment}"
  description = "Policy for Timestream access"
  
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = [
          "timestream:WriteRecords",
          "timestream:DescribeEndpoints"
        ]
        Effect = "Allow"
        Resource = "arn:aws:timestream:${var.aws_region}:${data.aws_caller_identity.current.account_id}:database/${aws_timestreamwrite_database.main.database_name}/table/${aws_timestreamwrite_table.main.table_name}"
      },
      {
        Action = [
          "timestream:Select",
          "timestream:DescribeEndpoints"
        ]
        Effect = "Allow"
        Resource = "arn:aws:timestream:${var.aws_region}:${data.aws_caller_identity.current.account_id}:database/${aws_timestreamwrite_database.main.database_name}/table/${aws_timestreamwrite_table.main.table_name}"
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "timestream_access" {
  role       = aws_iam_role.lambda_execution_role.name
  policy_arn = aws_iam_policy.timestream_access.arn
}

resource "aws_iam_policy" "sqs_access" {
  name        = "investment-performance-sqs-access-${var.environment}"
  description = "Policy for SQS access"
  
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = [
          "sqs:SendMessage",
          "sqs:ReceiveMessage",
          "sqs:DeleteMessage",
          "sqs:GetQueueAttributes"
        ]
        Effect = "Allow"
        Resource = [
          aws_sqs_queue.main.arn,
          aws_sqs_queue.dlq.arn
        ]
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "sqs_access" {
  role       = aws_iam_role.lambda_execution_role.name
  policy_arn = aws_iam_policy.sqs_access.arn
}

# API Handler Lambda Function
resource "aws_lambda_function" "api_handler" {
  function_name = "investment-performance-api-handler-${var.environment}"
  handler       = "bootstrap"
  runtime       = "provided.al2"
  role          = aws_iam_role.lambda_execution_role.arn
  memory_size   = var.lambda_memory_size
  timeout       = var.lambda_timeout
  
  s3_bucket = var.lambda_code_s3_bucket
  s3_key    = var.api_handler_function_s3_key
  
  environment {
    variables = {
      ENVIRONMENT             = var.environment
      TABLE_NAME              = aws_dynamodb_table.main.name
      TIMESTREAM_DATABASE_NAME = aws_timestreamwrite_database.main.database_name
      TIMESTREAM_TABLE_NAME   = aws_timestreamwrite_table.main.table_name
      QUEUE_URL               = aws_sqs_queue.main.url
    }
  }
  
  tracing_config {
    mode = "Active"
  }
  
  tags = {
    Environment = var.environment
    Project     = "InvestmentPerformanceCalculator"
  }
}

# Event Processor Lambda Function
resource "aws_lambda_function" "event_processor" {
  function_name = "investment-performance-event-processor-${var.environment}"
  handler       = "bootstrap"
  runtime       = "provided.al2"
  role          = aws_iam_role.lambda_execution_role.arn
  memory_size   = var.lambda_memory_size
  timeout       = var.lambda_timeout
  
  s3_bucket = var.lambda_code_s3_bucket
  s3_key    = var.event_processor_function_s3_key
  
  environment {
    variables = {
      ENVIRONMENT             = var.environment
      TABLE_NAME              = aws_dynamodb_table.main.name
      TIMESTREAM_DATABASE_NAME = aws_timestreamwrite_database.main.database_name
      TIMESTREAM_TABLE_NAME   = aws_timestreamwrite_table.main.table_name
      QUEUE_URL               = aws_sqs_queue.main.url
    }
  }
  
  tracing_config {
    mode = "Active"
  }
  
  tags = {
    Environment = var.environment
    Project     = "InvestmentPerformanceCalculator"
  }
}

# Performance Calculator Lambda Function
resource "aws_lambda_function" "performance_calculator" {
  function_name = "investment-performance-calculator-${var.environment}"
  handler       = "bootstrap"
  runtime       = "provided.al2"
  role          = aws_iam_role.lambda_execution_role.arn
  memory_size   = var.lambda_memory_size
  timeout       = var.lambda_timeout
  
  s3_bucket = var.lambda_code_s3_bucket
  s3_key    = var.performance_calculator_function_s3_key
  
  environment {
    variables = {
      ENVIRONMENT             = var.environment
      TABLE_NAME              = aws_dynamodb_table.main.name
      TIMESTREAM_DATABASE_NAME = aws_timestreamwrite_database.main.database_name
      TIMESTREAM_TABLE_NAME   = aws_timestreamwrite_table.main.table_name
      QUEUE_URL               = aws_sqs_queue.main.url
    }
  }
  
  tracing_config {
    mode = "Active"
  }
  
  tags = {
    Environment = var.environment
    Project     = "InvestmentPerformanceCalculator"
  }
}

# SQS Event Source Mapping for Event Processor
resource "aws_lambda_event_source_mapping" "event_processor_sqs" {
  event_source_arn = aws_sqs_queue.main.arn
  function_name    = aws_lambda_function.event_processor.arn
  batch_size       = 10
  enabled          = true
}

# API Gateway
resource "aws_api_gateway_rest_api" "main" {
  name        = "investment-performance-api-${var.environment}"
  description = "Investment Performance Calculator API"
  
  endpoint_configuration {
    types = ["REGIONAL"]
  }
  
  tags = {
    Environment = var.environment
    Project     = "InvestmentPerformanceCalculator"
  }
}

# API Gateway Root Resource Method
resource "aws_api_gateway_method" "root" {
  rest_api_id   = aws_api_gateway_rest_api.main.id
  resource_id   = aws_api_gateway_rest_api.main.root_resource_id
  http_method   = "ANY"
  authorization_type = "NONE"
}

# API Gateway Integration
resource "aws_api_gateway_integration" "root" {
  rest_api_id = aws_api_gateway_rest_api.main.id
  resource_id = aws_api_gateway_rest_api.main.root_resource_id
  http_method = aws_api_gateway_method.root.http_method
  
  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.api_handler.invoke_arn
}

# API Gateway Deployment
resource "aws_api_gateway_deployment" "main" {
  depends_on = [
    aws_api_gateway_integration.root
  ]
  
  rest_api_id = aws_api_gateway_rest_api.main.id
  stage_name  = var.api_gateway_stage_name
}

# API Gateway Lambda Permission
resource "aws_lambda_permission" "api_gateway" {
  statement_id  = "AllowAPIGatewayInvoke"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.api_handler.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_api_gateway_rest_api.main.execution_arn}/*/*"
}

# CloudWatch Alarms
resource "aws_cloudwatch_metric_alarm" "api_handler_errors" {
  alarm_name          = "ApiHandlerErrors-${var.environment}"
  alarm_description   = "Alarm for API Handler Lambda errors"
  comparison_operator = "GreaterThanOrEqualToThreshold"
  evaluation_periods  = 1
  metric_name         = "Errors"
  namespace           = "AWS/Lambda"
  period              = 60
  statistic           = "Sum"
  threshold           = 1
  treat_missing_data  = "notBreaching"
  
  dimensions = {
    FunctionName = aws_lambda_function.api_handler.function_name
  }
  
  alarm_actions = var.alarm_actions
  ok_actions    = var.ok_actions
  
  tags = {
    Environment = var.environment
    Project     = "InvestmentPerformanceCalculator"
  }
}

resource "aws_cloudwatch_metric_alarm" "event_processor_errors" {
  alarm_name          = "EventProcessorErrors-${var.environment}"
  alarm_description   = "Alarm for Event Processor Lambda errors"
  comparison_operator = "GreaterThanOrEqualToThreshold"
  evaluation_periods  = 1
  metric_name         = "Errors"
  namespace           = "AWS/Lambda"
  period              = 60
  statistic           = "Sum"
  threshold           = 1
  treat_missing_data  = "notBreaching"
  
  dimensions = {
    FunctionName = aws_lambda_function.event_processor.function_name
  }
  
  alarm_actions = var.alarm_actions
  ok_actions    = var.ok_actions
  
  tags = {
    Environment = var.environment
    Project     = "InvestmentPerformanceCalculator"
  }
}

resource "aws_cloudwatch_metric_alarm" "performance_calculator_errors" {
  alarm_name          = "PerformanceCalculatorErrors-${var.environment}"
  alarm_description   = "Alarm for Performance Calculator Lambda errors"
  comparison_operator = "GreaterThanOrEqualToThreshold"
  evaluation_periods  = 1
  metric_name         = "Errors"
  namespace           = "AWS/Lambda"
  period              = 60
  statistic           = "Sum"
  threshold           = 1
  treat_missing_data  = "notBreaching"
  
  dimensions = {
    FunctionName = aws_lambda_function.performance_calculator.function_name
  }
  
  alarm_actions = var.alarm_actions
  ok_actions    = var.ok_actions
  
  tags = {
    Environment = var.environment
    Project     = "InvestmentPerformanceCalculator"
  }
}

resource "aws_cloudwatch_metric_alarm" "api_gateway_4xx_errors" {
  alarm_name          = "ApiGateway4xxErrors-${var.environment}"
  alarm_description   = "Alarm for API Gateway 4xx errors"
  comparison_operator = "GreaterThanOrEqualToThreshold"
  evaluation_periods  = 1
  metric_name         = "4XXError"
  namespace           = "AWS/ApiGateway"
  period              = 60
  statistic           = "Sum"
  threshold           = 10
  treat_missing_data  = "notBreaching"
  
  dimensions = {
    ApiName = aws_api_gateway_rest_api.main.name
  }
  
  alarm_actions = var.alarm_actions
  ok_actions    = var.ok_actions
  
  tags = {
    Environment = var.environment
    Project     = "InvestmentPerformanceCalculator"
  }
}

resource "aws_cloudwatch_metric_alarm" "api_gateway_5xx_errors" {
  alarm_name          = "ApiGateway5xxErrors-${var.environment}"
  alarm_description   = "Alarm for API Gateway 5xx errors"
  comparison_operator = "GreaterThanOrEqualToThreshold"
  evaluation_periods  = 1
  metric_name         = "5XXError"
  namespace           = "AWS/ApiGateway"
  period              = 60
  statistic           = "Sum"
  threshold           = 1
  treat_missing_data  = "notBreaching"
  
  dimensions = {
    ApiName = aws_api_gateway_rest_api.main.name
  }
  
  alarm_actions = var.alarm_actions
  ok_actions    = var.ok_actions
  
  tags = {
    Environment = var.environment
    Project     = "InvestmentPerformanceCalculator"
  }
}

resource "aws_cloudwatch_metric_alarm" "sqs_queue_depth" {
  alarm_name          = "SQSQueueDepth-${var.environment}"
  alarm_description   = "Alarm for SQS queue depth"
  comparison_operator = "GreaterThanOrEqualToThreshold"
  evaluation_periods  = 5
  metric_name         = "ApproximateNumberOfMessagesVisible"
  namespace           = "AWS/SQS"
  period              = 60
  statistic           = "Average"
  threshold           = 100
  treat_missing_data  = "notBreaching"
  
  dimensions = {
    QueueName = aws_sqs_queue.main.name
  }
  
  alarm_actions = var.alarm_actions
  ok_actions    = var.ok_actions
  
  tags = {
    Environment = var.environment
    Project     = "InvestmentPerformanceCalculator"
  }
}

# Data Sources
data "aws_caller_identity" "current" {} 