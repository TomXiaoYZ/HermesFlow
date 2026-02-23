#!/usr/bin/env python3
"""
Test Polygon.io Starter plan access
Quick validation of API capabilities
"""

import os
import sys
import requests
import json
from datetime import datetime, timedelta

API_KEY = os.environ.get("POLYGON_API_KEY")
if not API_KEY:
    print("Error: POLYGON_API_KEY environment variable is not set")
    sys.exit(1)
BASE_URL = "https://api.polygon.io"

def test_quick():
    """Quick comprehensive test"""
    
    print("🧪 Polygon.io Starter Plan Validation")
    print("=" * 60)
    
    tests = [
        {
            "name": "Daily Aggregates (5 days)",
            "url": f"{BASE_URL}/v2/aggs/ticker/AAPL/range/1/day/2024-01-15/2024-01-19",
            "expected_count": 5
        },
        {
            "name": "Hourly Aggregates (1 day)",
            "url": f"{BASE_URL}/v2/aggs/ticker/TSLA/range/1/hour/2024-01-15/2024-01-15",
            "expected_count": 7  # Trading hours: 9:30-16:00 = 6.5 hours
        },
        {
            "name": "15-min Aggregates (1 day)",
            "url": f"{BASE_URL}/v2/aggs/ticker/GOOGL/range/15/minute/2024-01-15/2024-01-15",
            "expected_count": 26  # 6.5 hours / 0.25 = 26 bars
        }
    ]
    
    for i, test in enumerate(tests, 1):
        print(f"\n{i}. {test['name']}")
        print("-" * 60)
        
        try:
            response = requests.get(test['url'], params={"apiKey": API_KEY}, timeout=10)
            
            if response.status_code == 200:
                data = response.json()
                
                if data.get('status') == 'OK' and data.get('results'):
                    count = len(data['results'])
                    print(f"   ✅ Success! Fetched {count} bars")
                    
                    # Show first bar
                    first = data['results'][0]
                    date = datetime.fromtimestamp(first['t']/1000)
                    print(f"   📊 First bar: {date.strftime('%Y-%m-%d %H:%M')}")
                    print(f"      O: ${first['o']:.2f}, H: ${first['h']:.2f}, L: ${first['l']:.2f}, C: ${first['c']:.2f}")
                    print(f"      Volume: {first['v']:,}")
                else:
                    print(f"   ⚠️  Status: {data.get('status')}, Message: {data.get('message', 'No results')}")
            elif response.status_code == 403 or response.status_code == 401:
                data = response.json()
                print(f"   ❌ NOT AUTHORIZED")
                print(f"      {data.get('message', 'Permission denied')}")
            else:
                print(f"   ❌ HTTP {response.status_code}")
                
        except Exception as e:
            print(f"   ❌ Error: {e}")
    
    print("\n" + "=" * 60)
    print("✅ Test completed!\n")

if __name__ == "__main__":
    test_quick()
