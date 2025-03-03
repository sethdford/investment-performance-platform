# Sample Events for Testing

This directory contains sample events for testing the Lambda functions locally using the AWS SAM CLI.

## API Gateway Events

- **api-gateway-get-items.json**: Sample event for testing the API handler with a GET request to `/api/items`.

## SQS Events

- **sqs-event.json**: Sample event for testing the event processor with item creation and update events.

## Performance Calculation Events

- **performance-calculation-event.json**: Sample event for testing the performance calculator with a calculation request.

## Using the Events

You can use these events to test the Lambda functions locally using the AWS SAM CLI:

```bash
# Test the API handler
sam local invoke ApiHandlerFunction -e events/api-gateway-get-items.json

# Test the event processor
sam local invoke EventProcessorFunction -e events/sqs-event.json

# Test the performance calculator
sam local invoke PerformanceCalculatorFunction -e events/performance-calculation-event.json
```

## Creating Custom Events

You can create custom events by modifying the existing sample events or creating new ones. The events should follow the structure expected by the Lambda functions:

- API Gateway events should follow the [API Gateway Lambda proxy integration format](https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format).
- SQS events should follow the [SQS event format](https://docs.aws.amazon.com/lambda/latest/dg/with-sqs.html).

## Event Structure

### API Gateway Event

```json
{
  "version": "2.0",
  "routeKey": "GET /api/items",
  "rawPath": "/api/items",
  "rawQueryString": "limit=10",
  "headers": { ... },
  "queryStringParameters": {
    "limit": "10"
  },
  "requestContext": { ... },
  "isBase64Encoded": false
}
```

### SQS Event

```json
{
  "Records": [
    {
      "messageId": "19dd0b57-b21e-4ac1-bd88-01bbb068cb78",
      "receiptHandle": "MessageReceiptHandle",
      "body": "{\"event_type\":\"ITEM_CREATED\",\"item\":{...}}",
      "attributes": { ... },
      "messageAttributes": {},
      "md5OfBody": "7b270e59b47ff90a553787216d55d91d",
      "eventSource": "aws:sqs",
      "eventSourceARN": "arn:aws:sqs:us-east-1:123456789012:MyQueue",
      "awsRegion": "us-east-1"
    }
  ]
}
```

### Performance Calculation Event

```json
{
  "Records": [
    {
      "messageId": "19dd0b57-b21e-4ac1-bd88-01bbb068cb78",
      "receiptHandle": "MessageReceiptHandle",
      "body": "{\"portfolio_id\":\"portfolio123\",\"start_date\":\"2023-01-01T00:00:00Z\",\"end_date\":\"2023-12-31T23:59:59Z\",\"calculation_types\":[\"TWR\",\"MWR\"],\"request_id\":\"req-123456\"}",
      "attributes": { ... },
      "messageAttributes": {},
      "md5OfBody": "7b270e59b47ff90a553787216d55d91d",
      "eventSource": "aws:sqs",
      "eventSourceARN": "arn:aws:sqs:us-east-1:123456789012:PerformanceCalculationQueue",
      "awsRegion": "us-east-1"
    }
  ]
}
``` 