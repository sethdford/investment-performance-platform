# User Guide

This guide provides detailed information for users of the Investment Performance Calculator.

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Managing Portfolios](#managing-portfolios)
4. [Managing Items](#managing-items)
5. [Managing Transactions](#managing-transactions)
6. [Calculating Performance](#calculating-performance)
7. [Viewing Performance Metrics](#viewing-performance-metrics)
8. [Batch Calculations](#batch-calculations)
9. [API Reference](#api-reference)
10. [Troubleshooting](#troubleshooting)

## Introduction

The Investment Performance Calculator is a powerful tool for calculating investment performance metrics. It allows you to:

- Manage portfolios, items, and transactions
- Calculate performance metrics such as TWR, MWR, volatility, Sharpe ratio, etc.
- Compare portfolio performance against benchmarks
- Analyze performance over time with different intervals
- Calculate performance for multiple portfolios in parallel

## Getting Started

### Authentication

To use the Investment Performance Calculator API, you need to authenticate using a JWT token. You can obtain a token by calling the authentication endpoint with your credentials.

Example:

```bash
curl -X POST https://api.example.com/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "your-username", "password": "your-password"}'
```

Response:

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 3600
}
```

Use this token in the `Authorization` header for all subsequent requests:

```bash
curl -X GET https://api.example.com/v1/portfolios \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

### API Base URL

The base URL for the API is:

- Production: `https://api.example.com/v1`
- Development: `https://api-dev.example.com/v1`

## Managing Portfolios

### Creating a Portfolio

To create a portfolio, send a POST request to `/portfolios` with the portfolio data:

```bash
curl -X POST https://api.example.com/v1/portfolios \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Portfolio",
    "client_id": "client-123",
    "inception_date": "2022-01-01",
    "benchmark_id": "SPY"
  }'
```

Response:

```json
{
  "portfolio": {
    "id": "portfolio-123",
    "tenant_id": "tenant-123",
    "name": "My Portfolio",
    "client_id": "client-123",
    "inception_date": "2022-01-01",
    "benchmark_id": "SPY",
    "created_at": "2023-01-01T00:00:00Z",
    "updated_at": "2023-01-01T00:00:00Z",
    "status": "active",
    "metadata": {}
  }
}
```

### Getting a Portfolio

To get a portfolio, send a GET request to `/portfolios/{id}`:

```bash
curl -X GET https://api.example.com/v1/portfolios/portfolio-123 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

Response:

```json
{
  "portfolio": {
    "id": "portfolio-123",
    "tenant_id": "tenant-123",
    "name": "My Portfolio",
    "client_id": "client-123",
    "inception_date": "2022-01-01",
    "benchmark_id": "SPY",
    "created_at": "2023-01-01T00:00:00Z",
    "updated_at": "2023-01-01T00:00:00Z",
    "status": "active",
    "metadata": {}
  }
}
```

### Updating a Portfolio

To update a portfolio, send a PUT request to `/portfolios/{id}` with the updated data:

```bash
curl -X PUT https://api.example.com/v1/portfolios/portfolio-123 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Updated Portfolio",
    "benchmark_id": "QQQ",
    "status": "active"
  }'
```

Response:

```json
{
  "portfolio": {
    "id": "portfolio-123",
    "tenant_id": "tenant-123",
    "name": "My Updated Portfolio",
    "client_id": "client-123",
    "inception_date": "2022-01-01",
    "benchmark_id": "QQQ",
    "created_at": "2023-01-01T00:00:00Z",
    "updated_at": "2023-01-02T00:00:00Z",
    "status": "active",
    "metadata": {}
  }
}
```

### Deleting a Portfolio

To delete a portfolio, send a DELETE request to `/portfolios/{id}`:

```bash
curl -X DELETE https://api.example.com/v1/portfolios/portfolio-123 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

Response: 204 No Content

### Listing Portfolios

To list portfolios, send a GET request to `/portfolios`:

```bash
curl -X GET https://api.example.com/v1/portfolios \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

Response:

```json
{
  "items": [
    {
      "id": "portfolio-123",
      "tenant_id": "tenant-123",
      "name": "My Portfolio",
      "client_id": "client-123",
      "inception_date": "2022-01-01",
      "benchmark_id": "SPY",
      "created_at": "2023-01-01T00:00:00Z",
      "updated_at": "2023-01-01T00:00:00Z",
      "status": "active",
      "metadata": {}
    },
    {
      "id": "portfolio-456",
      "tenant_id": "tenant-123",
      "name": "Another Portfolio",
      "client_id": "client-456",
      "inception_date": "2022-02-01",
      "benchmark_id": "QQQ",
      "created_at": "2023-02-01T00:00:00Z",
      "updated_at": "2023-02-01T00:00:00Z",
      "status": "active",
      "metadata": {}
    }
  ],
  "next_token": "next-page-token"
}
```

You can use the `next_token` to get the next page of results:

```bash
curl -X GET https://api.example.com/v1/portfolios?next_token=next-page-token \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

## Managing Items

### Creating an Item

To create an item in a portfolio, send a POST request to `/portfolios/{id}/items` with the item data:

```bash
curl -X POST https://api.example.com/v1/portfolios/portfolio-123/items \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Apple Inc.",
    "description": "Technology company",
    "asset_class": "equity",
    "security_id": "AAPL"
  }'
```

Response:

```json
{
  "id": "item-123",
  "tenant_id": "tenant-123",
  "portfolio_id": "portfolio-123",
  "name": "Apple Inc.",
  "description": "Technology company",
  "asset_class": "equity",
  "security_id": "AAPL",
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-01T00:00:00Z",
  "status": "active",
  "metadata": {}
}
```

### Getting an Item

To get an item, send a GET request to `/items/{id}`:

```bash
curl -X GET https://api.example.com/v1/items/item-123 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

Response:

```json
{
  "id": "item-123",
  "tenant_id": "tenant-123",
  "portfolio_id": "portfolio-123",
  "name": "Apple Inc.",
  "description": "Technology company",
  "asset_class": "equity",
  "security_id": "AAPL",
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-01T00:00:00Z",
  "status": "active",
  "metadata": {}
}
```

### Updating an Item

To update an item, send a PUT request to `/items/{id}` with the updated data:

```bash
curl -X PUT https://api.example.com/v1/items/item-123 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Apple Inc.",
    "description": "Technology company that makes iPhones",
    "asset_class": "equity",
    "security_id": "AAPL",
    "status": "active"
  }'
```

Response:

```json
{
  "id": "item-123",
  "tenant_id": "tenant-123",
  "portfolio_id": "portfolio-123",
  "name": "Apple Inc.",
  "description": "Technology company that makes iPhones",
  "asset_class": "equity",
  "security_id": "AAPL",
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-02T00:00:00Z",
  "status": "active",
  "metadata": {}
}
```

### Deleting an Item

To delete an item, send a DELETE request to `/items/{id}`:

```bash
curl -X DELETE https://api.example.com/v1/items/item-123 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

Response: 204 No Content

### Listing Items in a Portfolio

To list items in a portfolio, send a GET request to `/portfolios/{id}/items`:

```bash
curl -X GET https://api.example.com/v1/portfolios/portfolio-123/items \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

Response:

```json
{
  "items": [
    {
      "id": "item-123",
      "tenant_id": "tenant-123",
      "portfolio_id": "portfolio-123",
      "name": "Apple Inc.",
      "description": "Technology company",
      "asset_class": "equity",
      "security_id": "AAPL",
      "created_at": "2023-01-01T00:00:00Z",
      "updated_at": "2023-01-01T00:00:00Z",
      "status": "active",
      "metadata": {}
    },
    {
      "id": "item-456",
      "tenant_id": "tenant-123",
      "portfolio_id": "portfolio-123",
      "name": "Microsoft Corp.",
      "description": "Technology company",
      "asset_class": "equity",
      "security_id": "MSFT",
      "created_at": "2023-01-01T00:00:00Z",
      "updated_at": "2023-01-01T00:00:00Z",
      "status": "active",
      "metadata": {}
    }
  ],
  "total": 2,
  "page": 1,
  "limit": 10
}
```

## Managing Transactions

### Creating a Transaction

To create a transaction for an item, send a POST request to `/items/{id}/transactions` with the transaction data:

```bash
curl -X POST https://api.example.com/v1/items/item-123/transactions \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "transaction_type": "buy",
    "amount": 10000,
    "quantity": 100,
    "price": 100,
    "transaction_date": "2022-01-01"
  }'
```

Response:

```json
{
  "id": "transaction-123",
  "tenant_id": "tenant-123",
  "portfolio_id": "portfolio-123",
  "item_id": "item-123",
  "transaction_type": "buy",
  "amount": 10000,
  "quantity": 100,
  "price": 100,
  "transaction_date": "2022-01-01",
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-01T00:00:00Z",
  "status": "settled",
  "metadata": {}
}
```

### Getting a Transaction

To get a transaction, send a GET request to `/transactions/{id}`:

```bash
curl -X GET https://api.example.com/v1/transactions/transaction-123 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

Response:

```json
{
  "id": "transaction-123",
  "tenant_id": "tenant-123",
  "portfolio_id": "portfolio-123",
  "item_id": "item-123",
  "transaction_type": "buy",
  "amount": 10000,
  "quantity": 100,
  "price": 100,
  "transaction_date": "2022-01-01",
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-01T00:00:00Z",
  "status": "settled",
  "metadata": {}
}
```

### Updating a Transaction

To update a transaction, send a PUT request to `/transactions/{id}` with the updated data:

```bash
curl -X PUT https://api.example.com/v1/transactions/transaction-123 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "transaction_type": "buy",
    "amount": 10500,
    "quantity": 100,
    "price": 105,
    "transaction_date": "2022-01-01",
    "status": "settled"
  }'
```

Response:

```json
{
  "id": "transaction-123",
  "tenant_id": "tenant-123",
  "portfolio_id": "portfolio-123",
  "item_id": "item-123",
  "transaction_type": "buy",
  "amount": 10500,
  "quantity": 100,
  "price": 105,
  "transaction_date": "2022-01-01",
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-02T00:00:00Z",
  "status": "settled",
  "metadata": {}
}
```

### Deleting a Transaction

To delete a transaction, send a DELETE request to `/transactions/{id}`:

```bash
curl -X DELETE https://api.example.com/v1/transactions/transaction-123 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

Response: 204 No Content

### Listing Transactions for an Item

To list transactions for an item, send a GET request to `/items/{id}/transactions`:

```bash
curl -X GET https://api.example.com/v1/items/item-123/transactions \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

Response:

```json
{
  "items": [
    {
      "id": "transaction-123",
      "tenant_id": "tenant-123",
      "portfolio_id": "portfolio-123",
      "item_id": "item-123",
      "transaction_type": "buy",
      "amount": 10000,
      "quantity": 100,
      "price": 100,
      "transaction_date": "2022-01-01",
      "created_at": "2023-01-01T00:00:00Z",
      "updated_at": "2023-01-01T00:00:00Z",
      "status": "settled",
      "metadata": {}
    },
    {
      "id": "transaction-456",
      "tenant_id": "tenant-123",
      "portfolio_id": "portfolio-123",
      "item_id": "item-123",
      "transaction_type": "sell",
      "amount": -5250,
      "quantity": -50,
      "price": 105,
      "transaction_date": "2022-02-01",
      "created_at": "2023-02-01T00:00:00Z",
      "updated_at": "2023-02-01T00:00:00Z",
      "status": "settled",
      "metadata": {}
    }
  ],
  "total": 2,
  "page": 1,
  "limit": 10
}
```

## Calculating Performance

### Calculating Performance for a Portfolio

To calculate performance for a portfolio, send a POST request to `/calculate` with the calculation parameters:

```bash
curl -X POST https://api.example.com/v1/calculate \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "portfolio_id": "portfolio-123",
    "start_date": "2022-01-01",
    "end_date": "2022-12-31",
    "include_details": true
  }'
```

Response:

```json
{
  "portfolio_id": "portfolio-123",
  "start_date": "2022-01-01",
  "end_date": "2022-12-31",
  "twr": 0.0823,
  "mwr": 0.0791,
  "volatility": 0.1245,
  "sharpe_ratio": 0.6612,
  "max_drawdown": -0.0512,
  "benchmark_id": "SPY",
  "benchmark_return": 0.0712,
  "tracking_error": 0.0231,
  "information_ratio": 0.4805,
  "calculated_at": "2023-01-01T00:00:00Z",
  "details": {
    "time_series": [
      {
        "date": "2022-01-31",
        "twr": 0.0123,
        "mwr": 0.0119,
        "volatility": 0.1102,
        "benchmark_return": 0.0098
      },
      {
        "date": "2022-02-28",
        "twr": 0.0156,
        "mwr": 0.0149,
        "volatility": 0.1134,
        "benchmark_return": 0.0112
      }
    ]
  }
}
```

### Getting Performance Metrics for a Portfolio

To get performance metrics for a portfolio, send a GET request to `/portfolios/{id}/performance`:

```bash
curl -X GET https://api.example.com/v1/portfolios/portfolio-123/performance?start_date=2022-01-01&end_date=2022-12-31&include_details=true \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

Response:

```json
{
  "twr": 0.0823,
  "mwr": 0.0791,
  "volatility": 0.1245,
  "sharpe_ratio": 0.6612,
  "max_drawdown": -0.0512,
  "benchmark_id": "SPY",
  "benchmark_return": 0.0712,
  "tracking_error": 0.0231,
  "information_ratio": 0.4805,
  "details": {
    "time_series": [
      {
        "date": "2022-01-31",
        "twr": 0.0123,
        "mwr": 0.0119,
        "volatility": 0.1102,
        "benchmark_return": 0.0098
      },
      {
        "date": "2022-02-28",
        "twr": 0.0156,
        "mwr": 0.0149,
        "volatility": 0.1134,
        "benchmark_return": 0.0112
      }
    ]
  }
}
```

### Getting Time-Series Performance Data for a Portfolio

To get time-series performance data for a portfolio, send a GET request to `/portfolios/{id}/performance/time-series`:

```bash
curl -X GET https://api.example.com/v1/portfolios/portfolio-123/performance/time-series?start_date=2022-01-01&end_date=2022-12-31&interval=monthly&include_benchmark=true \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

Response:

```json
[
  {
    "date": "2022-01-31",
    "twr": 0.0123,
    "mwr": 0.0119,
    "volatility": 0.1102,
    "benchmark_return": 0.0098
  },
  {
    "date": "2022-02-28",
    "twr": 0.0156,
    "mwr": 0.0149,
    "volatility": 0.1134,
    "benchmark_return": 0.0112
  },
  {
    "date": "2022-03-31",
    "twr": -0.0023,
    "mwr": -0.0025,
    "volatility": 0.1156,
    "benchmark_return": -0.0015
  }
]
```

## Batch Calculations

### Calculating Performance for Multiple Portfolios

To calculate performance for multiple portfolios, send a POST request to `/batch-calculate` with the calculation parameters:

```bash
curl -X POST https://api.example.com/v1/batch-calculate \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "portfolio_ids": ["portfolio-123", "portfolio-456"],
    "start_date": "2022-01-01",
    "end_date": "2022-12-31",
    "include_details": true
  }'
```

Response:

```json
{
  "results": {
    "portfolio-123": {
      "twr": 0.0823,
      "mwr": 0.0791,
      "volatility": 0.1245,
      "sharpe_ratio": 0.6612,
      "max_drawdown": -0.0512,
      "benchmark_id": "SPY",
      "benchmark_return": 0.0712,
      "tracking_error": 0.0231,
      "information_ratio": 0.4805,
      "details": {
        "time_series": [
          {
            "date": "2022-01-31",
            "twr": 0.0123,
            "mwr": 0.0119,
            "volatility": 0.1102,
            "benchmark_return": 0.0098
          },
          {
            "date": "2022-02-28",
            "twr": 0.0156,
            "mwr": 0.0149,
            "volatility": 0.1134,
            "benchmark_return": 0.0112
          }
        ]
      }
    },
    "portfolio-456": {
      "twr": 0.0654,
      "mwr": 0.0623,
      "volatility": 0.1345,
      "sharpe_ratio": 0.4865,
      "max_drawdown": -0.0623,
      "benchmark_id": "QQQ",
      "benchmark_return": 0.0534,
      "tracking_error": 0.0312,
      "information_ratio": 0.3846,
      "details": {
        "time_series": [
          {
            "date": "2022-01-31",
            "twr": 0.0098,
            "mwr": 0.0095,
            "volatility": 0.1245,
            "benchmark_return": 0.0076
          },
          {
            "date": "2022-02-28",
            "twr": 0.0123,
            "mwr": 0.0118,
            "volatility": 0.1289,
            "benchmark_return": 0.0087
          }
        ]
      }
    }
  },
  "duration_ms": 1234.56
} 