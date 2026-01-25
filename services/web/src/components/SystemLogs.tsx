"use client";

import React, { useState, useEffect, useRef } from "react";
import { Terminal, Search } from "lucide-react";

export interface LogEntry {
    timestamp: string;
    level: "INFO" | "WARN" | "ERROR";
    message: string;
    module?: string;
}

export default function SystemLogs({ logs }: { logs: LogEntry[] }) {
    const [search, setSearch] = useState("");
    const [activeModule, setActiveModule] = useState<"ALL" | "STRATEGY" | "DATA" | "SYSTEM">("ALL");
    const logsEndRef = useRef<HTMLDivElement>(null);

    // Removed internal WS effect

    useEffect(() => {
        logsEndRef.current?.scrollIntoView({ behavior: "smooth" });
    }, [logs]);

    const filteredLogs = logs.filter((log) => {
        const matchesSearch = log.message.toLowerCase().includes(search.toLowerCase());
        const matchesModule = activeModule === "ALL" || log.module === activeModule;
        return matchesSearch && matchesModule;
    });

    return (
        <div className="glass-panel rounded-3xl p-6 h-full flex flex-col">
            {/* Header */}
            <div className="flex items-center justify-between mb-4">
                <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-xl bg-slate-800 flex items-center justify-center border border-white/5">
                        <Terminal className="w-5 h-5 text-slate-400" />
                    </div>
                    <div>
                        <h3 className="text-lg font-bold text-white">System Logs</h3>
                        <p className="text-xs text-slate-500 font-medium uppercase tracking-wide">Real-time Activity</p>
                    </div>
                </div>
            </div>

            {/* Controls */}
            <div className="space-y-3 mb-4">
                {/* Search */}
                <div className="relative">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
                    <input
                        type="text"
                        placeholder="Search logs..."
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                        className="w-full pl-10 pr-4 py-2 bg-slate-900/50 border border-slate-700 rounded-lg text-xs text-slate-300 placeholder-slate-600 focus:outline-none focus:border-indigo-500/50 transition-colors"
                    />
                </div>

                {/* Module Tabs */}
                <div className="flex bg-slate-900/50 p-1 rounded-lg border border-slate-700/50">
                    {["ALL", "STRATEGY", "DATA", "SYSTEM"].map((mod) => (
                        <button
                            key={mod}
                            onClick={() => setActiveModule(mod as any)}
                            className={`flex-1 py-1 text-[10px] font-bold uppercase tracking-wider rounded-md transition-all ${activeModule === mod
                                ? "bg-slate-700 text-white shadow-sm"
                                : "text-slate-500 hover:text-slate-300 hover:bg-slate-800/50"
                                }`}
                        >
                            {mod}
                        </button>
                    ))}
                </div>
            </div>

            {/* Logs */}
            <div className="flex-1 overflow-y-auto custom-scrollbar font-mono text-[10px] sm:text-xs space-y-1 pr-2 min-h-0 bg-slate-950/30 rounded-xl p-2 border border-white/5">
                {filteredLogs.map((log, idx) => (
                    <LogLine key={idx} log={log} />
                ))}
                <div ref={logsEndRef} />
                {filteredLogs.length === 0 && (
                    <div className="text-center text-slate-600 py-8 italic">No logs found</div>
                )}
            </div>
        </div>
    );
}

function LogLine({ log }: { log: LogEntry }) {
    const levelColors = {
        INFO: "text-slate-400",
        WARN: "text-amber-400",
        ERROR: "text-rose-400",
    };

    return (
        <div className="flex items-start gap-2 hover:bg-white/5 px-2 py-1 rounded transition-colors break-all">
            <span className="text-slate-600 shrink-0 select-none">[{log.timestamp}]</span>
            <span className={`font-bold shrink-0 w-10 ${levelColors[log.level]}`}>{log.level}</span>
            <span className="text-slate-300 leading-relaxed">{log.message}</span>
        </div>
    );
}


