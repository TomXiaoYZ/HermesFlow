"use client";

import React, { useState, useEffect } from "react";
import { TrendingUp, TrendingDown, DollarSign, Clock } from "lucide-react";

interface TradeSignal {
    id: string;
    timestamp: string;
    symbol: string;
    side: "BUY" | "SELL";
    price: number;
    quantity: number;
    status: "PENDING" | "FILLED" | "REJECTED";
}

export interface TradeExecutionPanelProps {
    signals: TradeSignal[];
    portfolioValue: number;
    pnl24h: number;
}

export default function TradeExecutionPanel({ signals, portfolioValue, pnl24h }: TradeExecutionPanelProps) {

    return (
        <div className="bg-slate-900/50 backdrop-blur border border-slate-800/50 rounded-2xl p-6 shadow-2xl">
            {/* Header */}
            <div className="flex items-center justify-between mb-6">
                <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-500 to-teal-600 flex items-center justify-center">
                        <DollarSign className="w-5 h-5" />
                    </div>
                    <div>
                        <h3 className="text-lg font-semibold text-white">Trade Execution</h3>
                        <p className="text-xs text-slate-400">Live Signals</p>
                    </div>
                </div>
            </div>

            {/* Portfolio Summary */}
            <div className="grid grid-cols-2 gap-4 mb-6">
                <div className="p-4 rounded-xl bg-gradient-to-br from-emerald-500/10 to-teal-600/10 border border-emerald-500/30">
                    <div className="text-xs text-slate-400 mb-1">Total Equity</div>
                    <div className="text-2xl font-bold text-white">{portfolioValue.toFixed(2)} SOL</div>
                </div>
                <div className={`p-4 rounded-xl ${pnl24h >= 0 ? "bg-emerald-500/10 border-emerald-500/30" : "bg-red-500/10 border-red-500/30"} border`}>
                    <div className="text-xs text-slate-400 mb-1">PnL (24h)</div>
                    <div className={`text-2xl font-bold ${pnl24h >= 0 ? "text-emerald-400" : "text-red-400"}`}>
                        {pnl24h >= 0 ? "+" : ""}
                        {pnl24h.toFixed(2)}%
                    </div>
                </div>
            </div>

            {/* Recent Signals */}
            <div>
                <h4 className="text-sm font-medium text-slate-300 mb-3">Recent Signals</h4>
                <div className="space-y-2 max-h-64 overflow-y-auto custom-scrollbar">
                    {signals.length === 0 ? (
                        <div className="text-center py-8 text-slate-500 text-sm italic">No signals yet...</div>
                    ) : (
                        signals.map((signal) => (
                            <SignalCard key={signal.id} signal={signal} />
                        ))
                    )}
                </div>
            </div>
        </div>
    );
}

function SignalCard({ signal }: { signal: TradeSignal }) {
    const isBuy = signal.side === "BUY";
    const statusColors = {
        PENDING: "bg-yellow-500/20 border-yellow-500/50 text-yellow-400",
        FILLED: "bg-emerald-500/20 border-emerald-500/50 text-emerald-400",
        REJECTED: "bg-red-500/20 border-red-500/50 text-red-400",
    };

    return (
        <div className="flex items-center justify-between p-3 rounded-lg bg-slate-800/50 border border-slate-700/50 hover:border-slate-600 transition-colors">
            <div className="flex items-center gap-3">
                <div className={`p-2 rounded-lg ${isBuy ? "bg-emerald-500/20 text-emerald-400" : "bg-red-500/20 text-red-400"}`}>
                    {isBuy ? <TrendingUp className="w-4 h-4" /> : <TrendingDown className="w-4 h-4" />}
                </div>
                <div>
                    <div className="text-sm font-medium text-white">{signal.symbol}</div>
                    <div className="text-xs text-slate-400 flex items-center gap-2">
                        <Clock className="w-3 h-3" />
                        {new Date(signal.timestamp).toLocaleTimeString()}
                    </div>
                </div>
            </div>
            <div className="text-right">
                <div className="text-sm font-semibold text-white">{signal.price.toFixed(4)}</div>
                <div className={`text-xs px-2 py-0.5 rounded-full border ${statusColors[signal.status]}`}>{signal.status}</div>
            </div>
        </div>
    );
}
