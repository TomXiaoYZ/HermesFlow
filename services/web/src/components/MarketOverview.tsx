"use client";

import React, { useState, useEffect, useRef, useMemo } from "react";
import { Search, TrendingUp, TrendingDown, Clock, BarChart2, RefreshCw } from "lucide-react";
import { createChart, CandlestickSeries } from 'lightweight-charts';
import type { IChartApi, ISeriesApi, UTCTimestamp } from 'lightweight-charts';
import { cn } from "@/lib/utils";
import { authFetch } from "@/lib/api";

// Types
interface TokenSummary {
    address: string;
    symbol: string;
    name: string | null;
    price: number | null;
    volume_24h: number | null;
    change_24h: number | null;
    token_type?: string;
}

interface Candle {
    timestamp: number;
    open: number;
    high: number;
    low: number;
    close: number;
    volume: number;
}

const RESOLUTIONS = [
    { label: "1m", value: "1m" },
    { label: "15m", value: "15m" },
    { label: "1h", value: "1h" },
    { label: "4h", value: "4h" },
    { label: "1d", value: "1d" },
];

const STOCK_GROUPS = [
    { label: "Magnificent 7", symbols: ["AAPL", "MSFT", "GOOGL", "AMZN", "META", "NVDA", "TSLA"] },
    { label: "Indices", symbols: ["SPY", "QQQ", "DIA", "IWM"] },
    { label: "Commodities & Volatility", symbols: ["GLD", "UVXY"] },
];

// Use gateway URL directly from browser to bypass Next.js dev rewrite proxy issues
const API_BASE = process.env.NEXT_PUBLIC_API_URL || "";

