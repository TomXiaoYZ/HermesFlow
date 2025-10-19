# Build stage
FROM python:3.12-slim AS builder
WORKDIR /app

# Install dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir --user -r requirements.txt

# Runtime stage
FROM python:3.12-slim
WORKDIR /app

# Copy dependencies from builder
COPY --from=builder /root/.local /root/.local

# Copy application code
COPY src ./src

# Make sure scripts in .local are usable
ENV PATH=/root/.local/bin:$PATH
ENV PYTHONPATH=/app/src

EXPOSE 8030

CMD ["python", "-m", "uvicorn", "risk_engine.main:app", "--host", "0.0.0.0", "--port", "8030"]
