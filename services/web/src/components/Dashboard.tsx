"use client";

import React, { useState, useEffect } from "react";
import { Activity, TrendingUp, Database, Zap, LogOut, LayoutDashboard, Beaker, Search, Server, Settings } from "lucide-react";
import { useRouter } from "next/navigation";
import DataPipeline, { DataMetrics } from "@/components/DataPipeline";
import TradeExecutionPanel from "@/components/TradeExecutionPanel";
import SystemLogs, { LogEntry } from "@/components/SystemLogs";
import StrategyLab from "@/components/StrategyLab";
import { cn } from "@/lib/utils";
import SystemStatus from "@/components/SystemStatus";
import DataDiscovery from "@/components/DataDiscovery";
import dynamic from 'next/dynamic';
// Dynamic import to prevent SSR hydration issues with lightweight-charts
const MarketOverview = dynamic(() => import('@/components/MarketOverview'), {
    ssr: false,
    loading: () => (
        <div className="flex items-center justify-center h-full">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-500"></div>
        </div>
    )
});

// Settings Component
const ExchangeConfig = dynamic(() => import('@/components/Settings/ExchangeConfig'), { ssr: false });

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
    const router = useRouter();
    const [wsConnected, setWsConnected] = useState(false);
    const [systemHealth, setSystemHealth] = useState<"healthy" | "degraded" | "offline">("offline");
    const [activeTab, setActiveTab] = useState<"overview" | "market" | "strategy-lab" | "system" | "data-discovery" | "settings">("overview");

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
                        setMetrics(prev => ({
                            ...prev,
                            activeTokens: data.active_tokens,
                            birdeyeRequests: data.birdeye_requests
                        }));
                    }
                }

                if (type === "log") {
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

    const handleLogout = () => {
        localStorage.removeItem("token");
        localStorage.removeItem("lastActivity");
        router.push("/login");
    };

    return (
        <div className="flex h-screen bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-slate-900 via-[#030712] to-[#030712] text-white selection:bg-indigo-500/30 overflow-hidden">
            {/* Sidebar */}
            <aside className="w-64 bg-slate-950/50 backdrop-blur-xl border-r border-white/10 flex flex-col shadow-2xl z-50">
                {/* Logo Area */}
                <div className="h-16 flex items-center gap-3 px-6 border-b border-white/5 bg-slate-900/50">
                    <div className="relative group">
                        <div className="absolute -inset-1 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-lg blur opacity-25 group-hover:opacity-75 transition duration-1000 group-hover:duration-200"></div>
                        <div className="relative w-8 h-8 rounded-lg bg-slate-900 flex items-center justify-center border border-white/10">
                            <Zap className="w-4 h-4 text-indigo-400" />
                        </div>
                    </div>
                    <div>
                        <h1 className="text-lg font-bold bg-gradient-to-r from-white to-slate-400 bg-clip-text text-transparent">
                            HermesFlow
                        </h1>
                    </div>
                </div>

                {/* Navigation Items */}
                <nav className="flex-1 p-4 space-y-2 overflow-y-auto">
                    <NavItem
                        active={activeTab === "overview"}
                        onClick={() => setActiveTab("overview")}
                        icon={<LayoutDashboard className="w-5 h-5" />}
                        label="Overview"
                        color="indigo"
                    />
                    <NavItem
                        active={activeTab === "market"}
                        onClick={() => setActiveTab("market")}
                        icon={<TrendingUp className="w-5 h-5" />}
                        label="Market"
                        color="cyan"
                    />
                    <NavItem
                        active={activeTab === "strategy-lab"}
                        onClick={() => setActiveTab("strategy-lab")}
                        icon={<Beaker className="w-5 h-5" />}
                        label="Strategy Lab"
                        color="purple"
                    />
                    <NavItem
                        active={activeTab === "data-discovery"}
                        onClick={() => setActiveTab("data-discovery")}
                        icon={<Search className="w-5 h-5" />}
                        label="Data Discovery"
                        color="blue"
                    />
                    <NavItem
                        active={activeTab === "system"}
                        onClick={() => setActiveTab("system")}
                        icon={<Server className="w-5 h-5" />}

                        label="System Status"
                        color="emerald"
                    />
                    <div className="my-2 border-t border-white/5 mx-4"></div>
                    <NavItem
                        active={activeTab === "settings"}
                        onClick={() => setActiveTab("settings")}
                        icon={<Settings className="w-5 h-5" />}
                        label="Settings"
                        color="slate"
                    />
                </nav>

                {/* Logout Area */}
                <div className="p-4 border-t border-white/5 bg-slate-900/30">
                    <button
                        onClick={handleLogout}
                        className="flex items-center gap-3 w-full px-4 py-3 rounded-lg text-slate-400 hover:text-white hover:bg-red-500/10 hover:border-red-500/20 border border-transparent transition-all duration-200 group"
                    >
                        <LogOut className="w-5 h-5 group-hover:text-red-400 transition-colors" />
                        <span className="font-medium group-hover:text-red-100">Logout</span>
                        <span className="ml-auto text-xs bg-white/5 px-2 py-0.5 rounded text-slate-500 group-hover:text-red-300">Exit</span>
                    </button>
                </div>
            </aside>

            {/* Main Content Area */}
            <div className="flex-1 flex flex-col overflow-hidden relative">
                {/* Top Bar (Status) */}
                <div className="h-16 flex items-center justify-end gap-4 px-8 border-b border-white/5 bg-slate-950/30 backdrop-blur-sm">
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

                {/* Content Scrollable */}
                <main className="flex-1 overflow-y-auto p-8 scrollbar-thin scrollbar-thumb-white/10 scrollbar-track-transparent">
                    {/* Header Title for Context - Hide on Market tab (custom header inside) */}
                    {activeTab !== "market" && (
                        <div className="mb-8">
                            <h2 className="text-2xl font-bold text-white tracking-tight">
                                {activeTab === "overview" && "Mission Control"}
                                {activeTab === "strategy-lab" && "Alpha Strategy Lab"}
                                {activeTab === "data-discovery" && "Data Intelligence"}
                                {activeTab === "system" && "System Operations"}
                                {activeTab === "settings" && "Settings"}
                            </h2>
                            <p className="text-slate-400 mt-1">
                                {activeTab === "overview" && "Real-time market surveillance and execution monitoring."}
                                {activeTab === "strategy-lab" && "Design, backtest, and optimize trading strategies."}
                                {activeTab === "data-discovery" && "Explore market data, active tokens, and quality metrics."}
                                {activeTab === "system" && "Monitor infrastructure health and system logs."}
                                {activeTab === "settings" && "Configure external exchange connections and data preferences."}
                            </p>
                        </div>
                    )}

                    {activeTab === "overview" && (
                        <div className="space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
                            <div className="grid grid-cols-12 gap-6">
                                <div className="col-span-12">
                                    <DataPipeline metrics={metrics} />
                                </div>
                                <div className="col-span-12">
                                    <TradeExecutionPanel
                                        signals={signals}
                                        portfolioValue={portfolioValue}
                                        pnl24h={pnl24h}
                                    />
                                </div>
                            </div>
                        </div>
                    )}

                    {/* Market Tab - Always mounted to prevent chart destruction */}
                    <div className={cn(
                        "animate-in fade-in slide-in-from-bottom-4 duration-500 h-full",
                        activeTab !== "market" && "hidden"
                    )}>
                        <MarketOverview />
                    </div>

                    {activeTab === "strategy-lab" && (
                        <div className="animate-in fade-in slide-in-from-bottom-4 duration-500 h-full">
                            <StrategyLab />
                        </div>
                    )}

                    {activeTab === "data-discovery" && (
                        <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
                            <DataDiscovery />
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

                    {activeTab === "settings" && (
                        <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
                            <ExchangeConfig />
                        </div>
                    )}
                </main>
            </div>
        </div>
    );
}

// Subcomponents

function NavItem({ active, onClick, icon, label, color }: { active: boolean; onClick: () => void; icon: React.ReactNode; label: string; color: string }) {
    const colorStyles: Record<string, string> = {
        indigo: "text-indigo-400 group-hover:text-indigo-300",
        purple: "text-purple-400 group-hover:text-purple-300",
        blue: "text-blue-400 group-hover:text-blue-300",
        emerald: "text-emerald-400 group-hover:text-emerald-300",
        cyan: "text-cyan-400 group-hover:text-cyan-300",
        slate: "text-slate-400 group-hover:text-slate-300",
    };

    const activeBg: Record<string, string> = {
        indigo: "bg-indigo-500/10 border-indigo-500/50 text-white",
        purple: "bg-purple-500/10 border-purple-500/50 text-white",
        blue: "bg-blue-500/10 border-blue-500/50 text-white",
        emerald: "bg-emerald-500/10 border-emerald-500/50 text-white",
        cyan: "bg-cyan-500/10 border-cyan-500/50 text-white",
        slate: "bg-slate-500/10 border-slate-500/50 text-white",
    };

    const activeIndicator: Record<string, string> = {
        indigo: "bg-indigo-500 shadow-[0_0_10px_rgba(99,102,241,0.5)]",
        purple: "bg-purple-500 shadow-[0_0_10px_rgba(168,85,247,0.5)]",
        blue: "bg-blue-500 shadow-[0_0_10px_rgba(59,130,246,0.5)]",
        emerald: "bg-emerald-500 shadow-[0_0_10px_rgba(16,185,129,0.5)]",
        cyan: "bg-cyan-500 shadow-[0_0_10px_rgba(6,182,212,0.5)]",
        slate: "bg-slate-500 shadow-[0_0_10px_rgba(100,116,139,0.5)]",
    };

    return (
        <button
            onClick={onClick}
            className={cn(
                "flex items-center gap-3 w-full px-4 py-3 rounded-xl font-medium transition-all duration-300 border group relative overflow-hidden",
                active
                    ? activeBg[color] || "bg-slate-800 border-white/20 text-white"
                    : "border-transparent text-slate-400 hover:text-white hover:bg-white/5"
            )}
        >
            {active && (
                <div className={cn("absolute left-0 top-1/2 -translate-y-1/2 w-1 h-3/5 rounded-r-full", activeIndicator[color])}></div>
            )}

            <span className={cn("transition-colors relative z-10", active ? "text-white" : colorStyles[color])}>
                {icon}
            </span>
            <span className="relative z-10">{label}</span>
        </button>
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
