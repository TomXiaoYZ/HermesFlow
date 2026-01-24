"use client";

import React, { useState, useEffect } from "react";
import { Brain, Sparkles, Activity, Dna } from "lucide-react";
import { decodeGenome, getFeatureImportance } from "@/utils/genome";

interface StrategyStatus {
    active: boolean;
    generation: number;
    fitness: number;
    best_tokens: number[];
    timestamp: number;
}

export default function StrategyLab() {
    const [status, setStatus] = useState<StrategyStatus | null>(null);
    const [leaderboard, setLeaderboard] = useState<{ fitness: number, tokens: number[] }[]>([]);
    const [decodedFormula, setDecodedFormula] = useState<string>("Waiting for data...");
    const [featureCounts, setFeatureCounts] = useState<Record<string, number>>({});

    useEffect(() => {
        const fetchStatus = async () => {
            try {
                const res = await fetch("/api/v1/strategy/status");
                if (res.ok) {
                    const data = await res.json();
                    setStatus(data);
                    setStatus(data);
                    const tokens = data.best_tokens || data.formula || [];
                    if (tokens.length > 0) {
                        setDecodedFormula(decodeGenome(tokens));
                        setFeatureCounts(getFeatureImportance(tokens));
                    }
                }

                // Fetch Leaderboard
                const resPop = await fetch("/api/v1/strategy/population");
                if (resPop.ok) {
                    const popData = await resPop.json();
                    setLeaderboard(popData);
                }
            } catch (e) {
                console.error("Failed to fetch strategy status", e);
            }
        };

        fetchStatus();
        const interval = setInterval(fetchStatus, 3000);
        return () => clearInterval(interval);
    }, []);

    if (!status) {
        return (
            <div className="flex items-center justify-center h-96 text-slate-500 animate-pulse">
                Connecting to Evolutionary Kernel...
            </div>
        );
    }

    const factorExplanations: Record<string, string> = {
        "VolCluster": "Volatility Clustering: Measures if high volatility periods tend to follow each other.",
        "ln(Open)": "Log price: Normalized price level input.",
        "Liquidity": "Market Depth: Measures how easy it is to trade without moving price.",
        "Deviation": "Standard Deviation: Statistical measure of price volatility.",
        "Return": "Price Momentum: Rate of change in price over time.",
        "In(High)": "Log High Price: Used for detecting resistance levels.",
    };

    return (
        <div className="space-y-6 animate-in fade-in duration-500">
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                    <div className="w-12 h-12 rounded-2xl bg-gradient-to-br from-purple-500 to-indigo-600 flex items-center justify-center shadow-lg shadow-purple-900/20">
                        <Brain className="w-6 h-6 text-white" />
                    </div>
                    <div>
                        <h2 className="text-2xl font-bold text-white">Strategy Lab</h2>
                        <p className="text-sm text-slate-400">Deep Dive into Evolutionary Logic</p>
                    </div>
                </div>
                <div className="flex items-center gap-4">
                    <div className="text-right">
                        <div className="text-xs text-slate-500 uppercase tracking-wider font-semibold">Generation</div>
                        <div className="text-2xl font-mono text-purple-400">#{status.generation}</div>
                    </div>
                    <div className="text-right">
                        <div className="text-xs text-slate-500 uppercase tracking-wider font-semibold">Fitness Score</div>
                        <div className={`text-2xl font-mono ${status.fitness > 0 ? "text-emerald-400" : "text-slate-400"}`}>
                            {status.fitness === -999 ? "Pending" : status.fitness.toFixed(4)}
                        </div>
                    </div>
                </div>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                <div className="lg:col-span-2 bg-slate-900/50 backdrop-blur border border-purple-500/20 rounded-2xl p-8 relative overflow-hidden group">
                    <div className="absolute inset-0 bg-gradient-to-br from-purple-500/5 to-transparent opacity-50 group-hover:opacity-100 transition-opacity" />

                    <div className="relative z-10">
                        <div className="flex items-center gap-2 mb-4 text-purple-400">
                            <Dna className="w-5 h-5" />
                            <span className="text-sm font-semibold uppercase tracking-wider">Current Best Gene (Formula)</span>
                        </div>

                        <div className="font-mono text-xl sm:text-2xl md:text-3xl text-white leading-relaxed break-words bg-slate-950/50 p-6 rounded-xl border border-slate-800/50">
                            {decodedFormula}
                        </div>

                        <div className="mt-6 flex flex-wrap gap-2">
                            <span className="text-xs text-slate-500 uppercase tracking-wider mr-2 self-center">Raw Genome:</span>
                            {(status.best_tokens || (status as any).formula || []).map((t: number, i: number) => (
                                <div key={i} className="px-2 py-1 bg-slate-800/80 rounded text-xs font-mono text-slate-400 border border-slate-700">
                                    {t}
                                </div>
                            ))}
                        </div>
                    </div>
                </div>

                <div className="space-y-6">
                    <div className="bg-slate-900/50 backdrop-blur border border-slate-800/50 rounded-2xl p-6">
                        <div className="flex items-center gap-2 mb-4 text-emerald-400">
                            <Activity className="w-5 h-5" />
                            <span className="text-sm font-semibold uppercase tracking-wider">Feature Composition</span>
                        </div>

                        <div className="space-y-3">
                            {Object.entries(featureCounts)
                                .sort(([, a], [, b]) => b - a)
                                .map(([name, count]) => (
                                    <div key={name} className="flex items-center justify-between group">
                                        <div className="text-sm text-slate-300">{name}</div>
                                        <div className="flex items-center gap-3">
                                            <div className="w-24 h-1.5 bg-slate-800 rounded-full overflow-hidden">
                                                <div
                                                    className="h-full bg-emerald-500/50 group-hover:bg-emerald-400 transition-colors"
                                                    style={{ width: `${Math.min((count / ((status.best_tokens || (status as any).formula || []).length || 1)) * 100 * 3, 100)}%` }}
                                                />
                                            </div>
                                        </div>
                                    </div>
                                ))}
                        </div>
                    </div>

                    {/* Detailed Population Pool */}
                    <div className="bg-slate-900/50 backdrop-blur border border-slate-800/50 rounded-2xl p-6 flex flex-col h-[500px]">
                        <div className="flex items-center justify-between mb-4">
                            <div className="flex items-center gap-2 text-blue-400">
                                <Sparkles className="w-5 h-5" />
                                <span className="text-xs font-bold uppercase tracking-widest">Genomic Pool ({leaderboard.length || "Loading"})</span>
                            </div>
                            <div className="text-[10px] text-slate-500 uppercase font-mono">Sorted by Fitness</div>
                        </div>

                        <div className="flex-1 overflow-y-auto custom-scrollbar space-y-2 pr-2">
                            {leaderboard && leaderboard.map((gene, i) => (
                                <div key={i} className="flex items-center justify-between p-3 rounded-xl bg-slate-800/30 border border-white/5 hover:bg-slate-700/50 transition-colors group">
                                    <div className="flex items-center gap-3">
                                        <div className="w-6 h-6 rounded-md bg-slate-800 flex items-center justify-center text-[10px] font-mono text-slate-500 group-hover:text-white transition-colors">
                                            {i + 1}
                                        </div>
                                        <div className="flex flex-col">
                                            <div className="w-24 h-1.5 bg-slate-700 rounded-full overflow-hidden mb-1">
                                                <div className="h-full bg-blue-500" style={{ width: `${Math.max(gene.fitness * 20, 0)}%` }} />
                                            </div>
                                            <span className="text-[10px] text-slate-500 font-mono truncate w-32">
                                                Len: {gene.tokens.length} tokens
                                            </span>
                                        </div>
                                    </div>
                                    <div className="font-mono text-sm text-blue-300">
                                        {gene.fitness.toFixed(4)}
                                    </div>
                                </div>
                            ))}
                            {(!leaderboard || leaderboard.length === 0) && (
                                <div className="text-xs text-slate-500 text-center py-10 italic">
                                    Waiting for first generation...
                                </div>
                            )}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
