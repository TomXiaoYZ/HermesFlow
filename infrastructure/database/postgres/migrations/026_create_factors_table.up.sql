-- Factor Library - Single Table Design
-- Simplified schema with all metadata in one table

CREATE TABLE factors (
    id SERIAL PRIMARY KEY,
    
    -- 基本信息
    name VARCHAR(200) NOT NULL,
    slug VARCHAR(200) NOT NULL UNIQUE,
    category VARCHAR(100) NOT NULL,
    
    -- 计算逻辑
    rust_function VARCHAR(500),
    formula TEXT NOT NULL,
    latex_formula TEXT,
    
    -- 文档
    description TEXT NOT NULL,
    interpretation TEXT,
    
    -- 参数和示例 (JSONB for flexibility)
    parameters JSONB DEFAULT '[]',
    examples JSONB,
    
    -- 元数据
    output_range TEXT,
    normalization VARCHAR(50),
    computation_cost VARCHAR(20),
    min_bars_required INTEGER DEFAULT 0,
    
    -- 标签和引用
    tags TEXT[],
    refs JSONB,
    
    -- 状态
    is_active BOOLEAN DEFAULT true,
    version INTEGER DEFAULT 1,
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_factors_category ON factors(category);
CREATE INDEX idx_factors_slug ON factors(slug);
CREATE INDEX idx_factors_tags ON factors USING GIN(tags);
CREATE INDEX idx_factors_active ON factors(is_active);

-- Comments
COMMENT ON TABLE factors IS 'Technical analysis factor library with formulas and documentation';
COMMENT ON COLUMN factors.parameters IS 'JSONB array of parameter definitions with validation rules';
COMMENT ON COLUMN factors.examples IS 'JSONB object with input/output examples and calculations';
COMMENT ON COLUMN factors.tags IS 'Array of searchable tags for categorization';
