#!/usr/bin/env python3
import os
import psycopg2
import sys
from datetime import datetime, timedelta

# Configuration
PG_HOST = os.getenv("PG_HOST", "localhost")
PG_PORT = os.getenv("PG_PORT", "5432")
PG_DB = os.getenv("PG_DB", "hermesflow")
PG_USER = os.getenv("PG_USER", "postgres")
PG_PASS = os.getenv("PG_PASS", "")

def connect():
    try:
        conn = psycopg2.connect(
            host=PG_HOST,
            port=PG_PORT,
            database=PG_DB,
            user=PG_USER,
            password=PG_PASS
        )
        return conn
    except Exception as e:
        print(f"Error connecting to database: {e}")
        sys.exit(1)

def print_header(title):
    print(f"\n{'='*60}")
    print(f"  {title}")
    print(f"{'='*60}")

def check_completeness(cur):
    print_header("1. Historical Completeness Check (Target: ~30 Days due to API Limit)")
    # 1. Get List of all symbols present
    cur.execute("SELECT DISTINCT symbol FROM mkt_equity_candles")
    symbols = [row[0] for row in cur.fetchall()]
    
    print(f"Found {len(symbols)} symbols in database.")
    print(f"{'Symbol':<15} {'Res':<5} {'Oldest Date':<20} {'Count':<10} {'Exp(30d)':<10} {'Status'}")
    print("-" * 80)
    
    # Check each symbol for 1m, 15m, 1h, 4h, 1d, 1w
    # Adjusted for 30-day BirdEye Limit
    required_resolutions = {
        '1m': 40000,    # 30 * 1440 = 43200 (approx)
        '15m': 2800,    # 30 * 96 = 2880
        '1h': 700,      # 30 * 24 = 720
        '4h': 150,      # 30 * 6 = 180
        '1d': 28,       # 30
        '1w': 4         # 4 weeks
    }
    
    issues_found = 0
    
    for sym in symbols:
        for res, min_count in required_resolutions.items():
            # Check oldest record and count in last year
            query = """
            SELECT MIN(time), COUNT(*) 
            FROM mkt_equity_candles 
            WHERE symbol = %s AND resolution = %s AND time > NOW() - INTERVAL '1 year'
            """
            cur.execute(query, (sym, res))
            row = cur.fetchone()
            oldest, count = row if row else (None, 0)
            
            if count is None: count = 0
            
            # Formatting
            oldest_str = oldest.strftime('%Y-%m-%d') if oldest else "N/A"
            status = "✅"
            if count < min_count:
                status = "❌"
                issues_found += 1
                # Only print failures to keep log clean if many symbols
                print(f"{sym:<15} {res:<5} {oldest_str:<20} {count:<10} {min_count:<10} {status}")
            else:
                 # Optional: print success for debug, or just summarize
                 pass

    if issues_found == 0:
        print("\n✅ All symbols have sufficient 1-year data for all resolutions.")
    else:
        print(f"\n❌ Found {issues_found} missing data streams (Symbol/Resolution pairs).")

def check_consistency(cur):
    print_header("2. Aggregation Consistency Check (Sample 1 Day)")
    # Check if 1h High == Max(1m High) within that hour
    query = """
    WITH hourly_agg AS (
        SELECT 
            time_bucket('1 hour', time) as hour_bucket,
            symbol,
            MAX(high) as calc_high
        FROM mkt_equity_candles
        WHERE resolution = '1m' AND time > NOW() - INTERVAL '1 day'
        GROUP BY 1, 2
    )
    SELECT 
        h.time as bucket,
        h.symbol,
        h.high as stored_high,
        a.calc_high,
        (h.high - a.calc_high) as diff
    FROM mkt_equity_candles h
    JOIN hourly_agg a ON h.time = a.hour_bucket AND h.symbol = a.symbol
    WHERE h.resolution = '1h'
      AND ABS(h.high - a.calc_high) > 0.0001
    LIMIT 10;
    """
    cur.execute(query)
    rows = cur.fetchall()
    
    if not rows:
        print("✅ No aggregation inconsistencies found (Sampled 24h).")
    else:
        print("❌ Inconsistencies Found:")
        for row in rows:
            print(f"Time: {row[0]}, Symbol: {row[1]}, Stored High: {row[2]}, Calc High: {row[3]}, Diff: {row[4]}")

