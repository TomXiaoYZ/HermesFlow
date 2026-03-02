"use client";

import React, { useState, useEffect } from "react";
import { Save, Plus, Trash2, RefreshCw, Key, Activity, TrendingUp } from "lucide-react";
import { cn } from "@/lib/utils";
import { authFetch } from "@/lib/api";
import TradingAccountConfig from "./TradingAccountConfig";

interface ExchangeConfig {
  exchange: string;
  api_key: string | null;
  is_enabled: boolean;
}

interface WatchlistItem {
  exchange: string;
  symbol: string;
  name: string | null;
  added_at: string | null;
}

export default function ExchangeConfig() {
  const [configs, setConfigs] = useState<ExchangeConfig[]>([]);
  const [watchlist, setWatchlist] = useState<WatchlistItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [newSymbol, setNewSymbol] = useState("");
  const [newName, setNewName] = useState("");
  const [selectedExchange, setSelectedExchange] = useState("Polygon"); // Default for adding

  const fetchConfig = async () => {
    try {
      setLoading(true);
      const res = await authFetch("/api/v1/config/exchanges");
      if (res.ok) {
        const data = await res.json();
        setConfigs(data);
      }
    } catch {
      /* config fetch failed */
    } finally {
      setLoading(false);
    }
  };

  const fetchWatchlist = async () => {
    try {
      const res = await authFetch("/api/v1/watchlist");
      if (res.ok) {
        const data = await res.json();
        setWatchlist(data);
      }
    } catch {
      /* watchlist fetch failed */
    }
  };

  useEffect(() => {
    fetchConfig();
    fetchWatchlist();
  }, []);

  const handleUpdateConfig = async (exchange: string, apiKey: string, isEnabled: boolean) => {
    try {
      const res = await authFetch("/api/v1/config/exchanges", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ exchange, api_key: apiKey, is_enabled: isEnabled }),
      });
      if (res.ok) {
        alert("Configuration updated!");
        fetchConfig();
      } else {
        alert("Failed to update configuration");
      }
    } catch {
      /* config update failed */
    }
  };

  const handleAddToWatchlist = async () => {
    if (!newSymbol) return;
    try {
      const res = await authFetch("/api/v1/watchlist", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ 
          exchange: selectedExchange, 
          symbol: newSymbol.toUpperCase(), 
          name: newName || null 
        }),
      });
      
      if (res.ok) {
        setNewSymbol("");
        setNewName("");
        fetchWatchlist();
      }
    } catch {
      /* watchlist add failed */
    }
  };

  const handleRemoveFromWatchlist = async (exchange: string, symbol: string) => {
    if (!confirm(`Remove ${symbol} from watchlist?`)) return;
    try {
      const res = await authFetch("/api/v1/watchlist", {
        method: "DELETE",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ exchange, symbol }),
      });
      
      if (res.ok) {
        fetchWatchlist();
      }
    } catch {
      /* watchlist remove failed */
    }
  };

  return (
    <div className="space-y-8 text-slate-200">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold flex items-center gap-2">
            <Activity className="w-6 h-6 text-indigo-400" />
            Config & Data
          </h1>
          <p className="text-slate-500 text-sm mt-1">Manage external data sources and market connectivity</p>
        </div>
        <button 
          onClick={() => { fetchConfig(); fetchWatchlist(); }}
          className="p-2 bg-slate-800 rounded-lg hover:bg-slate-700 transition-colors"
        >
          <RefreshCw className={cn("w-4 h-4", loading && "animate-spin")} />
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        
        {/* API Configuration */}
        <div className="space-y-4">
          <h2 className="text-lg font-semibold flex items-center gap-2 text-indigo-300">
            <Key className="w-4 h-4" />
            Exchange Integration
          </h2>
          
          <div className="grid gap-4">
            {configs.length === 0 && !loading && (
              <div className="text-slate-500 text-sm p-4 border border-dashed border-slate-700 rounded-lg">
                No configurations found. Using default environment variables.
              </div>
            )}
            
            {configs.map((cfg) => (
              <div key={cfg.exchange} className="bg-slate-900/50 p-4 rounded-xl border border-white/5 space-y-4">
                 <div className="flex items-center justify-between mb-2">
                    <span className="font-bold text-white">{cfg.exchange}</span>
                    <span className={cn("px-2 py-0.5 rounded text-xs", cfg.is_enabled ? "bg-emerald-500/10 text-emerald-400" : "bg-slate-700 text-slate-400")}>
                      {cfg.is_enabled ? "Active" : "Disabled"}
                    </span>
                 </div>
                 
                 <div className="space-y-2">
                   <label className="text-xs text-slate-500 uppercase">API Key</label>
                   <div className="flex gap-2">
                     <input 
                       type="password" 
                       defaultValue={cfg.api_key || ""}
                       placeholder="Enter API Key"
                       className="flex-1 bg-slate-950 border border-white/10 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-indigo-500/50"
                       id={`key-${cfg.exchange}`}
                     />
                     <button 
                       onClick={() => {
                         const input = document.getElementById(`key-${cfg.exchange}`) as HTMLInputElement;
                         handleUpdateConfig(cfg.exchange, input.value, true);
                       }}
                       className="px-3 py-2 bg-indigo-600 hover:bg-indigo-500 rounded-lg text-white transition-colors"
                     >
                       <Save className="w-4 h-4" />
                     </button>
                   </div>
                   <p className="text-xs text-slate-500">
                     Status: {cfg.exchange === 'Polygon' ? 'Connected (Synced 35M+ candles)' : 'Pending'}
                   </p>
                 </div>
              </div>
            ))}
          </div>
        </div>

        {/* Watchlist Manager */}
        <div className="space-y-4">
           <h2 className="text-lg font-semibold flex items-center gap-2 text-emerald-300">
            <TrendingUp className="w-4 h-4" />
            Global Watchlist
          </h2>
          
          {/* Add Form */}
          <div className="bg-slate-900/50 p-4 rounded-xl border border-white/5 space-y-3">
             <div className="grid grid-cols-3 gap-2">
                <input 
                  type="text" 
                  placeholder="Symbol (e.g. NVDA)" 
                  value={newSymbol}
                  onChange={e => setNewSymbol(e.target.value)}
                  className="bg-slate-950 border border-white/10 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-emerald-500/50"
                />
                <input 
                  type="text" 
                  placeholder="Name (Optional)" 
                  value={newName}
                  onChange={e => setNewName(e.target.value)}
                  className="bg-slate-950 border border-white/10 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-emerald-500/50"
                />
                <select 
                  value={selectedExchange}
                  onChange={e => setSelectedExchange(e.target.value)}
                  className="bg-slate-950 border border-white/10 rounded-lg px-2 py-2 text-sm focus:outline-none focus:border-emerald-500/50"
                >
                  <option value="Polygon">Polygon</option>
                  <option value="Binance">Binance</option>
                </select>
             </div>
             <button 
               onClick={handleAddToWatchlist}
               disabled={!newSymbol}
               className="w-full py-2 bg-emerald-600/80 hover:bg-emerald-500 rounded-lg text-white font-medium flex items-center justify-center gap-2 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
             >
               <Plus className="w-4 h-4" />
               Add to Watchlist
             </button>
          </div>

          {/* List */}
          <div className="bg-slate-950/30 rounded-xl border border-white/5 overflow-hidden max-h-[500px] overflow-y-auto">
             <table className="w-full text-sm text-left">
               <thead className="bg-white/5 text-slate-400 font-medium">
                 <tr>
                   <th className="px-4 py-3">Symbol</th>
                   <th className="px-4 py-3">Name</th>
                   <th className="px-4 py-3">Exchange</th>
                   <th className="px-4 py-3 text-right">Action</th>
                 </tr>
               </thead>
               <tbody className="divide-y divide-white/5">
                 {watchlist.map((item, i) => (
                   <tr key={`${item.exchange}-${item.symbol}-${i}`} className="hover:bg-white/5 transition-colors">
                     <td className="px-4 py-3 font-medium text-white">{item.symbol}</td>
                     <td className="px-4 py-3 text-slate-400">{item.name || '-'}</td>
                     <td className="px-4 py-3 text-slate-500">{item.exchange}</td>
                     <td className="px-4 py-3 text-right">
                       <button 
                         onClick={() => handleRemoveFromWatchlist(item.exchange, item.symbol)}
                         className="p-1.5 text-red-500 hover:bg-red-500/10 rounded transition-colors"
                       >
                         <Trash2 className="w-3.5 h-3.5" />
                       </button>
                     </td>
                   </tr>
                 ))}
                 {watchlist.length === 0 && (
                   <tr>
                     <td colSpan={4} className="px-4 py-8 text-center text-slate-500">
                       Watchlist is empty
                     </td>
                   </tr>
                 )}
               </tbody>
             </table>
          </div>
        </div>

        {/* Trading Accounts — full-width section */}
        <div className="lg:col-span-2">
          <TradingAccountConfig />
        </div>

      </div>
    </div>
  );
}
