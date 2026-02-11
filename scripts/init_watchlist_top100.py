#!/usr/bin/env python3
"""
Initialize market_watchlist with top 100 US stocks by market cap
"""

import psycopg2
from psycopg2.extras import execute_values
import os

# Top 100 US stocks by market cap (as of 2024)
TOP_100_US_STOCKS = [
    # Tech Giants
    {"ticker": "AAPL", "name": "Apple Inc.", "market_cap": 3000000000000, "sector": "Technology"},
    {"ticker": "MSFT", "name": "Microsoft Corporation", "market_cap": 2800000000000, "sector": "Technology"},
    {"ticker": "GOOGL", "name": "Alphabet Inc. Class A", "market_cap": 1700000000000, "sector": "Technology"},
    {"ticker": "AMZN", "name": "Amazon.com Inc.", "market_cap": 1500000000000, "sector": "Consumer Cyclical"},
    {"ticker": "NVDA", "name": "NVIDIA Corporation", "market_cap": 1400000000000, "sector": "Technology"},
    {"ticker": "META", "name": "Meta Platforms Inc.", "market_cap": 900000000000, "sector": "Technology"},
    {"ticker": "TSLA", "name": "Tesla Inc.", "market_cap": 800000000000, "sector": "Consumer Cyclical"},
    
    # Finance
    {"ticker": "BRK.B", "name": "Berkshire Hathaway Inc. Class B", "market_cap": 750000000000, "sector": "Financial"},
    {"ticker": "V", "name": "Visa Inc.", "market_cap": 500000000000, "sector": "Financial"},
    {"ticker": "JPM", "name": "JPMorgan Chase & Co.", "market_cap": 450000000000, "sector": "Financial"},
    {"ticker": "MA", "name": "Mastercard Inc.", "market_cap": 400000000000, "sector": "Financial"},
    
    # Healthcare
    {"ticker": "UNH", "name": "UnitedHealth Group Inc.", "market_cap": 480000000000, "sector": "Healthcare"},
    {"ticker": "JNJ", "name": "Johnson & Johnson", "market_cap": 400000000000, "sector": "Healthcare"},
    {"ticker": "LLY", "name": "Eli Lilly and Company", "market_cap": 550000000000, "sector": "Healthcare"},
    {"ticker": "PFE", "name": "Pfizer Inc.", "market_cap": 160000000000, "sector": "Healthcare"},
    {"ticker": "ABBV", "name": "AbbVie Inc.", "market_cap": 280000000000, "sector": "Healthcare"},
    {"ticker": "MRK", "name": "Merck & Co. Inc.", "market_cap": 250000000000, "sector": "Healthcare"},
    
    # Consumer
    {"ticker": "PG", "name": "Procter & Gamble Co.", "market_cap": 360000000000, "sector": "Consumer Defensive"},
    {"ticker": "KO", "name": "Coca-Cola Co.", "market_cap": 260000000000, "sector": "Consumer Defensive"},
    {"ticker": "PEP", "name": "PepsiCo Inc.", "market_cap": 230000000000, "sector": "Consumer Defensive"},
    {"ticker": "COST", "name": "Costco Wholesale Corp.", "market_cap": 320000000000, "sector": "Consumer Defensive"},
    {"ticker": "WMT", "name": "Walmart Inc.", "market_cap": 450000000000, "sector": "Consumer Defensive"},
    {"ticker": "HD", "name": "Home Depot Inc.", "market_cap": 380000000000, "sector": "Consumer Cyclical"},
    {"ticker": "MCD", "name": "McDonald's Corp.", "market_cap": 200000000000, "sector": "Consumer Cyclical"},
    {"ticker": "NKE", "name": "Nike Inc. Class B", "market_cap": 150000000000, "sector": "Consumer Cyclical"},
    
    # Energy
    {"ticker": "XOM", "name": "Exxon Mobil Corp.", "market_cap": 420000000000, "sector": "Energy"},
    {"ticker": "CVX", "name": "Chevron Corp.", "market_cap": 280000000000, "sector": "Energy"},
    
    # Communication
    {"ticker": "DIS", "name": "Walt Disney Co.", "market_cap": 180000000000, "sector": "Communication Services"},
    {"ticker": "NFLX", "name": "Netflix Inc.", "market_cap": 200000000000, "sector": "Communication Services"},
    {"ticker": "CMCSA", "name": "Comcast Corp. Class A", "market_cap": 160000000000, "sector": "Communication Services"},
   {"ticker": "VZ", "name": "Verizon Communications Inc.", "market_cap": 170000000000, "sector": "Communication Services"},
    {"ticker": "T", "name": "AT&T Inc.", "market_cap": 120000000000, "sector": "Communication Services"},
    
    # More Tech
    {"ticker": "ORCL", "name": "Oracle Corp.", "market_cap": 280000000000, "sector": "Technology"},
    {"ticker": "CRM", "name": "Salesforce Inc.", "market_cap": 220000000000, "sector": "Technology"},
    {"ticker": "ADBE", "name": "Adobe Inc.", "market_cap": 240000000000, "sector": "Technology"},
    {"ticker": "CSCO", "name": "Cisco Systems Inc.", "market_cap": 200000000000, "sector": "Technology"},
    {"ticker": "AVGO", "name": "Broadcom Inc.", "market_cap": 600000000000, "sector": "Technology"},
    {"ticker": "INTC", "name": "Intel Corp.", "market_cap": 180000000000, "sector": "Technology"},
    {"ticker": "AMD", "name": "Advanced Micro Devices Inc.", "market_cap": 220000000000, "sector": "Technology"},
    {"ticker": "QCOM", "name": "Qualcomm Inc.", "market_cap": 190000000000, "sector": "Technology"},
    {"ticker": "TXN", "name": "Texas Instruments Inc.", "market_cap": 170000000000, "sector": "Technology"},
    
    # More Finance
    {"ticker": "BAC", "name": "Bank of America Corp.", "market_cap": 280000000000, "sector": "Financial"},
    {"ticker": "WFC", "name": "Wells Fargo & Co.", "market_cap": 190000000000, "sector": "Financial"},
    {"ticker": "MS", "name": "Morgan Stanley", "market_cap": 150000000000, "sector": "Financial"},
    {"ticker": "GS", "name": "Goldman Sachs Group Inc.", "market_cap": 130000000000, "sector": "Financial"},
    {"ticker": "BLK", "name": "BlackRock Inc.", "market_cap": 120000000000, "sector": "Financial"},
    {"ticker": "AXP", "name": "American Express Co.", "market_cap": 140000000000, "sector": "Financial"},
    
    # Industrial  
    {"ticker": "BA", "name": "Boeing Co.", "market_cap": 130000000000, "sector": "Industrials"},
    {"ticker": "CAT", "name": "Caterpillar Inc.", "market_cap": 150000000000, "sector": "Industrials"},
    {"ticker": "GE", "name": "General Electric Co.", "market_cap": 140000000000, "sector": "Industrials"},
    {"ticker": "HON", "name": "Honeywell International Inc.", "market_cap": 130000000000, "sector": "Industrials"},
    {"ticker": "UPS", "name": "United Parcel Service Inc. Class B", "market_cap": 130000000000, "sector": "Industrials"},
    
    # More stocks to reach 100
    {"ticker": "ACN", "name": "Accenture plc Class A", "market_cap": 210000000000, "sector": "Technology"},
    {"ticker": "INTU", "name": "Intuit Inc.", "market_cap": 160000000000, "sector": "Technology"},
    {"ticker": "IBM", "name": "International Business Machines Corp.", "market_cap": 170000000000, "sector": "Technology"},
    {"ticker": "NOW", "name": "ServiceNow Inc.", "market_cap": 140000000000, "sector": "Technology"},
    {"ticker": "PYPL", "name": "PayPal Holdings Inc.", "market_cap": 70000000000, "sector": "Financial"},
    {"ticker": "SCHW", "name": "Charles Schwab Corp.", "market_cap": 120000000000, "sector": "Financial"},
    {"ticker": "C", "name": "Citigroup Inc.", "market_cap": 110000000000, "sector": "Financial"},
    {"ticker": "BMY", "name": "Bristol-Myers Squibb Co.", "market_cap": 100000000000, "sector": "Healthcare"},
    {"ticker": "TMO", "name": "Thermo Fisher Scientific Inc.", "market_cap": 210000000000, "sector": "Healthcare"},
    {"ticker": "ABT", "name": "Abbott Laboratories", "market_cap": 190000000000, "sector": "Healthcare"},
    {"ticker": "DHR", "name": "Danaher Corp.", "market_cap": 180000000000, "sector": "Healthcare"},
    {"ticker": "PM", "name": "Philip Morris International Inc.", "market_cap": 140000000000, "sector": "Consumer Defensive"},
    {"ticker": "NEE", "name": "NextEra Energy Inc.", "market_cap": 140000000000, "sector": "Utilities"},
    {"ticker": "RTX", "name": "RTX Corp.", "market_cap": 130000000000, "sector": "Industrials"},
    {"ticker": "LMT", "name": "Lockheed Martin Corp.", "market_cap": 110000000000, "sector": "Industrials"},
{"ticker": "UNP", "name": "Union Pacific Corp.", "market_cap": 140000000000, "sector": "Industrials"},
    {"ticker": "LOW", "name": "Lowe's Companies Inc.", "market_cap": 130000000000, "sector": "Consumer Cyclical"},
    {"ticker": "SBUX", "name": "Starbucks Corp.", "market_cap": 110000000000, "sector": "Consumer Cyclical"},
    {"ticker": "MDT", "name": "Medtronic plc", "market_cap": 110000000000, "sector": "Healthcare"},
    {"ticker": "AMGN", "name": "Amgen Inc.", "market_cap": 140000000000, "sector": "Healthcare"},
    {"ticker": "GILD", "name": "Gilead Sciences Inc.", "market_cap": 100000000000, "sector": "Healthcare"},
    {"ticker": "CVS", "name": "CVS Health Corp.", "market_cap": 80000000000, "sector": "Healthcare"},
    {"ticker": "CI", "name": "Cigna Group", "market_cap": 90000000000, "sector": "Healthcare"},
    {"ticker": "SPGI", "name": "S&P Global Inc.", "market_cap": 130000000000, "sector": "Financial"},
    {"ticker": "CB", "name": "Chubb Ltd.", "market_cap": 100000000000, "sector": "Financial"},
    {"ticker": "MMC", "name": "Marsh McLennan Cos. Inc.", "market_cap": 100000000000, "sector": "Financial"},
    {"ticker": "TJX", "name": "TJX Companies Inc.", "market_cap": 110000000000, "sector": "Consumer Cyclical"},
    {"ticker": "BKNG", "name": "Booking Holdings Inc.", "market_cap": 130000000000, "sector": "Consumer Cyclical"},
    {"ticker": "AMAT", "name": "Applied Materials Inc.", "market_cap": 140000000000, "sector": "Technology"},
    {"ticker": "ADI", "name": "Analog Devices Inc.", "market_cap": 100000000000, "sector": "Technology"},
    {"ticker": "LRCX", "name": "Lam Research Corp.", "market_cap": 100000000000, "sector": "Technology"},
    {"ticker": "MU", "name": "Micron Technology Inc.", "market_cap": 110000000000, "sector": "Technology"},
    {"ticker": "ISRG", "name": "Intuitive Surgical Inc.", "market_cap": 120000000000, "sector": "Healthcare"},
    {"ticker": "REGN", "name": "Regeneron Pharmaceuticals Inc.", "market_cap": 100000000000, "sector": "Healthcare"},
    {"ticker": "VRTX", "name": "Vertex Pharmaceuticals Inc.", "market_cap": 110000000000, "sector": "Healthcare"},
    {"ticker": "ELV", "name": "Elevance Health Inc.", "market_cap": 110000000000, "sector": "Healthcare"},
    {"ticker": "ZTS", "name": "Zoetis Inc. Class A", "market_cap": 80000000000, "sector": "Healthcare"},
    {"ticker": "SYK", "name": "Stryker Corp.", "market_cap": 110000000000, "sector": "Healthcare"},
    {"ticker": "BDX", "name": "Becton Dickinson and Co.", "market_cap": 70000000000, "sector": "Healthcare"},
    {"ticker": "DUK", "name": "Duke Energy Corp.", "market_cap": 80000000000, "sector": "Utilities"},
    {"ticker": "SO", "name": "Southern Co.", "market_cap": 90000000000, "sector": "Utilities"},
    {"ticker": "D", "name": "Dominion Energy Inc.", "market_cap": 45000000000, "sector": "Utilities"},
]

