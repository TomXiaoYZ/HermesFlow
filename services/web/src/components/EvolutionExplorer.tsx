"use client";

import React, { useState, useEffect, useCallback } from "react";
import { Brain, ChevronDown, ChevronRight, Sparkles } from "lucide-react";
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
} from "recharts";
import { decodeGenome, getFeatureImportance, loadFactorConfigForExchange } from "@/utils/genome";

interface Exchange {
    key: string;
    exchange: string;
    resolution: string;
    factor_count: number;
}

interface BacktestData {
    pnl_percent: number;
    sharpe_ratio: number;
    max_drawdown: number;
    win_rate: number;
    total_trades: number;
    equity_curve?: { timestamp: number; value: number }[];
}

interface Generation {
    generation: number;
    fitness: number | null;
    best_genome: number[] | null;
    strategy_id: string | null;
    timestamp: string | null;
    oos_ic: number | null;
    backtest: BacktestData | null;
}

function formatTimeAgo(timestamp: string | null): string {
    if (!timestamp) return "—";
    const diff = Date.now() - new Date(timestamp).getTime();
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return "just now";
    if (mins < 60) return `${mins}m ago`;
    const hours = Math.floor(mins / 60);
    if (hours < 24) return `${hours}h ago`;
    return `${Math.floor(hours / 24)}d ago`;
}

function formatPercent(value: number | null | undefined): string {
    if (value == null) return "—";
    return `${value >= 0 ? "+" : ""}${(value * 100).toFixed(1)}%`;
}

function formatNumber(value: number | null | undefined, decimals = 2): string {
    if (value == null) return "—";
    return value.toFixed(decimals);
}

