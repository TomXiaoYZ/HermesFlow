import React, { useState, useEffect } from "react";
import { Play, Terminal, AlertCircle } from "lucide-react";
import { cn } from "@/lib/utils";
import { authFetch } from "@/lib/api";

export default function SqlIde({ initialQuery }: { initialQuery?: string }) {
    const [query, setQuery] = useState(initialQuery || "SELECT * FROM active_tokens LIMIT 10;");
    const [result, setResult] = useState<{ columns: string[], rows: any[][] } | null>(null);
    const [error, setError] = useState<string | null>(null);
    const [executing, setExecuting] = useState(false);

    useEffect(() => {
        if (initialQuery) setQuery(initialQuery);
    }, [initialQuery]);

    const runQuery = async () => {
        setExecuting(true);
        setError(null);
        setResult(null);

        try {
            const res = await authFetch("/api/v1/data/query", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ query })
            });

            const data = await res.json();

            if (!res.ok) {
                throw new Error(data.error || "Query failed");
            }

            setResult(data);
        } catch (e: any) {
            setError(e.message);
        } finally {
            setExecuting(false);
        }
    };

    return (
        <div className="flex flex-col h-full gap-4">
            {/* Editor Area */}
            <div className="flex flex-col gap-2">
                <div className="flex justify-between items-center">
                    <div className="flex items-center gap-2 text-slate-400 text-xs uppercase tracking-wider font-semibold">
                        <Terminal className="w-4 h-4" />
                        <span>SQL Query</span>
                    </div>
                    <button
                        onClick={runQuery}
                        disabled={executing}
                        className={cn(
                            "flex items-center gap-2 px-4 py-1.5 rounded-lg text-sm font-semibold transition-all",
                            executing
                                ? "bg-slate-800 text-slate-500 cursor-not-allowed"
                                : "bg-indigo-600 hover:bg-indigo-500 text-white shadow-lg shadow-indigo-500/20"
                        )}
                    >
                        <Play className={cn("w-3.5 h-3.5", executing && "animate-spin")} />
                        {executing ? "Running..." : "Run Query"}
                    </button>
                </div>

                <div className="relative group">
                    <textarea
                        value={query}
                        onChange={e => setQuery(e.target.value)}
                        className="w-full h-40 bg-slate-950 border border-white/10 rounded-xl p-4 font-mono text-sm text-slate-300 focus:outline-none focus:border-indigo-500/50 resize-y"
                        spellCheck={false}
                    />
                    <div className="absolute bottom-2 right-4 text-[10px] text-slate-600">
                        CMD+ENTER to run
                    </div>
                </div>
            </div>

            {/* Results Area */}
            <div className="flex-1 min-h-0 bg-slate-900/30 border border-white/5 rounded-xl overflow-hidden flex flex-col">
                {error && (
                    <div className="p-4 bg-red-500/10 text-red-400 text-sm flex items-center gap-3 border-b border-red-500/10">
                        <AlertCircle className="w-5 h-5" />
                        <span className="font-mono">{error}</span>
                    </div>
                )}

                {result && (
                    <div className="flex-1 overflow-auto">
                        <table className="w-full text-left text-sm whitespace-nowrap">
                            <thead className="bg-slate-950/50 sticky top-0 z-10">
                                <tr>
                                    {result.columns.map((col, i) => (
                                        <th key={i} className="px-4 py-3 font-medium text-slate-400 border-b border-white/5 font-mono text-xs">
                                            {col}
                                        </th>
                                    ))}
                                </tr>
                            </thead>
                            <tbody className="divide-y divide-white/5">
                                {result.rows.map((row, i) => (
                                    <tr key={i} className="hover:bg-white/5 transition-colors">
                                        {row.map((val, cellI) => (
                                            <td key={cellI} className="px-4 py-2 text-slate-300 font-mono text-xs border-r border-white/5 last:border-0">
                                                {formatValue(val)}
                                            </td>
                                        ))}
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}

                {!result && !error && !executing && (
                    <div className="flex-1 flex items-center justify-center text-slate-600 text-sm">
                        Run a query to see results
                    </div>
                )}
            </div>
        </div>
    );
}

function formatValue(val: any): string {
    if (val === null) return "NULL";
    if (typeof val === "boolean") return val ? "TRUE" : "FALSE";
    if (typeof val === "object") return JSON.stringify(val);
    return String(val);
}
