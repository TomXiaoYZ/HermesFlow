-- Seed all 33 factors into database
-- Complete factor library definition

-- Meme Indicators (9)
INSERT INTO factors (name, slug, category, rust_function, formula, description, interpretation, parameters, output_range, normalization, computation_cost, min_bars_required, tags, is_active) VALUES
('Log Returns', 'log_returns', 'meme', 'ret = ln(close/prev_close)', 'ret = ln(P[t] / P[t-1])', 'Logarithmic price returns', 'Positive = price increase, Negative = price decrease', '[]'::jsonb, 'unbounded', 'robust', 'low', 1, ARRAY['returns', 'meme'], true),
('Liquidity Health', 'liquidity_health', 'meme', 'MemeIndicators::liquidity_health', 'score = tanh(liquidity / (fdv * threshold))', 'Liquidity relative to market cap', 'Higher = better liquidity, >0.8 = healthy', '[]'::jsonb, '0-1', 'none', 'low', 1, ARRAY['liquidity', 'meme'], true),
('Buy/Sell Pressure', 'buy_sell_pressure', 'meme', 'MemeIndicators::buy_sell_imbalance', 'pressure = (close - open) / (high - low)', 'Intraday buying or selling pressure', '>0 = buying pressure, <0 = selling pressure', '[]'::jsonb, '-1 to 1', 'none', 'low', 1, ARRAY['pressure', 'meme'], true),
('FOMO Acceleration', 'fomo', 'meme', 'MemeIndicators::fomo_acceleration', 'fomo = d²volume/dt²', 'Second derivative of volume (acceleration)', 'High positive = FOMO buying, High negative = panic selling', '[]'::jsonb, 'unbounded', 'robust', 'low', 3, ARRAY['volume', 'fomo', 'meme'], true),
('Pump Deviation', 'pump_deviation', 'meme', 'MemeIndicators::pump_deviation', 'dev = (close - SMA) / SMA', 'Price deviation from moving average', '>0.5 = potential pump, <-0.5 = potential dump', '[{"name":"window","default":20}]'::jsonb, 'unbounded', 'robust', 'low', 20, ARRAY['pump', 'deviation', 'meme'], true),
('Log Volume', 'log_volume', 'meme', 'ln(volume + 1)', 'log_vol = ln(V + 1)', 'Natural log of trading volume', 'Higher = more activity', '[]'::jsonb, 'unbounded', 'robust', 'low', 1, ARRAY['volume'], true),
('Volatility Clustering', 'vol_clustering', 'meme', 'MemeIndicators::volatility_clustering', 'cluster = rolling_std(returns)', 'GARCH-style volatility measurement', 'High = periods of high volatility', '[{"name":"window","default":20}]'::jsonb, 'unbounded', 'robust', 'medium', 20, ARRAY['volatility', 'meme'], true),
('Momentum Reversal', 'momentum_reversal', 'meme', 'MemeIndicators::momentum_reversal', 'Binary signal for momentum reversal', '1 = reversal detected, 0 = no reversal', '[{"name":"window","default":20}]'::jsonb, '0-1', 'robust', 'medium', 20, ARRAY['reversal', 'momentum', 'meme'], true),
('RSI', 'rsi', 'meme', 'MemeIndicators::relative_strength', 'RSI = 100 - (100 / (1 + RS))', 'Relative Strength Index', '>70 overbought, <30 oversold', '[{"name":"period","default":14}]'::jsonb, '0-100', 'robust', 'medium', 14, ARRAY['rsi', 'oscillator'], true);

-- Moving Averages (4)
INSERT INTO factors (name, slug, category, rust_function, formula, description, interpretation, parameters, output_range, normalization, computation_cost, min_bars_required, tags, is_active) VALUES
('EMA 12', 'ema_12', 'moving_averages', 'MovingAverages::ema', 'EMA[t] = α×P[t] + (1-α)×EMA[t-1], α=2/(n+1)', 'Fast exponential moving average', 'Price above = bullish, Price below = bearish', '[{"name":"period","default":12}]'::jsonb, 'unbounded', 'custom', 'low', 12, ARRAY['trend', 'ema'], true),
('EMA 26', 'ema_26', 'moving_averages', 'MovingAverages::ema', 'EMA with 26 period', 'Medium-term trend indicator', 'Crossover with EMA12 = trend change', '[{"name":"period","default":26}]'::jsonb, 'unbounded', 'custom', 'low', 26, ARRAY['trend', 'ema'], true),
('EMA 50', 'ema_50', 'moving_averages', 'MovingAverages::ema', 'EMA with 50 period', 'Intermediate trend indicator', 'Common support/resistance level', '[{"name":"period","default":50}]'::jsonb, 'unbounded', 'custom', 'low', 50, ARRAY['trend', 'ema'], true),
('SMA 200', 'sma_200', 'moving_averages', 'MovingAverages::sma', 'SMA = (P1+P2+...+Pn)/n', 'Long-term trend (200-day MA)', 'Price above = bull market, Price below = bear market', '[{"name":"period","default":200}]'::jsonb, 'unbounded', 'custom', 'low', 200, ARRAY['trend', 'sma', 'long_term'], true);

