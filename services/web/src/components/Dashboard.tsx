"use client";

import React, { useState, useEffect } from "react";
import { Activity, TrendingUp, Database, Zap, AlertCircle } from "lucide-react";
import StrategyMonitor from "@/components/StrategyMonitor";
import DataPipeline, { DataMetrics } from "@/components/DataPipeline";
import TradeExecutionPanel from "@/components/TradeExecutionPanel";
import SystemLogs, { LogEntry } from "@/components/SystemLogs";
import StrategyLab from "@/components/StrategyLab";
import { cn } from "@/lib/utils";
import SystemStatus from "@/components/SystemStatus";

// Types
interface TradeSignal {
    id: string;
    timestamp: string;
    symbol: string;
    side: "BUY" | "SELL";
    price: number;
    quantity: number;
    status: "PENDING" | "FILLED" | "REJECTED";
}

export default function Dashboard() {
    const [wsConnected, setWsConnected] = useState(false);
    const [systemHealth, setSystemHealth] = useState<"healthy" | "degraded" | "offline">("offline");
    const [activeTab, setActiveTab] = useState<"overview" | "strategy-lab" | "system">("overview");

    // Centralized State
    const [signals, setSignals] = useState<TradeSignal[]>([]);
    const [portfolioValue, setPortfolioValue] = useState(0);
    const [pnl24h, setPnl24h] = useState(0);
    const [logs, setLogs] = useState<LogEntry[]>([]);
    const [heartbeats, setHeartbeats] = useState<Record<string, number>>({});
    const [metrics, setMetrics] = useState<DataMetrics>({
        heliusConnected: false,
        activeTokens: 0,
        staleSymbols: 0,
        gapSymbols: 0,
        lowLiqSymbols: 0,
    });

    // Strategy State
    const [currentGen, setCurrentGen] = useState(0);
    const [currentFitness, setCurrentFitness] = useState<number | null>(null);
    const [bestFormula, setBestFormula] = useState<number[]>([]);
    const [fitnessHistory, setFitnessHistory] = useState<{ gen: number; fitness: number }[]>([]);

    useEffect(() => {
        // WebSocket connection to data-engine via Gateway
        const ws = new WebSocket("ws://localhost:8080/ws");

        ws.onopen = () => {
            setWsConnected(true);
            setSystemHealth("healthy");
            setMetrics(p => ({ ...p, heliusConnected: true }));
        };

        ws.onclose = () => {
            setWsConnected(false);
            setSystemHealth("offline");
            setMetrics(p => ({ ...p, heliusConnected: false }));
        };

        ws.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                const { type, data } = message;

                if (type === "heartbeat") {
                    setHeartbeats(prev => ({
                        ...prev,
                        [data.service]: data.timestamp
                    }));
                }

                if (type === "signal") {
                    // ... existing signal code ...
                    // Trade Signal
                    // Map backend TradeSignal (id, side=Buy/Sell, etc) to frontend shape
                    const signal: TradeSignal = {
                        id: data.id,
                        timestamp: data.timestamp,
                        symbol: data.symbol,
                        side: data.side === "Buy" ? "BUY" : "SELL", // Rust uses "Buy"/"Sell", frontend expects "BUY"/"SELL"
                        price: data.price || 0,
                        quantity: data.quantity,
                        status: "PENDING" // Default
                    };
                    setSignals(prev => [signal, ...prev.slice(0, 19)]);

                    // Add to logs too
                    const log: LogEntry = {
                        timestamp: new Date().toLocaleTimeString(),
                        level: "INFO",
                        message: `Signal Received: ${signal.side} ${signal.symbol} @ ${signal.price}`,
                        module: "STRATEGY"
                    };
                    setLogs(prev => [log, ...prev.slice(0, 99)]);
                }

                if (type === "portfolio") {
                    if (data.cash !== undefined) {
                        setPortfolioValue(data.cash); // Simplified for MVP
                        // TODO: Calculate PnL from history
                    }
                }

                if (type === "market") {
                    // Market Data Update - Update metrics?
                    // if data has active count etc.
                }

                if (type === "metrics") {
                    if (data.active_tokens !== undefined) {
                        setMetrics(prev => ({ ...prev, activeTokens: data.active_tokens }));
                    }
                }

                if (type === "log" || data.strategy_id === "EvolutionaryKernel") {
                    // Handle Strategy Status
                    if (data.action === "Evolving") {
                        const genMatch = data.message?.match(/Gen (\d+)/);
                        if (genMatch) {
                            setCurrentGen(parseInt(genMatch[1]));
                        }
                    }

                    // Add to System Logs
                    const log: LogEntry = {
                        timestamp: new Date().toLocaleTimeString(),
                        level: data.level === "ERROR" ? "ERROR" : (data.level === "WARN" ? "WARN" : "INFO"),
                        message: data.message || JSON.stringify(data),
                        module: data.strategy_id ? "STRATEGY" : (data.target?.includes("data") ? "DATA" : "SYSTEM")
                    };
                    setLogs(prev => [log, ...prev.slice(0, 99)]);
                }

            } catch (e) {
                // console.error("Parse error", e);
            }
        };

        return () => ws.close();
    }, []);

    return (
        <div className="min-h-screen bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-slate-900 via-[#030712] to-[#030712] text-white selection:bg-indigo-500/30">
            {/* Header */}
            <header className="border-b border-white/5 backdrop-blur-md bg-slate-950/50 sticky top-0 z-50">
                <div className="container mx-auto px-6 h-16 flex items-center justify-between">
                    <div className="flex items-center gap-4">
                        <div className="relative group">
                            <div className="absolute -inset-1 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-lg blur opacity-25 group-hover:opacity-75 transition duration-1000 group-hover:duration-200"></div>
                            <div className="relative w-10 h-10 rounded-lg bg-slate-900 flex items-center justify-center border border-white/10">
                                <Zap className="w-5 h-5 text-indigo-400" />
                            </div>
                        </div>
                        <div>
                            <h1 className="text-xl font-bold bg-gradient-to-r from-white to-slate-400 bg-clip-text text-transparent">
                                HermesFlow
                            </h1>
                            <p className="text-[10px] uppercase tracking-wider text-slate-500 font-semibold">Autonomous Trading System</p>
                        </div>
                    </div>

                    {/* Navigation Tabs (Centered) */}
                    <div className="hidden md:flex bg-white/5 rounded-full p-1 border border-white/5 backdrop-blur">
                        <button
                            onClick={() => setActiveTab("overview")}
                            className={cn(
                                "px-6 py-1.5 rounded-full text-sm font-medium transition-all duration-300",
                                activeTab === "overview"
                                    ? "bg-indigo-500 text-white shadow-lg shadow-indigo-500/20"
                                    : "text-slate-400 hover:text-white hover:bg-white/5"
                            )}
                        >
                            Overview
                        </button>
                        <button
                            onClick={() => setActiveTab("strategy-lab")}
                            className={cn(
                                "px-6 py-1.5 rounded-full text-sm font-medium transition-all duration-300",
                                activeTab === "strategy-lab"
                                    ? "bg-purple-600 text-white shadow-lg shadow-purple-600/20"
                                    : "text-slate-400 hover:text-white hover:bg-white/5"
                            )}
                        >
                            Strategy Lab
                        </button>
                        <button
                            onClick={() => setActiveTab("system")}
                            className={cn(
                                "px-6 py-1.5 rounded-full text-sm font-medium transition-all duration-300",
                                activeTab === "system"
                                    ? "bg-emerald-600 text-white shadow-lg shadow-emerald-600/20"
                                    : "text-slate-400 hover:text-white hover:bg-white/5"
                            )}
                        >
                            System Status
                        </button>
                    </div>

                    {/* System Status */}
                    <div className="flex items-center gap-4">
                        <StatusBadge
                            label="Data Engine"
                            status={wsConnected ? "online" : "offline"}
                            icon={<Database className="w-3.5 h-3.5" />}
                        />
                        <StatusBadge
                            label="System Health"
                            status={systemHealth}
                            icon={<Activity className="w-3.5 h-3.5" />}
                        />
                    </div>
                </div>
            </header>

            {/* Main Content */}
            <main className="container mx-auto px-6 py-8">
                {activeTab === "overview" && (
                    <div className="grid grid-cols-12 gap-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
                        {/* Left Column: Strategy Monitor */}
                        <div className="col-span-12 lg:col-span-4 space-y-6">
                            <StrategyMonitor
                                currentGen={currentGen}
                                currentFitness={currentFitness}
                                bestFormula={bestFormula}
                                fitnessHistory={fitnessHistory}
                                evolutionRate={0} // TODO: calc rate
                                isEvolving={currentGen > 0}
                            />
                        </div>

                        {/* Middle Column: Data Pipeline + Execution (Expanded) */}
                        <div className="col-span-12 lg:col-span-8 space-y-6">
                            <DataPipeline metrics={metrics} />
                            <TradeExecutionPanel
                                signals={signals}
                                portfolioValue={portfolioValue}
                                pnl24h={pnl24h}
                            />
                        </div>
                    </div>
                )}

                {activeTab === "strategy-lab" && (
                    <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
                        <StrategyLab />
                    </div>
                )}

                {activeTab === "system" && (
                    <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
                        <SystemStatus
                            logs={logs}
                            heartbeats={heartbeats}
                            metrics={{
                                activeTokens: metrics.activeTokens,
                                heliusConnected: metrics.heliusConnected,
                                wsConnected: wsConnected
                            }}
                        />
                    </div>
                )}
            </main>
        </div>
    );
}

function StatusBadge({ label, status, icon }: { label: string; status: string; icon: React.ReactNode }) {
    const statusConfig = {
        online: "bg-emerald-500/10 border-emerald-500/20 text-emerald-400",
        healthy: "bg-emerald-500/10 border-emerald-500/20 text-emerald-400",
        degraded: "bg-yellow-500/10 border-yellow-500/20 text-yellow-400",
        offline: "bg-red-500/10 border-red-500/20 text-red-400",
    };

    const isActive = status === "online" || status === "healthy";
    const config = statusConfig[status as keyof typeof statusConfig] || statusConfig.offline;

    return (
        <div className={cn("flex items-center gap-2 px-3 py-1.5 rounded-full border text-xs font-medium transition-all duration-300", config)}>
            {icon}
            <span>{label}</span>
            <span className="relative flex h-2 w-2">
                {isActive && <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>}
                <span className={cn("relative inline-flex rounded-full h-2 w-2", isActive ? "bg-emerald-400" : "bg-red-400")}></span>
            </span>
        </div>
    );
}
