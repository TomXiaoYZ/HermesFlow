-- Migration 018: Data quality incident tracking table
-- Records detected quality issues for audit and auto-remediation workflows.

CREATE TABLE IF NOT EXISTS dq_incidents (
    id BIGSERIAL PRIMARY KEY,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    check_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    symbol VARCHAR(100),
    source VARCHAR(50),
    details JSONB,
    resolved_at TIMESTAMPTZ,
    resolution TEXT
);

CREATE INDEX IF NOT EXISTS idx_dq_incidents_detected
    ON dq_incidents (detected_at DESC);

CREATE INDEX IF NOT EXISTS idx_dq_incidents_type_severity
    ON dq_incidents (check_type, severity);