export default function EvolutionExplorer() {
    const [exchanges, setExchanges] = useState<Exchange[]>([]);
    const [activeExchange, setActiveExchange] = useState<string>("");
    const [generations, setGenerations] = useState<Generation[]>([]);
    const [selectedGen, setSelectedGen] = useState<number | null>(null);
    const [expandedGen, setExpandedGen] = useState<number | null>(null);
    const [expandedDetail, setExpandedDetail] = useState<BacktestData | null>(null);
    const [loading, setLoading] = useState(true);

    // Fetch exchanges on mount
    useEffect(() => {
        fetch("/api/v1/evolution/exchanges")
            .then((res) => res.json())
            .then((data) => {
                const exs: Exchange[] = data.exchanges || [];
                setExchanges(exs);
                if (exs.length > 0) {
                    setActiveExchange(exs[0].key);
                }
            })
            .catch(() => setExchanges([]));
    }, []);

    // Fetch generations when exchange changes
    const fetchGenerations = useCallback(async () => {
        if (!activeExchange) return;
        try {
            setLoading(true);
            await loadFactorConfigForExchange(activeExchange);
            const res = await fetch(
                `/api/v1/evolution/${activeExchange}/generations?limit=200`
            );
            const data = await res.json();
            setGenerations(data.generations || []);
        } catch {
            setGenerations([]);
        } finally {
            setLoading(false);
        }
    }, [activeExchange]);

    useEffect(() => {
        fetchGenerations();
        const interval = setInterval(fetchGenerations, 30000);
        return () => clearInterval(interval);
    }, [fetchGenerations]);

    // Fetch detail when expanding a row
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
                `/api/v1/evolution/${activeExchange}/generations/${gen}`
            );
            const data = await res.json();
            if (data.backtest) {
                setExpandedDetail(data.backtest);
            }
        } catch {
            // Detail fetch failed - row still shows summary
        }
    };

    // Derived data
    const chartData = [...generations]
        .reverse()
        .map((g) => ({
            gen: g.generation,
            fitness: g.fitness,
            oos_ic: g.oos_ic,
            hasBacktest: g.backtest != null,
        }));

    const bestGen = generations.reduce<Generation | null>((best, g) => {
        if (g.fitness == null) return best;
        if (!best || best.fitness == null || g.fitness > best.fitness) return g;
        return best;
    }, null);

    const featureImportance = bestGen?.best_genome
        ? getFeatureImportance(bestGen.best_genome)
        : {};
    const totalFeatureCount = Object.values(featureImportance).reduce(
        (a, b) => a + b,
        0
    );

    return (
        <div className="bg-slate-900/50 backdrop-blur border border-slate-800/50 rounded-2xl p-6 shadow-2xl">
            {/* Header */}
            <div className="flex items-center justify-between mb-6">
                <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-purple-500 to-pink-600 flex items-center justify-center">
                        <Brain className="w-5 h-5" />
                    </div>
                    <div>
                        <h3 className="text-lg font-semibold text-white">
                            Strategy Evolution Explorer
                        </h3>
                        <p className="text-xs text-slate-400">
                            {generations.length > 0
                                ? `${generations.length} generations loaded`
                                : "Awaiting data"}
                        </p>
                    </div>
                </div>

                {/* Exchange Tabs */}
                <div className="flex gap-2">
                    {exchanges.map((ex) => (
                        <button
                            key={ex.key}
                            onClick={() => setActiveExchange(ex.key)}
                            className={`rounded-full px-4 py-1.5 text-sm font-medium transition-all ${
                                activeExchange === ex.key
                                    ? "bg-purple-500/20 border border-purple-500/50 text-purple-300"
                                    : "bg-slate-800 border border-slate-700 text-slate-400 hover:bg-slate-700 hover:text-slate-300"
                            }`}
                        >
                            {ex.exchange}
                        </button>
                    ))}
                </div>
            </div>

            {loading && generations.length === 0 ? (
                <div className="flex items-center justify-center h-48">
                    <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-purple-500" />
                </div>
            ) : (
                <>
                    {/* Top Section: Chart + Best Strategy */}
                    <div className="grid grid-cols-12 gap-6 mb-6">
                        {/* Fitness Chart */}
                        <div className="col-span-8">
                            <h4 className="text-sm font-medium text-slate-300 mb-3">
                                Fitness Trend
                            </h4>
                            <div className="h-48 w-full">
                                <ResponsiveContainer width="100%" height="100%">
                                    <ComposedChart data={chartData}>
                                        <CartesianGrid
                                            strokeDasharray="3 3"
                                            stroke="#334155"
                                        />
                                        <XAxis
                                            dataKey="gen"
                                            stroke="#64748b"
                                            fontSize={10}
                                            label={{
                                                value: "Generation",
                                                position: "insideBottom",
                                                offset: -2,
                                                style: {
                                                    fill: "#64748b",
                                                    fontSize: 10,
                                                },
                                            }}
                                        />
                                        <YAxis
                                            stroke="#64748b"
                                            fontSize={10}
                                            label={{
                                                value: "IC",
                                                angle: -90,
                                                position: "insideLeft",
                                                style: {
                                                    fill: "#64748b",
                                                    fontSize: 10,
                                                },
                                            }}
                                        />
                                        <Tooltip
                                            contentStyle={{
                                                backgroundColor: "#1e293b",
                                                border: "1px solid #475569",
                                                borderRadius: "8px",
                                            }}
                                            labelStyle={{ color: "#cbd5e1" }}
                                            formatter={(
                                                value,
                                                name
                                            ) => [
                                                typeof value === "number"
                                                    ? value.toFixed(4)
                                                    : "—",
                                                name === "fitness"
                                                    ? "In-Sample IC"
                                                    : "OOS IC",
                                            ]}
                                            labelFormatter={(label) =>
                                                `Gen #${label}`
                                            }
                                        />
                                        <Line
                                            type="monotone"
                                            dataKey="fitness"
                                            stroke="#a78bfa"
                                            strokeWidth={2}
                                            dot={false}
                                            name="fitness"
                                        />
                                        <Line
                                            type="monotone"
                                            dataKey="oos_ic"
                                            stroke="#22d3ee"
                                            strokeWidth={1.5}
                                            strokeDasharray="5 3"
                                            dot={false}
                                            name="oos_ic"
                                            connectNulls
                                        />
                                        <Scatter
                                            dataKey="fitness"
                                            data={chartData.filter(
                                                (d) => d.hasBacktest
                                            )}
                                            fill="#a78bfa"
                                            shape="circle"
                                            r={3}
                                            onClick={(e: { gen?: number }) => {
                                                if (e.gen != null)
                                                    setSelectedGen(e.gen);
                                            }}
                                        />
                                    </ComposedChart>
                                </ResponsiveContainer>
                            </div>
                        </div>

                        {/* Best Strategy Card */}
                        <div className="col-span-4">
                            <h4 className="text-sm font-medium text-slate-300 mb-3">
                                Best Strategy
                            </h4>
                            {bestGen ? (
                                <div className="bg-slate-800/50 border border-slate-700/50 rounded-xl p-4 space-y-3">
                                    <div className="flex items-center justify-between">
                                        <span className="text-xs text-slate-400">
                                            Gen #{bestGen.generation}
                                        </span>
                                        <div className="flex items-center gap-1.5">
                                            <Sparkles className="w-3.5 h-3.5 text-purple-400" />
                                            <span className="text-sm font-semibold text-purple-300">
                                                IC:{" "}
                                                {bestGen.fitness?.toFixed(4)}
                                            </span>
                                        </div>
                                    </div>
                                    {bestGen.oos_ic != null && (
                                        <div className="text-xs text-cyan-400">
                                            OOS IC:{" "}
                                            {bestGen.oos_ic.toFixed(4)}
                                        </div>
                                    )}
                                    <div>
                                        <span className="text-xs text-slate-500 block mb-1">
                                            Formula
                                        </span>
                                        <code className="text-xs text-slate-300 bg-slate-900/50 px-2 py-1 rounded block break-all">
                                            {bestGen.best_genome
                                                ? decodeGenome(
                                                      bestGen.best_genome
                                                  )
                                                : "—"}
                                        </code>
                                    </div>
                                    {totalFeatureCount > 0 && (
                                        <div>
                                            <span className="text-xs text-slate-500 block mb-2">
                                                Feature Importance
                                            </span>
                                            <div className="space-y-1.5">
                                                {Object.entries(
                                                    featureImportance
                                                )
                                                    .sort(
                                                        ([, a], [, b]) => b - a
                                                    )
                                                    .map(([name, count]) => {
                                                        const pct =
                                                            totalFeatureCount >
                                                            0
                                                                ? (count /
                                                                      totalFeatureCount) *
                                                                  100
                                                                : 0;
                                                        return (
                                                            <div
                                                                key={name}
                                                                className="flex items-center gap-2"
                                                            >
                                                                <span className="text-xs text-slate-400 w-16 truncate">
                                                                    {name}
                                                                </span>
                                                                <div className="flex-1 h-2 bg-slate-900/50 rounded-full overflow-hidden">
                                                                    <div
                                                                        className="h-full bg-purple-500/60 rounded-full"
                                                                        style={{
                                                                            width: `${pct}%`,
                                                                        }}
                                                                    />
                                                                </div>
                                                                <span className="text-xs text-slate-500 w-8 text-right">
                                                                    {Math.round(
                                                                        pct
                                                                    )}
                                                                    %
                                                                </span>
                                                            </div>
                                                        );
                                                    })}
                                            </div>
                                        </div>
                                    )}
                                </div>
                            ) : (
                                <div className="text-sm text-slate-500 italic">
                                    No strategies evolved yet
                                </div>
                            )}
                        </div>
                    </div>

                    {/* Generation History Table */}
                    <div>
                        <h4 className="text-sm font-medium text-slate-300 mb-3">
                            Generation History
                        </h4>
                        <div className="overflow-hidden rounded-xl border border-slate-800/50">
                            {/* Table Header */}
                            <div className="grid grid-cols-[60px_100px_90px_90px_90px_80px_1fr] gap-2 px-4 py-2.5 bg-slate-800/30 border-b border-slate-800/50">
                                <span className="text-xs text-slate-400 uppercase tracking-wide">
                                    Gen
                                </span>
                                <span className="text-xs text-slate-400 uppercase tracking-wide">
                                    Fitness
                                </span>
                                <span className="text-xs text-slate-400 uppercase tracking-wide">
                                    PnL%
                                </span>
                                <span className="text-xs text-slate-400 uppercase tracking-wide">
                                    Sharpe
                                </span>
                                <span className="text-xs text-slate-400 uppercase tracking-wide">
                                    Max DD
                                </span>
                                <span className="text-xs text-slate-400 uppercase tracking-wide">
                                    Win Rate
                                </span>
                                <span className="text-xs text-slate-400 uppercase tracking-wide text-right">
                                    Time
                                </span>
                            </div>

                            {/* Table Rows */}
                            <div className="max-h-[320px] overflow-y-auto scrollbar-thin scrollbar-thumb-white/10 scrollbar-track-transparent">
                                {generations.map((g) => {
                                    const isBest =
                                        bestGen?.generation === g.generation;
                                    const isSelected =
                                        selectedGen === g.generation;
                                    const isExpanded =
                                        expandedGen === g.generation;
                                    const hasBacktest = g.backtest != null;

                                    return (
                                        <React.Fragment
                                            key={g.generation}
                                        >
                                            <button
                                                onClick={() => {
                                                    setSelectedGen(
                                                        g.generation
                                                    );
                                                    if (hasBacktest) {
                                                        handleExpandRow(
                                                            g.generation
                                                        );
                                                    }
                                                }}
                                                className={`w-full grid grid-cols-[60px_100px_90px_90px_90px_80px_1fr] gap-2 px-4 py-2 text-sm transition-colors hover:bg-slate-800/30 ${
                                                    isSelected
                                                        ? "bg-purple-500/10 border-l-2 border-purple-500"
                                                        : "border-l-2 border-transparent"
                                                } ${
                                                    isBest
                                                        ? "bg-purple-500/5"
                                                        : ""
                                                }`}
                                            >
                                                <span className="text-slate-300 font-mono flex items-center gap-1">
                                                    {hasBacktest ? (
                                                        isExpanded ? (
                                                            <ChevronDown className="w-3 h-3 text-slate-500" />
                                                        ) : (
                                                            <ChevronRight className="w-3 h-3 text-slate-500" />
                                                        )
                                                    ) : (
                                                        <span className="w-3" />
                                                    )}
                                                    {g.generation}
                                                </span>
                                                <span
                                                    className={`font-mono flex items-center gap-1 ${
                                                        g.fitness != null &&
                                                        g.fitness > 0
                                                            ? "text-emerald-400"
                                                            : "text-slate-400"
                                                    }`}
                                                >
                                                    {isBest && (
                                                        <span className="w-1.5 h-1.5 rounded-full bg-purple-400 inline-block" />
                                                    )}
                                                    {formatNumber(
                                                        g.fitness,
                                                        4
                                                    )}
                                                </span>
                                                <span
                                                    className={`font-mono ${
                                                        g.backtest
                                                            ? g.backtest
                                                                  .pnl_percent >=
                                                              0
                                                                ? "text-emerald-400"
                                                                : "text-red-400"
                                                            : "text-slate-600"
                                                    }`}
                                                >
                                                    {g.backtest
                                                        ? formatPercent(
                                                              g.backtest
                                                                  .pnl_percent
                                                          )
                                                        : "—"}
                                                </span>
                                                <span className="font-mono text-slate-400">
                                                    {g.backtest
                                                        ? formatNumber(
                                                              g.backtest
                                                                  .sharpe_ratio
                                                          )
                                                        : "—"}
                                                </span>
                                                <span
                                                    className={`font-mono ${
                                                        g.backtest
                                                            ? "text-red-400"
                                                            : "text-slate-600"
                                                    }`}
                                                >
                                                    {g.backtest
                                                        ? formatPercent(
                                                              g.backtest
                                                                  .max_drawdown
                                                          )
                                                        : "—"}
                                                </span>
                                                <span className="font-mono text-slate-400">
                                                    {g.backtest
                                                        ? formatPercent(
                                                              g.backtest
                                                                  .win_rate
                                                          )
                                                        : "—"}
                                                </span>
                                                <span className="text-slate-500 text-xs text-right">
                                                    {formatTimeAgo(
                                                        g.timestamp
                                                    )}
                                                </span>
                                            </button>

                                            {/* Expanded Detail */}
                                            {isExpanded && (
                                                <ExpandedBacktestDetail
                                                    generation={g}
                                                    detail={expandedDetail}
                                                    exchange={activeExchange}
                                                />
                                            )}
                                        </React.Fragment>
                                    );
                                })}

                                {generations.length === 0 && (
                                    <div className="px-4 py-8 text-center text-sm text-slate-500">
                                        No generations found for this exchange.
                                    </div>
                                )}
                            </div>
                        </div>
                    </div>
                </>
            )}
        </div>
    );
}

