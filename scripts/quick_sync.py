#!/usr/bin/env python3
"""
Quick sync script to import historical data from Polygon.io
"""

import os
import sys
import time
import psycopg2
from psycopg2.extras import execute_values
import requests
from datetime import datetime, timedelta
import argparse

# Polygon API configuration
API_KEY = os.getenv("POLYGON_API_KEY", "0_EQM4CgpMzXgdFs7y9rM7FEUyGZrDPH")
BASE_URL = "https://api.polygon.io"

# Configure requests session with retry logic
from requests.adapters import HTTPAdapter
from requests.packages.urllib3.util.retry import Retry

retry_strategy = Retry(
    total=5,
    backoff_factor=1,
    status_forcelist=[429, 500, 502, 503, 504],
    allowed_methods=["HEAD", "GET", "OPTIONS"]
)
adapter = HTTPAdapter(max_retries=retry_strategy)
http = requests.Session()
http.mount("https://", adapter)
http.mount("http://", adapter)

# Resolution mapping
RESOLUTION_MAP = {
    "1m": (1, "minute"),
    "15m": (15, "minute"),
    "1h": (1, "hour"),
    "4h": (4, "hour"),
    "1d": (1, "day"),
    "1w": (1, "week"),
}

def fetch_polygon_data(ticker, resolution, from_date, to_date):
    """Fetch data from Polygon API with chunking for large datasets"""
    multiplier, timespan = RESOLUTION_MAP[resolution]
    
    # Determine chunk size based on resolution to avoid hitting 50k limit
    chunk_sizes = {
        "1m": 30,    # 30 days = ~12k candles per trading day
        "15m": 90,   # 90 days
        "1h": 180,   # 180 days  
        "4h": 365,   # 1 year
        "1d": 1095,  # All 3 years
        "1w": 1095,  # All 3 years
    }
    
    chunk_days = chunk_sizes.get(resolution, 365)
    
    # Parse dates
    from_dt = datetime.strptime(from_date, "%Y-%m-%d")
    to_dt = datetime.strptime(to_date, "%Y-%m-%d")
    
    all_candles = []
    current_start = from_dt
    
    while current_start < to_dt:
        current_end = min(current_start + timedelta(days=chunk_days), to_dt)
        
        url = f"{BASE_URL}/v2/aggs/ticker/{ticker}/range/{multiplier}/{timespan}/{current_start.strftime('%Y-%m-%d')}/{current_end.strftime('%Y-%m-%d')}"
        params = {"apiKey": API_KEY, "limit": 50000}
        
        print(f"  Fetching chunk: {current_start.strftime('%Y-%m-%d')} to {current_end.strftime('%Y-%m-%d')}")
        try:
            response = http.get(url, params=params, timeout=30)
        except Exception as e:
            print(f"  ❌ Request failed: {e}")
            break
        
        if response.status_code != 200:
            print(f"  ❌ Error {response.status_code}: {response.text}")
            break
        
        data = response.json()
        if data.get("status") != "OK":
            if data.get("status") == "DELAYED":
                print(f"  ⚠️  Chunk delayed, skipping...")
            else:
                print(f"  ⚠️  No data: {data.get('status')}")
            current_start = current_end + timedelta(days=1)
            continue
        
        results = data.get("results", [])
        all_candles.extend(results)
        print(f"  ✅ Got {len(results)} candles (total: {len(all_candles)})")
        
        # Move to next chunk
        current_start = current_end + timedelta(days=1)
        
        # Small delay between chunks
        time.sleep(0.1)
    
    print(f"  📊 Total fetched: {len(all_candles)} candles")
    return all_candles

def insert_candles(cursor, ticker, resolution, candles):
    """Insert candles into database"""
    if not candles:
        return 0
    
    # Prepare data
    data = []
    for candle in candles:
        timestamp = datetime.fromtimestamp(candle['t'] / 1000)  # Convert ms to seconds
        data.append((
            timestamp,
            'Polygon',
            ticker,
            resolution,
            float(candle['o']),
            float(candle['h']),
            float(candle['l']),
            float(candle['c']),
            float(candle['v']),
            None,  # metadata
        ))
    
    # Bulk insert with conflict handling
    execute_values(
        cursor,
        """
        INSERT INTO mkt_equity_candles 
        (time, exchange, symbol, resolution, open, high, low, close, volume, metadata)
        VALUES %s
        ON CONFLICT (time, exchange, symbol, resolution) DO NOTHING
        """,
        data
    )
    
    return len(data)

