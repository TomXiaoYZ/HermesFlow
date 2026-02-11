import React, { useState } from "react";
import TableBrowser from "@/components/data-discovery/TableBrowser";
import QualityDashboard from "@/components/data-discovery/QualityDashboard";
import SqlIde from "@/components/data-discovery/SqlIde";

export default function DataDiscovery() {
    const [query, setQuery] = useState("SELECT * FROM active_tokens LIMIT 10;");
    const [queryKey, setQueryKey] = useState(0);

    const handleSelectTable = (tableName: string) => {
        setQuery(`SELECT * FROM ${tableName} ORDER BY 1 DESC LIMIT 20;`);
        setQueryKey(prev => prev + 1); // Force re-mount or re-init of IDE if needed
    };

    return (
        <div className="h-[calc(100vh-8rem)] flex border border-white/5 rounded-2xl overflow-hidden bg-slate-950/30 backdrop-blur-sm">
            {/* Left Sidebar */}
            <TableBrowser onSelectTable={handleSelectTable} />

            {/* Main Content */}
            <div className="flex-1 flex flex-col min-w-0">
                <div className="p-6 h-full flex flex-col gap-6 overflow-y-auto">
                    {/* Top: Quality Dashboard */}
                    <div>
                        <h2 className="text-lg font-semibold text-white mb-4">Data Engine Status</h2>
                        <QualityDashboard />
                    </div>

                    {/* Bottom: SQL IDE */}
                    <div className="flex-1 flex flex-col min-h-[500px]">
                        <h2 className="text-lg font-semibold text-white mb-4">SQL Explorer</h2>
                        <div className="flex-1">
                            <SqlIde key={queryKey} initialQuery={query} />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
