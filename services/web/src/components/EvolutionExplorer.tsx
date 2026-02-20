"use client";

import React, { useState, useEffect, useCallback } from "react";
import { ChevronDown, ChevronRight, Filter, RefreshCw, Dna, TrendingUp, TrendingDown, Minus } from "lucide-react";
import {
    ComposedChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    ResponsiveContainer,
    AreaChart,
    Area,
    Scatter,
    BarChart,
    Bar,
    Cell,
    ReferenceLine,
} from "recharts";
import {
    decodeGenome,
    getFeatureImportance,
    loadFactorConfigForExchange,
} from "@/utils/genome";

interface Exchange {
    key: string;
    exchange: string;
    resolution: string;
    factor_count: number;
}

type StrategyMode = "long_only" | "long_short";

interface SymbolOverview {
    symbol: string;
    mode?: string;
    latest_gen: number;
    best_fitness: number | null;
    best_oos_psr: number | null;
    best_pnl: number | null;
    sharpe_ratio: number | null;
    max_drawdown: number | null;
    win_rate: number | null;
    stagnation: number | null;
    fold_psrs: number[] | null;
    last_updated: string | null;
}

interface Trade {
    entry: number;
    exit: number;
    bars: number;
    pnl: number;
    direction?: "long" | "short";
}

interface BacktestMetrics {
    total_return?: number;
    final_equity?: number;
    sortino_ratio?: number;
    calmar_ratio?: number;
    profit_factor?: number;
    avg_win?: number;
    avg_loss?: number;
    max_win?: number;
    max_loss?: number;
    max_consecutive_wins?: number;
    max_consecutive_losses?: number;
    avg_holding_bars?: number;
    avg_trade_return?: number;
    trade_return_std?: number;
}

interface BacktestData {
    pnl_percent: number;
    sharpe_ratio: number;
    max_drawdown: number;
    win_rate: number;
    total_trades: number;
    equity_curve?: { timestamp: number; value: number }[];
    trades?: Trade[];
    metrics?: BacktestMetrics;
}

interface Generation {
    generation: number;
    fitness: number | null;
    best_genome: number[] | null;
    strategy_id: string | null;
    timestamp: string | null;
    oos_psr: number | null;
    stagnation?: number;
    fold_psrs?: number[];
    backtest: BacktestData | null;
}

const SYMBOL_NAMES: Record<string, string> = {
    AAPL: "Apple Inc",
    MSFT: "Microsoft Corp",
    GOOGL: "Alphabet Inc",
    AMZN: "Amazon.com Inc",
    META: "Meta Platforms",
    NVDA: "NVIDIA Corp",
    TSLA: "Tesla Inc",
    SPY: "S&P 500 ETF",
    QQQ: "Nasdaq 100 ETF",
    DIA: "Dow Jones ETF",
    IWM: "Russell 2000 ETF",
    UVXY: "Ultra VIX Short-Term Futures ETF",
    GLD: "Gold Shares",
};

