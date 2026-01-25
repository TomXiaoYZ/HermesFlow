-- Helper function to load factor configs from database
-- Used by strategy-generator to dynamically load active factors

CREATE OR REPLACE FUNCTION get_active_factor_configs()
RETURNS TABLE (
    slug TEXT,
    normalization TEXT,
    parameters JSONB
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        f.slug,
        f.normalization,
        f.parameters
    FROM factors f
    WHERE f.is_active = true
    ORDER BY f.id;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION get_active_factor_configs() IS 
'Returns configuration for all active factors for use in feature engineering';
