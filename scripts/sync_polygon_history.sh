#!/bin/bash
# Sync Polygon.io historical data for US stocks
# Usage: ./sync_polygon_history.sh

set -e

echo "🚀 Polygon.io Historical Data Sync"
echo "=================================="
echo ""

# Configuration
API_KEY="${POLYGON_API_KEY:-0_EQM4CgpMzXgdFs7y9rM7FEUyGZrDPH}"
TICKERS=(AAPL MSFT GOOGL TSLA NVDA AMZN META NFLX)
RESOLUTIONS=(1d 1h)

# Date range (last 30 days for testing)
END_DATE=$(date +%Y-%m-%d)
START_DATE=$(date -v-30d +%Y-%m-%d 2>/dev/null || date -d '30 days ago' +%Y-%m-%d)

echo "API Key: ${API_KEY:0:10}..."
echo "Date Range: $START_DATE to $END_DATE"
echo "Tickers: ${TICKERS[@]}"
echo "Resolutions: ${RESOLUTIONS[@]}"
echo ""

# Check if requests module is available
python3 -c "import requests" 2>/dev/null || {
    echo "❌ Python 'requests' module not found"
    echo "Installing..."
    pip3 install requests
}

# Create sync script
cat > /tmp/polygon_sync.py << 'EOF'
#!/usr/bin/env python3
import sys
import requests
import time
import json
from datetime import datetime

def fetch_aggregates(api_key, ticker, resolution, from_date, to_date):
    """Fetch aggregates from Polygon API"""
    
    # Map resolution
    if resolution == "1d":
        multiplier, timespan = 1, "day"
    elif resolution == "1h":
        multiplier, timespan = 1, "hour"
    elif resolution == "15m":
        multiplier, timespan = 15, "minute"
    elif resolution == "1m":
        multiplier, timespan = 1, "minute"
    else:
        multiplier, timespan = 1, "day"
    
    url = f"https://api.polygon.io/v2/aggs/ticker/{ticker}/range/{multiplier}/{timespan}/{from_date}/{to_date}"
    params = {"apiKey": api_key}
    
    try:
        response = requests.get(url, params=params, timeout=30)
        
        if response.status_code == 429:
            print(f"   ⚠️  Rate limit exceeded, waiting 60s...")
            time.sleep(60)
            response = requests.get(url, params=params, timeout=30)
        
        if response.status_code == 200:
            data = response.json()
            if data.get('results'):
                return data['results']
            else:
                return []
        else:
            print(f"   ❌ HTTP {response.status_code}: {response.text[:100]}")
            return None
            
    except Exception as e:
        print(f"   ❌ Error: {e}")
        return None

if __name__ == "__main__":
    api_key = sys.argv[1]
    ticker = sys.argv[2]
    resolution = sys.argv[3]
    from_date = sys.argv[4]
    to_date = sys.argv[5]
    
    results = fetch_aggregates(api_key, ticker, resolution, from_date, to_date)
    
    if results is not None:
        print(json.dumps(results))
        sys.exit(0)
    else:
        sys.exit(1)
EOF

chmod +x /tmp/polygon_sync.py

# Sync loop
total_calls=0
for ticker in "${TICKERS[@]}"; do
    for resolution in "${RESOLUTIONS[@]}"; do
        echo "📊 Fetching $ticker ($resolution)..."
        
        # Call Python script
        output=$(/tmp/polygon_sync.py "$API_KEY" "$ticker" "$resolution" "$START_DATE" "$END_DATE" 2>&1)
        
        if [ $? -eq 0 ]; then
            count=$(echo "$output" | python3 -c "import sys, json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo "0")
            echo "   ✅ Fetched $count candles"
            
            # TODO: Insert into database
            # echo "$output" | psql $DATABASE_URL -c "INSERT INTO ..."
        else
            echo "   ❌ Failed"
            echo "$output"
        fi
        
        total_calls=$((total_calls + 1))
        
        # Rate limiting: 5 calls/min, so wait 12 seconds between calls
        if [ $((total_calls % 5)) -eq 0 ]; then
            echo "   ⏳ Rate limit: waiting 60s after 5 calls..."
            sleep 60
        else
            echo "   ⏳ Waiting 3s..."
            sleep 3
        fi
    done
done

echo ""
echo "=================================="
echo "✅ Sync completed!"
echo "Total API calls: $total_calls"
echo "=================================="