-- MACD (3)
INSERT INTO factors (name, slug, category, rust_function, formula, description, interpretation, parameters, output_range, normalization, computation_cost, min_bars_required, tags, is_active) VALUES
('MACD Line', 'macd_line', 'momentum', 'MACD::macd', 'MACD = EMA(12) - EMA(26)', 'MACD line component', 'Positive = bullish momentum, Negative = bearish', '[{"name":"fast","default":12},{"name":"slow","default":26}]'::jsonb, 'unbounded', 'custom', 'low', 26, ARRAY['macd', 'momentum'], true),
('MACD Signal', 'macd_signal', 'momentum', 'MACD::macd', 'Signal = EMA(MACD, 9)', 'MACD signal line', 'Smoothed MACD for crossover signals', '[{"name":"signal","default":9}]'::jsonb, 'unbounded', 'custom', 'low', 35, ARRAY['macd', 'momentum'], true),
('MACD Histogram', 'macd_hist', 'momentum', 'MACD::macd', 'Histogram = MACD - Signal', 'MACD histogram', 'Expanding = strengthening momentum', '[]'::jsonb, 'unbounded', 'custom', 'low', 35, ARRAY['macd', 'momentum', 'histogram'], true);

-- Bollinger Bands (3)
INSERT INTO factors (name, slug, category, rust_function, formula, description, interpretation, parameters, output_range, normalization, computation_cost, min_bars_required, tags, is_active) VALUES
('Bollinger Bandwidth', 'bb_bandwidth', 'volatility', 'BollingerBands::bandwidth', 'BW = (Upper - Lower) / Middle', 'Bollinger Band width', 'Narrow = low volatility, Wide = high volatility', '[{"name":"period","default":20},{"name":"std","default":2}]'::jsonb, 'unbounded', 'robust', 'medium', 20, ARRAY['bollinger', 'volatility'], true),
('Bollinger %B', 'bb_percent_b', 'volatility', 'BollingerBands::percent_b', '%B = (Close - Lower) / (Upper - Lower)', 'Position within Bollinger Bands', '>1 = above upper band, <0 = below lower band', '[{"name":"period","default":20}]'::jsonb, 'unbounded', 'none', 'medium', 20, ARRAY['bollinger', 'overbought', 'oversold'], true),
('Bollinger Position', 'bb_position', 'volatility', 'bb_position = (close - middle) / middle', 'Distance from BB middle', 'Normalized distance from middle band', '[]'::jsonb, 'unbounded', 'robust', 'medium', 20, ARRAY['bollinger'], true);

-- ATR (1)
INSERT INTO factors (name, slug, category, rust_function, formula, description, interpretation, parameters, output_range, normalization, computation_cost, min_bars_required, tags, is_active) VALUES
('ATR Percent', 'atr_pct', 'volatility', 'ATR::atr_percent', 'ATR% = ATR / Close', 'Average True Range as percentage', 'Higher = more volatile', '[{"name":"period","default":14}]'::jsonb, '0-1', 'robust', 'low', 14, ARRAY['atr', 'volatility'], true);

-- Stochastic (2)
INSERT INTO factors (name, slug, category, rust_function, formula, description, interpretation, parameters, output_range, normalization, computation_cost, min_bars_required, tags, is_active) VALUES
('Stochastic %K', 'stoch_k', 'momentum', 'Stochastic::stochastic', '%K = (Close - Low14) / (High14 - Low14) × 100', 'Fast stochastic oscillator', '>80 overbought, <20 oversold', '[{"name":"k_period","default":14},{"name":"d_period","default":3}]'::jsonb, '0-100', 'custom', 'medium', 14, ARRAY['stochastic', 'oscillator', 'overbought'], true),
('Stochastic %D', 'stoch_d', 'momentum', 'Stochastic::stochastic', '%D = SMA(%K, 3)', 'Slow stochastic (signal line)', 'Crossover with %K = reversal signal', '[{"name":"d_period","default":3}]'::jsonb, '0-100', 'custom', 'medium', 17, ARRAY['stochastic', 'oscillator'], true);

