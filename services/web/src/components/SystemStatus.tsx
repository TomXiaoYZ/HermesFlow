import React, { useState, useEffect, useRef } from "react";
import { Server, Database, Activity, Terminal, Filter, Search, RefreshCw, ChevronDown, ChevronUp } from "lucide-react";
import { cn } from "@/lib/utils";
import { authFetch } from "@/lib/api";
import { LogEntry } from "./SystemLogs";

interface SystemStatusProps {
    logs: LogEntry[]; // Live logs from WS
    heartbeats?: Record<string, number>; // Map of service name -> last timestamp
    metrics: {
        activeTokens: number;
        heliusConnected: boolean;
        wsConnected: boolean;
    };
}

export default function SystemStatus({ logs: liveLogs, heartbeats, metrics }: SystemStatusProps) {
    const [selectedService, setSelectedService] = useState<string | null>(null);
    const [historicalLogs, setHistoricalLogs] = useState<LogEntry[]>([]);
    const [now, setNow] = useState(Date.now());
    const logContainerRef = useRef<HTMLDivElement>(null);

    // Filters
    const [keyword, setKeyword] = useState("");
    const [level, setLevel] = useState<"ALL" | "INFO" | "WARN" | "ERROR">("ALL");
    const [isLoading, setIsLoading] = useState(false);

    // Update 'now' every second to trigger re-render of statuses
    useEffect(() => {
        const interval = setInterval(() => setNow(Date.now()), 1000);
        return () => clearInterval(interval);
    }, []);

    // Helper: Strip ANSI codes and redundant tags
    const cleanMessage = (str: string, moduleName: string) => {
        // 1. Strip ANSI
        let clean = str.replace(/[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]/g, '');
        // 2. Remove redundant module tag if present (e.g. "[STRATEGY]")
        if (moduleName) {
            const tag = `[${moduleName}]`;
            clean = clean.replace(tag, "").trim();
            // Also handle case where module is STRATEGY but tag is [STRATEGY-ENGINE]
            if (moduleName === "STRATEGY") clean = clean.replace("[STRATEGY-ENGINE]", "").trim();
        }
        return clean;
    };

    // Helper: Determine Source Module
    const getModule = (log: LogEntry): string => {
        if (log.module) return log.module;
        // Fallback for raw objects not fully mapped
        return "SYSTEM";
    }

    // Status Check Helper
    const getStatus = (serviceName: string, fallbackOnline: boolean): "online" | "offline" | "unknown" => {
        if (heartbeats && heartbeats[serviceName]) {
            const lastSeen = heartbeats[serviceName];
            const diff = now - lastSeen;
            if (diff < 15000) return "online"; // 15s TTL
            return "offline";
        }
        if (fallbackOnline) return "online";
        return "unknown";
    };

    // Fetch Historical Logs
    useEffect(() => {
        const fetchHistory = async () => {
            setIsLoading(true);
            try {
                const params = new URLSearchParams({
                    service: selectedService || "all",
                    level: level,
                    limit: "100"
                });
                if (keyword) params.append("keyword", keyword);

                const res = await authFetch(`/api/logs?${params.toString()}`);
                const data = await res.json();

                if (Array.isArray(data)) {
                    const mappedLogs: LogEntry[] = data.map((d: any) => ({
                        timestamp: new Date(d.timestamp * 1000).toLocaleTimeString(),
                        level: d.level,
                        message: d.message,
                        module: d.container_name.replace("hermesflow-", "").replace("-1", "").toUpperCase()
                    }));
                    setHistoricalLogs(mappedLogs);
                }
            } catch (e) {
                // Ignore
            } finally {
                setIsLoading(false);
            }
        };

        const timer = setTimeout(fetchHistory, 300);
        return () => clearTimeout(timer);
    }, [selectedService, level, keyword]);

    // Live Logs Filter
    const filteredLiveLogs = liveLogs.filter(l => {
        const m = l.module;
        const msg = l.message;
        if (level !== "ALL" && l.level !== level) return false;
        if (keyword && !msg.toLowerCase().includes(keyword.toLowerCase())) return false;

        if (!selectedService) return true;

        switch (selectedService) {
            case "gateway": return true;
            case "data-engine": return m?.includes("DATA");
            case "strategy-engine": return m?.includes("STRATEGY");
            case "execution-engine": return m?.includes("EXECUTION");
            case "strategy-generator": return m?.includes("GENERATOR");
            case "redis": return msg.includes("Redis");
            case "timescaledb": return m?.includes("TIMESCALE") || msg.toLowerCase().includes("postgres");
            case "clickhouse": return m?.includes("CLICKHOUSE") || msg.toLowerCase().includes("clickhouse");
            case "vector": return m?.includes("VECTOR") || msg.toLowerCase().includes("vector");
            default: return true;
        }
    });

    const displayLogs = React.useMemo(() => {
        const all = [...filteredLiveLogs, ...historicalLogs];
        const uniqueMap = new Map();

        all.forEach(log => {
            // Dedupe simply
            const key = `${log.timestamp}-${log.message}`;
            if (!uniqueMap.has(key)) {
                uniqueMap.set(key, log);
            }
        });

        // Return mostly raw order (Newest first)
        return Array.from(uniqueMap.values());
    }, [filteredLiveLogs, historicalLogs]);


    const handleServiceClick = (service: string) => {
        setSelectedService(prev => prev === service ? null : service);
    };

    return (
        <div className="grid grid-cols-1 lg:grid-cols-4 gap-6 h-[calc(100vh-140px)] pb-6">
            {/* Left: Service Health Cards (Scrollable) */}
            <div className="lg:col-span-1 flex flex-col gap-4 overflow-y-auto pr-2 custom-scrollbar">
                <div className="flex items-center justify-between sticky top-0 bg-[#030712] py-2 z-10">
                    <h3 className="text-xs font-semibold text-slate-500 uppercase tracking-wider">Services</h3>
                    {selectedService && (
                        <button
                            onClick={() => setSelectedService(null)}
                            className="text-[10px] text-indigo-400 hover:text-indigo-300 flex items-center gap-1"
                        >
                            <Filter className="w-3 h-3" /> Clear
                        </button>
                    )}
                </div>

                <div className="space-y-3">
                    <ServiceCard
                        name="Gateway"
                        status={metrics.wsConnected ? "online" : "offline"}
                        icon={<Server className="w-4 h-4 text-indigo-400" />}
                        details="API Gateway"
                        selected={selectedService === "gateway"}
                        onClick={() => handleServiceClick("gateway")}
                    />
                    <ServiceCard
                        name="Data Engine"
                        status={getStatus("data-engine", metrics.activeTokens > 0)}
                        icon={<Database className="w-4 h-4 text-cyan-400" />}
                        details={`Tokens: ${metrics.activeTokens}`}
                        selected={selectedService === "data-engine"}
                        onClick={() => handleServiceClick("data-engine")}
                    />
                    <ServiceCard
                        name="Strategy Engine"
                        status={getStatus("strategy-engine", false)}
                        icon={<Activity className="w-4 h-4 text-purple-400" />}
                        details="Signals"
                        selected={selectedService === "strategy-engine"}
                        onClick={() => handleServiceClick("strategy-engine")}
                    />
                    <ServiceCard
                        name="Execution Engine"
                        status={getStatus("execution-engine", false)}
                        icon={<Terminal className="w-4 h-4 text-emerald-400" />}
                        details="Trader"
                        selected={selectedService === "execution-engine"}
                        onClick={() => handleServiceClick("execution-engine")}
                    />
                    <ServiceCard
                        name="Strategy Gen"
                        status={getStatus("strategy-generator", false)}
                        icon={<Activity className="w-4 h-4 text-pink-400" />}
                        details="Evolution"
                        selected={selectedService === "strategy-generator"}
                        onClick={() => handleServiceClick("strategy-generator")}
                    />
                    <ServiceCard
                        name="Redis"
                        status={metrics.wsConnected ? "online" : "unknown"}
                        icon={<Database className="w-4 h-4 text-red-400" />}
                        details="Broker"
                        selected={selectedService === "redis"}
                        onClick={() => handleServiceClick("redis")}
                    />
                    <ServiceCard
                        name="TimescaleDB"
                        status="online"
                        icon={<Database className="w-4 h-4 text-amber-400" />}
                        details="Postgres"
                        selected={selectedService === "timescaledb"}
                        onClick={() => handleServiceClick("timescaledb")}
                    />
                    <ServiceCard
                        name="ClickHouse"
                        status="online"
                        icon={<Database className="w-4 h-4 text-yellow-400" />}
                        details="Analytics"
                        selected={selectedService === "clickhouse"}
                        onClick={() => handleServiceClick("clickhouse")}
                    />
                    <ServiceCard
                        name="Vector"
                        status="online"
                        icon={<Activity className="w-4 h-4 text-orange-400" />}
                        details="Logs"
                        selected={selectedService === "vector"}
                        onClick={() => handleServiceClick("vector")}
                    />
                </div>
            </div>

            {/* Right: Log Console (Full Height) */}
            <div className="lg:col-span-3 bg-slate-900/50 rounded-xl border border-white/5 backdrop-blur-sm flex flex-col overflow-hidden shadow-xl">
                {/* Header */}
                <div className="px-4 py-3 border-b border-white/5 bg-white/5 flex items-center gap-4 justify-between shrink-0">
                    <div className="flex items-center gap-3">
                        <Terminal className="w-4 h-4 text-slate-400" />
                        <h3 className="text-sm font-semibold text-slate-200">
                            {selectedService ? `Logs: ${selectedService}` : "System Logs"}
                        </h3>
                        <span className="text-[10px] bg-slate-800 text-slate-400 px-2 py-0.5 rounded-full border border-white/5">
                            {displayLogs.length} events
                        </span>
                    </div>

                    <div className="flex items-center gap-3">
                        <div className="relative w-48">
                            <Search className="w-3.5 h-3.5 absolute left-3 top-2.5 text-slate-500" />
                            <input
                                type="text"
                                placeholder="Search..."
                                value={keyword}
                                onChange={(e) => setKeyword(e.target.value)}
                                className="w-full bg-slate-950/50 border border-white/10 rounded-lg py-1.5 pl-9 pr-4 text-xs focus:outline-none focus:border-indigo-500/50 transition-colors"
                            />
                        </div>
                        <select
                            value={level}
                            onChange={(e) => setLevel(e.target.value as any)}
                            className="bg-slate-950/50 border border-white/10 rounded-lg px-3 py-1.5 text-xs focus:outline-none focus:border-indigo-500/50"
                        >
                            <option value="ALL">All Levels</option>
                            <option value="INFO">INFO</option>
                            <option value="WARN">WARN</option>
                            <option value="ERROR">ERROR</option>
                        </select>
                        {isLoading && <RefreshCw className="w-3.5 h-3.5 animate-spin text-slate-400" />}
                    </div>
                </div>

                {/* Log Table Header */}
                <div className="grid grid-cols-12 gap-4 px-4 py-2 border-b border-white/5 bg-slate-950/30 text-[10px] font-semibold text-slate-500 uppercase tracking-wider shrink-0">
                    <div className="col-span-2">Timestamp</div>
                    <div className="col-span-1">Level</div>
                    <div className="col-span-2">Service</div>
                    <div className="col-span-7">Message</div>
                </div>

                {/* Log Content - Scrollable Table */}
                <div className="flex-1 overflow-y-auto custom-scrollbar bg-[#050608]" ref={logContainerRef}>
                    <div className="flex flex-col min-h-full">
                        {displayLogs.length === 0 ? (
                            <div className="flex-1 flex flex-col items-center justify-center text-slate-600 gap-2 min-h-[200px]">
                                <Terminal className="w-8 h-8 opacity-20" />
                                <span>No logs found via filters.</span>
                            </div>
                        ) : (
                            displayLogs.map((log, i) => {
                                const moduleName = getModule(log);
                                const isStrategy = moduleName.includes("STRATEGY");
                                const isData = moduleName.includes("DATA");
                                const isError = log.level === "ERROR";
                                const isWarn = log.level === "WARN";

                                return (
                                    <div key={i} className="group grid grid-cols-12 gap-4 px-4 py-1.5 hover:bg-white/5 text-xs font-mono border-b border-white/[0.02] transition-colors items-start">
                                        <div className="col-span-2 text-slate-500 whitespace-nowrap overflow-hidden text-ellipsis">
                                            {log.timestamp}
                                        </div>
                                        <div className={cn("col-span-1 font-bold",
                                            isError ? "text-red-400" : (isWarn ? "text-yellow-400" : "text-blue-400")
                                        )}>
                                            {log.level}
                                        </div>
                                        <div className="col-span-2 truncate font-medium">
                                            <span className={cn(
                                                "px-1.5 py-0.5 rounded-[3px] text-[10px]",
                                                isStrategy ? "bg-purple-500/10 text-purple-400" :
                                                    isData ? "bg-cyan-500/10 text-cyan-400" :
                                                        "bg-slate-700/30 text-slate-400"
                                            )}>
                                                {moduleName.split("-")[0]}
                                            </span>
                                        </div>
                                        <div className="col-span-7 text-slate-300 break-words whitespace-pre-wrap leading-relaxed">
                                            {cleanMessage(log.message, moduleName)}
                                        </div>
                                    </div>
                                );
                            })
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}

function ServiceCard({
    name,
    status,
    icon,
    details,
    selected,
    onClick
}: {
    name: string,
    status: "online" | "offline" | "unknown",
    icon: React.ReactNode,
    details: string,
    selected?: boolean,
    onClick?: () => void
}) {
    const statusColor = {
        online: "text-emerald-400",
        offline: "text-red-400",
        unknown: "text-slate-500"
    };

    // Condensed card design
    return (
        <div
            onClick={onClick}
            className={cn(
                "p-3 rounded-lg border backdrop-blur-sm transition-all duration-200 group cursor-pointer relative",
                status === "online" ? "bg-emerald-500/[0.02] border-emerald-500/20 hover:bg-emerald-500/[0.05]" :
                    status === "offline" ? "bg-red-500/[0.02] border-red-500/20 hover:bg-red-500/[0.05]" :
                        "bg-slate-500/[0.02] border-slate-500/20 hover:bg-slate-500/[0.05]",
                selected && "ring-1 ring-indigo-500 border-indigo-500/50 bg-indigo-500/[0.05]"
            )}
        >
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                    <div className="p-1.5 bg-slate-900/50 rounded-md border border-white/5 text-slate-400 group-hover:text-slate-200 transition-colors">
                        {icon}
                    </div>
                    <div>
                        <h3 className="text-xs font-medium text-slate-300 group-hover:text-white transition-colors">{name}</h3>
                        <p className="text-[10px] text-slate-500">{details}</p>
                    </div>
                </div>
                <div className={cn("flex items-center gap-1.5 px-1.5 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-slate-950/30 border border-white/5", statusColor[status])}>
                    <div className={cn("w-1 h-1 rounded-full", status === "online" ? "bg-emerald-500 animate-pulse" : (status === "offline" ? "bg-red-500" : "bg-slate-500"))}></div>
                    {status}
                </div>
            </div>
        </div>
    );
}
