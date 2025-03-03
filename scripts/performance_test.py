#!/usr/bin/env python3

import argparse
import concurrent.futures
import json
import requests
import time
import uuid
from datetime import datetime, timedelta

def generate_portfolio():
    """Generate a random portfolio."""
    portfolio_id = str(uuid.uuid4())
    return {
        "name": f"Test Portfolio {portfolio_id}",
        "client_id": "test-client",
        "inception_date": (datetime.now() - timedelta(days=365)).strftime("%Y-%m-%d"),
        "benchmark_id": "SPY"
    }

def generate_item(portfolio_id):
    """Generate a random item."""
    item_id = str(uuid.uuid4())
    return {
        "name": f"Test Item {item_id}",
        "description": "Test item for performance testing",
        "asset_class": "equity",
        "security_id": "AAPL",
        "portfolio_id": portfolio_id
    }

def generate_transaction(item_id, portfolio_id):
    """Generate a random transaction."""
    transaction_id = str(uuid.uuid4())
    return {
        "transaction_type": "buy",
        "amount": 10000,
        "quantity": 100,
        "price": 100,
        "transaction_date": (datetime.now() - timedelta(days=30)).strftime("%Y-%m-%d"),
        "item_id": item_id,
        "portfolio_id": portfolio_id
    }

def create_portfolio(api_url, token):
    """Create a portfolio."""
    portfolio = generate_portfolio()
    response = requests.post(
        f"{api_url}/portfolios",
        headers={"Authorization": f"Bearer {token}", "Content-Type": "application/json"},
        json=portfolio
    )
    if response.status_code == 201:
        return response.json()["portfolio"]["id"]
    else:
        print(f"Failed to create portfolio: {response.status_code} {response.text}")
        return None

def create_item(api_url, token, portfolio_id):
    """Create an item."""
    item = generate_item(portfolio_id)
    response = requests.post(
        f"{api_url}/portfolios/{portfolio_id}/items",
        headers={"Authorization": f"Bearer {token}", "Content-Type": "application/json"},
        json=item
    )
    if response.status_code == 201:
        return response.json()["id"]
    else:
        print(f"Failed to create item: {response.status_code} {response.text}")
        return None

def create_transaction(api_url, token, item_id, portfolio_id):
    """Create a transaction."""
    transaction = generate_transaction(item_id, portfolio_id)
    response = requests.post(
        f"{api_url}/items/{item_id}/transactions",
        headers={"Authorization": f"Bearer {token}", "Content-Type": "application/json"},
        json=transaction
    )
    if response.status_code == 201:
        return response.json()["id"]
    else:
        print(f"Failed to create transaction: {response.status_code} {response.text}")
        return None

def calculate_performance(api_url, token, portfolio_id):
    """Calculate performance for a portfolio."""
    request = {
        "portfolio_id": portfolio_id,
        "start_date": (datetime.now() - timedelta(days=365)).strftime("%Y-%m-%d"),
        "end_date": datetime.now().strftime("%Y-%m-%d"),
        "include_details": True
    }
    start_time = time.time()
    response = requests.post(
        f"{api_url}/calculate",
        headers={"Authorization": f"Bearer {token}", "Content-Type": "application/json"},
        json=request
    )
    end_time = time.time()
    duration = end_time - start_time
    
    if response.status_code == 200:
        return duration
    else:
        print(f"Failed to calculate performance: {response.status_code} {response.text}")
        return None

def batch_calculate_performance(api_url, token, portfolio_ids):
    """Batch calculate performance for multiple portfolios."""
    request = {
        "portfolio_ids": portfolio_ids,
        "start_date": (datetime.now() - timedelta(days=365)).strftime("%Y-%m-%d"),
        "end_date": datetime.now().strftime("%Y-%m-%d"),
        "include_details": True
    }
    start_time = time.time()
    response = requests.post(
        f"{api_url}/batch-calculate",
        headers={"Authorization": f"Bearer {token}", "Content-Type": "application/json"},
        json=request
    )
    end_time = time.time()
    duration = end_time - start_time
    
    if response.status_code == 200:
        return duration
    else:
        print(f"Failed to batch calculate performance: {response.status_code} {response.text}")
        return None

