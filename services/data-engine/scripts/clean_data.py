import os
import psycopg2
from urllib.parse import urlparse

# Configuration
PG_HOST = os.getenv("DATA_ENGINE__POSTGRES__HOST", "localhost")
PG_PORT = os.getenv("DATA_ENGINE__POSTGRES__PORT", "5432")
PG_DB = os.getenv("DATA_ENGINE__POSTGRES__DATABASE", "hermesflow")
PG_USER = os.getenv("DATA_ENGINE__POSTGRES__USERNAME", "postgres")
PG_PASS = os.getenv("DATA_ENGINE__POSTGRES__PASSWORD", "password")

def get_db_connection():
    try:
        conn = psycopg2.connect(
            host=PG_HOST,
            port=PG_PORT,
            dbname=PG_DB,
            user=PG_USER,
            password=PG_PASS
        )
        return conn
    except Exception as e:
        print(f"❌ Failed to connect to DB: {e}")
        return None

def clean_dirty_data():
    conn = get_db_connection()
    if not conn:
        return

    try:
        cur = conn.cursor()
        
        # Count before
        print("🔍 Checking for dirty data (Low <= 0)...")
        cur.execute("SELECT COUNT(*) FROM mkt_equity_candles WHERE low <= 0")
        count_before = cur.fetchone()[0]
        print(f"Found {count_before} invalid rows.")

        if count_before > 0:
            print("🧹 Deleting invalid rows...")
            cur.execute("DELETE FROM mkt_equity_candles WHERE low <= 0")
            deleted_count = cur.rowcount
            conn.commit()
            print(f"✅ Deleted {deleted_count} rows.")
        else:
            print("✨ No dirty data found. Database is clean.")

        cur.close()
        conn.close()

    except Exception as e:
        print(f"❌ Error during cleanup: {e}")

if __name__ == "__main__":
    clean_dirty_data()