function formatTimeAgo(timestamp: string | null): string {
    if (!timestamp) return "—";
    const diff = Date.now() - new Date(timestamp).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return "now";
    if (mins < 60) return `${mins}m`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h`;
    return `${Math.floor(hours / 24)}d`;
}

function fmtPct(value: number | null | undefined): string {
    if (value == null) return "—";
    return `${value >= 0 ? "+" : ""}${(value * 100).toFixed(1)}%`;
}

function fmtNum(value: number | null | undefined, decimals = 4): string {
    if (value == null) return "—";
    return value.toFixed(decimals);
}

export default function EvolutionExplorer() {
    const [exchanges, setExchanges] = useState<Exchange[]>([]);
    const [activeExchange, setActiveExchange] = useState<string>("");
    const [activeMode, setActiveMode] = useState<StrategyMode>("long_only");
    const [overview, setOverview] = useState<SymbolOverview[]>([]);
    const [selectedSymbol, setSelectedSymbol] = useState<string | null>(null);
    const [generations, setGenerations] = useState<Generation[]>([]);
    const [expandedGen, setExpandedGen] = useState<number | null>(null);
    const [expandedDetail, setExpandedDetail] = useState<BacktestData | null>(null);
    const [loading, setLoading] = useState(true);
    const [detailLoading, setDetailLoading] = useState(false);
    const [latestDetail, setLatestDetail] = useState<{ gen: Generation; bt: BacktestData } | null>(null);

    // Load exchanges
    useEffect(() => {
        fetch("/api/v1/evolution/exchanges")
            .then((res) => res.json())
            .then((data) => {
                const exs: Exchange[] = data.exchanges || [];
                setExchanges(exs);
                if (exs.length > 0) setActiveExchange(exs[0].key);
            })
            .catch(() => setExchanges([]));
    }, []);

    // Fetch overview for the active exchange and mode
    const fetchOverview = useCallback(async () => {
        if (!activeExchange) return;
        try {
            setLoading(true);
            const res = await fetch(`/api/v1/evolution/${activeExchange}/overview?mode=${activeMode}`);
            const data = await res.json();
            const symbols: SymbolOverview[] = data.symbols || [];
            setOverview(symbols);
            if (symbols.length > 0 && !selectedSymbol) {
                setSelectedSymbol(symbols[0].symbol);
            }
        } catch {
            setOverview([]);
        } finally {
            setLoading(false);
        }
    }, [activeExchange, activeMode, selectedSymbol]);

    useEffect(() => {
        fetchOverview();
        const interval = setInterval(fetchOverview, 30000);
        return () => clearInterval(interval);
    }, [fetchOverview]);

    // Fetch per-symbol generations when selected symbol or mode changes
    const fetchSymbolGenerations = useCallback(async () => {
        if (!activeExchange || !selectedSymbol) return;
        try {
            setDetailLoading(true);
            await loadFactorConfigForExchange(activeExchange);
            const res = await fetch(
                `/api/v1/evolution/${activeExchange}/${selectedSymbol}/generations?limit=200&mode=${activeMode}`
            );
            const data = await res.json();
            setGenerations(data.generations || []);
        } catch {
            setGenerations([]);
        } finally {
            setDetailLoading(false);
        }
    }, [activeExchange, selectedSymbol, activeMode]);

    useEffect(() => {
        fetchSymbolGenerations();
        const interval = setInterval(fetchSymbolGenerations, 15000);
        return () => clearInterval(interval);
    }, [fetchSymbolGenerations]);

    // Auto-fetch latest backtest detail when generations change
    useEffect(() => {
        if (!activeExchange || !selectedSymbol || generations.length === 0) {
            setLatestDetail(null);
            return;
        }
        const latestBt = generations.find((g) => g.backtest != null);
        if (!latestBt) {
            setLatestDetail(null);
            return;
        }
        (async () => {
            try {
                const res = await fetch(
                    `/api/v1/evolution/${activeExchange}/${selectedSymbol}/generations/${latestBt.generation}?mode=${activeMode}`
                );
                const data = await res.json();
                if (data.backtest) {
                    const bt = data.backtest;
                    if (bt.equity_curve && Array.isArray(bt.equity_curve)) {
                        bt.equity_curve = bt.equity_curve.map(
                            (pt: { t?: number; i?: number; equity?: number }) => ({
                                timestamp: (pt.t || pt.i || 0) * 1000,
                                value: pt.equity ?? 0,
                            })
                        );
                    }
                    setLatestDetail({ gen: { ...latestBt, ...data }, bt });
                }
            } catch {
                /* latest detail fetch failed */
            }
        })();
    }, [activeExchange, selectedSymbol, activeMode, generations]);

    const handleExpandRow = async (gen: number) => {
        if (expandedGen === gen) {
            setExpandedGen(null);
            setExpandedDetail(null);
            return;
        }
        setExpandedGen(gen);
        setExpandedDetail(null);
        try {
            const res = await fetch(
                `/api/v1/evolution/${activeExchange}/${selectedSymbol}/generations/${gen}?mode=${activeMode}`
            );
            const data = await res.json();
            if (data.backtest) {
                const bt = data.backtest;
                if (bt.equity_curve && Array.isArray(bt.equity_curve)) {
                    bt.equity_curve = bt.equity_curve.map(
                        (pt: { t?: number; equity?: number }) => ({
                            timestamp: (pt.t || 0) * 1000,
                            value: pt.equity ?? 0,
                        })
                    );
                }
                setExpandedDetail(bt);
            }
        } catch {
            /* detail fetch failed */
        }
    };

    // Derived
    const chartData = [...generations].reverse().map((g) => ({
        gen: g.generation,
        fitness: g.fitness,
        oos_psr: g.oos_psr,
        hasBacktest: g.backtest != null,
    }));

    const bestGen = generations.reduce<Generation | null>((best, g) => {
        if (g.fitness == null) return best;
        if (!best || best.fitness == null || g.fitness > best.fitness) return g;
        return best;
    }, null);

    const backtestGens = generations.filter((g) => g.backtest != null);
    const latestGen = generations[0];
    const selectedOverview = overview.find((s) => s.symbol === selectedSymbol);

    return (
        <div className="flex flex-col h-full bg-[#030305]">
            {/* Header */}
            <div className="flex items-center justify-between px-5 py-3 border-b border-white/5 bg-slate-950/80 backdrop-blur-md shrink-0">
                <div className="flex items-center gap-3">
                    <Dna className="w-4 h-4 text-indigo-400" />
                    {exchanges.map((ex) => (
                        <button
                            key={ex.key}
                            onClick={() => {
                                setActiveExchange(ex.key);
                                setSelectedSymbol(null);
                                setOverview([]);
                            }}
                            className={`px-3 py-1 text-xs font-medium rounded-md transition-all cursor-pointer ${
                                activeExchange === ex.key
                                    ? "bg-indigo-500/15 border border-indigo-500/40 text-indigo-300"
                                    : "bg-white/5 border border-white/5 text-slate-400 hover:bg-white/10 hover:text-slate-200"
                            }`}
                        >
                            {ex.exchange}
                        </button>
                    ))}
                    <span className="w-px h-4 bg-white/10 mx-1" />
                    {(["long_only", "long_short"] as StrategyMode[]).map((m) => (
                        <button
                            key={m}
                            onClick={() => {
                                setActiveMode(m);
                                setGenerations([]);
                                setLatestDetail(null);
                            }}
                            className={`px-2.5 py-1 text-[10px] font-medium rounded-md transition-all cursor-pointer ${
                                activeMode === m
                                    ? "bg-violet-500/15 border border-violet-500/40 text-violet-300"
                                    : "bg-white/5 border border-white/5 text-slate-400 hover:bg-white/10 hover:text-slate-200"
                            }`}
                        >
                            {m === "long_only" ? "Long Only" : "Long/Short"}
                        </button>
                    ))}
                    <span className="text-[10px] text-slate-600 font-mono ml-1">
                        {latestGen ? `Gen #${latestGen.generation}` : "—"}
                    </span>
                </div>
                <div className="flex items-center gap-3">
                    <button
                        onClick={() => { fetchOverview(); fetchSymbolGenerations(); }}
                        className="p-1.5 rounded-md text-slate-500 hover:text-slate-200 hover:bg-white/5 transition-colors cursor-pointer"
                    >
                        <RefreshCw className={`w-3.5 h-3.5 ${loading ? "animate-spin" : ""}`} />
                    </button>
                </div>
            </div>

            {loading && overview.length === 0 ? (
                <div className="flex-1 flex items-center justify-center">
                    <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-indigo-500" />
                </div>
            ) : (
                <div className="flex-1 min-h-0 flex">
                    {/* Left Panel: Symbol Overview Grid */}
                    <div className="w-[340px] shrink-0 border-r border-white/5 overflow-y-auto custom-scrollbar">
                        {/* Summary bar */}
                        <div className="px-3 py-2.5 border-b border-white/5 bg-black/20">
                            <div className="flex items-center justify-between text-[10px]">
                                <span className="text-slate-500">{overview.length} symbols</span>
                                <div className="flex items-center gap-3">
                                    <span className="flex items-center gap-1 text-emerald-500/70">
                                        <span className="w-1.5 h-1.5 rounded-full bg-emerald-500" />
                                        {overview.filter(s => getStatus(s, overview) === "improving").length}
                                    </span>
                                    <span className="flex items-center gap-1 text-amber-500/70">
                                        <span className="w-1.5 h-1.5 rounded-full bg-amber-500" />
                                        {overview.filter(s => getStatus(s, overview) === "plateau").length}
                                    </span>
                                    <span className="flex items-center gap-1 text-red-500/70">
                                        <span className="w-1.5 h-1.5 rounded-full bg-red-500" />
                                        {overview.filter(s => getStatus(s, overview) === "stagnant").length}
                                    </span>
                                </div>
                            </div>
                        </div>
                        <div className="p-2 space-y-1">
                            {overview.map((sym) => {
                                const isSelected = selectedSymbol === sym.symbol;
                                const status = getStatus(sym, overview);
                                const statusBorderColor = status === "improving" ? "border-l-emerald-500/60" : status === "plateau" ? "border-l-amber-500/40" : "border-l-red-500/40";
                                return (
                                    <button
                                        key={sym.symbol}
                                        onClick={() => setSelectedSymbol(sym.symbol)}
                                        className={`w-full text-left px-3 py-2 rounded-lg border-l-2 transition-all cursor-pointer ${statusBorderColor} ${
                                            isSelected
                                                ? "bg-indigo-500/8 border border-r border-t border-b border-indigo-500/30"
                                                : "bg-transparent hover:bg-white/[0.03] border border-r-transparent border-t-transparent border-b-transparent"
                                        }`}
                                    >
                                        <div className="flex items-center justify-between">
                                            <div className="flex items-center gap-2">
                                                <span className="text-[11px] font-bold text-slate-200 w-10">
                                                    {sym.symbol}
                                                </span>
                                                <span className="text-[9px] text-slate-600 truncate max-w-[80px]">
                                                    {SYMBOL_NAMES[sym.symbol] || ""}
                                                </span>
                                            </div>
                                            <div className="flex items-center gap-1.5">
                                                <StatusIcon status={status} />
                                                <span className="text-[9px] text-slate-700 font-mono">#{sym.latest_gen}</span>
                                            </div>
                                        </div>
                                        <div className="flex items-center gap-3 mt-1.5">
                                            <div className="flex items-center gap-1">
                                                <span className="text-[8px] text-slate-600 uppercase">IS</span>
                                                <span className={`text-[10px] font-mono font-bold ${
                                                    (sym.best_fitness ?? 0) > 0.05 ? "text-emerald-400" : (sym.best_fitness ?? 0) > 0 ? "text-emerald-400/60" : "text-slate-500"
                                                }`}>
                                                    {fmtNum(sym.best_fitness)}
                                                </span>
                                            </div>
                                            <div className="flex items-center gap-1">
                                                <span className="text-[8px] text-slate-600 uppercase">OOS</span>
                                                <span className={`text-[10px] font-mono ${
                                                    (sym.best_oos_psr ?? 0) > 0 ? "text-cyan-400/80" : "text-slate-600"
                                                }`}>
                                                    {fmtNum(sym.best_oos_psr)}
                                                </span>
                                            </div>
                                            <div className="flex items-center gap-1 ml-auto">
                                                <span className="text-[8px] text-slate-600 uppercase">Stag</span>
                                                <span className={`text-[10px] font-mono ${
                                                    (sym.stagnation ?? 0) > 200 ? "text-red-400/70" : (sym.stagnation ?? 0) > 50 ? "text-amber-400/70" : "text-slate-500"
                                                }`}>
                                                    {sym.stagnation ?? "—"}
                                                </span>
                                            </div>
                                        </div>
                                        <div className="flex items-center gap-3 mt-1">
                                            <div className="flex items-center gap-1">
                                                <span className="text-[8px] text-slate-600 uppercase">BT</span>
                                                <span className={`text-[10px] font-mono font-bold ${
                                                    sym.best_pnl != null ? (sym.best_pnl >= 0 ? "text-emerald-400" : "text-red-400") : "text-slate-600"
                                                }`}>
                                                    {sym.best_pnl != null ? fmtPct(sym.best_pnl) : "—"}
                                                </span>
                                            </div>
                                            <div className="flex items-center gap-1">
                                                <span className="text-[8px] text-slate-600 uppercase">Sharpe</span>
                                                <span className={`text-[10px] font-mono ${
                                                    sym.sharpe_ratio != null ? (sym.sharpe_ratio >= 0 ? "text-emerald-400/70" : "text-red-400/70") : "text-slate-600"
                                                }`}>
                                                    {fmtNum(sym.sharpe_ratio, 2)}
                                                </span>
                                            </div>
                                            <div className="flex items-center gap-1 ml-auto">
                                                <span className="text-[8px] text-slate-600 uppercase">WR</span>
                                                <span className={`text-[10px] font-mono ${
                                                    sym.win_rate != null ? (sym.win_rate > 0.5 ? "text-emerald-400/70" : "text-red-400/70") : "text-slate-600"
                                                }`}>
                                                    {sym.win_rate != null ? `${(sym.win_rate * 100).toFixed(0)}%` : "—"}
                                                </span>
                                            </div>
                                        </div>
                                    </button>
                                );
                            })}
                        </div>
                        {overview.length === 0 && (
                            <div className="text-center py-12 text-xs text-slate-600">
                                No symbols evolving yet.
                            </div>
                        )}
                    </div>

                    {/* Right Panel: Selected Symbol Detail */}
                    <div className="flex-1 min-h-0 overflow-y-auto custom-scrollbar">
                        {selectedSymbol && selectedOverview ? (
                            <>
                                {/* Symbol header */}
                                <div className="flex items-center justify-between px-5 py-3 border-b border-white/5 bg-slate-950/50">
                                    <div className="flex items-center gap-3">
                                        <span className="text-sm font-bold text-white">
                                            {selectedSymbol}
                                        </span>
                                        <span className="text-xs text-slate-500">
                                            {SYMBOL_NAMES[selectedSymbol] || ""}
                                        </span>
                                        <StatusBadge status={getStatus(selectedOverview, overview)} />
                                        <span className="text-[10px] text-slate-600 font-mono">
                                            Gen #{selectedOverview.latest_gen}
                                        </span>
                                    </div>
                                    {detailLoading && (
                                        <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-indigo-500" />
                                    )}
                                </div>

                                {/* Metric summary */}
                                <div className="grid grid-cols-8 gap-2 px-5 py-3 border-b border-white/5 bg-slate-950/30">
                                    <MetricCell label="IS PSR" value={fmtNum(selectedOverview.best_fitness)} positive={(selectedOverview.best_fitness ?? 0) > 0} />
                                    <MetricCell label="OOS PSR" value={fmtNum(selectedOverview.best_oos_psr)} positive={(selectedOverview.best_oos_psr ?? 0) > 0} />
                                    <MetricCell label="Backtest PnL" value={selectedOverview.best_pnl != null ? fmtPct(selectedOverview.best_pnl) : "—"} positive={(selectedOverview.best_pnl ?? 0) >= 0} />
                                    <MetricCell label="Sharpe" value={fmtNum(selectedOverview.sharpe_ratio, 2)} positive={(selectedOverview.sharpe_ratio ?? 0) >= 0} />
                                    <MetricCell label="Max DD" value={selectedOverview.max_drawdown != null ? fmtPct(selectedOverview.max_drawdown) : "—"} positive={false} />
                                    <MetricCell label="Win Rate" value={selectedOverview.win_rate != null ? fmtPct(selectedOverview.win_rate) : "—"} positive={(selectedOverview.win_rate ?? 0) > 0.5} />
                                    <MetricCell label="Trades" value={latestDetail?.bt?.total_trades != null ? String(latestDetail.bt.total_trades) : "—"} />
                                    <MetricCell label="Stagnation" value={selectedOverview.stagnation != null ? String(selectedOverview.stagnation) : "—"} positive={(selectedOverview.stagnation ?? 0) < 50} />
                                </div>

                                {/* Fold PSR Mini Bars */}
                                {selectedOverview.fold_psrs && selectedOverview.fold_psrs.length > 0 && (
                                    <div className="flex items-center gap-2 px-5 py-2 border-b border-white/5 bg-slate-950/20">
                                        <span className="text-[9px] text-slate-600 uppercase tracking-wider font-bold shrink-0">
                                            Folds (K={selectedOverview.fold_psrs.length})
                                        </span>
                                        <div className="flex items-end gap-1 h-10 flex-1">
                                            {selectedOverview.fold_psrs.map((pnl, i) => {
                                                const maxAbs = Math.max(...selectedOverview.fold_psrs!.map(Math.abs), 0.01);
                                                const height = Math.abs(pnl) / maxAbs * 100;
                                                return (
                                                    <div key={i} className="flex-1 flex flex-col items-center justify-end h-full">
                                                        <div
                                                            className={`w-full rounded-sm ${pnl >= 0 ? "bg-emerald-500/60" : "bg-red-500/60"}`}
                                                            style={{ height: `${Math.max(height, 4)}%` }}
                                                            title={`Fold ${i + 1}: ${pnl.toFixed(4)}`}
                                                        />
                                                        <span className="text-[7px] text-slate-600 mt-0.5 font-mono">{pnl.toFixed(2)}</span>
                                                    </div>
                                                );
                                            })}
                                        </div>
                                    </div>
                                )}

                                {/* Fitness Chart */}
                                {chartData.length > 0 && (
                                    <div className="px-5 pt-4 pb-2">
                                        <div className="bg-slate-900/30 border border-white/5 rounded-xl backdrop-blur-sm">
                                            <div className="flex items-center justify-between px-4 pt-3 pb-1">
                                                <h4 className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                                                    Fitness Trend — {selectedSymbol}
                                                </h4>
                                                <div className="flex items-center gap-4 text-[10px] text-slate-600">
                                                    <span className="flex items-center gap-1.5">
                                                        <span className="w-4 h-[2px] bg-indigo-400 rounded-full inline-block" />
                                                        IS PnL
                                                    </span>
                                                    <span className="flex items-center gap-1.5">
                                                        <span className="w-4 h-[2px] bg-cyan-400 rounded-full inline-block opacity-60" style={{ borderBottom: '1px dashed' }} />
                                                        OOS PnL
                                                    </span>
                                                    <span className="flex items-center gap-1.5">
                                                        <span className="w-2 h-2 bg-indigo-400 rounded-full inline-block" />
                                                        Backtested
                                                    </span>
                                                </div>
                                            </div>
                                            <div className="h-56 px-2 pb-3">
                                                <ResponsiveContainer width="100%" height="100%">
                                                    <ComposedChart data={chartData}>
                                                        <CartesianGrid strokeDasharray="3 3" stroke="#334155" strokeOpacity={0.3} />
                                                        <XAxis dataKey="gen" stroke="#475569" fontSize={10} tickLine={false} axisLine={false} />
                                                        <YAxis stroke="#475569" fontSize={11} tickLine={false} axisLine={false} tickFormatter={(v) => v.toFixed(3)} width={50} />
                                                        <Tooltip
                                                            contentStyle={{ backgroundColor: "rgba(2, 6, 23, 0.95)", border: "1px solid rgba(255,255,255,0.1)", borderRadius: "8px", fontSize: 11 }}
                                                            labelStyle={{ color: "#94a3b8", fontWeight: "bold" }}
                                                            formatter={(value: number | string | undefined, name: string | undefined) => [typeof value === "number" ? value.toFixed(6) : "—", name === "fitness" ? "IS PSR" : "OOS PSR"]}
                                                            labelFormatter={(l) => `Generation #${l}`}
                                                        />
                                                        <Line type="monotone" dataKey="fitness" stroke="#818cf8" strokeWidth={1.5} dot={false} name="fitness" />
                                                        <Line type="monotone" dataKey="oos_psr" stroke="#22d3ee" strokeWidth={1} strokeDasharray="4 3" dot={false} name="oos_psr" connectNulls />
                                                        <Scatter dataKey="fitness" data={chartData.filter((d) => d.hasBacktest)} fill="#818cf8" shape="circle" r={3.5} />
                                                    </ComposedChart>
                                                </ResponsiveContainer>
                                            </div>
                                        </div>
                                    </div>
                                )}

                                {/* Best formula */}
                                {bestGen?.best_genome && (
                                    <div className="px-5 py-2">
                                        <div className="bg-slate-900/30 border border-white/5 rounded-lg p-3">
                                            <h5 className="text-[9px] text-slate-600 uppercase tracking-widest mb-1.5 font-bold">
                                                Best Formula
                                            </h5>
                                            <code className="text-[10px] text-slate-300 font-mono leading-relaxed break-all block">
                                                {decodeGenome(bestGen.best_genome)}
                                            </code>
                                        </div>
                                    </div>
                                )}

                                {/* Latest Backtest Panel or Overview Summary */}
                                {latestDetail ? (
                                    <LatestBacktestPanel
                                        detail={latestDetail}
                                        symbol={selectedSymbol}
                                    />
                                ) : selectedOverview.best_pnl != null && (
                                    <BacktestSummaryPanel
                                        overview={selectedOverview}
                                        symbol={selectedSymbol}
                                    />
                                )}

                                {/* Generation Table */}
                                <div className="px-5 pb-5 pt-2">
                                    <div className="bg-slate-900/30 border border-white/5 rounded-xl backdrop-blur-sm overflow-hidden">
                                        <div className="flex items-center justify-between px-4 py-2.5 border-b border-white/5">
                                            <h4 className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                                                Generation History — {selectedSymbol}
                                            </h4>
                                            <span className="text-[10px] text-slate-600">
                                                {backtestGens.length} backtests / {generations.length} gens
                                            </span>
                                        </div>

                                        <div className="grid grid-cols-[60px_80px_80px_90px_80px_80px_60px_60px] gap-1 px-4 py-2 bg-black/20 border-b border-white/5 text-[9px] text-slate-600 uppercase tracking-wider font-bold sticky top-0 z-10">
                                            <span>Gen</span>
                                            <span className="text-right">IS PnL</span>
                                            <span className="text-right">OOS PnL</span>
                                            <span className="text-right">BT PnL</span>
                                            <span className="text-right">Sharpe</span>
                                            <span className="text-right">Max DD</span>
                                            <span className="text-right">WR</span>
                                            <span className="text-right">Time</span>
                                        </div>

                                        <div className="max-h-[400px] overflow-y-auto custom-scrollbar">
                                            {generations.map((g, idx) => {
                                                const isBest = bestGen?.generation === g.generation;
                                                const isExpanded = expandedGen === g.generation;
                                                const hasBt = g.backtest != null;
                                                const isOdd = idx % 2 === 1;

                                                return (
                                                    <React.Fragment key={g.generation}>
                                                        <button
                                                            onClick={() => { if (hasBt) handleExpandRow(g.generation); }}
                                                            className={`w-full grid grid-cols-[60px_80px_80px_90px_80px_80px_60px_60px] gap-1 px-4 py-2 text-[11px] transition-colors border-l-2 cursor-pointer ${
                                                                isExpanded
                                                                    ? "bg-indigo-500/10 border-l-indigo-500"
                                                                    : isBest
                                                                      ? `${isOdd ? "bg-white/[0.02]" : "bg-transparent"} border-l-amber-500/50`
                                                                      : `${isOdd ? "bg-white/[0.02]" : "bg-transparent"} border-l-transparent hover:bg-white/[0.04]`
                                                            }`}
                                                        >
                                                            <span className="text-slate-400 font-mono flex items-center gap-1">
                                                                {hasBt ? (
                                                                    isExpanded ? <ChevronDown className="w-3 h-3 text-slate-600" /> : <ChevronRight className="w-3 h-3 text-slate-600" />
                                                                ) : (
                                                                    <span className="w-3" />
                                                                )}
                                                                {isBest && <span className="w-1.5 h-1.5 rounded-full bg-amber-500 shadow-[0_0_6px_#f59e0b]" />}
                                                                {g.generation}
                                                            </span>
                                                            <span className={`text-right font-mono ${g.fitness != null && g.fitness > 0 ? "text-emerald-400" : "text-slate-500"}`}>
                                                                {fmtNum(g.fitness)}
                                                            </span>
                                                            <span className="text-right font-mono text-cyan-400/60">
                                                                {fmtNum(g.oos_psr)}
                                                            </span>
                                                            <span className={`text-right font-mono ${g.backtest ? g.backtest.pnl_percent >= 0 ? "text-emerald-400" : "text-red-400" : "text-slate-700"}`}>
                                                                {g.backtest ? fmtPct(g.backtest.pnl_percent) : "—"}
                                                            </span>
                                                            <span className={`text-right font-mono ${g.backtest ? g.backtest.sharpe_ratio >= 0 ? "text-slate-300" : "text-red-400" : "text-slate-700"}`}>
                                                                {g.backtest ? fmtNum(g.backtest.sharpe_ratio, 2) : "—"}
                                                            </span>
                                                            <span className={`text-right font-mono ${g.backtest ? "text-red-400/70" : "text-slate-700"}`}>
                                                                {g.backtest ? fmtPct(g.backtest.max_drawdown) : "—"}
                                                            </span>
                                                            <span className={`text-right font-mono ${g.backtest ? g.backtest.win_rate > 0.5 ? "text-emerald-400/70" : "text-slate-400" : "text-slate-700"}`}>
                                                                {g.backtest ? `${(g.backtest.win_rate * 100).toFixed(0)}%` : "—"}
                                                            </span>
                                                            <span className="text-right text-slate-600 font-mono">
                                                                {formatTimeAgo(g.timestamp)}
                                                            </span>
                                                        </button>

                                                        {isExpanded && (
                                                            <BacktestDetail generation={g} detail={expandedDetail} exchange={activeExchange} symbol={selectedSymbol || ""} mode={activeMode} />
                                                        )}
                                                    </React.Fragment>
                                                );
                                            })}

                                            {generations.length === 0 && (
                                                <div className="px-4 py-12 text-center text-xs text-slate-600">
                                                    No generation data for {selectedSymbol}.
                                                </div>
                                            )}
                                        </div>
                                    </div>
                                </div>
                            </>
                        ) : (
                            <div className="flex-1 flex items-center justify-center h-full">
                                <div className="text-center text-slate-600 text-xs">
                                    Select a symbol from the left panel
                                </div>
                            </div>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
}

/* Sub-components */

type EvolutionStatus = "improving" | "plateau" | "stagnant";

function getStatus(sym: SymbolOverview, _all: SymbolOverview[]): EvolutionStatus {
    const fitness = sym.best_fitness ?? 0;
    const oos = sym.best_oos_psr ?? 0;

    // Early generations — still exploring
    if (sym.latest_gen < 30) return "improving";

    // Overfitting: high IS PSR but negative OOS (strategy fails out-of-sample)
    if (fitness > 1.0 && oos < -1.0) return "stagnant";

    // OOS not significantly positive
    if (oos <= 0) return "plateau";

    // Both IS and OOS show significant positive PSR z-scores
    if (fitness > 0.5 && oos > 0.5) return "improving";

    return "plateau";
}

function StatusIcon({ status }: { status: EvolutionStatus }) {
    if (status === "improving") return <TrendingUp className="w-3 h-3 text-emerald-400" />;
    if (status === "plateau") return <Minus className="w-3 h-3 text-amber-400" />;
    return <TrendingDown className="w-3 h-3 text-red-400" />;
}

function StatusBadge({ status }: { status: EvolutionStatus }) {
    const colors = {
        improving: "bg-emerald-500/10 text-emerald-400 border-emerald-500/20",
        plateau: "bg-amber-500/10 text-amber-400 border-amber-500/20",
        stagnant: "bg-red-500/10 text-red-400 border-red-500/20",
    };
    return (
        <span className={`text-[9px] font-bold px-1.5 py-0.5 rounded border ${colors[status]}`}>
            {status}
        </span>
    );
}

function MetricCell({ label, value, positive }: { label: string; value: string; positive?: boolean }) {
    return (
        <div className="bg-white/[0.03] rounded px-2.5 py-1.5 border border-white/5">
            <span className="text-[9px] text-slate-600 block">{label}</span>
            <span className={`text-xs font-mono font-bold tabular-nums ${
                positive === true ? "text-emerald-400" :
                positive === false ? "text-red-400" :
                "text-slate-200"
            }`}>
                {value}
            </span>
        </div>
    );
}

function TradePnlChart({ trades }: { trades: Trade[] }) {
    const data = trades.map((t, i) => ({
        idx: i + 1,
        pnl: +(t.pnl * 100).toFixed(2),
        isShort: t.direction === "short",
    }));
    // Cumulative PnL
    let cum = 0;
    const cumData = trades.map((t, i) => {
        cum += t.pnl * 100;
        return { idx: i + 1, cum: +cum.toFixed(2) };
    });

    return (
        <div className="space-y-3">
            {/* Per-trade PnL bar chart */}
            <div>
                <h5 className="text-[9px] text-slate-600 uppercase tracking-widest mb-1.5 font-bold">
                    Per-Trade PnL (%)
                </h5>
                <div className="h-32 bg-white/[0.02] rounded-lg border border-white/5 p-1">
                    <ResponsiveContainer width="100%" height="100%">
                        <BarChart data={data} barCategoryGap={1}>
                            <CartesianGrid strokeDasharray="3 3" stroke="#334155" strokeOpacity={0.2} />
                            <XAxis dataKey="idx" stroke="#475569" fontSize={8} tickLine={false} axisLine={false} interval={Math.max(0, Math.floor(data.length / 10) - 1)} />
                            <YAxis stroke="#475569" fontSize={8} tickLine={false} axisLine={false} tickFormatter={(v) => `${v}%`} width={36} />
                            <Tooltip
                                contentStyle={{ backgroundColor: "rgba(2, 6, 23, 0.95)", border: "1px solid rgba(255,255,255,0.1)", borderRadius: "8px", fontSize: 10 }}
                                formatter={(value: number | string | undefined) => [`${typeof value === "number" ? value.toFixed(2) : "0"}%`, "PnL"]}
                                labelFormatter={(l) => `Trade #${l}`}
                            />
                            <ReferenceLine y={0} stroke="#475569" strokeOpacity={0.5} />
                            <Bar dataKey="pnl" radius={[1, 1, 0, 0]}>
                                {data.map((d, i) => (
                                    <Cell
                                        key={i}
                                        fill={d.isShort ? (d.pnl >= 0 ? "#a78bfa" : "#f87171") : (d.pnl >= 0 ? "#34d399" : "#f87171")}
                                        fillOpacity={0.7}
                                    />
                                ))}
                            </Bar>
                        </BarChart>
                    </ResponsiveContainer>
                </div>
            </div>

            {/* Cumulative trade PnL */}
            <div>
                <h5 className="text-[9px] text-slate-600 uppercase tracking-widest mb-1.5 font-bold">
                    Cumulative Trade PnL (%)
                </h5>
                <div className="h-28 bg-white/[0.02] rounded-lg border border-white/5 p-1">
                    <ResponsiveContainer width="100%" height="100%">
                        <AreaChart data={cumData}>
                            <defs>
                                <linearGradient id="cumPnlGrad" x1="0" y1="0" x2="0" y2="1">
                                    <stop offset="5%" stopColor="#818cf8" stopOpacity={0.3} />
                                    <stop offset="95%" stopColor="#818cf8" stopOpacity={0} />
                                </linearGradient>
                            </defs>
                            <CartesianGrid strokeDasharray="3 3" stroke="#334155" strokeOpacity={0.2} />
                            <XAxis dataKey="idx" stroke="#475569" fontSize={8} tickLine={false} axisLine={false} interval={Math.max(0, Math.floor(cumData.length / 10) - 1)} />
                            <YAxis stroke="#475569" fontSize={8} tickLine={false} axisLine={false} tickFormatter={(v) => `${v}%`} width={36} />
                            <Tooltip
                                contentStyle={{ backgroundColor: "rgba(2, 6, 23, 0.95)", border: "1px solid rgba(255,255,255,0.1)", borderRadius: "8px", fontSize: 10 }}
                                formatter={(value: number | string | undefined) => [`${typeof value === "number" ? value.toFixed(2) : "0"}%`, "Cumulative"]}
                                labelFormatter={(l) => `After Trade #${l}`}
                            />
                            <ReferenceLine y={0} stroke="#475569" strokeOpacity={0.5} />
                            <Area type="monotone" dataKey="cum" stroke="#818cf8" strokeWidth={1.5} fill="url(#cumPnlGrad)" />
                        </AreaChart>
                    </ResponsiveContainer>
                </div>
            </div>
        </div>
    );
}

function BacktestSummaryPanel({
    overview,
    symbol,
}: {
    overview: SymbolOverview;
    symbol: string;
}) {
    const foldPsrs = overview.fold_psrs;
    const foldData = foldPsrs
        ? foldPsrs.map((psr, i) => ({
              idx: i + 1,
              psr: +psr.toFixed(2),
          }))
        : [];

    return (
        <div className="px-5 py-3">
            <div className="bg-slate-900/30 border border-white/5 rounded-xl backdrop-blur-sm overflow-hidden">
                <div className="flex items-center justify-between px-4 pt-3 pb-2 border-b border-white/5">
                    <h4 className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                        Latest Backtest — {symbol} (Gen #{overview.latest_gen})
                    </h4>
                    <span className="text-[10px] text-slate-600">
                        from overview
                    </span>
                </div>

                <div className="p-4 space-y-4">
                    {/* Key metrics row — same 8-col layout as LatestBacktestPanel */}
                    <div className="grid grid-cols-8 gap-1.5">
                        <MetricCell label="PnL" value={fmtPct(overview.best_pnl)} positive={(overview.best_pnl ?? 0) >= 0} />
                        <MetricCell label="Sharpe" value={fmtNum(overview.sharpe_ratio, 2)} positive={(overview.sharpe_ratio ?? 0) >= 0} />
                        <MetricCell label="Sortino" value="—" />
                        <MetricCell label="Profit Factor" value="—" />
                        <MetricCell label="Max DD" value={overview.max_drawdown != null ? fmtPct(overview.max_drawdown) : "—"} positive={false} />
                        <MetricCell label="Win Rate" value={overview.win_rate != null ? fmtPct(overview.win_rate) : "—"} positive={(overview.win_rate ?? 0) > 0.5} />
                        <MetricCell label="Avg Win" value="—" />
                        <MetricCell label="Avg Loss" value="—" />
                    </div>

                    {/* Extended trade stats row — same 6-col layout */}
                    <div className="grid grid-cols-6 gap-1.5">
                        <MetricCell label="Profit Factor" value="—" />
                        <MetricCell label="Calmar" value="—" />
                        <MetricCell label="Avg Hold Bars" value="—" />
                        <MetricCell label="Max Consec W" value="—" />
                        <MetricCell label="Max Consec L" value="—" />
                        <MetricCell label="Avg Trade Ret" value="—" />
                    </div>

                    {/* Charts row — same 2-col layout */}
                    <div className="grid grid-cols-2 gap-4">
                        <div>
                            <h5 className="text-[9px] text-slate-600 uppercase tracking-widest mb-1.5 font-bold">
                                Equity Curve
                            </h5>
                            <div className="h-44 flex items-center justify-center text-xs text-slate-600 bg-white/[0.02] rounded-lg border border-white/5">
                                No equity curve data
                            </div>
                        </div>

                        <div>
                            {foldData.length > 0 ? (
                                <>
                                    <h5 className="text-[9px] text-slate-600 uppercase tracking-widest mb-1.5 font-bold">
                                        Fold PSR (K={foldData.length})
                                    </h5>
                                    <div className="h-44 bg-white/[0.02] rounded-lg border border-white/5 p-1">
                                        <ResponsiveContainer width="100%" height="100%">
                                            <BarChart data={foldData} barCategoryGap={4}>
                                                <CartesianGrid strokeDasharray="3 3" stroke="#334155" strokeOpacity={0.2} />
                                                <XAxis dataKey="idx" stroke="#475569" fontSize={8} tickLine={false} axisLine={false} tickFormatter={(v) => `F${v}`} />
                                                <YAxis stroke="#475569" fontSize={8} tickLine={false} axisLine={false} width={36} />
                                                <Tooltip
                                                    contentStyle={{ backgroundColor: "rgba(2, 6, 23, 0.95)", border: "1px solid rgba(255,255,255,0.1)", borderRadius: "8px", fontSize: 10 }}
                                                    formatter={(value: number | string | undefined) => [typeof value === "number" ? value.toFixed(2) : "0", "PSR z-score"]}
                                                    labelFormatter={(l) => `Fold #${l}`}
                                                />
                                                <ReferenceLine y={0} stroke="#475569" strokeOpacity={0.5} />
                                                <Bar dataKey="psr" radius={[2, 2, 0, 0]}>
                                                    {foldData.map((d, i) => (
                                                        <Cell
                                                            key={i}
                                                            fill={d.psr >= 0 ? "#34d399" : "#f87171"}
                                                            fillOpacity={0.7}
                                                        />
                                                    ))}
                                                </Bar>
                                            </BarChart>
                                        </ResponsiveContainer>
                                    </div>
                                </>
                            ) : (
                                <>
                                    <h5 className="text-[9px] text-slate-600 uppercase tracking-widest mb-1.5 font-bold">
                                        Per-Trade PnL (%)
                                    </h5>
                                    <div className="h-44 flex items-center justify-center text-xs text-slate-600 bg-white/[0.02] rounded-lg border border-white/5">
                                        No trade data
                                    </div>
                                </>
                            )}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}

function LatestBacktestPanel({
    detail,
    symbol,
}: {
    detail: { gen: Generation; bt: BacktestData };
    symbol: string;
}) {
    const { gen, bt } = detail;
    const m = bt.metrics;
    const trades = bt.trades || [];
    const wins = trades.filter((t) => t.pnl > 0);
    const losses = trades.filter((t) => t.pnl <= 0);

    return (
        <div className="px-5 py-3">
            <div className="bg-slate-900/30 border border-white/5 rounded-xl backdrop-blur-sm overflow-hidden">
                <div className="flex items-center justify-between px-4 pt-3 pb-2 border-b border-white/5">
                    <h4 className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">
                        Latest Backtest — {symbol} (Gen #{gen.generation})
                    </h4>
                    <span className="text-[10px] text-slate-600">
                        {(() => {
                            const longCount = trades.filter(t => t.direction !== "short").length;
                            const shortCount = trades.filter(t => t.direction === "short").length;
                            if (shortCount > 0) {
                                return `${trades.length} trades (${longCount}L / ${shortCount}S)`;
                            }
                            return `${trades.length} trades`;
                        })()}
                    </span>
                </div>

                <div className="p-4 space-y-4">
                    {/* Key metrics row */}
                    <div className="grid grid-cols-8 gap-1.5">
                        <MetricCell label="PnL" value={fmtPct(bt.pnl_percent)} positive={bt.pnl_percent >= 0} />
                        <MetricCell label="Sharpe" value={fmtNum(bt.sharpe_ratio, 2)} positive={bt.sharpe_ratio >= 0} />
                        <MetricCell label="Sortino" value={fmtNum(m?.sortino_ratio, 2)} positive={(m?.sortino_ratio ?? 0) >= 0} />
                        <MetricCell label="Profit Factor" value={fmtNum(m?.profit_factor, 2)} positive={(m?.profit_factor ?? 0) >= 1} />
                        <MetricCell label="Max DD" value={fmtPct(bt.max_drawdown)} positive={false} />
                        <MetricCell label="Win Rate" value={fmtPct(bt.win_rate)} positive={bt.win_rate > 0.5} />
                        <MetricCell label="Avg Win" value={fmtPct(m?.avg_win)} positive />
                        <MetricCell label="Avg Loss" value={fmtPct(m?.avg_loss)} positive={false} />
                    </div>

                    {/* Extended trade stats row */}
                    {m && (
                        <div className="grid grid-cols-6 gap-1.5">
                            <MetricCell label="Profit Factor" value={fmtNum(m.profit_factor, 2)} positive={(m.profit_factor ?? 0) >= 1} />
                            <MetricCell label="Calmar" value={fmtNum(m.calmar_ratio, 2)} positive={(m.calmar_ratio ?? 0) >= 0} />
                            <MetricCell label="Avg Hold Bars" value={m.avg_holding_bars != null ? m.avg_holding_bars.toFixed(1) : "—"} />
                            <MetricCell label="Max Consec W" value={m.max_consecutive_wins != null ? String(m.max_consecutive_wins) : "—"} positive={true} />
                            <MetricCell label="Max Consec L" value={m.max_consecutive_losses != null ? String(m.max_consecutive_losses) : "—"} positive={false} />
                            <MetricCell label="Avg Trade Ret" value={fmtPct(m.avg_trade_return)} positive={(m.avg_trade_return ?? 0) >= 0} />
                        </div>
                    )}

                    {/* Charts row: Equity curve + Trade PnL */}
                    <div className="grid grid-cols-2 gap-4">
                        {/* Equity curve */}
                        <div>
                            <h5 className="text-[9px] text-slate-600 uppercase tracking-widest mb-1.5 font-bold">
                                Equity Curve
                            </h5>
                            {bt.equity_curve && bt.equity_curve.length > 0 ? (
                                <div className="h-44 bg-white/[0.02] rounded-lg border border-white/5 p-1">
                                    <ResponsiveContainer width="100%" height="100%">
                                        <AreaChart data={bt.equity_curve}>
                                            <defs>
                                                <linearGradient id="eqLatest" x1="0" y1="0" x2="0" y2="1">
                                                    <stop offset="5%" stopColor="#818cf8" stopOpacity={0.2} />
                                                    <stop offset="95%" stopColor="#818cf8" stopOpacity={0} />
                                                </linearGradient>
                                            </defs>
                                            <CartesianGrid strokeDasharray="3 3" stroke="#334155" strokeOpacity={0.2} />
                                            <XAxis dataKey="timestamp" stroke="#475569" fontSize={8} tickLine={false} axisLine={false} hide />
                                            <YAxis stroke="#475569" fontSize={8} tickLine={false} axisLine={false} tickFormatter={(v) => `${(v * 100).toFixed(0)}%`} width={36} />
                                            <Tooltip
                                                contentStyle={{ backgroundColor: "rgba(2, 6, 23, 0.95)", border: "1px solid rgba(255,255,255,0.1)", borderRadius: "8px", fontSize: 10 }}
                                                formatter={(value: number | string | undefined) => [`${typeof value === "number" ? (value * 100).toFixed(2) : "0"}%`, "Equity"]}
                                            />
                                            <Area type="monotone" dataKey="value" stroke="#818cf8" strokeWidth={1.5} fill="url(#eqLatest)" />
                                        </AreaChart>
                                    </ResponsiveContainer>
                                </div>
                            ) : (
                                <div className="h-44 flex items-center justify-center text-xs text-slate-600 bg-white/[0.02] rounded-lg border border-white/5">
                                    No equity curve data
                                </div>
                            )}
                        </div>

                        {/* Trade PnL distribution */}
                        <div>
                            {trades.length > 0 ? (
                                <TradePnlChart trades={trades} />
                            ) : (
                                <div className="h-44 flex items-center justify-center text-xs text-slate-600 bg-white/[0.02] rounded-lg border border-white/5">
                                    No trade data
                                </div>
                            )}
                        </div>
                    </div>

                    {/* Trade Log */}
                    {trades.length > 0 && (
                        <div>
                            <h5 className="text-[9px] text-slate-600 uppercase tracking-widest mb-1.5 font-bold">
                                Trade Log
                                <span className="ml-2 text-emerald-500/70">{wins.length}W</span>
                                <span className="ml-1 text-red-500/70">{losses.length}L</span>
                            </h5>
                            <div className="max-h-52 overflow-y-auto custom-scrollbar bg-white/[0.02] rounded-lg border border-white/5">
                                <table className="w-full text-[10px]">
                                    <thead className="sticky top-0 bg-slate-950/90 z-10">
                                        <tr className="text-slate-600 uppercase tracking-wider">
                                            <th className="text-left px-2 py-1.5 font-bold">#</th>
                                            <th className="text-left px-2 py-1.5 font-bold">Dir</th>
                                            <th className="text-right px-2 py-1.5 font-bold">Entry Bar</th>
                                            <th className="text-right px-2 py-1.5 font-bold">Exit Bar</th>
                                            <th className="text-right px-2 py-1.5 font-bold">Duration</th>
                                            <th className="text-right px-2 py-1.5 font-bold">PnL</th>
                                            <th className="text-right px-2 py-1.5 font-bold">Cum PnL</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {(() => {
                                            let cumPnl = 0;
                                            return trades.map((trade, i) => {
                                                cumPnl += trade.pnl;
                                                return (
                                                    <tr key={i} className={`border-t border-white/5 ${i % 2 === 1 ? "bg-white/[0.02]" : ""}`}>
                                                        <td className="px-2 py-1 text-slate-500 font-mono">{i + 1}</td>
                                                        <td className={`px-2 py-1 font-mono font-bold ${trade.direction === "short" ? "text-red-400" : "text-emerald-400"}`}>
                                                            {trade.direction === "short" ? "SHORT" : "LONG"}
                                                        </td>
                                                        <td className="px-2 py-1 text-right text-slate-400 font-mono">{trade.entry}</td>
                                                        <td className="px-2 py-1 text-right text-slate-400 font-mono">{trade.exit}</td>
                                                        <td className="px-2 py-1 text-right text-slate-400 font-mono">{trade.bars}b</td>
                                                        <td className={`px-2 py-1 text-right font-mono font-bold ${trade.pnl >= 0 ? "text-emerald-400" : "text-red-400"}`}>
                                                            {fmtPct(trade.pnl)}
                                                        </td>
                                                        <td className={`px-2 py-1 text-right font-mono ${cumPnl >= 0 ? "text-indigo-400/70" : "text-red-400/70"}`}>
                                                            {fmtPct(cumPnl)}
                                                        </td>
                                                    </tr>
                                                );
                                            });
                                        })()}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}

function BacktestDetail({
    generation,
    detail,
    exchange,
    symbol,
    mode,
}: {
    generation: Generation;
    detail: BacktestData | null;
    exchange: string;
    symbol: string;
    mode: StrategyMode;
}) {
    const bt = detail || generation.backtest;
    if (!bt) return null;

    const m = bt.metrics;
    const featureImportance = generation.best_genome
        ? getFeatureImportance(generation.best_genome)
        : {};
    const totalFeatureCount = Object.values(featureImportance).reduce((a, b) => a + b, 0);

    const handleRerun = async () => {
        if (!generation.best_genome) return;
        try {
            await fetch(`/api/v1/evolution/${exchange}/backtest`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    genome: generation.best_genome,
                    token_address: symbol,
                    mode,
                }),
            });
        } catch {
            /* rerun failed */
        }
    };

    return (
        <div className="bg-black/30 border-b border-white/5 px-6 py-5 backdrop-blur-sm">
            <div className="grid grid-cols-12 gap-5">
                {/* Left: Formula + Metrics + Features */}
                <div className="col-span-4 space-y-4">
                    {generation.best_genome && (
                        <div>
                            <h5 className="text-[9px] text-slate-600 uppercase tracking-widest mb-1.5 font-bold">
                                Decoded Formula
                            </h5>
                            <code className="text-[10px] text-slate-300 font-mono leading-relaxed break-all block bg-white/[0.03] rounded p-2 border border-white/5">
                                {decodeGenome(generation.best_genome)}
                            </code>
                        </div>
                    )}

                    {/* Extended 8-metric grid */}
                    <div>
                        <h5 className="text-[9px] text-slate-600 uppercase tracking-widest mb-2 font-bold">
                            Backtest Metrics
                        </h5>
                        <div className="grid grid-cols-2 gap-1.5">
                            <MetricCell label="PnL" value={fmtPct(bt.pnl_percent)} positive={bt.pnl_percent >= 0} />
                            <MetricCell label="Sharpe" value={fmtNum(bt.sharpe_ratio, 2)} positive={bt.sharpe_ratio >= 0} />
                            <MetricCell label="Sortino" value={fmtNum(m?.sortino_ratio, 2)} positive={(m?.sortino_ratio ?? 0) >= 0} />
                            <MetricCell label="Calmar" value={fmtNum(m?.calmar_ratio, 2)} positive={(m?.calmar_ratio ?? 0) >= 0} />
                            <MetricCell label="Profit Factor" value={fmtNum(m?.profit_factor, 2)} positive={(m?.profit_factor ?? 0) >= 1} />
                            <MetricCell label="Max DD" value={fmtPct(bt.max_drawdown)} positive={false} />
                            <MetricCell label="Win Rate" value={fmtPct(bt.win_rate)} positive={bt.win_rate != null && bt.win_rate > 0.5} />
                            <MetricCell label="Trades" value={bt.total_trades?.toLocaleString() ?? "—"} />
                        </div>
                    </div>

                    {/* Trade Statistics */}
                    {m && (
                        <div>
                            <h5 className="text-[9px] text-slate-600 uppercase tracking-widest mb-2 font-bold">
                                Trade Statistics
                            </h5>
                            <div className="grid grid-cols-2 gap-1.5 text-[10px]">
                                <div className="bg-white/[0.03] rounded px-2 py-1 border border-white/5">
                                    <span className="text-[8px] text-slate-600 block">Avg Win</span>
                                    <span className="font-mono text-emerald-400">{fmtPct(m.avg_win)}</span>
                                </div>
                                <div className="bg-white/[0.03] rounded px-2 py-1 border border-white/5">
                                    <span className="text-[8px] text-slate-600 block">Avg Loss</span>
                                    <span className="font-mono text-red-400">{fmtPct(m.avg_loss)}</span>
                                </div>
                                <div className="bg-white/[0.03] rounded px-2 py-1 border border-white/5">
                                    <span className="text-[8px] text-slate-600 block">Max Win</span>
                                    <span className="font-mono text-emerald-400/70">{fmtPct(m.max_win)}</span>
                                </div>
                                <div className="bg-white/[0.03] rounded px-2 py-1 border border-white/5">
                                    <span className="text-[8px] text-slate-600 block">Max Loss</span>
                                    <span className="font-mono text-red-400/70">{fmtPct(m.max_loss)}</span>
                                </div>
                                <div className="bg-white/[0.03] rounded px-2 py-1 border border-white/5">
                                    <span className="text-[8px] text-slate-600 block">Consec Wins</span>
                                    <span className="font-mono text-slate-300">{m.max_consecutive_wins ?? "—"}</span>
                                </div>
                                <div className="bg-white/[0.03] rounded px-2 py-1 border border-white/5">
                                    <span className="text-[8px] text-slate-600 block">Consec Losses</span>
                                    <span className="font-mono text-slate-300">{m.max_consecutive_losses ?? "—"}</span>
                                </div>
                                <div className="col-span-2 bg-white/[0.03] rounded px-2 py-1 border border-white/5">
                                    <span className="text-[8px] text-slate-600 block">Avg Holding Bars</span>
                                    <span className="font-mono text-slate-300">{m.avg_holding_bars != null ? m.avg_holding_bars.toFixed(1) : "—"}</span>
                                </div>
                            </div>
                        </div>
                    )}

                    {/* Fold Performance */}
                    {generation.fold_psrs && generation.fold_psrs.length > 0 && (
                        <div>
                            <h5 className="text-[9px] text-slate-600 uppercase tracking-widest mb-2 font-bold">
                                Fold Performance (K={generation.fold_psrs.length})
                            </h5>
                            <div className="flex items-end gap-1 h-12">
                                {generation.fold_psrs.map((pnl, i) => {
                                    const maxAbs = Math.max(...generation.fold_psrs!.map(Math.abs), 0.01);
                                    const height = Math.abs(pnl) / maxAbs * 100;
                                    return (
                                        <div key={i} className="flex-1 flex flex-col items-center justify-end h-full">
                                            <div
                                                className={`w-full rounded-sm ${pnl >= 0 ? "bg-emerald-500/60" : "bg-red-500/60"}`}
                                                style={{ height: `${Math.max(height, 4)}%` }}
                                                title={`Fold ${i + 1}: ${pnl.toFixed(4)}`}
                                            />
                                            <span className="text-[7px] text-slate-600 mt-0.5 font-mono">{pnl.toFixed(2)}</span>
                                        </div>
                                    );
                                })}
                            </div>
                        </div>
                    )}

                    {totalFeatureCount > 0 && (
                        <div>
                            <h5 className="text-[9px] text-slate-600 uppercase tracking-widest mb-2 font-bold">
                                Feature Usage
                            </h5>
                            <div className="space-y-1">
                                {Object.entries(featureImportance)
                                    .sort(([, a], [, b]) => b - a)
                                    .map(([name, count]) => {
                                        const pct = (count / totalFeatureCount) * 100;
                                        return (
                                            <div key={name} className="flex items-center gap-2">
                                                <span className="text-[10px] text-slate-500 w-20 truncate font-mono">{name}</span>
                                                <div className="flex-1 h-1 bg-white/5 rounded-full overflow-hidden">
                                                    <div className="h-full bg-indigo-500/40 rounded-full" style={{ width: `${pct}%` }} />
                                                </div>
                                                <span className="text-[9px] text-slate-600 w-6 text-right font-mono">{Math.round(pct)}%</span>
                                            </div>
                                        );
                                    })}
                            </div>
                        </div>
                    )}

                    <button
                        onClick={handleRerun}
                        className="px-3 py-1.5 text-[10px] font-bold uppercase tracking-wider rounded-md bg-white/5 border border-white/10 text-slate-400 hover:bg-white/10 hover:text-slate-200 transition-colors cursor-pointer"
                    >
                        Re-run Backtest
                    </button>
                </div>

                {/* Right: Equity curve + Trade log */}
                <div className="col-span-8 space-y-4">
                    <div>
                        <h5 className="text-[9px] text-slate-600 uppercase tracking-widest mb-2 font-bold">
                            Equity Curve
                        </h5>
                        {detail?.equity_curve && detail.equity_curve.length > 0 ? (
                            <div className="h-56 bg-white/[0.02] rounded-lg border border-white/5 p-2">
                                <ResponsiveContainer width="100%" height="100%">
                                    <AreaChart data={detail.equity_curve}>
                                        <defs>
                                            <linearGradient id={`eq-${generation.generation}`} x1="0" y1="0" x2="0" y2="1">
                                                <stop offset="5%" stopColor="#818cf8" stopOpacity={0.2} />
                                                <stop offset="95%" stopColor="#818cf8" stopOpacity={0} />
                                            </linearGradient>
                                        </defs>
                                        <CartesianGrid strokeDasharray="3 3" stroke="#334155" strokeOpacity={0.2} />
                                        <XAxis dataKey="timestamp" stroke="#475569" fontSize={9} tickLine={false} axisLine={false} tickFormatter={(v) => new Date(v).toLocaleDateString(undefined, { month: "short", day: "numeric" })} />
                                        <YAxis stroke="#475569" fontSize={9} tickLine={false} axisLine={false} tickFormatter={(v) => `${(v * 100).toFixed(0)}%`} />
                                        <Tooltip
                                            contentStyle={{ backgroundColor: "rgba(2, 6, 23, 0.95)", border: "1px solid rgba(255,255,255,0.1)", borderRadius: "8px", fontSize: 11 }}
                                            labelStyle={{ color: "#94a3b8" }}
                                            formatter={(value: number | string | undefined) => [`${typeof value === "number" ? (value * 100).toFixed(2) : "0.00"}%`, "Equity"]}
                                            labelFormatter={(v) => new Date(v).toLocaleString()}
                                        />
                                        <Area type="monotone" dataKey="value" stroke="#818cf8" strokeWidth={1.5} fill={`url(#eq-${generation.generation})`} />
                                    </AreaChart>
                                </ResponsiveContainer>
                            </div>
                        ) : (
                            <div className="h-56 flex items-center justify-center text-xs text-slate-600 bg-white/[0.02] rounded-lg border border-white/5">
                                {detail === null ? (
                                    <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-indigo-500" />
                                ) : (
                                    "No equity curve data"
                                )}
                            </div>
                        )}
                    </div>

                    {/* Trade Log */}
                    {detail?.trades && detail.trades.length > 0 && (
                        <div>
                            <h5 className="text-[9px] text-slate-600 uppercase tracking-widest mb-2 font-bold">
                                Trade Log ({detail.trades.length} trades)
                            </h5>
                            <div className="max-h-40 overflow-y-auto custom-scrollbar bg-white/[0.02] rounded-lg border border-white/5">
                                <table className="w-full text-[10px]">
                                    <thead className="sticky top-0 bg-slate-950/90">
                                        <tr className="text-slate-600 uppercase tracking-wider">
                                            <th className="text-left px-2 py-1.5 font-bold">#</th>
                                            <th className="text-left px-2 py-1.5 font-bold">Dir</th>
                                            <th className="text-right px-2 py-1.5 font-bold">Entry</th>
                                            <th className="text-right px-2 py-1.5 font-bold">Exit</th>
                                            <th className="text-right px-2 py-1.5 font-bold">Bars</th>
                                            <th className="text-right px-2 py-1.5 font-bold">PnL</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        {detail.trades.map((trade, i) => (
                                            <tr key={i} className={`border-t border-white/5 ${i % 2 === 1 ? "bg-white/[0.02]" : ""}`}>
                                                <td className="px-2 py-1 text-slate-500 font-mono">{i + 1}</td>
                                                <td className={`px-2 py-1 font-mono font-bold ${trade.direction === "short" ? "text-red-400" : "text-emerald-400"}`}>
                                                    {trade.direction === "short" ? "SHORT" : "LONG"}
                                                </td>
                                                <td className="px-2 py-1 text-right text-slate-400 font-mono">{trade.entry}</td>
                                                <td className="px-2 py-1 text-right text-slate-400 font-mono">{trade.exit}</td>
                                                <td className="px-2 py-1 text-right text-slate-400 font-mono">{trade.bars}</td>
                                                <td className={`px-2 py-1 text-right font-mono font-bold ${trade.pnl >= 0 ? "text-emerald-400" : "text-red-400"}`}>
                                                    {fmtPct(trade.pnl)}
                                                </td>
                                            </tr>
                                        ))}
                                    </tbody>
                                </table>
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
