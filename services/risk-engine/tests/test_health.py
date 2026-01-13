"""Tests for health endpoint"""

from fastapi.testclient import TestClient
from risk_engine.main import app

client = TestClient(app)


def test_health_check():
    """Test health check endpoint"""
    response = client.get("/health")
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "healthy"
    assert data["service"] == "risk-engine"
    assert data["version"] == "0.1.0"


def test_health_check_response_structure():
    """Test health check response structure"""
    response = client.get("/health")
    data = response.json()
    assert "status" in data
    assert "service" in data
    assert "version" in data
