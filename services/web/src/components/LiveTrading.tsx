"use client";

import React, { useState, useEffect, useCallback } from "react";
import { cn } from "@/lib/utils";

// Types

interface TradeSignal {
    id: string;
    timestamp: string;
    symbol: string;
    side: "BUY" | "SELL";
    price: number;
    quantity: number;
    status: "PENDING" | "FILLED" | "REJECTED";
}

interface Position {
    symbol: string;
    quantity: number;
    market_value: number;
}

interface PortfolioData {
    cash: number;
    total_equity: number;
    positions: Position[];
}

interface TradeOrder {
    order_id: string;
    symbol: string;
    side: string;
    quantity: string;
    filled_qty: string | null;
    avg_price: string | null;
    status: string;
    strategy_id: string | null;
    mode: string | null;
    account_id: string | null;
    created_at: string | null;
}

interface StrategyDetail {
    strategy_id: string;
    orders: TradeOrder[];
    generation: {
        exchange: string;
        symbol: string;
        mode: string;
        generation: number;
        fitness: number;
        metadata: Record<string, unknown>;
        timestamp: string;
    } | null;
}

interface DBPosition {
    account_id: string;
    exchange: string;
    symbol: string;
    quantity: string;
    avg_price: string;
    current_price: string | null;
    market_value: string | null;
    unrealized_pnl: string | null;
    updated_at: string | null;
    price_time: string | null;
}

interface AccountSummary {
    account_id: string;
    label: string;
    broker_account: string | null;
    mode: string;
    is_enabled: boolean;
    max_order_value: string;
    max_positions: number;
    max_daily_loss: string;
    position_count: number;
    total_value: string;
    total_unrealized_pnl: string;
}

interface LiveTradingProps {
    signals: TradeSignal[];
    portfolioData: PortfolioData;
}

const GATEWAY_BASE = "http://localhost:8080";