def sync_task(conn, exchange, symbol, resolution, from_date, to_date):
    """Sync a single task"""
    cursor = conn.cursor()
    
    print(f"\n[{exchange}/{symbol}/{resolution}]")
    
    # Mark as syncing
    cursor.execute(
        "UPDATE market_sync_status SET status = 'syncing' WHERE exchange = %s AND symbol = %s AND resolution = %s",
        (exchange, symbol, resolution)
    )
    conn.commit()
    
    try:
        # Fetch data
        candles = fetch_polygon_data(symbol, resolution, from_date, to_date)
        
        # Insert into DB
        inserted = insert_candles(cursor, symbol, resolution, candles)
        print(f"  ✅ Inserted {inserted} candles")
        
        # Update status
        cursor.execute(
            """
            UPDATE market_sync_status 
            SET status = 'completed',
                total_candles = %s,
                last_sync_at = NOW(),
                last_synced_time = (
                    SELECT MAX(time) FROM mkt_equity_candles 
                    WHERE exchange = %s AND symbol = %s AND resolution = %s
                )
            WHERE exchange = %s AND symbol = %s AND resolution = %s
            """,
            (inserted, exchange, symbol, resolution, exchange, symbol, resolution)
        )
        conn.commit()
        
        return True
        
    except Exception as e:
        print(f"  ❌ Error: {e}")
        cursor.execute(
            "UPDATE market_sync_status SET status = 'failed', error_message = %s WHERE exchange = %s AND symbol = %s AND resolution = %s",
            (str(e), exchange, symbol, resolution)
        )
        conn.commit()
        return False

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--limit", type=int, default=470, help="Number of tasks to process")
    parser.add_argument("--exchange", default="Polygon", help="Exchange filter")
    args = parser.parse_args()
    
    # Connect to database
    db_url = os.getenv("DATABASE_URL", "postgresql://postgres:password@localhost:5432/hermesflow")
    print(f"Connecting to database...")
    conn = psycopg2.connect(db_url)
    cursor = conn.cursor()
    
    # Get pending tasks
    cursor.execute(
        """
        SELECT 
            s.exchange,
            s.symbol,
            s.resolution,
            COALESCE(w.sync_from_date, '2023-01-01'::DATE),
            COALESCE(w.priority, 50)
        FROM market_sync_status s
        INNER JOIN market_watchlist w ON s.exchange = w.exchange AND s.symbol = w.symbol
        WHERE s.status = 'pending'
          AND s.exchange = %s
          AND w.is_active = true
        ORDER BY w.priority DESC, s.exchange, s.symbol, s.resolution
        LIMIT %s
        """,
        (args.exchange, args.limit)
    )
    
    tasks = cursor.fetchall()
    print(f"\n{'='*70}")
    print(f"Found {len(tasks)} pending tasks")
    print(f"{'='*70}")
    
    if not tasks:
        print("✅ No pending tasks!")
        return
    
    # Process each task
    success_count = 0
    for i, (exchange, symbol, resolution, from_date, priority) in enumerate(tasks, 1):
        print(f"\n[{i}/{len(tasks)}] Priority: {priority}")
        
        from_str = from_date.strftime("%Y-%m-%d")
        # Use fixed historical date to avoid system time issues
        to_str = "2025-12-31"
        
        if sync_task(conn, exchange, symbol, resolution, from_str, to_str):
            success_count += 1
        
        # Rate limiting (300ms between requests)
        time.sleep(0.3)
    
    # Show summary
    print(f"\n{'='*70}")
    print(f"✅ Sync completed: {success_count}/{len(tasks)} tasks successful")
    print(f"{'='*70}")
    
    # Show status
    cursor.execute(
        """
        SELECT status, COUNT(*) 
        FROM market_sync_status 
        WHERE exchange = %s
        GROUP BY status
        """,
        (args.exchange,)
    )
    
    print("\nSync Status:")
    for status, count in cursor.fetchall():
        print(f"  - {status}: {count}")
    
    conn.close()

if __name__ == "__main__":
    main()
