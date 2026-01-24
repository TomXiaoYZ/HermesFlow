"""Main FastAPI application for Risk Engine"""

from fastapi import FastAPI
from hermes_common import setup_logging
from .health import router as health_router

# Configure structured logging
setup_logging("risk-engine")

app = FastAPI(
    title="Risk Engine",
    description="HermesFlow Risk Engine Service",
    version="0.1.0"
)

app.include_router(health_router)

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8030)
