"use client";

import React, { useState, useEffect } from "react";
import { Database, CloudLightning, AlertTriangle, CheckCircle2 } from "lucide-react";

interface DataMetrics {
    heliusConnected: boolean;
    activeTokens: number;
    staleSymbols: number;
    gapSymbols: number;
    lowLiqSymbols: number;
}

export default function DataPipeline() {
    const [metrics, setMetrics] = useState<DataMetrics>({
        heliusConnected: false,
        activeTokens: 0,
        staleSymbols: 0,
        gapSymbols: 0,
        lowLiqSymbols: 0,
    });

    useEffect(() => {
        const ws = new WebSocket("ws://localhost:8080/ws");

        ws.onopen = () => {
            setMetrics((prev) => ({ ...prev, heliusConnected: true }));
        };

        ws.onclose = () => {
            setMetrics((prev) => ({ ...prev, heliusConnected: false }));
        };

        // Poll Prometheus metrics endpoint
        const fetchMetrics = async () => {
            try {
                const res = await fetch("/metrics");
                const text = await res.text();

                // Parse Prometheus format (simple regex for demo)
                const activeTokensMatch = text.match(/(?:dq_active_symbols|data_engine_active_symbols_count) (\d+)/);
                const staleMatch = text.match(/dq_stale_symbols (\d+)/);
                const gapMatch = text.match(/dq_gap_symbols (\d+)/);
                const lowLiqMatch = text.match(/dq_low_liquidity_symbols (\d+)/);

                setMetrics((prev) => ({
                    ...prev,
                    activeTokens: activeTokensMatch ? parseInt(activeTokensMatch[1]) : prev.activeTokens,
                    staleSymbols: staleMatch ? parseInt(staleMatch[1]) : prev.staleSymbols,
                    gapSymbols: gapMatch ? parseInt(gapMatch[1]) : prev.gapSymbols,
                    lowLiqSymbols: lowLiqMatch ? parseInt(lowLiqMatch[1]) : prev.lowLiqSymbols,
                }));
            } catch (e) {
                console.error("Failed to fetch metrics", e);
            }
        };

        fetchMetrics();
        const interval = setInterval(fetchMetrics, 10000); // Poll every 10s

        return () => {
            ws.close();
            clearInterval(interval);
        };
    }, []);

    const hasIssues = metrics.staleSymbols > 0 || metrics.gapSymbols > 0 || metrics.lowLiqSymbols > 0;

    return (
        <div className="bg-slate-900/50 backdrop-blur border border-slate-800/50 rounded-2xl p-6 shadow-2xl">
            {/* Header */}
            <div className="flex items-center justify-between mb-6">
                <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-cyan-500 to-blue-600 flex items-center justify-center">
                        <Database className="w-5 h-5" />
                    </div>
                    <div>
                        <h3 className="text-lg font-semibold text-white">Data Pipeline</h3>
                        <p className="text-xs text-slate-400">Real-time Ingestion</p>
                    </div>
                </div>
            </div>

            {/* Connection Status */}
            <div className="space-y-4 mb-6">
                <StatusRow
                    label="Helius WebSocket"
                    status={metrics.heliusConnected ? "Connected" : "Disconnected"}
                    icon={<CloudLightning className="w-4 h-4" />}
                    healthy={metrics.heliusConnected}
                />
                <StatusRow
                    label="Active Tokens"
                    status={`${metrics.activeTokens} tracked`}
                    icon={<Database className="w-4 h-4" />}
                    healthy={metrics.activeTokens > 0}
                />
            </div>

            {/* Data Quality Alerts */}
            <div>
                <h4 className="text-sm font-medium text-slate-300 mb-3">Data Quality</h4>
                <div className="space-y-2">
                    {metrics.staleSymbols > 0 && (
                        <AlertBadge severity="warning" message={`${metrics.staleSymbols} stale symbols detected`} />
                    )}
                    {metrics.gapSymbols > 0 && (
                        <AlertBadge severity="warning" message={`${metrics.gapSymbols} symbols with data gaps`} />
                    )}
                    {metrics.lowLiqSymbols > 0 && (
                        <AlertBadge severity="error" message={`${metrics.lowLiqSymbols} tokens below liquidity threshold`} />
                    )}
                    {!hasIssues && (
                        <div className="flex items-center gap-2 text-sm text-emerald-400">
                            <CheckCircle2 className="w-4 h-4" />
                            <span>All quality checks passed</span>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}

function StatusRow({ label, status, icon, healthy }: { label: string; status: string; icon: React.ReactNode; healthy: boolean }) {
    return (
        <div className="flex items-center justify-between p-3 rounded-lg bg-slate-800/50 border border-slate-700/50">
            <div className="flex items-center gap-3">
                <div className={`p-2 rounded-lg ${healthy ? "bg-emerald-500/20 text-emerald-400" : "bg-red-500/20 text-red-400"}`}>
                    {icon}
                </div>
                <div>
                    <div className="text-sm font-medium text-white">{label}</div>
                    <div className="text-xs text-slate-400">{status}</div>
                </div>
            </div>
            <div className={`w-2 h-2 rounded-full ${healthy ? "bg-emerald-400 animate-pulse" : "bg-red-400"}`} />
        </div>
    );
}

function AlertBadge({ severity, message }: { severity: "warning" | "error"; message: string }) {
    const colors = {
        warning: "bg-yellow-500/20 border-yellow-500/50 text-yellow-400",
        error: "bg-red-500/20 border-red-500/50 text-red-400",
    };

    return (
        <div className={`flex items-center gap-2 p-2 rounded-lg border ${colors[severity]}`}>
            <AlertTriangle className="w-4 h-4" />
            <span className="text-xs font-medium">{message}</span>
        </div>
    );
}
