# futu-bridge - FOLDER_INDEX

> Python FastAPI HTTP bridge between HermesFlow execution-engine and Futu OpenD SDK. Translates REST calls into Futu SDK operations for HK stock trading.

## Module Map

```
app/
  main.py              # FastAPI app: lifespan (OpenD connection), endpoints
                       #   GET  /health           - Connection status
                       #   POST /api/order         - Place order
                       #   DELETE /api/order/{id}   - Cancel order
                       #   GET  /api/positions      - Query positions
                       #   GET  /api/account        - Account summary
```

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Returns `ok` if OpenD connected, `degraded` otherwise |
| POST | `/api/order` | Place order (symbol, side, qty, type, limit_price) |
| DELETE | `/api/order/{order_id}` | Cancel pending order |
| GET | `/api/positions` | List current positions |
| GET | `/api/account` | Account summary (NAV, cash, buying power) |

## Config (env vars)
- `FUTU_OPEND_HOST` (default: `127.0.0.1`)
- `FUTU_OPEND_PORT` (default: `11111`)
- `FUTU_TRD_ENV` (default: `SIMULATE`, options: `SIMULATE`/`REAL`)
- `FUTU_CONNECT_TIMEOUT` (default: `10` seconds)

## Market Detection
Auto-detects market from symbol prefix: `US.` → US, `HK.` → HK, `SH./SZ.` → CN

## Dependencies
- FastAPI, Pydantic, uvicorn
- `futu-api` (Futu OpenD Python SDK)
