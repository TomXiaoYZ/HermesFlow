#!/usr/bin/env python3
"""
Quick test script for Polygon.io API
Tests Basic (free) plan capabilities
"""

import requests
import json
from datetime import datetime, timedelta

API_KEY = "0_EQM4CgpMzXgdFs7y9rM7FEUyGZrDPH"
BASE_URL = "https://api.polygon.io"

def test_daily_aggregates():
    """Test fetching daily OHLCV data"""
    print("=" * 60)
    print("Test 1: Daily Aggregates (AAPL)")
    print("=" * 60)
    
    # Get yesterday's data
    yesterday = (datetime.now() - timedelta(days=1)).strftime("%Y-%m-%d")
    
    url = f"{BASE_URL}/v2/aggs/ticker/AAPL/range/1/day/{yesterday}/{yesterday}"
    params = {"apiKey": API_KEY}
    
    print(f"Request: GET {url}")
    
    response = requests.get(url, params=params)
    print(f"Status: {response.status_code}")
    
    if response.status_code == 200:
        data = response.json()
        print(f"\nResponse:")
        print(json.dumps(data, indent=2))
        
        if data.get('results'):
            bar = data['results'][0]
            print(f"\n✅ Success!")
            print(f"   Date: {datetime.fromtimestamp(bar['t']/1000).strftime('%Y-%m-%d')}")
            print(f"   Open: ${bar['o']:.2f}")
            print(f"   High: ${bar['h']:.2f}")
            print(f"   Low: ${bar['l']:.2f}")
            print(f"   Close: ${bar['c']:.2f}")
            print(f"   Volume: {bar['v']:,}")
        else:
            print(f"\n⚠️  No data returned (might be weekend/holiday)")
    else:
        print(f"❌ Error: {response.text}")
    
    print()

def test_ticker_details():
    """Test fetching ticker details"""
    print("=" * 60)
    print("Test 2: Ticker Details (AAPL)")
    print("=" * 60)
    
    url = f"{BASE_URL}/v3/reference/tickers/AAPL"
    params = {"apiKey": API_KEY}
    
    print(f"Request: GET {url}")
    
    response = requests.get(url, params=params)
    print(f"Status: {response.status_code}")
    
    if response.status_code == 200:
        data = response.json()
        
        if data.get('results'):
            ticker = data['results']
            print(f"\n✅ Success!")
            print(f"   Ticker: {ticker.get('ticker')}")
            print(f"   Name: {ticker.get('name')}")
            print(f"   Market: {ticker.get('market')}")
            print(f"   Locale: {ticker.get('locale')}")
            print(f"   Type: {ticker.get('type')}")
            print(f"   Active: {ticker.get('active')}")
    else:
        print(f"❌ Error: {response.text}")
    
    print()

def test_historical_range():
    """Test fetching a week of daily data"""
    print("=" * 60)
    print("Test 3: Historical Range (AAPL - 1 week)")
    print("=" * 60)
    
    end_date = (datetime.now() - timedelta(days=1)).strftime("%Y-%m-%d")
    start_date = (datetime.now() - timedelta(days=7)).strftime("%Y-%m-%d")
    
    url = f"{BASE_URL}/v2/aggs/ticker/AAPL/range/1/day/{start_date}/{end_date}"
    params = {"apiKey": API_KEY}
    
    print(f"Request: GET {url}")
    print(f"Range: {start_date} to {end_date}")
    
    response = requests.get(url, params=params)
    print(f"Status: {response.status_code}")
    
    if response.status_code == 200:
        data = response.json()
        
        if data.get('results'):
            results = data['results']
            print(f"\n✅ Success! Fetched {len(results)} bars")
            print(f"\nFirst 3 bars:")
            for bar in results[:3]:
                date = datetime.fromtimestamp(bar['t']/1000).strftime('%Y-%m-%d')
                print(f"   {date}: O=${bar['o']:.2f} H=${bar['h']:.2f} L=${bar['l']:.2f} C=${bar['c']:.2f} V={bar['v']:,}")
        else:
            print(f"\n⚠️  No data returned")
    else:
        print(f"❌ Error: {response.text}")
    
    print()

def test_rate_limit():
    """Test multiple tickers to see rate limiting"""
    print("=" * 60)
    print("Test 4: Rate Limit Test (5 tickers)")
    print("=" * 60)
    
    tickers = ["AAPL", "MSFT", "GOOGL", "TSLA", "NVDA"]
    yesterday = (datetime.now() - timedelta(days=1)).strftime("%Y-%m-%d")
    
    print(f"Basic plan limit: 5 API calls/minute")
    print(f"Fetching {len(tickers)} tickers...\n")
    
    for i, ticker in enumerate(tickers, 1):
        url = f"{BASE_URL}/v2/aggs/ticker/{ticker}/range/1/day/{yesterday}/{yesterday}"
        params = {"apiKey": API_KEY}
        
        print(f"{i}. {ticker}...", end=" ")
        response = requests.get(url, params=params)
        
        if response.status_code == 200:
            data = response.json()
            if data.get('results'):
                print(f"✅ Close: ${data['results'][0]['c']:.2f}")
            else:
                print("⚠️  No data")
        elif response.status_code == 429:
            print("❌ Rate limit exceeded!")
            break
        else:
            print(f"❌ Error {response.status_code}")
    
    print()

if __name__ == "__main__":
    print("\n🧪 Polygon.io API Test Suite")
    print("=" * 60)
    print(f"API Key: {API_KEY[:10]}...")
    print(f"Plan: Basic (Free)")
    print(f"Limit: 5 API calls/minute\n")
    
    try:
        test_daily_aggregates()
        test_ticker_details()
        test_historical_range()
        test_rate_limit()
        
        print("=" * 60)
        print("✅ All tests completed!")
        print("=" * 60)
        
    except Exception as e:
        print(f"\n❌ Test failed with error: {e}")
