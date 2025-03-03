#!/usr/bin/env python3

import argparse
import concurrent.futures
import json
import requests
import time
import uuid
from datetime import datetime, timedelta
import statistics
import matplotlib.pyplot as plt
import numpy as np

def send_request(api_url, token, endpoint, method="GET", data=None):
    """Send a request to the API."""
    headers = {"Authorization": f"Bearer {token}", "Content-Type": "application/json"}
    
    start_time = time.time()
    
    if method == "GET":
        response = requests.get(f"{api_url}/{endpoint}", headers=headers)
    elif method == "POST":
        response = requests.post(f"{api_url}/{endpoint}", headers=headers, json=data)
    else:
        raise ValueError(f"Unsupported method: {method}")
    
    end_time = time.time()
    duration = end_time - start_time
    
    return {
        "status_code": response.status_code,
        "duration": duration,
        "success": 200 <= response.status_code < 300
    }

def run_load_test(api_url, token, endpoint, method="GET", data=None, requests_per_second=10, duration_seconds=60, concurrency=10):
    """Run a load test."""
    print(f"Running load test on {api_url}/{endpoint} with {requests_per_second} requests per second for {duration_seconds} seconds")
    
    # Calculate total number of requests
    total_requests = requests_per_second * duration_seconds
    
    # Calculate delay between requests
    delay = 1.0 / requests_per_second
    
    # Create a list to store results
    results = []
    
    # Start time
    start_time = time.time()
    
    # Create a thread pool
    with concurrent.futures.ThreadPoolExecutor(max_workers=concurrency) as executor:
        # Submit tasks
        futures = []
        for i in range(total_requests):
            # Submit task
            future = executor.submit(send_request, api_url, token, endpoint, method, data)
            futures.append(future)
            
            # Sleep to maintain request rate
            time.sleep(delay)
            
            # Print progress
            if (i + 1) % 100 == 0:
                print(f"Submitted {i + 1}/{total_requests} requests")
        
        # Collect results
        for future in concurrent.futures.as_completed(futures):
            result = future.result()
            results.append(result)
    
    # End time
    end_time = time.time()
    
    # Calculate actual duration
    actual_duration = end_time - start_time
    
    # Calculate actual requests per second
    actual_rps = len(results) / actual_duration
    
    # Calculate success rate
    success_count = sum(1 for r in results if r["success"])
    success_rate = success_count / len(results) * 100
    
    # Calculate response time statistics
    durations = [r["duration"] for r in results]
    avg_duration = statistics.mean(durations)
    min_duration = min(durations)
    max_duration = max(durations)
    p50_duration = statistics.median(durations)
    p90_duration = np.percentile(durations, 90)
    p95_duration = np.percentile(durations, 95)
    p99_duration = np.percentile(durations, 99)
    
    # Print results
    print("\nLoad Test Results:")
    print(f"Total Requests: {len(results)}")
    print(f"Successful Requests: {success_count} ({success_rate:.2f}%)")
    print(f"Failed Requests: {len(results) - success_count} ({100 - success_rate:.2f}%)")
    print(f"Actual Duration: {actual_duration:.2f}s")
    print(f"Actual Requests Per Second: {actual_rps:.2f}")
    print("\nResponse Time Statistics:")
    print(f"Average: {avg_duration * 1000:.2f}ms")
    print(f"Min: {min_duration * 1000:.2f}ms")
    print(f"Max: {max_duration * 1000:.2f}ms")
    print(f"P50: {p50_duration * 1000:.2f}ms")
    print(f"P90: {p90_duration * 1000:.2f}ms")
    print(f"P95: {p95_duration * 1000:.2f}ms")
    print(f"P99: {p99_duration * 1000:.2f}ms")
    
    # Create a histogram of response times
    plt.figure(figsize=(10, 6))
    plt.hist(durations, bins=50, alpha=0.75)
    plt.xlabel("Response Time (s)")
    plt.ylabel("Count")
    plt.title("Response Time Distribution")
    plt.grid(True)
    plt.savefig("response_time_histogram.png")
    print("\nResponse time histogram saved to response_time_histogram.png")
    
    # Create a line chart of response times over time
    plt.figure(figsize=(10, 6))
    plt.plot(durations)
    plt.xlabel("Request Number")
    plt.ylabel("Response Time (s)")
    plt.title("Response Time Over Time")
    plt.grid(True)
    plt.savefig("response_time_line.png")
    print("Response time line chart saved to response_time_line.png")
    
    return {
        "total_requests": len(results),
        "successful_requests": success_count,
        "success_rate": success_rate,
        "actual_duration": actual_duration,
        "actual_rps": actual_rps,
        "response_time": {
            "avg": avg_duration,
            "min": min_duration,
            "max": max_duration,
            "p50": p50_duration,
            "p90": p90_duration,
            "p95": p95_duration,
            "p99": p99_duration
        }
    }

def main():
    """Main function."""
    parser = argparse.ArgumentParser(description="Load test for Investment Performance Calculator API")
    parser.add_argument("--api-url", required=True, help="API URL")
    parser.add_argument("--token", required=True, help="Authentication token")
    parser.add_argument("--endpoint", required=True, help="API endpoint to test")
    parser.add_argument("--method", default="GET", choices=["GET", "POST"], help="HTTP method")
    parser.add_argument("--data-file", help="JSON file containing request data (for POST)")
    parser.add_argument("--rps", type=int, default=10, help="Requests per second")
    parser.add_argument("--duration", type=int, default=60, help="Test duration in seconds")
    parser.add_argument("--concurrency", type=int, default=10, help="Concurrency level")
    parser.add_argument("--output", help="Output file for results (JSON)")
    
    args = parser.parse_args()
    
    # Load data from file if specified
    data = None
    if args.data_file and args.method == "POST":
        with open(args.data_file, 'r') as f:
            data = json.load(f)
    
    # Run load test
    results = run_load_test(
        args.api_url,
        args.token,
        args.endpoint,
        args.method,
        data,
        args.rps,
        args.duration,
        args.concurrency
    )
    
    # Save results to file if specified
    if args.output:
        with open(args.output, 'w') as f:
            json.dump(results, f, indent=2)
        print(f"\nResults saved to {args.output}")

if __name__ == "__main__":
    main() 