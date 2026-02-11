#!/usr/bin/env python3
import os
import time
import json
import urllib.request
import urllib.error
from datetime import datetime

# Configuration
BIRDEYE_API_KEY = os.getenv("DATA_ENGINE__BIRDEYE__API_KEY", "")
JUPITER_API_KEY = os.getenv("DATA_ENGINE__JUPITER__API_KEY", "")
J_URL = "https://api.jup.ag/price/v2" # Fallback to v2, or use v3 with key
SOL_MINT = "So11111111111111111111111111111111111111112"

def print_header(title):
    print(f"\n{'='*60}")
    print(f"  {title}")
    print(f"{'='*60}")

def make_request(url, headers=None):
    if headers is None:
        headers = {}
    
    # Add User-Agent to avoid 403 blocks from some APIs
    headers['User-Agent'] = 'HermesFlow-Audit/1.0'
    
    req = urllib.request.Request(url, headers=headers)
    start = time.time()
    try:
        with urllib.request.urlopen(req) as response:
            latency = (time.time() - start) * 1000
            data = response.read()
            text = data.decode('utf-8')
            return {
                'status': response.status,
                'latency': latency,
                'json': json.loads(text),
                'text': text
            }
    except urllib.error.HTTPError as e:
        latency = (time.time() - start) * 1000
        return {
            'status': e.code,
            'latency': latency,
            'error': str(e),
            'text': e.read().decode('utf-8')
        }
    except Exception as e:
        latency = (time.time() - start) * 1000
        return {
            'status': 0,
            'latency': latency,
            'error': str(e)
        }

def audit_birdeye():
    print_header("1. BirdEye API Audit (Historical/Meta)")
    
    if not BIRDEYE_API_KEY:
        print("⚠️  Skipping BirdEye audit: DATA_ENGINE__BIRDEYE__API_KEY not set.")
        return

    headers = {
        "X-API-KEY": BIRDEYE_API_KEY,
        "accept": "application/json"
    }
    
    # 1.1 Token Overview
    print("\n[Check 1.1] Token Overview (SOL)")
    url = f"https://public-api.birdeye.so/defi/token_overview?address={SOL_MINT}"
    resp = make_request(url, headers)
    
    print(f"Status: {resp['status']} | Latency: {resp['latency']:.1f}ms")
    
    if resp['status'] == 200:
        data = resp['json']
        if data.get('success'):
            token_data = data.get('data', {})
            print(f"✅ Success. Symbol: {token_data.get('symbol')}, Price: {token_data.get('price')}")
        else:
            print(f"❌ API Error: {data.get('message')}")
    else:
        print(f"❌ HTTP Error: {resp.get('error')}")

    # 1.2 Historical Data
    print("\n[Check 1.2] Historical Data (last 24h)")
    now = int(time.time())
    start_ts = now - 86400
    url = f"https://public-api.birdeye.so/defi/ohlcv?address={SOL_MINT}&type=1H&time_from={start_ts}&time_to={now}"
    resp = make_request(url, headers)
    
    if resp['status'] == 200:
        data = resp['json']
        items = data.get('data', {}).get('items', [])
        print(f"✅ Retrieved {len(items)} hourly candles.")
    else:
        print(f"❌ Failed to get history: {resp['status']}")

def audit_helius():
    print_header("3.1 Helius RPC Audit (Solana Liveness)")
    # Public RPC for audit if key not available, or use key if present
    # Using public for generic reachability check
    url = "https://api.mainnet-beta.solana.com"
    payload = json.dumps({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getHealth"
    }).encode('utf-8')
    
    headers = {'Content-Type': 'application/json'}
    
    start = time.time()
    try:
        req = urllib.request.Request(url, data=payload, headers=headers)
        with urllib.request.urlopen(req) as response:
            latency = (time.time() - start) * 1000
            data = json.loads(response.read().decode('utf-8'))
            print(f"Status: {response.status} | Latency: {latency:.1f}ms")
            if data.get('result') == 'ok':
                 print("✅ Solana Network (RPC) Reachable.")
            else:
                 print(f"⚠️  RPC Health Response: {data}")
    except Exception as e:
        print(f"❌ Helius/RPC Error: {e}")

def audit_okx():
    print_header("3.2 OKX API Audit (CEX Status)")
    url = "https://www.okx.com/api/v5/public/status"
    resp = make_request(url)
    print(f"Status: {resp['status']} | Latency: {resp['latency']:.1f}ms")
    if resp['status'] == 200:
        data = resp['json']
        if data.get('code') == '0':
            print("✅ OKX API Reachable.")
        else:
            print(f"⚠️  OKX Reported Issue: {data}")
    else:
        print(f"❌ OKX Unreachable: {resp.get('error')}")

def audit_polygon():
    print_header("3.3 Massive/Polygon Audit (Market Status)")
    url = "https://api.polygon.io/v1/marketstatus/now"
    # Note: Polygon requires key even for status often, but let's try or handle 401 as 'Reachable but Auth needed'
    resp = make_request(url)
    print(f"Status: {resp['status']} | Latency: {resp['latency']:.1f}ms")
    if resp['status'] == 200:
        print("✅ Polygon API Reachable.")
    elif resp['status'] == 401:
        print("✅ Polygon API Reachable (Auth Required).")
    else:
        print(f"❌ Polygon Unreachable: {resp.get('error')}")

def audit_others():
    print_header("3. Multi-Source Reachability")
    audit_helius()
    audit_okx()
    audit_polygon()

def audit_jupiter():
    print_header("2. Jupiter Price API Audit (Real-time)")
    
    # Poll 5 times to check stability
    print(f"Polling {J_URL} for SOL price (5 iterations)...")
    
    headers = {}
    if JUPITER_API_KEY:
        headers["x-api-key"] = JUPITER_API_KEY
    else:
        print("⚠️  Warning: DATA_ENGINE__JUPITER__API_KEY is not set. Requests may fail (401).")

    success_cnt = 0
    latencies = []
    
    for i in range(5):
        # GET /price?ids=...
        url = f"{J_URL}?ids={SOL_MINT}"
        resp = make_request(url, headers)
        
        if resp['status'] == 200:
            data = resp['json']
            # data format: {"data": {"So111...": { "id": "...", "type": "derivedPrice", "price": "..." }}}
            price_data = data.get('data', {}).get(SOL_MINT, {})
            price = price_data.get('price')
            
            if price:
                print(f"Iter {i+1}: ✅ Price={price} | Latency={resp['latency']:.1f}ms")
                success_cnt += 1
                latencies.append(resp['latency'])
            else:
                print(f"Iter {i+1}: ⚠️  Response OK but empty price.")
        else:
            print(f"Iter {i+1}: ❌ HTTP {resp['status']}")
            
        time.sleep(1)
            
    print(f"\nSummary: Success {success_cnt}/5")
    if latencies:
        avg_lat = sum(latencies)/len(latencies)
        print(f"Avg Latency: {avg_lat:.1f}ms")

def main():
    print("Starting Data Source Audit (Standard Lib)...")
    audit_birdeye()
    audit_jupiter()
    print("\nAudit Complete.")

if __name__ == "__main__":
    main()
