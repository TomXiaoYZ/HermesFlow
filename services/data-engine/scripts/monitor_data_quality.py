#!/usr/bin/env python3
import redis
import json
import time
import os
import signal
import sys
from datetime import datetime
from collections import defaultdict

# Configuration
REDIS_URL = os.getenv("REDIS_URL", "redis://localhost:6379")
CHANNELS = ["market_data", "portfolio_updates", "strategy_logs"]

def signal_handler(sig, frame):
    print("\n[Monitor] Stopping...")
    sys.exit(0)

signal.signal(signal.SIGINT, signal_handler)

def main():
    print(f"[Monitor] Connecting to Redis at {REDIS_URL}...")
    try:
        r = redis.from_url(REDIS_URL)
        pubsub = r.pubsub()
        for ch in CHANNELS:
            pubsub.subscribe(ch)
            print(f"[Monitor] Subscribed to channel: {ch}")
    except Exception as e:
        print(f"[Monitor] Failed to connect: {e}")
        return

    print("[Monitor] Listening for events... (Press Ctrl+C to stop)")
    
    stats = {
        "count": 0,
        "latency_sum": 0,
        "sources": defaultdict(int),
        "start_time": time.time(),
        "errors": 0
    }

    last_report_time = time.time()

    for message in pubsub.listen():
        if message['type'] != 'message':
            continue
            
        try:
            now = time.time() * 1000 # ms
            channel = message['channel'].decode('utf-8')
            payload_str = message['data'].decode('utf-8')
            
            # Try to parse JSON
            data = json.loads(payload_str)
            
            # Latency Check
            event_ts = data.get('timestamp')
            latency = 0
            if event_ts:
                # Handle potential different timestamp formats if needed, assuming ms int
                latency = now - float(event_ts)
            
            # Source Check
            source = data.get('source', 'unknown')
            
            # Update Stats
            stats["count"] += 1
            stats["latency_sum"] += max(0, latency)
            stats["sources"][source] += 1
            
            # Report every 5 seconds
            if time.time() - last_report_time > 5:
                report(stats)
                # Reset rolling stats
                stats["count"] = 0
                stats["latency_sum"] = 0
                # stats["sources"] = defaultdict(int) # Keep cumulative source counts for now
                last_report_time = time.time()
                
        except json.JSONDecodeError:
            stats["errors"] += 1
        except Exception as e:
            # print(f"Error parsing: {e}")
            stats["errors"] += 1

def report(stats):
    if stats["count"] == 0:
        print(f"[{datetime.now().strftime('%H:%M:%S')}] No events received.")
        return

    avg_latency = stats["latency_sum"] / stats["count"]
    fps = stats["count"] / 5.0
    
    print(f"[{datetime.now().strftime('%H:%M:%S')}] "
          f"Rate: {fps:.1f} msg/s | "
          f"Avg Latency: {avg_latency:.1f}ms | "
          f"Sources: {dict(stats['sources'])}")

if __name__ == "__main__":
    main()
