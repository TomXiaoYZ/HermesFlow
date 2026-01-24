FROM mcr.microsoft.com/devcontainers/python:3.12

WORKDIR /app

# Copy requirements first to leverage Docker cache
COPY requirements.txt .

# Install Python dependencies including Playwright
RUN pip install --upgrade pip && \
    pip install --no-cache-dir --default-timeout=300 -r requirements.txt

# Install Playwright browsers AFTER installing the playwright package from requirements.txt
# This ensures version compatibility
RUN playwright install chromium && \
    playwright install-deps chromium

# Copy application code
COPY xscraper ./xscraper
COPY api.py .

# Create data directory for cookies
RUN mkdir -p /app/data

# Expose port
EXPOSE 8000

# Set environment variables
ENV PYTHONUNBUFFERED=1

# Run the FastAPI application
CMD ["uvicorn", "api:app", "--host", "0.0.0.0", "--port", "8000"]
