"""
Futu Bridge - HTTP bridge between HermesFlow execution-engine (Rust) and Futu OpenD (Python SDK).

Exposes REST endpoints for order management and account queries.
Futu OpenD must be running and accessible at FUTU_OPEND_HOST:FUTU_OPEND_PORT.
"""

import asyncio
import os
import logging
from concurrent.futures import ThreadPoolExecutor
from contextlib import asynccontextmanager
from enum import Enum
from typing import Optional

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from futu import (
    OpenQuoteContext,
    OpenSecTradeContext,
    TrdEnv,
    TrdSide,
    OrderType,
    TrdMarket,
    RET_OK,
    ModifyOrderOp,
)

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("futu-bridge")

# --- Config ---
OPEND_HOST = os.getenv("FUTU_OPEND_HOST", "127.0.0.1")
OPEND_PORT = int(os.getenv("FUTU_OPEND_PORT", "11111"))
TRD_ENV_STR = os.getenv("FUTU_TRD_ENV", "SIMULATE")

TRD_ENV_MAP = {
    "SIMULATE": TrdEnv.SIMULATE,
    "REAL": TrdEnv.REAL,
}


def get_trd_env() -> TrdEnv:
    return TRD_ENV_MAP.get(TRD_ENV_STR.upper(), TrdEnv.SIMULATE)


def detect_market(symbol: str) -> TrdMarket:
    """Detect trading market from symbol prefix."""
    if symbol.startswith("US."):
        return TrdMarket.US
    elif symbol.startswith("HK."):
        return TrdMarket.HK
    elif symbol.startswith("SH.") or symbol.startswith("SZ."):
        return TrdMarket.CN
    return TrdMarket.US


# --- Globals ---
trd_ctx: Optional[OpenSecTradeContext] = None
quote_ctx: Optional[OpenQuoteContext] = None


CONNECT_TIMEOUT = int(os.getenv("FUTU_CONNECT_TIMEOUT", "10"))

_executor = ThreadPoolExecutor(max_workers=1)


def _connect_futu():
    """Blocking function to connect to Futu OpenD. Runs in a thread."""
    trd = OpenSecTradeContext(
        host=OPEND_HOST, port=OPEND_PORT, filter_trdmarket=TrdMarket.US, security_firm=None
    )
    quote = OpenQuoteContext(host=OPEND_HOST, port=OPEND_PORT)
    return trd, quote


@asynccontextmanager
async def lifespan(app: FastAPI):
    global trd_ctx, quote_ctx
    logger.info(f"Connecting to Futu OpenD at {OPEND_HOST}:{OPEND_PORT} (env={TRD_ENV_STR})")
    try:
        loop = asyncio.get_running_loop()
        trd_ctx, quote_ctx = await asyncio.wait_for(
            loop.run_in_executor(_executor, _connect_futu),
            timeout=CONNECT_TIMEOUT,
        )
        logger.info("Futu OpenD connected")
    except asyncio.TimeoutError:
        logger.warning(f"Futu OpenD connection timed out after {CONNECT_TIMEOUT}s, starting in degraded mode")
        trd_ctx = None
        quote_ctx = None
    except Exception as e:
        logger.error(f"Failed to connect to Futu OpenD: {e}")
        trd_ctx = None
        quote_ctx = None
    yield
    if trd_ctx:
        trd_ctx.close()
    if quote_ctx:
        quote_ctx.close()
    logger.info("Futu contexts closed")


app = FastAPI(title="Futu Bridge", version="1.0.0", lifespan=lifespan)


# --- Models ---
class SideEnum(str, Enum):
    buy = "Buy"
    sell = "Sell"


class OrderTypeEnum(str, Enum):
    market = "Market"
    limit = "Limit"
    market_on_close = "MarketOnClose"


class PlaceOrderRequest(BaseModel):
    symbol: str
    side: SideEnum
    quantity: float
    order_type: OrderTypeEnum = OrderTypeEnum.market
    limit_price: Optional[float] = None


class OrderResponse(BaseModel):
    order_id: str
    status: str
    broker: str = "Futu"


class PositionItem(BaseModel):
    symbol: str
    quantity: float
    avg_cost: float
    market_value: float
    unrealized_pnl: float


class AccountSummaryResponse(BaseModel):
    net_liquidation: float
    cash: float
    buying_power: float
    currency: str