function ExpandedBacktestDetail({
    generation,
    detail,
    exchange,
}: {
    generation: Generation;
    detail: BacktestData | null;
    exchange: string;
}) {
    const bt = detail || generation.backtest;
    if (!bt) return null;

    const handleRerunBacktest = async () => {
        if (!generation.best_genome) return;
        try {
            await fetch(`/api/v1/backtest/run`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    genome: generation.best_genome,
                    token_address: "UNIVERSAL",
                    exchange,
                }),
            });
        } catch {
            // Backtest request failed
        }
    };

    return (
        <div className="bg-slate-800/20 border-t border-slate-800/50 px-6 py-4">
            <div className="grid grid-cols-2 gap-6">
                {/* Metrics Summary */}
                <div className="space-y-3">
                    <h5 className="text-xs text-slate-400 uppercase tracking-wide mb-2">
                        Backtest Metrics
                    </h5>
                    <div className="grid grid-cols-2 gap-3">
                        <MetricItem
                            label="PnL"
                            value={formatPercent(bt.pnl_percent)}
                            positive={bt.pnl_percent >= 0}
                        />
                        <MetricItem
                            label="Sharpe Ratio"
                            value={formatNumber(bt.sharpe_ratio)}
                        />
                        <MetricItem
                            label="Max Drawdown"
                            value={formatPercent(bt.max_drawdown)}
                            positive={false}
                        />
                        <MetricItem
                            label="Win Rate"
                            value={formatPercent(bt.win_rate)}
                        />
                        <MetricItem
                            label="Total Trades"
                            value={bt.total_trades?.toString() ?? "—"}
                        />
                    </div>
                    {generation.best_genome && (
                        <div className="mt-3">
                            <span className="text-xs text-slate-500 block mb-1">
                                Decoded Formula
                            </span>
                            <code className="text-xs text-slate-300 bg-slate-900/50 px-2 py-1 rounded block break-all">
                                {decodeGenome(generation.best_genome)}
                            </code>
                        </div>
                    )}
                    <button
                        onClick={handleRerunBacktest}
                        className="mt-2 px-3 py-1.5 text-xs font-medium rounded-lg bg-purple-500/20 border border-purple-500/50 text-purple-300 hover:bg-purple-500/30 transition-colors"
                    >
                        Re-run Backtest
                    </button>
                </div>

                {/* Equity Curve */}
                <div>
                    <h5 className="text-xs text-slate-400 uppercase tracking-wide mb-2">
                        Equity Curve
                    </h5>
                    {detail?.equity_curve && detail.equity_curve.length > 0 ? (
                        <div className="h-40">
                            <ResponsiveContainer width="100%" height="100%">
                                <AreaChart data={detail.equity_curve}>
                                    <defs>
                                        <linearGradient
                                            id="equityGradient"
                                            x1="0"
                                            y1="0"
                                            x2="0"
                                            y2="1"
                                        >
                                            <stop
                                                offset="5%"
                                                stopColor="#a78bfa"
                                                stopOpacity={0.3}
                                            />
                                            <stop
                                                offset="95%"
                                                stopColor="#a78bfa"
                                                stopOpacity={0}
                                            />
                                        </linearGradient>
                                    </defs>
                                    <CartesianGrid
                                        strokeDasharray="3 3"
                                        stroke="#334155"
                                    />
                                    <XAxis
                                        dataKey="timestamp"
                                        stroke="#64748b"
                                        fontSize={9}
                                        tickFormatter={(v) =>
                                            new Date(v).toLocaleDateString(
                                                undefined,
                                                {
                                                    month: "short",
                                                    day: "numeric",
                                                }
                                            )
                                        }
                                    />
                                    <YAxis
                                        stroke="#64748b"
                                        fontSize={9}
                                        tickFormatter={(v) =>
                                            `${(v * 100).toFixed(0)}%`
                                        }
                                    />
                                    <Tooltip
                                        contentStyle={{
                                            backgroundColor: "#1e293b",
                                            border: "1px solid #475569",
                                            borderRadius: "8px",
                                        }}
                                        labelStyle={{ color: "#cbd5e1" }}
                                        formatter={(value) => [
                                            typeof value === "number"
                                                ? `${(value * 100).toFixed(2)}%`
                                                : "—",
                                            "Equity",
                                        ]}
                                    />
                                    <Area
                                        type="monotone"
                                        dataKey="value"
                                        stroke="#a78bfa"
                                        strokeWidth={2}
                                        fill="url(#equityGradient)"
                                    />
                                </AreaChart>
                            </ResponsiveContainer>
                        </div>
                    ) : (
                        <div className="h-40 flex items-center justify-center text-sm text-slate-500">
                            {detail === null ? (
                                <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-purple-500" />
                            ) : (
                                "No equity curve data"
                            )}
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}

function MetricItem({
    label,
    value,
    positive,
}: {
    label: string;
    value: string;
    positive?: boolean;
}) {
    return (
        <div className="bg-slate-900/30 rounded-lg px-3 py-2">
            <span className="text-xs text-slate-500 block">{label}</span>
            <span
                className={`text-sm font-semibold ${
                    positive === true
                        ? "text-emerald-400"
                        : positive === false
                          ? "text-red-400"
                          : "text-white"
                }`}
            >
                {value}
            </span>
        </div>
    );
}