-- CCI (1)
INSERT INTO factors (name, slug, category, rust_function, formula, description, interpretation, parameters, output_range, normalization, computation_cost, min_bars_required, tags, is_active) VALUES
('CCI', 'cci', 'momentum', 'CCI::cci_normalized', 'CCI = (TP - SMA(TP)) / (0.015 × MD)', 'Commodity Channel Index', '>100 overbought, <-100 oversold', '[{"name":"period","default":20}]'::jsonb, 'unbounded', 'custom', 'medium', 20, ARRAY['cci', 'oscillator', 'cyclical'], true);

-- Williams %R (1)
INSERT INTO factors (name, slug, category, rust_function, formula, description, interpretation, parameters, output_range, normalization, computation_cost, min_bars_required, tags, is_active) VALUES
('Williams %R', 'williams_r', 'momentum', 'WilliamsR::williams_r_normalized', '%R = -100 × (High14 - Close) / (High14 - Low14)', 'Williams Percent R oscillator', 'Near 0 = overbought, Near 100 = oversold (normalized)', '[{"name":"period","default":14}]'::jsonb, '0-1', 'custom', 'medium', 14, ARRAY['williams', 'oscillator', 'overbought'], true);

-- VWAP (2)
INSERT INTO factors (name, slug, category, rust_function, formula, description, interpretation, parameters, output_range, normalization, computation_cost, min_bars_required, tags, is_active) VALUES
('VWAP Deviation', 'vwap_dev', 'volume', 'VWAP::vwap_deviation', 'VWAP_dev = (Close - VWAP) / VWAP', 'Distance from VWAP', 'Positive = above VWAP (bullish), Negative = below (bearish)', '[]'::jsonb, 'unbounded', 'robust', 'medium', 1, ARRAY['vwap', 'volume', 'deviation'], true),
('Rolling VWAP Deviation', 'vwap_roll_dev', 'volume', 'VWAP::vwap_rolling', 'Rolling VWAP (20-period)', 'Short-term volume-weighted average', 'More responsive than cumulative VWAP', '[{"name":"window","default":20}]'::jsonb, 'unbounded', 'robust', 'medium', 20, ARRAY['vwap', 'volume'], true);

-- OBV (1)
INSERT INTO factors (name, slug, category, rust_function, formula, description, interpretation, parameters, output_range, normalization, computation_cost, min_bars_required, tags, is_active) VALUES
('OBV % Change', 'obv_pct', 'volume', 'OBV::obv_pct_change', 'OBV_pct = (OBV[t] - OBV[t-1]) / |OBV[t-1]|', 'On-Balance Volume momentum', 'Positive = accumulation, Negative = distribution', '[]'::jsonb, 'unbounded', 'robust', 'low', 2, ARRAY['obv', 'volume', 'momentum'], true);

-- MFI (1)
INSERT INTO factors (name, slug, category, rust_function, formula, description, interpretation, parameters, output_range, normalization, computation_cost, min_bars_required, tags, is_active) VALUES
('MFI', 'mfi', 'volume', 'MFI::mfi_normalized', 'MFI = 100 - (100 / (1 + MFR))', 'Money Flow Index (volume-weighted RSI)', '>80 overbought, <20 oversold', '[{"name":"period","default":14}]'::jsonb, '-1 to 1', 'custom', 'medium', 14, ARRAY['mfi', 'volume', 'rsi'], true);

-- Additional Features (5)
INSERT INTO factors (name, slug, category, rust_function, formula, description, interpretation, parameters, output_range, normalization, computation_cost, min_bars_required, tags, is_active) VALUES
('High-Low Range', 'hl_range', 'volatility', 'hl_range = (high - low) / close', 'Intraday price range', 'Higher = more intraday volatility', '[]'::jsonb, 'unbounded', 'robust', 'low', 1, ARRAY['range', 'volatility'], true),
('Close Position', 'close_pos', 'price_action', 'close_pos = (close - low) / (high - low)', 'Where close is in daily range', '1 = close at high, 0 = close at low', '[]'::jsonb, '0-1', 'none', 'low', 1, ARRAY['position', 'range'], true),
('Volume Trend', 'vol_trend', 'volume', 'vol_trend = (vol - vol_prev) / vol_prev', 'Volume change rate', 'Positive = increasing volume', '[]'::jsonb, 'unbounded', 'robust', 'low', 2, ARRAY['volume', 'trend'], true),
('Momentum 10', 'momentum_10', 'momentum', 'momentum = (close - close[t-10]) / close[t-10]', '10-period rate of change', 'Short-term momentum', '[{"name":"period","default":10}]'::jsonb, 'unbounded', 'robust', 'low', 10, ARRAY['momentum', 'roc'], true),
('Momentum 20', 'momentum_20', 'momentum', 'momentum = (close - close[t-20]) / close[t-20]', '20-period rate of change', 'Medium-term momentum', '[{"name":"period","default":20}]'::jsonb, 'unbounded', 'robust', 'low', 20, ARRAY['momentum', 'roc'], true);
