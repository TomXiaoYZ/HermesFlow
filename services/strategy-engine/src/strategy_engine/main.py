"""Main FastAPI application for Strategy Engine"""

from fastapi import FastAPI
from hermes_common import setup_logging
from .health import router as health_router

setup_logging("strategy-engine")

app = FastAPI(
    title="Strategy Engine",
    description="HermesFlow Strategy Engine Service",
    version="0.1.0"
)

app.include_router(health_router)

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8040)