export default function LiveTrading({ signals, portfolioData }: LiveTradingProps) {
    const [selectedAccount, setSelectedAccount] = useState<string>("all");
    const [tradeHistory, setTradeHistory] = useState<TradeOrder[]>([]);
    const [dbPositions, setDbPositions] = useState<DBPosition[]>([]);
    const [accountSummary, setAccountSummary] = useState<AccountSummary[]>([]);
    const [selectedStrategy, setSelectedStrategy] = useState<StrategyDetail | null>(null);
    const [showDrilldown, setShowDrilldown] = useState(false);
    const [loading, setLoading] = useState(false);

    const modeForAccount = accountSummary.find((a) => a.account_id === selectedAccount)?.mode;

    const fetchTradeHistory = useCallback(async () => {
        const params = new URLSearchParams({ limit: "50" });
        if (selectedAccount !== "all" && modeForAccount) {
            params.set("mode", modeForAccount);
        }
        try {
            const res = await fetch(`${GATEWAY_BASE}/api/v1/trades/history?${params}`);
            if (res.ok) {
                const data = await res.json();
                if (Array.isArray(data)) setTradeHistory(data);
            }
        } catch {
            // Gateway unavailable — keep stale data
        }
    }, [selectedAccount, modeForAccount]);

    const fetchPositions = useCallback(async () => {
        try {
            const res = await fetch(`${GATEWAY_BASE}/api/v1/trades/positions`);
            if (res.ok) {
                const data = await res.json();
                if (Array.isArray(data)) setDbPositions(data);
            }
        } catch {
            // Gateway unavailable
        }
    }, []);

    const fetchAccountSummary = useCallback(async () => {
        try {
            const res = await fetch(`${GATEWAY_BASE}/api/v1/trades/account-summary`);
            if (res.ok) {
                const data = await res.json();
                if (Array.isArray(data)) setAccountSummary(data);
            }
        } catch {
            // Gateway unavailable
        }
    }, []);

    useEffect(() => {
        fetchTradeHistory();
        fetchPositions();
        fetchAccountSummary();
        const interval = setInterval(() => {
            fetchTradeHistory();
            fetchPositions();
            fetchAccountSummary();
        }, 15000);
        return () => clearInterval(interval);
    }, [fetchTradeHistory, fetchPositions, fetchAccountSummary]);

    const openDrilldown = async (strategyId: string) => {
        setLoading(true);
        setShowDrilldown(true);
        try {
            const res = await fetch(
                `${GATEWAY_BASE}/api/v1/trades/strategy/${encodeURIComponent(strategyId)}`
            );
            if (res.ok) {
                const data = await res.json();
                setSelectedStrategy(data);
            }
        } catch {
            // Gateway unavailable
        } finally {
            setLoading(false);
        }
    };

    // Filter DB positions by selected account
    const filteredPositions =
        selectedAccount === "all"
            ? dbPositions
            : dbPositions.filter((p) => p.account_id === selectedAccount);

    // Filter live positions from WebSocket
    const livePositions = portfolioData.positions;

    // Aggregate stats
    const totalPositions = accountSummary.reduce((sum, a) => sum + a.position_count, 0);

    // Selected account detail
    const selectedAccountDetail = accountSummary.find((a) => a.account_id === selectedAccount);

    return (
        <div className="space-y-6">
            {/* Account Header */}
            <div className="bg-slate-900/50 border border-white/10 rounded-xl p-6">
                <div className="flex items-center justify-between">
                    <div>
                        <div className="flex items-center gap-2">
                            <span className="text-lg font-semibold text-white">
                                {selectedAccount === "all"
                                    ? "All Accounts"
                                    : selectedAccountDetail?.label || selectedAccount}
                            </span>
                            {selectedAccountDetail && (
                                <>
                                    <span className="text-sm text-slate-400">
                                        {selectedAccountDetail.broker_account}
                                    </span>
                                    <span
                                        className={cn(
                                            "text-xs font-bold px-2 py-0.5 rounded-full",
                                            selectedAccountDetail.is_enabled
                                                ? "bg-emerald-500/20 text-emerald-400 border border-emerald-500/30"
                                                : "bg-red-500/20 text-red-400 border border-red-500/30"
                                        )}
                                    >
                                        {selectedAccountDetail.is_enabled ? "Enabled" : "Disabled"}
                                    </span>
                                </>
                            )}
                            {selectedAccount === "all" && (
                                <span className="text-xs font-bold px-2 py-0.5 rounded-full bg-yellow-500/20 text-yellow-400 border border-yellow-500/30">
                                    Paper
                                </span>
                            )}
                        </div>
                        <div className="flex gap-6 mt-2 text-sm text-slate-400">
                            <span>
                                Net Liq:{" "}
                                <span className="text-white font-medium">
                                    ${portfolioData.total_equity.toLocaleString(undefined, { minimumFractionDigits: 2 })}
                                </span>
                            </span>
                            <span>
                                Cash:{" "}
                                <span className="text-white font-medium">
                                    ${portfolioData.cash.toLocaleString(undefined, { minimumFractionDigits: 2 })}
                                </span>
                            </span>
                            <span>
                                Positions:{" "}
                                <span className="text-white font-medium">
                                    {selectedAccount === "all"
                                        ? totalPositions || livePositions.length || dbPositions.length
                                        : selectedAccountDetail?.position_count ?? 0}
                                </span>
                            </span>
                            {selectedAccountDetail && (
                                <>
                                    <span>
                                        Max Order:{" "}
                                        <span className="text-white font-medium">${selectedAccountDetail.max_order_value}</span>
                                    </span>
                                    <span>
                                        Max Positions:{" "}
                                        <span className="text-white font-medium">{selectedAccountDetail.max_positions}</span>
                                    </span>
                                    <span>
                                        Max Daily Loss:{" "}
                                        <span className="text-white font-medium">${selectedAccountDetail.max_daily_loss}</span>
                                    </span>
                                </>
                            )}
                        </div>
                    </div>
                </div>
            </div>

            {/* Account Tabs */}
            <div className="flex gap-2">
                <button
                    onClick={() => setSelectedAccount("all")}
                    className={cn(
                        "px-4 py-2 rounded-lg text-sm font-medium transition-all border",
                        selectedAccount === "all"
                            ? "bg-orange-500/20 border-orange-500/50 text-orange-300"
                            : "bg-slate-800/50 border-white/10 text-slate-400 hover:text-white hover:border-white/20"
                    )}
                >
                    All Accounts
                </button>
                {accountSummary.map((account) => (
                    <button
                        key={account.account_id}
                        onClick={() => setSelectedAccount(account.account_id)}
                        className={cn(
                            "px-4 py-2 rounded-lg text-sm font-medium transition-all border",
                            selectedAccount === account.account_id
                                ? "bg-orange-500/20 border-orange-500/50 text-orange-300"
                                : "bg-slate-800/50 border-white/10 text-slate-400 hover:text-white hover:border-white/20"
                        )}
                    >
                        {account.label}
                    </button>
                ))}
            </div>

            {/* Account Summary Cards (shown in "All" view) */}
            {selectedAccount === "all" && accountSummary.length > 0 && (
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {accountSummary.map((account) => (
                        <div
                            key={account.account_id}
                            className="bg-slate-900/50 border border-white/5 rounded-xl p-4 cursor-pointer hover:border-white/20 transition-colors"
                            onClick={() => setSelectedAccount(account.account_id)}
                        >
                            <div className="flex items-center justify-between mb-3">
                                <div className="flex items-center gap-2">
                                    <span className="text-sm font-semibold text-white">{account.label}</span>
                                    <span className="text-xs text-slate-500">{account.broker_account}</span>
                                </div>
                                <span
                                    className={cn(
                                        "text-xs font-medium px-2 py-0.5 rounded-full border",
                                        account.is_enabled
                                            ? "bg-emerald-500/20 text-emerald-400 border-emerald-500/30"
                                            : "bg-red-500/20 text-red-400 border-red-500/30"
                                    )}
                                >
                                    {account.is_enabled ? "Active" : "Disabled"}
                                </span>
                            </div>
                            <div className="grid grid-cols-4 gap-3">
                                <div>
                                    <p className="text-xs text-slate-500">Positions</p>
                                    <p className="text-sm font-semibold text-white">{account.position_count}</p>
                                </div>
                                <div>
                                    <p className="text-xs text-slate-500">Market Value</p>
                                    <p className="text-sm font-semibold text-white">
                                        ${parseFloat(account.total_value || "0").toLocaleString(undefined, { minimumFractionDigits: 2 })}
                                    </p>
                                </div>
                                <div>
                                    <p className="text-xs text-slate-500">Unrealized PnL</p>
                                    {(() => {
                                        const pnl = parseFloat(account.total_unrealized_pnl || "0");
                                        return (
                                            <p className={cn("text-sm font-semibold", pnl >= 0 ? "text-emerald-400" : "text-red-400")}>
                                                {pnl >= 0 ? "+" : ""}{pnl.toLocaleString(undefined, { minimumFractionDigits: 2 })}
                                            </p>
                                        );
                                    })()}
                                </div>
                                <div>
                                    <p className="text-xs text-slate-500">Mode</p>
                                    <p className="text-sm font-semibold text-white capitalize">
                                        {account.mode.replace("_", " ")}
                                    </p>
                                </div>
                            </div>
                        </div>
                    ))}
                </div>
            )}

            {/* Active Positions */}
            <div className="bg-slate-900/50 border border-white/10 rounded-xl overflow-hidden">
                <div className="px-6 py-4 border-b border-white/5">
                    <h3 className="text-sm font-semibold text-slate-300 uppercase tracking-wider">Active Positions</h3>
                </div>
                <div className="overflow-x-auto">
                    <table className="w-full text-sm">
                        <thead>
                            <tr className="text-slate-500 text-xs uppercase">
                                {selectedAccount === "all" && <th className="text-left px-6 py-3">Account</th>}
                                <th className="text-left px-6 py-3">Symbol</th>
                                <th className="text-right px-6 py-3">Qty</th>
                                <th className="text-right px-6 py-3">Avg Cost</th>
                                <th className="text-right px-6 py-3">Last Price</th>
                                <th className="text-right px-6 py-3">Mkt Value</th>
                                <th className="text-right px-6 py-3">PnL</th>
                            </tr>
                        </thead>
                        <tbody>
                            {livePositions.length > 0
                                ? livePositions.map((pos) => {
                                      const unitCost = pos.market_value / (pos.quantity || 1);
                                      const pnl = pos.market_value - pos.quantity * unitCost;
                                      return (
                                          <tr key={pos.symbol} className="border-t border-white/5 hover:bg-white/5 transition-colors">
                                              {selectedAccount === "all" && <td className="px-6 py-3 text-slate-400">—</td>}
                                              <td className="px-6 py-3 font-medium text-white">{pos.symbol}</td>
                                              <td className="px-6 py-3 text-right text-slate-300">{pos.quantity}</td>
                                              <td className="px-6 py-3 text-right text-slate-300">${unitCost.toFixed(2)}</td>
                                              <td className="px-6 py-3 text-right text-slate-300">—</td>
                                              <td className="px-6 py-3 text-right text-slate-300">
                                                  ${pos.market_value.toLocaleString(undefined, { minimumFractionDigits: 2 })}
                                              </td>
                                              <td className={cn("px-6 py-3 text-right font-medium", pnl >= 0 ? "text-emerald-400" : "text-red-400")}>
                                                  {pnl >= 0 ? "+" : ""}{pnl.toFixed(2)}
                                              </td>
                                          </tr>
                                      );
                                  })
                                : filteredPositions.map((pos) => {
                                      const qty = parseFloat(pos.quantity);
                                      const avgPrice = parseFloat(pos.avg_price);
                                      const curPrice = pos.current_price ? parseFloat(pos.current_price) : null;
                                      const mktValue = pos.market_value ? parseFloat(pos.market_value) : null;
                                      const pnl = pos.unrealized_pnl ? parseFloat(pos.unrealized_pnl) : 0;
                                      const costBasis = Math.abs(qty) * avgPrice;
                                      const pnlPct = costBasis > 0 ? (pnl / costBasis) * 100 : 0;
                                      const accountLabel = accountSummary.find((a) => a.account_id === pos.account_id)?.label || pos.account_id;
                                      return (
                                          <tr key={`${pos.account_id}-${pos.symbol}`} className="border-t border-white/5 hover:bg-white/5 transition-colors">
                                              {selectedAccount === "all" && (
                                                  <td className="px-6 py-3 text-slate-400 text-xs">{accountLabel}</td>
                                              )}
                                              <td className="px-6 py-3 font-medium text-white">{pos.symbol}</td>
                                              <td className="px-6 py-3 text-right text-slate-300">{qty}</td>
                                              <td className="px-6 py-3 text-right text-slate-300">${avgPrice.toFixed(2)}</td>
                                              <td className="px-6 py-3 text-right text-slate-300">
                                                  {curPrice != null ? `$${curPrice.toFixed(2)}` : "—"}
                                              </td>
                                              <td className="px-6 py-3 text-right text-slate-300">
                                                  {mktValue != null ? `$${mktValue.toLocaleString(undefined, { minimumFractionDigits: 2 })}` : "—"}
                                              </td>
                                              <td className={cn("px-6 py-3 text-right font-medium", pnl >= 0 ? "text-emerald-400" : "text-red-400")}>
                                                  {pnl >= 0 ? "+" : ""}{pnl.toFixed(2)} ({pnlPct >= 0 ? "+" : ""}{pnlPct.toFixed(1)}%)
                                              </td>
                                          </tr>
                                      );
                                  })}
                            {livePositions.length === 0 && filteredPositions.length === 0 && (
                                <tr>
                                    <td colSpan={selectedAccount === "all" ? 8 : 7} className="px-6 py-8 text-center text-slate-500">
                                        No positions found
                                    </td>
                                </tr>
                            )}
                        </tbody>
                    </table>
                </div>
            </div>

            {/* Trade History */}
            <div className="bg-slate-900/50 border border-white/10 rounded-xl overflow-hidden">
                <div className="px-6 py-4 border-b border-white/5">
                    <h3 className="text-sm font-semibold text-slate-300 uppercase tracking-wider">Trade History</h3>
                </div>
                <div className="overflow-x-auto">
                    <table className="w-full text-sm">
                        <thead>
                            <tr className="text-slate-500 text-xs uppercase">
                                <th className="text-left px-6 py-3">Time</th>
                                <th className="text-left px-6 py-3">Symbol</th>
                                <th className="text-left px-6 py-3">Side</th>
                                <th className="text-right px-6 py-3">Qty</th>
                                <th className="text-right px-6 py-3">Price</th>
                                <th className="text-left px-6 py-3">Status</th>
                                <th className="text-left px-6 py-3">Mode</th>
                                <th className="text-left px-6 py-3">Strategy</th>
                            </tr>
                        </thead>
                        <tbody>
                            {tradeHistory.map((order) => (
                                <tr
                                    key={order.order_id}
                                    className="border-t border-white/5 hover:bg-white/5 transition-colors cursor-pointer"
                                    onClick={() => order.strategy_id && openDrilldown(order.strategy_id)}
                                >
                                    <td className="px-6 py-3 text-slate-400 font-mono text-xs">
                                        {order.created_at ? new Date(order.created_at).toLocaleTimeString() : "—"}
                                    </td>
                                    <td className="px-6 py-3 font-medium text-white">{order.symbol}</td>
                                    <td className="px-6 py-3">
                                        <span
                                            className={cn(
                                                "text-xs font-bold px-2 py-0.5 rounded",
                                                order.side === "Buy" || order.side === "BUY"
                                                    ? "bg-emerald-500/20 text-emerald-400"
                                                    : "bg-red-500/20 text-red-400"
                                            )}
                                        >
                                            {order.side}
                                        </span>
                                    </td>
                                    <td className="px-6 py-3 text-right text-slate-300">{order.quantity}</td>
                                    <td className="px-6 py-3 text-right text-slate-300">
                                        {order.avg_price ? `$${parseFloat(order.avg_price).toFixed(2)}` : "—"}
                                    </td>
                                    <td className="px-6 py-3">
                                        <StatusBadge status={order.status} />
                                    </td>
                                    <td className="px-6 py-3">
                                        {order.mode ? (
                                            <span className="text-xs font-medium px-2 py-0.5 rounded bg-slate-700/50 text-slate-300 capitalize">
                                                {order.mode.replace("_", " ")}
                                            </span>
                                        ) : (
                                            <span className="text-slate-500">—</span>
                                        )}
                                    </td>
                                    <td className="px-6 py-3 text-xs text-slate-400 max-w-[200px] truncate" title={order.strategy_id || ""}>
                                        {order.strategy_id || "—"}
                                    </td>
                                </tr>
                            ))}
                            {tradeHistory.length === 0 && (
                                <tr>
                                    <td colSpan={8} className="px-6 py-8 text-center text-slate-500">
                                        No trade history found
                                    </td>
                                </tr>
                            )}
                        </tbody>
                    </table>
                </div>
            </div>

            {/* Recent Signals (real-time via WebSocket) */}
            <div className="bg-slate-900/50 border border-white/10 rounded-xl overflow-hidden">
                <div className="px-6 py-4 border-b border-white/5">
                    <h3 className="text-sm font-semibold text-slate-300 uppercase tracking-wider">Recent Signals</h3>
                </div>
                <div className="divide-y divide-white/5">
                    {signals.slice(0, 10).map((sig) => (
                        <div key={sig.id} className="px-6 py-3 flex items-center gap-4">
                            <span className="relative flex h-2 w-2">
                                <span
                                    className={cn(
                                        "animate-ping absolute inline-flex h-full w-full rounded-full opacity-75",
                                        sig.status === "PENDING" ? "bg-yellow-400" : sig.status === "FILLED" ? "bg-emerald-400" : "bg-red-400"
                                    )}
                                ></span>
                                <span
                                    className={cn(
                                        "relative inline-flex rounded-full h-2 w-2",
                                        sig.status === "PENDING" ? "bg-yellow-400" : sig.status === "FILLED" ? "bg-emerald-400" : "bg-red-400"
                                    )}
                                ></span>
                            </span>
                            <span className={cn("text-xs font-bold", sig.side === "BUY" ? "text-emerald-400" : "text-red-400")}>
                                {sig.side}
                            </span>
                            <span className="text-sm font-medium text-white">{sig.symbol}</span>
                            <span className="text-sm text-slate-400">x{sig.quantity}</span>
                            <span className="text-sm text-slate-400">@ ${sig.price.toFixed(2)}</span>
                            <span className="ml-auto">
                                <StatusBadge status={sig.status} />
                            </span>
                        </div>
                    ))}
                    {signals.length === 0 && (
                        <div className="px-6 py-8 text-center text-slate-500">No recent signals</div>
                    )}
                </div>
            </div>

            {/* Strategy Drill-down Modal */}
            {showDrilldown && (
                <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" onClick={() => setShowDrilldown(false)}>
                    <div className="bg-slate-900 border border-white/10 rounded-2xl w-full max-w-2xl max-h-[80vh] overflow-y-auto shadow-2xl" onClick={(e) => e.stopPropagation()}>
                        <div className="px-6 py-4 border-b border-white/5 flex items-center justify-between">
                            <h3 className="text-lg font-semibold text-white">Strategy Detail</h3>
                            <button onClick={() => setShowDrilldown(false)} className="text-slate-400 hover:text-white text-xl leading-none">&times;</button>
                        </div>
                        {loading ? (
                            <div className="px-6 py-12 text-center">
                                <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-500 mx-auto"></div>
                            </div>
                        ) : selectedStrategy ? (
                            <div className="p-6 space-y-6">
                                <div>
                                    <p className="text-xs text-slate-500 uppercase tracking-wider mb-1">Strategy ID</p>
                                    <p className="text-sm font-mono text-white break-all">{selectedStrategy.strategy_id}</p>
                                </div>

                                {selectedStrategy.generation && (
                                    <div className="grid grid-cols-2 gap-4">
                                        <MetricCard label="Generation" value={`#${selectedStrategy.generation.generation}`} />
                                        <MetricCard label="Fitness (PnL)" value={selectedStrategy.generation.fitness.toFixed(4)} />
                                        <MetricCard label="Mode" value={selectedStrategy.generation.mode} />
                                        <MetricCard label="Exchange" value={selectedStrategy.generation.exchange} />
                                        {selectedStrategy.generation.metadata && (
                                            <>
                                                {typeof selectedStrategy.generation.metadata.sharpe === "number" && (
                                                    <MetricCard label="Sharpe" value={(selectedStrategy.generation.metadata.sharpe as number).toFixed(2)} />
                                                )}
                                                {typeof selectedStrategy.generation.metadata.max_drawdown === "number" && (
                                                    <MetricCard label="Max DD" value={`${((selectedStrategy.generation.metadata.max_drawdown as number) * 100).toFixed(1)}%`} />
                                                )}
                                                {typeof selectedStrategy.generation.metadata.win_rate === "number" && (
                                                    <MetricCard label="Win Rate" value={`${((selectedStrategy.generation.metadata.win_rate as number) * 100).toFixed(1)}%`} />
                                                )}
                                                {typeof selectedStrategy.generation.metadata.sortino === "number" && (
                                                    <MetricCard label="Sortino" value={(selectedStrategy.generation.metadata.sortino as number).toFixed(2)} />
                                                )}
                                            </>
                                        )}
                                    </div>
                                )}

                                {!selectedStrategy.generation && (
                                    <p className="text-sm text-slate-500">No generation metadata found for this strategy.</p>
                                )}

                                <div>
                                    <p className="text-xs text-slate-500 uppercase tracking-wider mb-2">Orders ({selectedStrategy.orders.length})</p>
                                    <div className="space-y-2">
                                        {selectedStrategy.orders.map((o) => (
                                            <div key={o.order_id} className="flex items-center gap-3 text-sm bg-slate-800/50 rounded-lg px-4 py-2">
                                                <span className="text-slate-400 font-mono text-xs">
                                                    {o.created_at ? new Date(o.created_at).toLocaleString() : "—"}
                                                </span>
                                                <span className={cn("font-bold text-xs", o.side === "Buy" || o.side === "BUY" ? "text-emerald-400" : "text-red-400")}>
                                                    {o.side}
                                                </span>
                                                <span className="text-white font-medium">{o.symbol}</span>
                                                <span className="text-slate-400">x{o.quantity}</span>
                                                <span className="ml-auto">
                                                    <StatusBadge status={o.status} />
                                                </span>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            </div>
                        ) : (
                            <div className="px-6 py-12 text-center text-slate-500">No data available</div>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
}

// Sub-components

function StatusBadge({ status }: { status: string }) {
    const upper = status.toUpperCase();
    const styles: Record<string, string> = {
        FILLED: "bg-emerald-500/20 text-emerald-400 border-emerald-500/30",
        REJECTED: "bg-red-500/20 text-red-400 border-red-500/30",
        FAILED: "bg-red-500/20 text-red-400 border-red-500/30",
        CANCELED: "bg-red-500/20 text-red-400 border-red-500/30",
        CANCELLED: "bg-red-500/20 text-red-400 border-red-500/30",
        PENDING: "bg-yellow-500/20 text-yellow-400 border-yellow-500/30",
        NEW: "bg-blue-500/20 text-blue-400 border-blue-500/30",
        PARTIALLY_FILLED: "bg-cyan-500/20 text-cyan-400 border-cyan-500/30",
    };

    return (
        <span className={cn("text-xs font-medium px-2 py-0.5 rounded-full border", styles[upper] || "bg-slate-500/20 text-slate-400 border-slate-500/30")}>
            {status}
        </span>
    );
}

function MetricCard({ label, value }: { label: string; value: string }) {
    return (
        <div className="bg-slate-800/50 rounded-lg px-4 py-3">
            <p className="text-xs text-slate-500 mb-0.5">{label}</p>
            <p className="text-sm font-semibold text-white">{value}</p>
        </div>
    );
}
