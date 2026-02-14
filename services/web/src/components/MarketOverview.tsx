"use client";

import React, { useState, useEffect, useRef, useMemo, useCallback } from "react";
import { Search, TrendingUp, TrendingDown, Clock, BarChart2, RefreshCw } from "lucide-react";
import { createChart, CandlestickSeries } from 'lightweight-charts';
import { cn } from "@/lib/utils";

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
    const [exchange, setExchange] = useState("Birdeye");

    const chartRef = useRef<any>(null);
    const candlestickSeriesRef = useRef<any>(null);

    // Callback ref to initialize chart when container is ready
    const chartContainerRef = useCallback((container: HTMLDivElement | null) => {
        console.log('[Chart Init Callback] Container ref callback triggered');
        console.log('[Chart Init Callback] Container exists:', !!container);
        console.log('[Chart Init Callback] Chart already initialized:', !!chartRef.current);

        if (!container) {
            console.log('[Chart Init Callback] Container is null, skipping');
            return;
        }

        if (chartRef.current) {
            console.log('[Chart Init Callback] Chart already exists, skipping re-initialization');
            return;
        }

        console.log('[Chart Init Callback] Initializing chart...');

        try {
            const { width, height } = container.getBoundingClientRect();
            console.log('[Chart Init Callback] Container dimensions:', { width, height });

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

            console.log('[Chart Init Callback] Chart created successfully');

            const candlestickSeries = chart.addSeries(CandlestickSeries);
            console.log('[Chart Init Callback] Candlestick series created');

            candlestickSeries.applyOptions({
                upColor: '#22c55e',
                downColor: '#ef4444',
                borderUpColor: '#22c55e',
                borderDownColor: '#ef4444',
                wickUpColor: '#22c55e',
                wickDownColor: '#ef4444',
            });

            console.log('[Chart Init Callback] Series options applied');

            chartRef.current = chart;
            candlestickSeriesRef.current = candlestickSeries;

            console.log('[Chart Init Callback] Refs stored');

            // Handle resize
            const handleResize = () => {
                console.log('[Chart Resize] Triggered');
                if (container && chartRef.current) {
                    const newWidth = container.clientWidth;
                    const newHeight = container.clientHeight;
                    chartRef.current.applyOptions({
                        width: Math.floor(newWidth),
                        height: Math.floor(newHeight),
                    });
                }
            };

            window.addEventListener('resize', handleResize);
            console.log('[Chart Init Callback] Complete!');

        } catch (error) {
            console.error('[Chart Init Callback] Error creating chart:', error);
        }
    }, []);

    // Fetch Tokens
    useEffect(() => {
        const fetchTokens = async () => {
            try {
                const res = await fetch(`${API_BASE}/api/v1/data/market/tokens`);
                const json = await res.json();
                if (json.tokens) {
                    setTokens(json.tokens);
                    if (!selectedSymbol && json.tokens.length > 0) {
                        setSelectedSymbol(json.tokens[0].symbol);
                    }
                }
            } catch (e) {
                console.error("Failed to fetch tokens", e);
            }
        };
        fetchTokens();
        const interval = setInterval(fetchTokens, 60000);
        return () => clearInterval(interval);
    }, [selectedSymbol]);



    // Fetch Candles
    useEffect(() => {
        console.log('[Candles Effect] Triggered with:', {
            selectedSymbol,
            resolution,
            exchange,
            tokensLength: tokens.length
        });

        if (!selectedSymbol) {
            console.log('[Candles Effect] No symbol selected, returning');
            return;
        }

        const fetchCandles = async () => {
            console.log('[Candles Effect] Starting fetch for:', selectedSymbol, 'resolution:', resolution);
            setLoading(true);
            try {
                const now = Date.now();
                let start = now - 24 * 60 * 60 * 1000;
                if (resolution === "1m") start = now - 6 * 60 * 60 * 1000;
                else if (resolution === "15m") start = now - 3 * 24 * 60 * 60 * 1000;
                else if (resolution === "1h") start = now - 14 * 24 * 60 * 60 * 1000;
                else if (resolution === "4h") start = now - 30 * 24 * 60 * 60 * 1000;
                else if (resolution === "1d") start = now - 365 * 24 * 60 * 60 * 1000;

                const token = tokens.find(t => t.symbol === selectedSymbol);
                if (!token) {
                    console.warn('[Candles Effect] Token not found for symbol:', selectedSymbol);
                    return;
                }

                const url = `${API_BASE}/api/v1/data/market/${token.address}/history?resolution=${resolution}&exchange=${exchange}&start=${start}&end=${now}&limit=5000`;
                console.log('[Candles Effect] Fetching URL:', url);

                const res = await fetch(url);
                const json = await res.json();

                console.log('[Candles Effect] Response:', {
                    dataLength: json.data?.length || 0,
                    resolution,
                    symbol: selectedSymbol
                });

                if (json.data) {
                    setCandles(json.data);
                    setLastUpdated(new Date());
                }
            } catch (e) {
                console.error('[Candles Effect] Fetch error:', e);
                setCandles([]);
            } finally {
                setLoading(false);
            }
        };
        fetchCandles();
        const interval = setInterval(fetchCandles, 60000);
        return () => {
            console.log('[Candles Effect] Cleanup for:', selectedSymbol, resolution);
            clearInterval(interval);
        };
    }, [selectedSymbol, resolution, exchange, tokens]); // CHANGED: Added `tokens` back as it's needed for `token.address`

    // Cleanup effect when component unmounts
    useEffect(() => {
        const handleResize = () => {
            console.log('[Chart Resize] Global handler triggered');
            if (chartRef.current) {
                const container = document.querySelector('[data-chart-container]') as HTMLDivElement;
                if (container) {
                    const newWidth = container.clientWidth;
                    const newHeight = container.clientHeight;
                    chartRef.current.applyOptions({
                        width: Math.floor(newWidth),
                        height: Math.floor(newHeight),
                    });
                }
            }
        };

        // Note: resize listener is now added in callback ref
        // This effect is only for cleanup on unmount

        return () => {
            console.log('[Chart Cleanup] Component unmounting, removing resize listener');
            window.removeEventListener('resize', handleResize);
            if (chartRef.current) {
                console.log('[Chart Cleanup] Removing chart instance');
                chartRef.current.remove();
                chartRef.current = null;
                candlestickSeriesRef.current = null;
            }
        };
    }, []);

    // Derived: Selected Token
    const selectedToken = tokens.find(t => t.symbol === selectedSymbol);

    // Update chart data when candles change
    useEffect(() => {
        console.log('[Chart Data Update] Effect triggered with candles.length:', candles.length);
        console.log('[Chart Data Update] candlestickSeriesRef.current exists:', !!candlestickSeriesRef.current);

        if (!candlestickSeriesRef.current || candles.length === 0) {
            console.warn('[Chart Data Update] Returning early - no series or no candles');
            return;
        }

        const formattedData = candles.map(candle => ({
            time: Math.floor(candle.timestamp / 1000) as any,
            open: candle.open,
            high: candle.high,
            low: candle.low,
            close: candle.close,
        }));

        console.log('[Chart Data Update] Setting data with', formattedData.length, 'candles');
        console.log('[Chart Data Update] First candle:', formattedData[0]);
        console.log('[Chart Data Update] Last candle:', formattedData[formattedData.length - 1]);

        candlestickSeriesRef.current.setData(formattedData);

        // Auto-fit the timeScale to show all data
        if (chartRef.current) {
            console.log('[Chart Data Update] Fitting time scale to content');
            chartRef.current.timeScale().fitContent();
        }

        console.log('[Chart Data Update] Complete!');
    }, [candles]);

    const filteredTokens = useMemo(() => {
        return tokens.filter(t => {
            const matchesSearch = t.symbol.toLowerCase().includes(searchQuery.toLowerCase()) ||
                (t.name && t.name.toLowerCase().includes(searchQuery.toLowerCase()));

            if (!matchesSearch) return false;

            if (exchange === "Polygon") {
                return t.token_type === "stock";
            } else {
                return t.token_type !== "stock"; // Default to crypto for others
            }
        });
    }, [tokens, searchQuery, exchange]);

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
            {/* Sidebar */}
            <div className="w-80 flex flex-col gap-4 bg-slate-950/30 rounded-2xl border border-white/5 p-4 overflow-hidden">
                {/* Exchange Selector */}
                <div className="space-y-2">
                    <label className="text-xs font-semibold text-slate-500 uppercase tracking-wider">Exchange</label>
                    <div className="flex gap-1 bg-slate-900/50 p-1 rounded-lg border border-white/5">
                        {["Birdeye", "Jupiter", "Polygon"].map(ex => (
                            <button
                                key={ex}
                                onClick={() => setExchange(ex)}
                                className={cn(
                                    "flex-1 px-3 py-1.5 rounded-md text-xs font-medium transition-all duration-200 text-center",
                                    exchange === ex ? "bg-indigo-500 text-white shadow" : "text-slate-400 hover:text-white hover:bg-white/5"
                                )}
                            >
                                {ex}
                            </button>
                        ))}
                    </div>
                </div>

                <div className="w-full h-px bg-white/5 my-1"></div>

                {/* Search */}
                <div className="relative">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
                    <input
                        type="text"
                        placeholder="Search tokens..."
                        className="w-full bg-slate-900/50 border border-white/10 rounded-lg pl-9 pr-3 py-2 text-sm text-slate-200 focus:outline-none focus:border-indigo-500/50 transition-colors"
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                    />
                </div>

                {/* Token List */}
                <div className="flex-1 overflow-y-auto space-y-1 pr-1 scrollbar-thin scrollbar-thumb-white/10 scrollbar-track-transparent">
                    {filteredTokens.map(token => (
                        <button
                            key={token.symbol}
                            onClick={() => setSelectedSymbol(token.symbol)}
                            className={cn(
                                "w-full flex items-center justify-between p-3 rounded-xl transition-all duration-200 border border-transparent",
                                selectedSymbol === token.symbol
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
                    ))}
                    {filteredTokens.length === 0 && (
                        <div className="text-center py-8 text-slate-500 text-sm">No tokens found.</div>
                    )}
                </div>
            </div>

            {/* Main Chart Area */}
            <div className="flex-1 flex flex-col gap-6 bg-slate-950/30 rounded-2xl border border-white/5 p-6 overflow-hidden">
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
                                        onClick={() => {
                                            console.log('[Resolution] Changing from', resolution, 'to', res.value);
                                            setResolution(res.value);
                                        }}
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
    );
}