def main():
    # Get database URL from environment
    db_url = os.environ.get("DATABASE_URL", "postgresql://postgres:password@localhost:5432/hermesflow")
    
    print("=" * 70)
    print("Initializing Top 100 US Stocks Watchlist")
    print("=" * 70)
    print(f"Database: {db_url}")
    print(f"Total stocks: {len(TOP_100_US_STOCKS)}")
    print()
    
    # Connect to database
    conn = psycopg2.connect(db_url)
    cur = conn.cursor()
    
    # Prepare data for insertion
    # Enable all timeframes for top stocks
    data = []
    for i, stock in enumerate(TOP_100_US_STOCKS, 1):
        # Priority: higher market cap = higher priority
        priority = 100 - (i // 10)  # 100, 99, 98, ..., 90 for top 10, etc
        
        data.append((
            'Polygon',                    # exchange
            stock['ticker'],              # symbol
            'stock',                      # asset_type
            stock['name'],                # name
            None,                         # base_currency
            None,                         # quote_currency
            True,  # enabled_1m
            False, # enabled_5m
            True,  # enabled_15m
            False, # enabled_30m
            True,  # enabled_1h
            True,  # enabled_4h
            True,  # enabled_1d
            False, # enabled_1w
            True,                         # is_active
            priority,                     # priority
            '2023-01-01',                 # sync_from_date
            f'{{"market_cap": {stock["market_cap"]}, "sector": "{stock["sector"]}"}}',  # metadata
            None,                         # notes
        ))
    
    # Insert using execute_values (efficient batch insert)
    execute_values(
        cur,
        """
        INSERT INTO market_watchlist 
        (exchange, symbol, asset_type, name,
         base_currency, quote_currency,
         enabled_1m, enabled_5m, enabled_15m, enabled_30m,
         enabled_1h, enabled_4h, enabled_1d, enabled_1w,
         is_active, priority, sync_from_date, metadata, notes)
        VALUES %s
        ON CONFLICT (exchange, symbol) DO UPDATE SET
            name = EXCLUDED.name,
            metadata = EXCLUDED.metadata,
            priority = EXCLUDED.priority
        """,
        data
    )
    
    conn.commit()
    
    # Verify insertion
    cur.execute("SELECT COUNT(*) FROM market_watchlist WHERE exchange = 'Polygon'")
    count = cur.fetchone()[0]
    
    print(f"✅ Inserted/updated {count} stocks in market_watchlist")
    print()
    
    # Show sync tasks created
    cur.execute("""
        SELECT 
            COUNT(*) as task_count,
            status
        FROM market_sync_status
        WHERE exchange = 'Polygon'
        GROUP BY status
        ORDER BY status
    """)
    
    print("Sync tasks created:")
    for row in cur.fetchall():
        print(f"  - {row[1]}: {row[0]} tasks")
    
    conn.close()
    
    print()
    print("=" * 70)
    print("✅ Initialization complete!")
    print("=" * 70)
    print()
    print("The auto-sync worker will now process these pending tasks.")
    print("You can monitor progress with:")
    print("  - Check watchlist: SELECT * FROM market_watchlist LIMIT 10;")
    print("  - Check sync status: SELECT * FROM market_sync_status;")
    print()

if __name__ == "__main__":
    main()
