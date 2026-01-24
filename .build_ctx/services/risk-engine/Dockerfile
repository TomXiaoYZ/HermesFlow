FROM python:3.11-slim

WORKDIR /app

# Copy shared library
COPY infrastructure/python/hermes_common /libs/hermes_common

# Copy service code
COPY services/risk-engine /app

# Install packages
RUN pip install --no-cache-dir /libs/hermes_common
RUN pip install --no-cache-dir .

EXPOSE 8030

CMD ["uvicorn", "risk_engine.main:app", "--host", "0.0.0.0", "--port", "8030"]
