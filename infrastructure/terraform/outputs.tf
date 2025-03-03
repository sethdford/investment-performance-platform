output "api_gateway_url" {
  description = "URL of the API Gateway"
  value       = "${aws_api_gateway_deployment.main.invoke_url}"
}

output "dynamodb_table_name" {
  description = "Name of the DynamoDB table"
  value       = aws_dynamodb_table.main.name
}

output "timestream_database_name" {
  description = "Name of the Timestream database"
  value       = aws_timestreamwrite_database.main.database_name
}

output "timestream_table_name" {
  description = "Name of the Timestream table"
  value       = aws_timestreamwrite_table.main.table_name
}

output "sqs_queue_url" {
  description = "URL of the SQS queue"
  value       = aws_sqs_queue.main.url
}

output "api_handler_function_name" {
  description = "Name of the API Handler Lambda function"
  value       = aws_lambda_function.api_handler.function_name
}

output "event_processor_function_name" {
  description = "Name of the Event Processor Lambda function"
  value       = aws_lambda_function.event_processor.function_name
}

output "performance_calculator_function_name" {
  description = "Name of the Performance Calculator Lambda function"
  value       = aws_lambda_function.performance_calculator.function_name
} 