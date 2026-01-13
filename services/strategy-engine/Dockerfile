FROM python:3.11-slim

WORKDIR /app

# Copy shared library
COPY infrastructure/python/hermes_common /libs/hermes_common

# Copy service code
COPY services/strategy-engine /app

# Install packages
RUN pip install --no-cache-dir /libs/hermes_common
RUN pip install --no-cache-dir .

EXPOSE 8040

CMD ["uvicorn", "strategy_engine.main:app", "--host", "0.0.0.0", "--port", "8040"]
