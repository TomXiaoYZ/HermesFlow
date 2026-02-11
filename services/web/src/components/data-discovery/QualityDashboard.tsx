import React, { useEffect, useState } from "react";
import { Activity, Clock, Database, RefreshCw, AlertTriangle, CheckCircle, Play } from "lucide-react";
import { cn } from "@/lib/utils";

interface QualityMetrics {
    snapshots: MetricStatus;
    candles_1m: MetricStatus;
    candles_15m: MetricStatus;
    candles_1h: MetricStatus;
    candles_4h: MetricStatus;
    candles_1d: MetricStatus;
    token_discovery: MetricStatus;
    active_tokens: number;
}

interface MetricStatus {
    latest: string | null;
    lag_seconds: number | null;
    status: string;
}

export default function QualityDashboard() {
    const [metrics, setMetrics] = useState<QualityMetrics | null>(null);
    const [loading, setLoading] = useState(true);
    const [triggering, setTriggering] = useState<string | null>(null);

    const fetchMetrics = async () => {
        try {
            const res = await fetch("/api/v1/data/quality");
            if (res.ok) {
                const data = await res.json();
                setMetrics(data);
            }
        } catch (e) {
            console.error(e);
        } finally {
            setLoading(false);
        }
    };

    const triggerTask = async (taskName: string, endpoint: string) => {
        setTriggering(taskName);
        try {
            const res = await fetch(endpoint, { method: "POST" });
            if (res.ok) {
                // Short refetch buffer
                setTimeout(fetchMetrics, 2000);
            } else {
                console.error("Failed to trigger task");
            }
        } catch (e) {
            console.error(e);
        } finally {
            setTriggering(null);
        }
    };

    useEffect(() => {
        fetchMetrics();
        const interval = setInterval(fetchMetrics, 5000);
        return () => clearInterval(interval);
    }, []);

    if (loading && !metrics) {
        return <div className="p-4 text-slate-400 animate-pulse">Loading Data Engine Metrics...</div>;
    }

    if (!metrics) return <div className="p-4 text-red-400">Failed to load metrics. Is Data Engine running?</div>;

    return (
        <div className="flex flex-col gap-4 mb-6">
            {/* Top Row: Core Status */}
            <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
                <QualityCard
                    title="Active Tokens"
                    value={metrics.active_tokens.toString()}
                    status="healthy"
                    icon={<Database className="w-5 h-5 text-indigo-400" />}
                    subtext="Total Tracked"
                />
                <QualityCard
                    title="Realtime Ingestion"
                    value={formatLag(metrics.snapshots.lag_seconds)}
                    status={metrics.snapshots.status}
                    icon={<Activity className="w-5 h-5 text-emerald-400" />}
                    subtext={`Last: ${formatTime(metrics.snapshots.latest)}`}
                />
                <QualityCard
                    title="Token Discovery"
                    value={formatLag(metrics.token_discovery.lag_seconds)}
                    status={metrics.token_discovery.status}
                    icon={<RefreshCw className={`w-5 h-5 text-purple-400 ${triggering === 'discovery' ? 'animate-spin' : ''}`} />}
                    subtext={`Last: ${formatTime(metrics.token_discovery.latest)}`}
                    action={() => triggerTask('discovery', '/api/v1/data/tasks/discovery')}
                    actionLabel="Run Scan"
                />
                <QualityCard
                    title="Control Plane"
                    value="ACTIVE"
                    status="healthy"
                    icon={<CheckCircle className="w-5 h-5 text-blue-400" />}
                    subtext="System Operational"
                    hideValue
                />
            </div>

            {/* Aggregation Status Row */}
            <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
                <AggregationCard res="1m" metric={metrics.candles_1m} />
                <AggregationCard res="15m" metric={metrics.candles_15m} />
                <AggregationCard res="1h" metric={metrics.candles_1h} />
                <AggregationCard res="4h" metric={metrics.candles_4h} />
                {/* Last one with trigger */}
                <AggregationCard
                    res="Daily"
                    metric={metrics.candles_1d}
                    trigger={() => triggerTask('aggregation', '/api/v1/data/tasks/aggregation')}
                    isTriggering={triggering === 'aggregation'}
                />
            </div>
        </div>
    );
}