def run_test(api_url, token, num_portfolios, num_items_per_portfolio, num_transactions_per_item, concurrency):
    """Run the performance test."""
    print(f"Running performance test with {num_portfolios} portfolios, {num_items_per_portfolio} items per portfolio, {num_transactions_per_item} transactions per item, and concurrency {concurrency}")
    
    # Create portfolios
    print("Creating portfolios...")
    portfolio_ids = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=concurrency) as executor:
        futures = [executor.submit(create_portfolio, api_url, token) for _ in range(num_portfolios)]
        for future in concurrent.futures.as_completed(futures):
            portfolio_id = future.result()
            if portfolio_id:
                portfolio_ids.append(portfolio_id)
    
    print(f"Created {len(portfolio_ids)} portfolios")
    
    # Create items
    print("Creating items...")
    items = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=concurrency) as executor:
        futures = []
        for portfolio_id in portfolio_ids:
            for _ in range(num_items_per_portfolio):
                futures.append(executor.submit(create_item, api_url, token, portfolio_id))
        
        for future in concurrent.futures.as_completed(futures):
            item_id = future.result()
            if item_id:
                items.append((item_id, portfolio_id))
    
    print(f"Created {len(items)} items")
    
    # Create transactions
    print("Creating transactions...")
    transactions = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=concurrency) as executor:
        futures = []
        for item_id, portfolio_id in items:
            for _ in range(num_transactions_per_item):
                futures.append(executor.submit(create_transaction, api_url, token, item_id, portfolio_id))
        
        for future in concurrent.futures.as_completed(futures):
            transaction_id = future.result()
            if transaction_id:
                transactions.append(transaction_id)
    
    print(f"Created {len(transactions)} transactions")
    
    # Calculate performance for each portfolio
    print("Calculating performance for each portfolio...")
    durations = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=concurrency) as executor:
        futures = [executor.submit(calculate_performance, api_url, token, portfolio_id) for portfolio_id in portfolio_ids]
        for future in concurrent.futures.as_completed(futures):
            duration = future.result()
            if duration:
                durations.append(duration)
    
    if durations:
        avg_duration = sum(durations) / len(durations)
        min_duration = min(durations)
        max_duration = max(durations)
        print(f"Performance calculation: avg={avg_duration:.2f}s, min={min_duration:.2f}s, max={max_duration:.2f}s")
    
    # Batch calculate performance
    print("Batch calculating performance...")
    batch_duration = batch_calculate_performance(api_url, token, portfolio_ids)
    if batch_duration:
        print(f"Batch performance calculation: {batch_duration:.2f}s")
    
    return {
        "portfolios": len(portfolio_ids),
        "items": len(items),
        "transactions": len(transactions),
        "individual_calculation": {
            "count": len(durations),
            "avg_duration": avg_duration if durations else None,
            "min_duration": min_duration if durations else None,
            "max_duration": max_duration if durations else None
        },
        "batch_calculation": {
            "duration": batch_duration
        }
    }

def main():
    """Main function."""
    parser = argparse.ArgumentParser(description="Performance test for Investment Performance Calculator API")
    parser.add_argument("--api-url", required=True, help="API URL")
    parser.add_argument("--token", required=True, help="Authentication token")
    parser.add_argument("--portfolios", type=int, default=5, help="Number of portfolios to create")
    parser.add_argument("--items", type=int, default=3, help="Number of items per portfolio")
    parser.add_argument("--transactions", type=int, default=10, help="Number of transactions per item")
    parser.add_argument("--concurrency", type=int, default=5, help="Concurrency level")
    parser.add_argument("--output", help="Output file for results (JSON)")
    
    args = parser.parse_args()
    
    results = run_test(
        args.api_url,
        args.token,
        args.portfolios,
        args.items,
        args.transactions,
        args.concurrency
    )
    
    # Print results
    print("\nTest Results:")
    print(f"Portfolios: {results['portfolios']}")
    print(f"Items: {results['items']}")
    print(f"Transactions: {results['transactions']}")
    
    if results['individual_calculation']['count'] > 0:
        print("\nIndividual Performance Calculation:")
        print(f"Count: {results['individual_calculation']['count']}")
        print(f"Average Duration: {results['individual_calculation']['avg_duration']:.2f}s")
        print(f"Min Duration: {results['individual_calculation']['min_duration']:.2f}s")
        print(f"Max Duration: {results['individual_calculation']['max_duration']:.2f}s")
    
    if results['batch_calculation']['duration']:
        print("\nBatch Performance Calculation:")
        print(f"Duration: {results['batch_calculation']['duration']:.2f}s")
    
    # Save results to file if specified
    if args.output:
        with open(args.output, 'w') as f:
            json.dump(results, f, indent=2)
        print(f"\nResults saved to {args.output}")

if __name__ == "__main__":
    main()