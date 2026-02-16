import React, { useEffect, useState } from "react";
import { Table as TableIcon, Search, ChevronRight } from "lucide-react";
import { cn } from "@/lib/utils";

interface TableInfo {
    name: string;
    size?: string;
}

export default function TableBrowser({ onSelectTable }: { onSelectTable: (tableName: string) => void }) {
    const [tables, setTables] = useState<TableInfo[]>([]);
    const [filter, setFilter] = useState("");
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        fetch("/api/v1/data/tables")
            .then(res => res.json())
            .then(data => {
                if (data.tables) setTables(data.tables);
            })
            .catch(() => { /* fetch failed */ })
            .finally(() => setLoading(false));
    }, []);

    const filtered = tables.filter(t => t.name.toLowerCase().includes(filter.toLowerCase()));

    return (
        <div className="h-full flex flex-col bg-slate-900/50 border-r border-white/5 w-64">
            <div className="p-4 border-b border-white/5">
                <div className="relative">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
                    <input
                        type="text"
                        placeholder="Search tables..."
                        value={filter}
                        onChange={e => setFilter(e.target.value)}
                        className="w-full bg-slate-950 border border-white/10 rounded-lg py-2 pl-9 pr-4 text-sm text-slate-300 focus:outline-none focus:border-indigo-500/50"
                    />
                </div>
            </div>

            <div className="flex-1 overflow-y-auto p-2 space-y-1">
                {loading ? (
                    <div className="text-center p-4 text-slate-500 text-xs">Loading schema...</div>
                ) : filtered.length === 0 ? (
                    <div className="text-center p-4 text-slate-500 text-xs">No tables found</div>
                ) : (
                    filtered.map(table => (
                        <button
                            key={table.name}
                            onClick={() => onSelectTable(table.name)}
                            className="w-full flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-white/5 text-left group transition-colors"
                        >
                            <TableIcon className="w-4 h-4 text-slate-500 group-hover:text-indigo-400 transition-colors" />
                            <span className="text-sm text-slate-400 group-hover:text-slate-200 truncate flex-1 font-mono">
                                {table.name}
                            </span>
                            <ChevronRight className="w-3 h-3 text-slate-700 group-hover:text-slate-500 opacity-0 group-hover:opacity-100" />
                        </button>
                    ))
                )}
            </div>
        </div>
    );
}
