"use client";

import React, { useState, useEffect, useRef } from "react";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";
import { Activity, ArrowUpRight, ArrowDownRight, DollarSign, Wallet, CloudLightning } from "lucide-react";
import { cn } from "@/lib/utils";

// Types matching MarketDataUpdate from Rust
interface MarketDataUpdate {
    symbol: string;
    price: number;
    volume: number;
    timestamp: string; // ISO 8601
    source: string;
}

interface PortfolioUpdate {
    timestamp: string;
    cash: number;
    positions: any[];
    total_equity: number;
}

interface SignalLog {
    id: number;
    symbol: string;
    side: "BUY" | "SELL";
    price: number;
    time: string;
}

interface PnlPoint {
    time: string;
    pnl: number;
}

export default function Dashboard() {
    const [balance, setBalance] = useState(0.00);
    const [pnlData, setPnlData] = useState<PnlPoint[]>([]);
    const [activeSignals, setActiveSignals] = useState<SignalLog[]>([]);
    const [logs, setLogs] = useState<string[]>([]);
    const [connected, setConnected] = useState(false);

    const wsRef = useRef<WebSocket | null>(null);

    useEffect(() => {
        // Connect to Data Engine WebSocket
        const ws = new WebSocket("ws://localhost:8080/ws");
        wsRef.current = ws;

        ws.onopen = () => {
            setConnected(true);
            addLog("System connected to Data Engine.");
        };

        ws.onclose = () => {
            setConnected(false);
            addLog("Disconnected from Data Engine.");
        };

        ws.onmessage = (event) => {
            try {
                const rawData = JSON.parse(event.data);

                // Check if it's a Portfolio Update (has total_equity)
                if (typeof rawData.total_equity === 'number') {
                    const portfolioData = rawData as PortfolioUpdate;
                    setBalance(portfolioData.total_equity);
                    // Update generic PnL chart live
                    setPnlData(prev => {
                        const newPoint = {
                            time: new Date(portfolioData.timestamp).toLocaleTimeString(),
                            pnl: portfolioData.total_equity
                        };
                        return [...prev.slice(-50), newPoint];
                    });
                    // Log it occasionally or just update silently? 
                    // For debugging let's log once if it changes significantly? No, spammy.
                } else if (rawData.action && rawData.message) {
                    // It's a Strategy Log
                    const log = rawData as any; // Quick cast
                    const timestamp = new Date(log.timestamp).toLocaleTimeString();
                    addLog(`[${timestamp}] [${log.action}] ${log.symbol}: ${log.message}`);
                } else if (rawData.symbol) {
                    // It's likely market data
                    handleMarketData(rawData as MarketDataUpdate);
                }
            } catch (e) {
                console.error("Failed to parse WS message", e);
            }
        };

        return () => {
            ws.close();
        };
    }, []);

    const addLog = (msg: string) => {
        setLogs(prev => [`[${new Date().toLocaleTimeString()}] ${msg}`, ...prev.slice(0, 50)]);
    };

    const handleMarketData = (data: MarketDataUpdate) => {
        // Simulate Strategy Logic for Visualization (in reality, this comes from execution-engine events)
        // For Phase 4 Demo: Just show the ticker updates as logs or "signals" if price creates a new high/low

        // For visualization: Update "Equity" randomly based on price moves just to show liveliness
        // Real implementation would listen to "PortfolioUpdate" events

        // Example: If it's a known symbol
        if (data.symbol === "SPY" || data.symbol === "AAPL" || data.symbol.includes("USDC")) {
            // Just log it
            // addLog(`Tick: ${data.symbol} @ ${data.price}`);
        }
    };

    // Mock auto-generating signals to prove UI works while waiting for real backtest integration
    // Remove this when fully live
    useEffect(() => {
        /* 
           This mock effect can be removed. 
           We rely on WS `handleMarketData` or specific `TradeSignal` events if we add them to WS.
           Currently WS only sends `MarketDataUpdate`.
           To visualize signals, we need `TradeSignal` events on the WS too.
           For now, let's keep the UI static or minimal until we pipe TradeSignals to WS.
        */
    }, []);

    return (
        <div className="min-h-screen bg-neutral-950 text-white p-6 font-sans">
            <header className="flex justify-between items-center mb-8 border-b border-neutral-800 pb-4">
                <div>
                    <h1 className="text-2xl font-bold bg-gradient-to-r from-blue-400 to-purple-500 bg-clip-text text-transparent">
                        HermesFlow
                    </h1>
                    <p className="text-neutral-400 text-sm">AlphaGPT Microservices Migration</p>
                </div>
                <div className="flex items-center gap-4">
                    <div className={cn("flex items-center gap-2 px-4 py-2 rounded-full border", connected ? "bg-green-900/20 border-green-800" : "bg-red-900/20 border-red-800")}>
                        <div className={cn("w-2 h-2 rounded-full animate-pulse", connected ? "bg-green-500" : "bg-red-500")} />
                        <span className={cn("text-sm font-medium", connected ? "text-green-400" : "text-red-400")}>{connected ? "System Online" : "Disconnected"}</span>
                    </div>
                </div>
            </header>

            <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
                {/* Key Metrics */}
                <div className="md:col-span-1 space-y-4">
                    <MetricCard
                        title="Total Equity"
                        value={`${balance.toLocaleString(undefined, { minimumFractionDigits: 2 })} SOL`}
                        icon={<Wallet className="text-blue-400" />}
                        change="+0.0%"
                    />
                    <MetricCard
                        title="Active Data Streams"
                        value={connected ? "Active" : "Offline"}
                        icon={<CloudLightning className="text-yellow-400" />}
                        change=""
                    />
                </div>

                {/* Main Chart */}
                <div className="md:col-span-3 bg-neutral-900/50 border border-neutral-800 rounded-xl p-6">
                    <h3 className="text-lg font-semibold mb-4 text-neutral-200">Equity Curve (Live)</h3>
                    <div className="h-[300px] w-full">
                        <ResponsiveContainer width="100%" height="100%">
                            <LineChart data={pnlData}>
                                <CartesianGrid strokeDasharray="3 3" stroke="#333" />
                                <XAxis dataKey="time" stroke="#666" />
                                <YAxis stroke="#666" domain={['auto', 'auto']} />
                                <Tooltip
                                    contentStyle={{ backgroundColor: "#111", border: "1px solid #333" }}
                                    itemStyle={{ color: "#fff" }}
                                />
                                <Line
                                    type="stepAfter"
                                    dataKey="pnl"
                                    stroke="#8b5cf6"
                                    strokeWidth={2}
                                    dot={false}
                                />
                            </LineChart>
                        </ResponsiveContainer>
                    </div>
                </div>

                {/* Recent Signals / Ticks */}
                <div className="md:col-span-2 bg-neutral-900/50 border border-neutral-800 rounded-xl p-6">
                    <h3 className="text-lg font-semibold mb-4 text-neutral-200">Recent Signals</h3>
                    <div className="space-y-3">
                        {activeSignals.length === 0 && <div className="text-neutral-500 italic">No signals yet...</div>}
                        {activeSignals.map((signal) => (
                            <div key={signal.id} className="flex justify-between items-center p-3 bg-neutral-900 rounded-lg border border-neutral-800/50">
                                <div>{signal.symbol} {signal.side}</div>
                                <div>{signal.price}</div>
                            </div>
                        ))}
                    </div>
                </div>

                {/* Logs / Terminal */}
                <div className="md:col-span-2 bg-neutral-950 border border-neutral-800 rounded-xl p-6 font-mono text-sm">
                    <h3 className="text-lg font-semibold mb-4 text-neutral-200 font-sans">System Logs</h3>
                    <div className="space-y-2 h-[200px] overflow-y-auto text-neutral-400 custom-scrollbar">
                        {logs.map((log, i) => (
                            <div key={i}>{log}</div>
                        ))}
                    </div>
                </div>
            </div>
        </div>
    );
}

function MetricCard({ title, value, icon, change }: any) {
    return (
        <div className="bg-neutral-900/50 border border-neutral-800 rounded-xl p-4 flex items-center justify-between">
            <div>
                <div className="text-neutral-400 text-sm mb-1">{title}</div>
                <div className="text-2xl font-bold">{value}</div>
                <div className="text-green-400 text-xs mt-1">{change}</div>
            </div>
            <div className="p-3 bg-neutral-800/50 rounded-lg">
                {icon}
            </div>
        </div>
    );
}