function AggregationCard({ res, metric, trigger, isTriggering }: { res: string, metric: MetricStatus, trigger?: () => void, isTriggering?: boolean }) {
    return (
        <div className={cn("p-3 rounded-lg border bg-slate-900/50 backdrop-blur flex flex-col justify-between",
            metric.status === 'healthy' ? "border-slate-800" : "border-yellow-500/30 bg-yellow-900/10"
        )}>
            <div className="flex justify-between items-center mb-1">
                <span className="text-xs text-slate-400 font-mono">{res} Agg</span>
                <div className={cn("w-2 h-2 rounded-full",
                    metric.status === 'healthy' ? "bg-emerald-500" : "bg-yellow-500"
                )} />
            </div>
            <div className="text-lg font-bold text-slate-200">
                {formatLag(metric.lag_seconds)}
            </div>
            <div className="flex justify-between items-end mt-1">
                <div className="text-[10px] text-slate-500">{formatTimeShort(metric.latest)}</div>
                {trigger && (
                    <button
                        onClick={trigger}
                        disabled={isTriggering}
                        className="text-[10px] bg-blue-500/20 hover:bg-blue-500/40 text-blue-300 px-2 py-0.5 rounded transition-colors flex items-center gap-1"
                    >
                        {isTriggering ? <RefreshCw className="w-3 h-3 animate-spin" /> : <Play className="w-3 h-3" />}
                        Run
                    </button>
                )}
            </div>
        </div>
    )
}

function QualityCard({ title, value, status, icon, subtext, action, actionLabel, hideValue }: any) {
    const statusColors: any = {
        healthy: "border-emerald-500/20 bg-emerald-500/5 text-emerald-400",
        degraded: "border-yellow-500/20 bg-yellow-500/5 text-yellow-400",
        stale: "border-orange-500/20 bg-orange-500/5 text-orange-400",
        empty: "border-red-500/20 bg-red-500/5 text-slate-400",
        error: "border-red-500/20 bg-red-500/5 text-red-400",
    };

    const colorClass = statusColors[status] || statusColors.error;

    return (
        <div className={cn("p-4 rounded-xl border backdrop-blur-sm transition-all duration-300 flex flex-col justify-between min-h-[110px]", colorClass)}>
            <div>
                <div className="flex justify-between items-start mb-2">
                    <span className="opacity-70 text-xs font-medium uppercase tracking-wider">{title}</span>
                    {icon}
                </div>
                {!hideValue && (
                    <div className="text-2xl font-bold mb-1">
                        {value}
                    </div>
                )}
            </div>
            <div className="flex justify-between items-center mt-auto pt-2">
                <div className="text-[10px] opacity-60 font-mono">
                    {subtext}
                </div>
                {action && (
                    <button
                        onClick={action}
                        className="text-[10px] bg-white/10 hover:bg-white/20 px-2 py-1 rounded transition-colors"
                    >
                        {actionLabel}
                    </button>
                )}
            </div>
        </div>
    );
}

function formatLag(seconds: number | null) {
    if (seconds === null) return "N/A";
    if (seconds < 60) return `${seconds}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
    return `${Math.floor(seconds / 3600)}h`;
}

function formatTime(iso: string | null) {
    if (!iso) return "Never";
    const date = new Date(iso);
    return date.toLocaleTimeString("en-US", {
        timeZone: "Asia/Shanghai",
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        hour12: false
    }) + " (BJ)";
}

function formatTimeShort(iso: string | null) {
    if (!iso) return "--/-- --:--";
    const date = new Date(iso);
    return date.toLocaleDateString("en-US", {
        timeZone: "Asia/Shanghai",
        month: 'numeric',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
        hour12: false
    });
}