# --- Helpers ---
def map_order_type(ot: OrderTypeEnum) -> OrderType:
    if ot == OrderTypeEnum.limit:
        return OrderType.NORMAL
    elif ot == OrderTypeEnum.market_on_close:
        return OrderType.MARKET
    return OrderType.MARKET


def map_side(side: SideEnum) -> TrdSide:
    if side == SideEnum.buy:
        return TrdSide.BUY
    return TrdSide.SELL


# --- Endpoints ---
@app.get("/health")
async def health():
    connected = trd_ctx is not None
    return {"status": "ok" if connected else "degraded", "opend_connected": connected}


@app.post("/api/order", response_model=OrderResponse)
async def place_order(req: PlaceOrderRequest):
    if not trd_ctx:
        raise HTTPException(status_code=503, detail="Futu OpenD not connected")

    trd_env = get_trd_env()
    futu_side = map_side(req.side)
    futu_order_type = map_order_type(req.order_type)

    price = req.limit_price if req.limit_price else 0.0

    # For market orders, we need to get a reference price from quote context
    if req.order_type == OrderTypeEnum.market and quote_ctx:
        ret, data = quote_ctx.get_market_snapshot([req.symbol])
        if ret == RET_OK and not data.empty:
            price = float(data["last_price"].iloc[0])
            logger.info(f"Market order reference price for {req.symbol}: {price}")

    logger.info(
        f"Placing order: {req.side.value} {req.symbol} x{req.quantity} "
        f"type={req.order_type.value} price={price}"
    )

    ret, data = trd_ctx.place_order(
        price=price,
        qty=req.quantity,
        code=req.symbol,
        trd_side=futu_side,
        order_type=futu_order_type,
        trd_env=trd_env,
    )

    if ret != RET_OK:
        logger.error(f"place_order failed: {data}")
        raise HTTPException(status_code=500, detail="Order placement failed")

    order_id = str(data["order_id"].iloc[0])
    status = str(data["order_status"].iloc[0])
    logger.info(f"Order placed: id={order_id} status={status}")

    return OrderResponse(order_id=order_id, status=status)


@app.delete("/api/order/{order_id}")
async def cancel_order(order_id: str):
    if not trd_ctx:
        raise HTTPException(status_code=503, detail="Futu OpenD not connected")

    trd_env = get_trd_env()
    ret, data = trd_ctx.modify_order(
        modify_order_op=ModifyOrderOp.CANCEL,
        order_id=order_id,
        qty=0,
        price=0,
        trd_env=trd_env,
    )

    if ret != RET_OK:
        logger.error(f"cancel_order failed: {data}")
        raise HTTPException(status_code=500, detail="Order cancellation failed")

    return {"status": "cancelled", "order_id": order_id}


@app.get("/api/positions", response_model=list[PositionItem])
async def get_positions():
    if not trd_ctx:
        raise HTTPException(status_code=503, detail="Futu OpenD not connected")

    trd_env = get_trd_env()
    ret, data = trd_ctx.position_list_query(trd_env=trd_env)

    if ret != RET_OK:
        logger.error(f"position_list_query failed: {data}")
        raise HTTPException(status_code=500, detail="Failed to query positions")

    positions = []
    for _, row in data.iterrows():
        positions.append(
            PositionItem(
                symbol=row["code"],
                quantity=float(row["qty"]),
                avg_cost=float(row["cost_price"]),
                market_value=float(row["market_val"]),
                unrealized_pnl=float(row["pl_val"]),
            )
        )

    return positions


@app.get("/api/account", response_model=AccountSummaryResponse)
async def get_account():
    if not trd_ctx:
        raise HTTPException(status_code=503, detail="Futu OpenD not connected")

    trd_env = get_trd_env()
    ret, data = trd_ctx.accinfo_query(trd_env=trd_env)

    if ret != RET_OK:
        logger.error(f"accinfo_query failed: {data}")
        raise HTTPException(status_code=500, detail="Failed to query account info")

    row = data.iloc[0]
    return AccountSummaryResponse(
        net_liquidation=float(row.get("total_assets", 0)),
        cash=float(row.get("cash", 0)),
        buying_power=float(row.get("avl_withdrawal_cash", 0)),
        currency=str(row.get("currency", "USD")),
    )
