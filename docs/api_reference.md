# API Reference

This document provides a comprehensive reference for the Investment Performance Calculator API.

## Base URL

```
https://api.example.com/v1
```

## Authentication

All API requests require authentication using a JWT token. Include the token in the `Authorization` header:

```
Authorization: Bearer <your_token>
```

## Error Handling

The API uses standard HTTP status codes to indicate the success or failure of requests:

- `200 OK`: The request was successful
- `201 Created`: The resource was successfully created
- `400 Bad Request`: The request was invalid
- `401 Unauthorized`: Authentication failed
- `403 Forbidden`: The authenticated user doesn't have permission
- `404 Not Found`: The requested resource was not found
- `409 Conflict`: The request conflicts with the current state
- `422 Unprocessable Entity`: Validation error
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error

Error responses have the following format:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "A human-readable error message",
    "details": {
      "field_name": "Specific error for this field"
    }
  },
  "request_id": "unique-request-identifier"
}
```

## Endpoints

### Portfolios

#### Create Portfolio

```
POST /portfolios
```

Creates a new portfolio.

**Request Body:**

```json
{
  "name": "My Portfolio",
  "description": "My investment portfolio",
  "currency": "USD",
  "benchmark_id": "benchmark-456",
  "tags": ["retirement", "long-term"]
}
```

**Response:**

```json
{
  "portfolio_id": "portfolio-123",
  "name": "My Portfolio",
  "description": "My investment portfolio",
  "currency": "USD",
  "benchmark_id": "benchmark-456",
  "tags": ["retirement", "long-term"],
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-01T00:00:00Z"
}
```

#### Get Portfolio

```
GET /portfolios/{portfolio_id}
```

Retrieves a portfolio by ID.

**Response:**

```json
{
  "portfolio_id": "portfolio-123",
  "name": "My Portfolio",
  "description": "My investment portfolio",
  "currency": "USD",
  "benchmark_id": "benchmark-456",
  "tags": ["retirement", "long-term"],
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-01T00:00:00Z"
}
```

#### Update Portfolio

```
PUT /portfolios/{portfolio_id}
```

Updates an existing portfolio.

**Request Body:**

```json
{
  "name": "Updated Portfolio Name",
  "description": "Updated description",
  "currency": "USD",
  "benchmark_id": "benchmark-789",
  "tags": ["retirement", "long-term", "aggressive"]
}
```

**Response:**

```json
{
  "portfolio_id": "portfolio-123",
  "name": "Updated Portfolio Name",
  "description": "Updated description",
  "currency": "USD",
  "benchmark_id": "benchmark-789",
  "tags": ["retirement", "long-term", "aggressive"],
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-02T00:00:00Z"
}
```

#### Delete Portfolio

```
DELETE /portfolios/{portfolio_id}
```

Deletes a portfolio.

**Response:**

```
204 No Content
```

#### List Portfolios

```
GET /portfolios
```

Lists all portfolios.

**Query Parameters:**

- `page`: Page number (default: 1)
- `limit`: Number of items per page (default: 20, max: 100)
- `sort`: Field to sort by (default: created_at)
- `order`: Sort order (asc or desc, default: desc)
- `tag`: Filter by tag

**Response:**

```json
{
  "items": [
    {
      "portfolio_id": "portfolio-123",
      "name": "My Portfolio",
      "description": "My investment portfolio",
      "currency": "USD",
      "benchmark_id": "benchmark-456",
      "tags": ["retirement", "long-term"],
      "created_at": "2023-01-01T00:00:00Z",
      "updated_at": "2023-01-01T00:00:00Z"
    },
    {
      "portfolio_id": "portfolio-456",
      "name": "Another Portfolio",
      "description": "Another investment portfolio",
      "currency": "EUR",
      "benchmark_id": "benchmark-789",
      "tags": ["short-term", "aggressive"],
      "created_at": "2023-01-02T00:00:00Z",
      "updated_at": "2023-01-02T00:00:00Z"
    }
  ],
  "pagination": {
    "total": 42,
    "page": 1,
    "limit": 20,
    "pages": 3
  }
}
```

### Items

#### Create Item

```
POST /portfolios/{portfolio_id}/items
```

Creates a new item in a portfolio.

**Request Body:**

```json
{
  "name": "Apple Inc.",
  "symbol": "AAPL",
  "type": "stock",
  "description": "Apple Inc. stock",
  "tags": ["technology", "blue-chip"]
}
```

**Response:**

```json
{
  "item_id": "item-123",
  "portfolio_id": "portfolio-123",
  "name": "Apple Inc.",
  "symbol": "AAPL",
  "type": "stock",
  "description": "Apple Inc. stock",
  "tags": ["technology", "blue-chip"],
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-01T00:00:00Z"
}
```

#### Get Item

```
GET /items/{item_id}
```

Retrieves an item by ID.

**Response:**

```json
{
  "item_id": "item-123",
  "portfolio_id": "portfolio-123",
  "name": "Apple Inc.",
  "symbol": "AAPL",
  "type": "stock",
  "description": "Apple Inc. stock",
  "tags": ["technology", "blue-chip"],
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-01T00:00:00Z"
}
```

#### Update Item

```
PUT /items/{item_id}
```

Updates an existing item.

**Request Body:**

```json
{
  "name": "Apple Inc.",
  "symbol": "AAPL",
  "type": "stock",
  "description": "Updated description",
  "tags": ["technology", "blue-chip", "dividend"]
}
```

**Response:**

```json
{
  "item_id": "item-123",
  "portfolio_id": "portfolio-123",
  "name": "Apple Inc.",
  "symbol": "AAPL",
  "type": "stock",
  "description": "Updated description",
  "tags": ["technology", "blue-chip", "dividend"],
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-02T00:00:00Z"
}
```

#### Delete Item

```
DELETE /items/{item_id}
```

Deletes an item.

**Response:**

```
204 No Content
```

#### List Items

```
GET /portfolios/{portfolio_id}/items
```

Lists all items in a portfolio.

**Query Parameters:**

- `page`: Page number (default: 1)
- `limit`: Number of items per page (default: 20, max: 100)
- `sort`: Field to sort by (default: created_at)
- `order`: Sort order (asc or desc, default: desc)
- `type`: Filter by item type
- `tag`: Filter by tag

**Response:**

```json
{
  "items": [
    {
      "item_id": "item-123",
      "portfolio_id": "portfolio-123",
      "name": "Apple Inc.",
      "symbol": "AAPL",
      "type": "stock",
      "description": "Apple Inc. stock",
      "tags": ["technology", "blue-chip"],
      "created_at": "2023-01-01T00:00:00Z",
      "updated_at": "2023-01-01T00:00:00Z"
    },
    {
      "item_id": "item-456",
      "portfolio_id": "portfolio-123",
      "name": "Microsoft Corporation",
      "symbol": "MSFT",
      "type": "stock",
      "description": "Microsoft Corporation stock",
      "tags": ["technology", "blue-chip"],
      "created_at": "2023-01-02T00:00:00Z",
      "updated_at": "2023-01-02T00:00:00Z"
    }
  ],
  "pagination": {
    "total": 42,
    "page": 1,
    "limit": 20,
    "pages": 3
  }
}
```

### Transactions

#### Create Transaction

```
POST /items/{item_id}/transactions
```

Creates a new transaction for an item.

**Request Body:**

```json
{
  "date": "2023-01-01",
  "type": "buy",
  "quantity": 10,
  "price": 150.00,
  "currency": "USD",
  "fees": 7.99,
  "notes": "Initial purchase"
}
```

**Response:**

```json
{
  "transaction_id": "transaction-123",
  "item_id": "item-123",
  "portfolio_id": "portfolio-123",
  "date": "2023-01-01",
  "type": "buy",
  "quantity": 10,
  "price": 150.00,
  "currency": "USD",
  "fees": 7.99,
  "notes": "Initial purchase",
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-01T00:00:00Z"
}
```

#### Get Transaction

```
GET /transactions/{transaction_id}
```

Retrieves a transaction by ID.

**Response:**

```json
{
  "transaction_id": "transaction-123",
  "item_id": "item-123",
  "portfolio_id": "portfolio-123",
  "date": "2023-01-01",
  "type": "buy",
  "quantity": 10,
  "price": 150.00,
  "currency": "USD",
  "fees": 7.99,
  "notes": "Initial purchase",
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-01T00:00:00Z"
}
```

#### Update Transaction

```
PUT /transactions/{transaction_id}
```

Updates an existing transaction.

**Request Body:**

```json
{
  "date": "2023-01-01",
  "type": "buy",
  "quantity": 15,
  "price": 150.00,
  "currency": "USD",
  "fees": 7.99,
  "notes": "Updated notes"
}
```

**Response:**

```json
{
  "transaction_id": "transaction-123",
  "item_id": "item-123",
  "portfolio_id": "portfolio-123",
  "date": "2023-01-01",
  "type": "buy",
  "quantity": 15,
  "price": 150.00,
  "currency": "USD",
  "fees": 7.99,
  "notes": "Updated notes",
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-02T00:00:00Z"
}
```

#### Delete Transaction

```
DELETE /transactions/{transaction_id}
```

Deletes a transaction.

**Response:**

```
204 No Content
```

#### List Transactions

```
GET /items/{item_id}/transactions
```

Lists all transactions for an item.

**Query Parameters:**

- `page`: Page number (default: 1)
- `limit`: Number of items per page (default: 20, max: 100)
- `sort`: Field to sort by (default: date)
- `order`: Sort order (asc or desc, default: desc)
- `type`: Filter by transaction type
- `start_date`: Filter by start date (inclusive)
- `end_date`: Filter by end date (inclusive)

**Response:**

```json
{
  "items": [
    {
      "transaction_id": "transaction-123",
      "item_id": "item-123",
      "portfolio_id": "portfolio-123",
      "date": "2023-01-01",
      "type": "buy",
      "quantity": 10,
      "price": 150.00,
      "currency": "USD",
      "fees": 7.99,
      "notes": "Initial purchase",
      "created_at": "2023-01-01T00:00:00Z",
      "updated_at": "2023-01-01T00:00:00Z"
    },
    {
      "transaction_id": "transaction-456",
      "item_id": "item-123",
      "portfolio_id": "portfolio-123",
      "date": "2023-01-15",
      "type": "buy",
      "quantity": 5,
      "price": 155.00,
      "currency": "USD",
      "fees": 7.99,
      "notes": "Additional purchase",
      "created_at": "2023-01-15T00:00:00Z",
      "updated_at": "2023-01-15T00:00:00Z"
    }
  ],
  "pagination": {
    "total": 42,
    "page": 1,
    "limit": 20,
    "pages": 3
  }
}
```

### Performance

#### Calculate Performance

```
POST /portfolios/{portfolio_id}/calculate
```

Calculates performance metrics for a portfolio.

**Request Body:**

```json
{
  "start_date": "2023-01-01",
  "end_date": "2023-12-31",
  "benchmark_id": "benchmark-456",
  "include_details": true
}
```

**Response:**

```json
{
  "portfolio_id": "portfolio-123",
  "start_date": "2023-01-01",
  "end_date": "2023-12-31",
  "metrics": {
    "twr": 0.0823,
    "mwr": 0.0791,
    "volatility": 0.1245,
    "sharpe_ratio": 0.6612,
    "max_drawdown": -0.0512,
    "benchmark_id": "benchmark-456",
    "benchmark_return": 0.0712,
    "tracking_error": 0.0231,
    "information_ratio": 0.4805
  },
  "details": {
    "time_series": [
      {
        "date": "2023-01-31",
        "twr": 0.0123,
        "mwr": 0.0119,
        "volatility": 0.1102,
        "benchmark_return": 0.0098
      },
      {
        "date": "2023-02-28",
        "twr": 0.0245,
        "mwr": 0.0238,
        "volatility": 0.1125,
        "benchmark_return": 0.0201
      }
    ]
  },
  "duration_ms": 1234.56
}
```

#### Get Performance

```
GET /portfolios/{portfolio_id}/performance
```

Retrieves the latest performance metrics for a portfolio.

**Query Parameters:**

- `start_date`: Start date (required)
- `end_date`: End date (required)
- `benchmark_id`: Benchmark ID (optional)
- `include_details`: Whether to include time series details (default: false)

**Response:**

```json
{
  "portfolio_id": "portfolio-123",
  "start_date": "2023-01-01",
  "end_date": "2023-12-31",
  "metrics": {
    "twr": 0.0823,
    "mwr": 0.0791,
    "volatility": 0.1245,
    "sharpe_ratio": 0.6612,
    "max_drawdown": -0.0512,
    "benchmark_id": "benchmark-456",
    "benchmark_return": 0.0712,
    "tracking_error": 0.0231,
    "information_ratio": 0.4805
  },
  "details": {
    "time_series": [
      {
        "date": "2023-01-31",
        "twr": 0.0123,
        "mwr": 0.0119,
        "volatility": 0.1102,
        "benchmark_return": 0.0098
      },
      {
        "date": "2023-02-28",
        "twr": 0.0245,
        "mwr": 0.0238,
        "volatility": 0.1125,
        "benchmark_return": 0.0201
      }
    ]
  },
  "last_calculated_at": "2023-12-31T23:59:59Z"
}
```

#### Batch Calculate Performance

```
POST /batch-calculate
```

Calculates performance metrics for multiple portfolios.

**Request Body:**

```json
{
  "portfolio_ids": ["portfolio-123", "portfolio-456"],
  "start_date": "2023-01-01",
  "end_date": "2023-12-31",
  "benchmark_id": "benchmark-789",
  "include_details": true
}
```

**Response:**

```json
{
  "results": {
    "portfolio-123": {
      "twr": 0.0823,
      "mwr": 0.0791,
      "volatility": 0.1245,
      "sharpe_ratio": 0.6612,
      "max_drawdown": -0.0512,
      "benchmark_id": "benchmark-789",
      "benchmark_return": 0.0712,
      "tracking_error": 0.0231,
      "information_ratio": 0.4805,
      "details": {
        "time_series": [
          {
            "date": "2023-01-31",
            "twr": 0.0123,
            "mwr": 0.0119,
            "volatility": 0.1102,
            "benchmark_return": 0.0098
          },
          {
            "date": "2023-02-28",
            "twr": 0.0245,
            "mwr": 0.0238,
            "volatility": 0.1125,
            "benchmark_return": 0.0201
          }
        ]
      }
    },
    "portfolio-456": {
      "twr": 0.0956,
      "mwr": 0.0912,
      "volatility": 0.1356,
      "sharpe_ratio": 0.7045,
      "max_drawdown": -0.0478,
      "benchmark_id": "benchmark-789",
      "benchmark_return": 0.0712,
      "tracking_error": 0.0267,
      "information_ratio": 0.9138,
      "details": {
        "time_series": [
          {
            "date": "2023-01-31",
            "twr": 0.0145,
            "mwr": 0.0139,
            "volatility": 0.1201,
            "benchmark_return": 0.0098
          },
          {
            "date": "2023-02-28",
            "twr": 0.0289,
            "mwr": 0.0276,
            "volatility": 0.1234,
            "benchmark_return": 0.0201
          }
        ]
      }
    }
  },
  "duration_ms": 2345.67
}
```

### Benchmarks

#### List Benchmarks

```
GET /benchmarks
```

Lists all available benchmarks.

**Query Parameters:**

- `page`: Page number (default: 1)
- `limit`: Number of items per page (default: 20, max: 100)
- `sort`: Field to sort by (default: name)
- `order`: Sort order (asc or desc, default: asc)

**Response:**

```json
{
  "items": [
    {
      "benchmark_id": "benchmark-123",
      "name": "S&P 500",
      "symbol": "SPX",
      "description": "Standard & Poor's 500 Index",
      "currency": "USD",
      "created_at": "2023-01-01T00:00:00Z",
      "updated_at": "2023-01-01T00:00:00Z"
    },
    {
      "benchmark_id": "benchmark-456",
      "name": "NASDAQ Composite",
      "symbol": "IXIC",
      "description": "NASDAQ Composite Index",
      "currency": "USD",
      "created_at": "2023-01-01T00:00:00Z",
      "updated_at": "2023-01-01T00:00:00Z"
    }
  ],
  "pagination": {
    "total": 42,
    "page": 1,
    "limit": 20,
    "pages": 3
  }
}
```

#### Get Benchmark

```
GET /benchmarks/{benchmark_id}
```

Retrieves a benchmark by ID.

**Response:**

```json
{
  "benchmark_id": "benchmark-123",
  "name": "S&P 500",
  "symbol": "SPX",
  "description": "Standard & Poor's 500 Index",
  "currency": "USD",
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-01T00:00:00Z"
}
```

## Data Models

### Portfolio

| Field | Type | Description |
|-------|------|-------------|
| portfolio_id | string | Unique identifier for the portfolio |
| name | string | Name of the portfolio |
| description | string | Description of the portfolio |
| currency | string | Base currency of the portfolio |
| benchmark_id | string | ID of the benchmark for the portfolio |
| tags | array of strings | Tags for the portfolio |
| created_at | string (ISO 8601) | Creation timestamp |
| updated_at | string (ISO 8601) | Last update timestamp |

### Item

| Field | Type | Description |
|-------|------|-------------|
| item_id | string | Unique identifier for the item |
| portfolio_id | string | ID of the portfolio the item belongs to |
| name | string | Name of the item |
| symbol | string | Symbol of the item |
| type | string | Type of the item (stock, bond, etc.) |
| description | string | Description of the item |
| tags | array of strings | Tags for the item |
| created_at | string (ISO 8601) | Creation timestamp |
| updated_at | string (ISO 8601) | Last update timestamp |

### Transaction

| Field | Type | Description |
|-------|------|-------------|
| transaction_id | string | Unique identifier for the transaction |
| item_id | string | ID of the item the transaction belongs to |
| portfolio_id | string | ID of the portfolio the transaction belongs to |
| date | string (YYYY-MM-DD) | Date of the transaction |
| type | string | Type of the transaction (buy, sell, dividend, etc.) |
| quantity | number | Quantity of the item |
| price | number | Price per unit |
| currency | string | Currency of the transaction |
| fees | number | Fees associated with the transaction |
| notes | string | Notes for the transaction |
| created_at | string (ISO 8601) | Creation timestamp |
| updated_at | string (ISO 8601) | Last update timestamp |

### Performance Metrics

| Field | Type | Description |
|-------|------|-------------|
| twr | number | Time-Weighted Return |
| mwr | number | Money-Weighted Return |
| volatility | number | Volatility (standard deviation of returns) |
| sharpe_ratio | number | Sharpe Ratio |
| max_drawdown | number | Maximum Drawdown |
| benchmark_id | string | ID of the benchmark |
| benchmark_return | number | Return of the benchmark |
| tracking_error | number | Tracking Error |
| information_ratio | number | Information Ratio |

## Rate Limiting

The API is rate limited to protect against abuse. The current limits are:

- 100 requests per minute per IP address
- 1000 requests per hour per IP address

Rate limit information is included in the response headers:

- `X-RateLimit-Limit`: The maximum number of requests allowed in the current period
- `X-RateLimit-Remaining`: The number of requests remaining in the current period
- `X-RateLimit-Reset`: The time at which the current rate limit window resets (Unix timestamp)

If you exceed the rate limit, you will receive a `429 Too Many Requests` response. 