export default function MarketOverview() {
    const [tokens, setTokens] = useState<TokenSummary[]>([]);
    const [selectedSymbol, setSelectedSymbol] = useState<string | null>(null);
    const [searchQuery, setSearchQuery] = useState("");
    const [resolution, setResolution] = useState("1h");
    const [candles, setCandles] = useState<Candle[]>([]);
    const [loading, setLoading] = useState(false);
    const [lastUpdated, setLastUpdated] = useState<Date>(new Date());
    const [exchange, setExchange] = useState("Polygon");
    const [chartReady, setChartReady] = useState(false);

    const chartContainerRef = useRef<HTMLDivElement>(null);
    const chartRef = useRef<IChartApi | null>(null);
    const candlestickSeriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null);

    // Initialize chart when container mounts via ResizeObserver (avoids 0x0 dimensions)
    useEffect(() => {
        const container = chartContainerRef.current;
        if (!container) return;

        let observer: ResizeObserver | null = null;

        const initChart = () => {
            if (chartRef.current) return;

            const { width, height } = container.getBoundingClientRect();
            if (width === 0 || height === 0) return; // Wait for layout

            const chart = createChart(container, {
                width: Math.floor(width),
                height: Math.floor(height),
                layout: {
                    background: { color: 'transparent' },
                    textColor: '#94a3b8',
                },
                grid: {
                    vertLines: { color: 'rgba(255,255,255,0.05)' },
                    horzLines: { color: 'rgba(255,255,255,0.05)' },
                },
                timeScale: {
                    borderColor: 'rgba(255,255,255,0.1)',
                    timeVisible: true,
                },
                rightPriceScale: {
                    borderColor: 'rgba(255,255,255,0.1)',
                },
            });

            const series = chart.addSeries(CandlestickSeries);
            series.applyOptions({
                upColor: '#22c55e',
                downColor: '#ef4444',
                borderUpColor: '#22c55e',
                borderDownColor: '#ef4444',
                wickUpColor: '#22c55e',
                wickDownColor: '#ef4444',
            });

            chartRef.current = chart;
            candlestickSeriesRef.current = series;
            setChartReady(true);
        };

        observer = new ResizeObserver((entries) => {
            for (const entry of entries) {
                const { width, height } = entry.contentRect;
                if (!chartRef.current) {
                    initChart();
                } else {
                    chartRef.current.applyOptions({
                        width: Math.floor(width),
                        height: Math.floor(height),
                    });
                }
            }
        });
        observer.observe(container);

        return () => {
            observer?.disconnect();
            if (chartRef.current) {
                chartRef.current.remove();
                chartRef.current = null;
                candlestickSeriesRef.current = null;
                setChartReady(false);
            }
        };
    }, [selectedSymbol]);

    // Fetch Tokens
    useEffect(() => {
        const fetchTokens = async () => {
            try {
                const res = await authFetch(`${API_BASE}/api/v1/data/market/tokens`);
                const json = await res.json();
                if (json.tokens) {
                    setTokens(json.tokens);
                }
            } catch {
                // Token fetch failed — will retry on next interval
            }
        };
        fetchTokens();
        const interval = setInterval(fetchTokens, 60000);
        return () => clearInterval(interval);
    }, []);

    // Auto-select first matching token when exchange changes or tokens load
    useEffect(() => {
        if (tokens.length === 0) return;
        const isStock = exchange === "Polygon";
        const matching = tokens.filter(t =>
            isStock ? t.token_type === "stock" : t.token_type !== "stock"
        );
        if (matching.length > 0 && !matching.find(t => t.symbol === selectedSymbol)) {
            setSelectedSymbol(matching[0].symbol);
        }
    }, [exchange, tokens, selectedSymbol]);



    // Resolution-aware time windows and limits
    const RESOLUTION_CONFIG: Record<string, { windowMs: number; limit: number }> = {
        "1m":  { windowMs: 6 * 60 * 60 * 1000, limit: 360 },
        "15m": { windowMs: 3 * 24 * 60 * 60 * 1000, limit: 288 },
        "1h":  { windowMs: 14 * 24 * 60 * 60 * 1000, limit: 336 },
        "4h":  { windowMs: 30 * 24 * 60 * 60 * 1000, limit: 180 },
        "1d":  { windowMs: 365 * 24 * 60 * 60 * 1000, limit: 365 },
    };

    // Simple in-memory candle cache to avoid re-fetching on tab switches
    const candleCacheRef = useRef<Map<string, { data: Candle[]; ts: number }>>(new Map());

    // Fetch Candles
    useEffect(() => {
        if (!selectedSymbol) return;

        const fetchCandles = async () => {
            const token = tokens.find(t => t.symbol === selectedSymbol);
            if (!token) return;

            const cacheKey = `${token.address}:${resolution}:${exchange}`;
            const cached = candleCacheRef.current.get(cacheKey);
            const CACHE_TTL = resolution === "1m" ? 30000 : 60000;
            if (cached && Date.now() - cached.ts < CACHE_TTL) {
                setCandles(cached.data);
                return;
            }

            setLoading(true);
            try {
                const now = Date.now();
                const config = RESOLUTION_CONFIG[resolution] || { windowMs: 24 * 60 * 60 * 1000, limit: 500 };
                const start = now - config.windowMs;

                const url = `${API_BASE}/api/v1/data/market/${token.address}/history?resolution=${resolution}&exchange=${exchange}&start=${start}&end=${now}&limit=${config.limit}`;
                const res = await authFetch(url);
                const json = await res.json();

                if (json.data) {
                    setCandles(json.data);
                    setLastUpdated(new Date());
                    candleCacheRef.current.set(cacheKey, { data: json.data, ts: Date.now() });
                }
            } catch {
                setCandles([]);
            } finally {
                setLoading(false);
            }
        };
        fetchCandles();
        const interval = setInterval(fetchCandles, 60000);
        return () => clearInterval(interval);
    }, [selectedSymbol, resolution, exchange, tokens]);

    // Derived: Selected Token
    const selectedToken = tokens.find(t => t.symbol === selectedSymbol);

    // Update chart data when candles change
    useEffect(() => {
        if (!candlestickSeriesRef.current || candles.length === 0) return;

        const formattedData = candles.map(candle => ({
            time: Math.floor(candle.timestamp / 1000) as UTCTimestamp,
            open: candle.open,
            high: candle.high,
            low: candle.low,
            close: candle.close,
        }));

        candlestickSeriesRef.current.setData(formattedData);
        chartRef.current?.timeScale().fitContent();
    }, [candles, chartReady]);

    const filteredTokens = useMemo(() => {
        return tokens.filter(t => {
            const matchesSearch = t.symbol.toLowerCase().includes(searchQuery.toLowerCase()) ||
                (t.name && t.name.toLowerCase().includes(searchQuery.toLowerCase()));

            if (!matchesSearch) return false;

            if (exchange === "Polygon") {
                return t.token_type === "stock";
            } else {
                return t.token_type !== "stock";
            }
        });
    }, [tokens, searchQuery, exchange]);

    // Group tokens by category for Polygon exchange
    const groupedTokens = useMemo(() => {
        if (exchange !== "Polygon") return null;

        const tokenMap = new Map(filteredTokens.map(t => [t.symbol, t]));
        const groups: { label: string; tokens: TokenSummary[] }[] = [];

        for (const group of STOCK_GROUPS) {
            const matched = group.symbols
                .map(s => tokenMap.get(s))
                .filter((t): t is TokenSummary => t !== undefined);
            if (matched.length > 0) {
                groups.push({ label: group.label, tokens: matched });
            }
        }

        // Ungrouped tokens (not in any predefined group)
        const allGrouped = new Set(STOCK_GROUPS.flatMap(g => g.symbols));
        const ungrouped = filteredTokens.filter(t => !allGrouped.has(t.symbol));
        if (ungrouped.length > 0) {
            groups.push({ label: "Other", tokens: ungrouped });
        }

        return groups;
    }, [filteredTokens, exchange]);

    const formatPrice = (val: number | null) => {
        if (val === null) return "-";
        return val < 1 ? val.toFixed(6) : val.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
    };

    const formatVolume = (val: number | null) => {
        if (val === null) return "-";
        if (val >= 1_000_000) return `$${(val / 1_000_000).toFixed(2)}M`;
        if (val >= 1_000) return `$${(val / 1_000).toFixed(2)}K`;
        return `$${val.toFixed(2)}`;
    };

    return (
        <div className="flex h-[calc(100vh-8rem)] gap-6 text-white">
            {/* Full Width Layout */}
            <div className="flex-1 flex flex-col gap-4 overflow-hidden">
                {/* Heatmap Grid: All 13 symbols at a glance */}
                <div className="shrink-0 bg-slate-950/30 rounded-2xl border border-white/5 p-4">
                    <div className="flex items-center justify-between mb-3">
                        <h3 className="text-xs font-bold text-slate-400 uppercase tracking-widest">Market Overview</h3>
                        <span className="text-[10px] text-slate-600">
                            Updated {lastUpdated.toLocaleTimeString()}
                        </span>
                    </div>
                    <div className="space-y-3">
                        {(groupedTokens || []).map(group => (
                            <div key={group.label}>
                                <div className="text-[9px] text-slate-600 uppercase tracking-widest mb-1.5 font-semibold">{group.label}</div>
                                <div className="flex gap-1.5 flex-wrap">
                                    {group.tokens.map(token => {
                                        const change = token.change_24h || 0;
                                        const isUp = change >= 0;
                                        const intensity = Math.min(Math.abs(change) / 3, 1);
                                        const isActive = selectedSymbol === token.symbol;
                                        return (
                                            <button
                                                key={token.symbol}
                                                onClick={() => setSelectedSymbol(token.symbol)}
                                                className={cn(
                                                    "flex flex-col items-center px-3 py-2 rounded-lg border transition-all cursor-pointer min-w-[80px]",
                                                    isActive
                                                        ? "border-indigo-500/50 bg-indigo-500/10 shadow-lg shadow-indigo-500/5"
                                                        : "border-white/5 hover:border-white/10 hover:bg-white/[0.03]",
                                                )}
                                                style={{
                                                    backgroundColor: isActive
                                                        ? undefined
                                                        : isUp
                                                            ? `rgba(34, 197, 94, ${intensity * 0.08})`
                                                            : `rgba(239, 68, 68, ${intensity * 0.08})`,
                                                }}
                                            >
                                                <span className="text-[11px] font-bold text-slate-200">{token.symbol}</span>
                                                <span className="text-[10px] font-mono text-slate-400 mt-0.5">${formatPrice(token.price)}</span>
                                                <span className={cn(
                                                    "text-[10px] font-mono font-bold mt-0.5",
                                                    isUp ? "text-emerald-400" : "text-red-400"
                                                )}>
                                                    {isUp ? "+" : ""}{change.toFixed(2)}%
                                                </span>
                                            </button>
                                        );
                                    })}
                                </div>
                            </div>
                        ))}
                    </div>
                </div>

                {/* Chart Area */}
                <div className="flex-1 flex flex-col bg-slate-950/30 rounded-2xl border border-white/5 p-6 overflow-hidden min-h-0">
                {selectedToken ? (
                    <>
                        {/* Header */}
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-4">
                                <div className="w-12 h-12 rounded-xl bg-indigo-500/20 flex items-center justify-center border border-indigo-500/30">
                                    <span className="font-bold text-lg text-indigo-300">{selectedToken.symbol[0]}</span>
                                </div>
                                <div>
                                    <h2 className="text-2xl font-bold flex items-center gap-2">
                                        {selectedToken.symbol}
                                        <span className="text-sm font-normal text-slate-500 px-2 py-0.5 rounded-md bg-white/5">{selectedToken.name}</span>
                                    </h2>
                                    <div className="flex items-center gap-4 text-sm mt-1">
                                        <span className="font-mono text-xl text-white font-semibold">${formatPrice(selectedToken.price)}</span>
                                        <span className={cn("px-2 py-0.5 rounded text-xs font-semibold", (selectedToken.change_24h || 0) >= 0 ? "bg-emerald-500/10 text-emerald-400" : "bg-red-500/10 text-red-400")}>
                                            {(selectedToken.change_24h || 0) >= 0 ? "+" : ""}{(selectedToken.change_24h || 0).toFixed(2)}%
                                        </span>
                                        <span className="text-slate-500 flex items-center gap-1">
                                            <BarChart2 className="w-3 h-3" />
                                            Vol: {selectedToken.volume_24h ? formatVolume(selectedToken.volume_24h) : "N/A"}
                                        </span>
                                    </div>
                                </div>
                            </div>

                            {/* Resolution Selector */}
                            <div className="flex items-center gap-2 bg-slate-900/50 p-1 rounded-lg border border-white/5">
                                {RESOLUTIONS.map(res => (
                                    <button
                                        key={res.value}
                                        onClick={() => setResolution(res.value)}
                                        className={cn(
                                            "px-3 py-1.5 rounded-md text-xs font-medium transition-all duration-200",
                                            resolution === res.value ? "bg-indigo-500 text-white shadow" : "text-slate-400 hover:text-white hover:bg-white/5"
                                        )}
                                    >
                                        {res.label}
                                    </button>
                                ))}
                                <div className="w-px h-6 bg-white/10 mx-1"></div>
                                <button
                                    className="p-1.5 rounded-md text-slate-400 hover:text-white hover:bg-white/5 transition-colors"
                                    onClick={() => setLastUpdated(new Date())}
                                >
                                    <RefreshCw className={cn("w-3.5 h-3.5", loading && "animate-spin")} />
                                </button>
                            </div>
                        </div>

                        {/* Candlestick Chart */}
                        <div className="flex-1 relative group">
                            {/* Always render container for chart ref */}
                            <div
                                ref={chartContainerRef}
                                className="w-full h-full"
                            />
                            {/* Overlay when no data */}
                            {candles.length === 0 && (
                                <div className="absolute inset-0 flex items-center justify-center text-slate-500 flex-col gap-2 bg-slate-950/30 backdrop-blur-sm">
                                    <Clock className="w-8 h-8 opacity-50" />
                                    <span>{loading ? "Loading data..." : "No data available"}</span>
                                    <span className="text-xs text-slate-600">Try switching timeframe or exchange</span>
                                </div>
                            )}
                        </div>
                    </>
                ) : (
                    <div className="h-full flex flex-col items-center justify-center text-slate-500 gap-4">
                        <TrendingUp className="w-12 h-12 text-slate-700" />
                        <p>Select a token to view market data</p>
                    </div>
                )}
                </div>
            </div>
        </div>
    );
}