def check_gaps(cur):
    print_header("3. Gap Detection (Last 24h, 1m candles)")
    query = """
    SELECT
      symbol,
      time AS gap_start,
      LEAD(time) OVER (PARTITION BY symbol ORDER BY time) AS next_time,
      LEAD(time) OVER (PARTITION BY symbol ORDER BY time) - time AS duration
    FROM mkt_equity_candles
    WHERE time > NOW() - INTERVAL '24 hours'
    AND resolution = '1m'
    """
    # Note: Wrap above in CTE to filter where duration > 2 mins (to allow minor jitter)
    wrapped_query = f"""
    WITH gaps AS ({query})
    SELECT * FROM gaps WHERE duration > INTERVAL '2 minutes' LIMIT 20;
    """
    cur.execute(wrapped_query)
    rows = cur.fetchall()
    
    if not rows:
        print("✅ No significant gaps found (> 2 mins).")
    else:
        print("❌ Gaps Found:")
        for row in rows:
            print(f"Symbol: {row[0]}, Start: {row[1]}, Next: {row[2]}, Duration: {row[3]}")

def check_anomalies(cur):
    print_header("4. Data Anomalies (Dirty Data)")
    query = """
    SELECT count(*) FROM mkt_equity_candles 
    WHERE high < low OR volume < 0 OR close <= 0;
    """
    cur.execute(query)
    count = cur.fetchone()[0]
    
    if count == 0:
        print("✅ No dirty data found (High < Low, Negative Volume).")
    else:
        print(f"❌ Found {count} rows with invalid data!")
        # Print sample
        query_sample = """
        SELECT time, symbol, high, low, volume 
        FROM mkt_equity_candles 
        WHERE high < low OR volume < 0 OR close <= 0
        LIMIT 5;
        """
        cur.execute(query_sample)
        rows = cur.fetchall()
        print("Sample Invalid Rows:")
        for r in rows:
            print(f"  Time: {r[0]}, Symbol: {r[1]}, High: {r[2]}, Low: {r[3]}, Vol: {r[4]}")

def check_freshness(cur):
    print_header("5. Real-Time Freshness (Liveness)")
    
    # 1. Snapshots (Raw Data)
    cur.execute("SELECT MAX(timestamp) FROM mkt_equity_snapshots")
    last_snap = cur.fetchone()[0]
    
    # 2. Candles (Aggregated)
    cur.execute("SELECT MAX(time) FROM mkt_equity_candles WHERE resolution = '15m'")
    last_candle = cur.fetchone()[0]

    # 3. Active Tokens (Discovery)
    cur.execute("SELECT MAX(last_updated) FROM active_tokens")
    last_token_update = cur.fetchone()[0]

    now = datetime.now(last_snap.tzinfo if last_snap else None)
    
    print(f"{'Metric':<20} {'Latest Timestamp':<30} {'Lag':<15} {'Status'}")
    print("-" * 75)
    
    # Check Snapshots
    if last_snap:
        lag = now - last_snap
        status = "✅ Live" if lag < timedelta(minutes=5) else "⚠️  Stale"
        print(f"{'Snapshots':<20} {str(last_snap):<30} {str(lag):<15} {status}")
    else:
        print(f"{'Snapshots':<20} {'None':<30} {'N/A':<15} ❌ Empty")

    # Check Candles
    if last_candle:
        # 15m candle might be up to 15m old + delay
        lag = now - last_candle
        status = "✅ Live" if lag < timedelta(minutes=20) else "⚠️  Stale"
        print(f"{'Candles (15m)':<20} {str(last_candle):<30} {str(lag):<15} {status}")
    else:
        print(f"{'Candles (15m)':<20} {'None':<30} {'N/A':<15} ❌ Empty")
        
    # Check Discovery
    if last_token_update:
        lag = now - last_token_update
        # Runs every hour
        status = "✅ Live" if lag < timedelta(hours=1, minutes=10) else "⚠️  Stale"
        print(f"{'Token Discovery':<20} {str(last_token_update):<30} {str(lag):<15} {status}")
    else:
        print(f"{'Token Discovery':<20} {'None':<30} {'N/A':<15} ❌ Empty")

def main():
    print("Starting Data Quality Audit...")
    conn = connect()
    cur = conn.cursor()
    
    try:
        check_completeness(cur)
        check_consistency(cur)
        check_gaps(cur)
        check_anomalies(cur)
        check_freshness(cur)
    finally:
        cur.close()
        conn.close()
        print("\nAudit Complete.")

if __name__ == "__main__":
    main()
