"use client";

import React, { useState, useEffect } from "react";
import { TrendingUp, Brain, Activity, Sparkles } from "lucide-react";
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";

interface StrategyStatus {
    generation: number;
    fitness: number | null;
    bestTokens: number[];
    timestamp: string;
}

export interface StrategyMonitorProps {
    currentGen: number;
    currentFitness: number | null;
    bestFormula: number[];
    fitnessHistory: { gen: number; fitness: number }[];
    evolutionRate: number;
    isEvolving: boolean;
}

export default function StrategyMonitor({
    currentGen,
    currentFitness,
    bestFormula,
    fitnessHistory,
    evolutionRate,
    isEvolving
}: StrategyMonitorProps) {
    const hasFitness = currentFitness !== null && currentFitness > -999;

    return (
        <div className="bg-slate-900/50 backdrop-blur border border-slate-800/50 rounded-2xl p-6 shadow-2xl">
            {/* Header */}
            <div className="flex items-center justify-between mb-6">
                <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-purple-500 to-pink-600 flex items-center justify-center">
                        <Brain className="w-5 h-5" />
                    </div>
                    <div>
                        <h3 className="text-lg font-semibold text-white">Strategy Generator</h3>
                        <p className="text-xs text-slate-400">Evolutionary Kernel</p>
                    </div>
                </div>
                {isEvolving && (
                    <div className="flex items-center gap-2 px-3 py-1 rounded-full bg-purple-500/20 border border-purple-500/50">
                        <Sparkles className="w-4 h-4 text-purple-400 animate-pulse" />
                        <span className="text-xs text-purple-400 font-medium">Evolving</span>
                    </div>
                )}
            </div>

            {/* Key Metrics Grid */}
            <div className="grid grid-cols-2 gap-4 mb-6">
                <MetricCard
                    label="Generation"
                    value={currentGen.toLocaleString()}
                    trend={evolutionRate > 0 ? `${evolutionRate} gen/min` : ""}
                    icon={<TrendingUp className="w-4 h-4" />}
                    color="blue"
                />
                <MetricCard
                    label="Fitness Score"
                    value={hasFitness ? currentFitness!.toFixed(4) : "Pending"}
                    trend={hasFitness ? "Active" : "Awaiting Data"}
                    icon={<Activity className="w-4 h-4" />}
                    color={hasFitness ? "emerald" : "slate"}
                />
            </div>

            {/* Fitness Trend Chart */}
            {fitnessHistory.length > 0 && (
                <div className="mb-6">
                    <h4 className="text-sm font-medium text-slate-300 mb-3">Fitness Trend</h4>
                    <div className="h-32 w-full">
                        <ResponsiveContainer width="100%" height="100%">
                            <LineChart data={fitnessHistory}>
                                <CartesianGrid strokeDasharray="3 3" stroke="#334155" />
                                <XAxis dataKey="gen" stroke="#64748b" fontSize={10} />
                                <YAxis stroke="#64748b" fontSize={10} />
                                <Tooltip
                                    contentStyle={{ backgroundColor: "#1e293b", border: "1px solid #475569", borderRadius: "8px" }}
                                    labelStyle={{ color: "#cbd5e1" }}
                                    itemStyle={{ color: "#a78bfa" }}
                                />
                                <Line type="monotone" dataKey="fitness" stroke="#a78bfa" strokeWidth={2} dot={false} />
                            </LineChart>
                        </ResponsiveContainer>
                    </div>
                </div>
            )}

            {/* Best Formula Display */}
            <div>
                <h4 className="text-sm font-medium text-slate-300 mb-2">Current Best Formula</h4>
                <div className="flex flex-wrap gap-2">
                    {bestFormula.length > 0 ? (
                        bestFormula.map((token, idx) => (
                            <div key={idx} className="px-2 py-1 rounded bg-slate-800/80 border border-slate-700 text-xs font-mono text-slate-300">
                                {token}
                            </div>
                        ))
                    ) : (
                        <div className="text-xs text-slate-500 italic">No formula generated yet</div>
                    )}
                </div>
            </div>
        </div>
    );
}

function MetricCard({ label, value, trend, icon, color }: { label: string; value: string; trend: string; icon: React.ReactNode; color: string }) {
    const colorClasses = {
        blue: "from-blue-500/20 to-blue-600/20 border-blue-500/30 text-blue-400",
        emerald: "from-emerald-500/20 to-emerald-600/20 border-emerald-500/30 text-emerald-400",
        slate: "from-slate-500/20 to-slate-600/20 border-slate-500/30 text-slate-400",
    };

    return (
        <div className={`bg-gradient-to-br ${colorClasses[color as keyof typeof colorClasses]} border rounded-xl p-4`}>
            <div className="flex items-center justify-between mb-2">
                <span className="text-xs text-slate-400">{label}</span>
                {icon}
            </div>
            <div className="text-xl font-bold text-white mb-1">{value}</div>
            {trend && <div className="text-xs text-slate-400">{trend}</div>}
        </div>
    );
}