function TokenButton({ token, isSelected, onSelect, formatPrice }: {
    token: TokenSummary;
    isSelected: boolean;
    onSelect: (symbol: string) => void;
    formatPrice: (val: number | null) => string;
}) {
    return (
        <button
            onClick={() => onSelect(token.symbol)}
            className={cn(
                "w-full flex items-center justify-between p-3 rounded-xl transition-all duration-200 border border-transparent",
                isSelected
                    ? "bg-indigo-500/10 border-indigo-500/30 shadow-lg shadow-indigo-500/10"
                    : "hover:bg-white/5 hover:border-white/5"
            )}
        >
            <div className="text-left">
                <div className="font-bold text-sm text-slate-200">{token.symbol}</div>
                <div className="text-xs text-slate-500 truncate max-w-[100px]">{token.name || "Unknown"}</div>
            </div>
            <div className="text-right">
                <div className="font-mono text-sm text-slate-300">${formatPrice(token.price)}</div>
                <div className={cn("text-xs flex items-center justify-end gap-1", (token.change_24h || 0) >= 0 ? "text-emerald-400" : "text-red-400")}>
                    {(token.change_24h || 0) >= 0 ? <TrendingUp className="w-3 h-3" /> : <TrendingDown className="w-3 h-3" />}
                    {Math.abs(token.change_24h || 0).toFixed(2)}%
                </div>
            </div>
        </button>
    );
}
