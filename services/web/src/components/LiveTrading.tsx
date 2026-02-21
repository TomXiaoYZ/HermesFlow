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
    cost_basis: string;
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
    initial_capital: string;
    position_count: number;
    total_cost_basis: string;
    total_market_value: string;
    unrealized_pnl: string;
    cash_balance: string;
    net_liquidation: string;
    total_commissions: string;
    total_trades: number;
    realized_pnl: string;
    cache_updated_at: string | null;
}

interface LiveTradingProps {
    signals: TradeSignal[];
    portfolioData: PortfolioData;
}

const GATEWAY_BASE = "http://localhost:8080";

export default function LiveTrading({ signals, portfolioData }: LiveTradingProps) {
    const [selectedAccount, setSelectedAccount] = useState<string>("");
    const [tradeHistory, setTradeHistory] = useState<TradeOrder[]>([]);
    const [dbPositions, setDbPositions] = useState<DBPosition[]>([]);
    const [accountSummary, setAccountSummary] = useState<AccountSummary[]>([]);
    const [selectedStrategy, setSelectedStrategy] = useState<StrategyDetail | null>(null);
    const [showDrilldown, setShowDrilldown] = useState(false);
    const [loading, setLoading] = useState(false);

    // Auto-select first account when data loads (or if current selection becomes invalid)
    useEffect(() => {
        if (accountSummary.length > 0 && !accountSummary.find((a) => a.account_id === selectedAccount)) {
            setSelectedAccount(accountSummary[0].account_id);
        }
    }, [accountSummary, selectedAccount]);

    const modeForAccount = accountSummary.find((a) => a.account_id === selectedAccount)?.mode;

    const fetchTradeHistory = useCallback(async () => {
        const params = new URLSearchParams({ limit: "50" });
        if (modeForAccount) {
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
    const filteredPositions = dbPositions.filter((p) => p.account_id === selectedAccount);

    // Selected account detail
    const selectedAccountDetail = accountSummary.find((a) => a.account_id === selectedAccount);

    return (
        <div className="space-y-6">
            {/* Account Financial Overview */}
            {selectedAccountDetail && <AccountOverview account={selectedAccountDetail} />}

            {/* Account Tabs */}
            <div className="flex gap-2">
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

            {/* Active Positions */}
            <div className="bg-slate-900/50 border border-white/10 rounded-xl overflow-hidden">
                <div className="px-6 py-4 border-b border-white/5">
                    <h3 className="text-sm font-semibold text-slate-300 uppercase tracking-wider">Active Positions</h3>
                </div>
                <div className="overflow-x-auto">
                    <table className="w-full text-sm">
                        <thead>
                            <tr className="text-slate-500 text-xs uppercase">
                                <th className="text-left px-6 py-3">Symbol</th>
                                <th className="text-right px-6 py-3">Qty</th>
                                <th className="text-right px-6 py-3">Avg Cost</th>
                                <th className="text-right px-6 py-3">Last Price</th>
                                <th className="text-right px-6 py-3">Cost Basis</th>
                                <th className="text-right px-6 py-3">Mkt Value</th>
                                <th className="text-right px-6 py-3">PnL ($)</th>
                                <th className="text-right px-6 py-3">PnL (%)</th>
                                <th className="text-right px-6 py-3">Weight</th>
                            </tr>
                        </thead>
                        <tbody>
                            {filteredPositions.map((pos) => {
                                const qty = parseFloat(pos.quantity);
                                const avgPrice = parseFloat(pos.avg_price);
                                const curPrice = pos.current_price ? parseFloat(pos.current_price) : null;
                                const costBasis = parseFloat(pos.cost_basis);
                                const mktValue = pos.market_value ? parseFloat(pos.market_value) : null;
                                const pnl = pos.unrealized_pnl ? parseFloat(pos.unrealized_pnl) : 0;
                                const pnlPct = costBasis > 0 ? (pnl / costBasis) * 100 : 0;
                                const totalMktVal = selectedAccountDetail ? parseFloat(selectedAccountDetail.total_market_value) : 0;
                                const weight = totalMktVal > 0 && mktValue != null ? (Math.abs(mktValue) / totalMktVal) * 100 : 0;
                                return (
                                    <tr key={`${pos.account_id}-${pos.symbol}`} className="border-t border-white/5 hover:bg-white/5 transition-colors">
                                        <td className="px-6 py-3 font-medium text-white">{pos.symbol}</td>
                                        <td className="px-6 py-3 text-right text-slate-300">{qty}</td>
                                        <td className="px-6 py-3 text-right text-slate-300">${avgPrice.toFixed(2)}</td>
                                        <td className="px-6 py-3 text-right text-slate-300">
                                            {curPrice != null ? `$${curPrice.toFixed(2)}` : "—"}
                                        </td>
                                        <td className="px-6 py-3 text-right text-slate-300">
                                            ${costBasis.toLocaleString(undefined, { minimumFractionDigits: 2 })}
                                        </td>
                                        <td className="px-6 py-3 text-right text-slate-300">
                                            {mktValue != null ? `$${mktValue.toLocaleString(undefined, { minimumFractionDigits: 2 })}` : "—"}
                                        </td>
                                        <td className={cn("px-6 py-3 text-right font-medium", pnl >= 0 ? "text-emerald-400" : "text-red-400")}>
                                            {pnl >= 0 ? "+" : ""}${Math.abs(pnl).toLocaleString(undefined, { minimumFractionDigits: 2 })}
                                        </td>
                                        <td className={cn("px-6 py-3 text-right font-medium", pnlPct >= 0 ? "text-emerald-400" : "text-red-400")}>
                                            {pnlPct >= 0 ? "+" : ""}{pnlPct.toFixed(1)}%
                                        </td>
                                        <td className="px-6 py-3 text-right text-slate-400">
                                            {weight.toFixed(1)}%
                                        </td>
                                    </tr>
                                );
                            })}
                            {filteredPositions.length === 0 && (
                                <tr>
                                    <td colSpan={9} className="px-6 py-8 text-center text-slate-500">
                                        No open positions
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
                                        {order.created_at ? new Date(order.created_at).toLocaleString() : "—"}
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
                                    <td className="px-6 py-3 text-xs text-slate-400 max-w-[200px] truncate" title={order.strategy_id || ""}>
                                        {order.strategy_id || "—"}
                                    </td>
                                </tr>
                            ))}
                            {tradeHistory.length === 0 && (
                                <tr>
                                    <td colSpan={7} className="px-6 py-8 text-center text-slate-500">
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
                    {signals.length === 0 && tradeHistory.filter(o => o.status === "Filled").length > 0 && (
                        <>
                            <div className="px-6 py-2 text-xs text-slate-500 bg-slate-800/30">
                                Recent Activity (from trade history)
                            </div>
                            {tradeHistory.filter(o => o.status === "Filled").slice(0, 10).map((order) => (
                                <div key={order.order_id} className="px-6 py-3 flex items-center gap-4">
                                    <span className="relative flex h-2 w-2">
                                        <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-400"></span>
                                    </span>
                                    <span className={cn("text-xs font-bold", order.side === "Buy" || order.side === "BUY" ? "text-emerald-400" : "text-red-400")}>
                                        {order.side}
                                    </span>
                                    <span className="text-sm font-medium text-white">{order.symbol}</span>
                                    <span className="text-sm text-slate-400">x{order.quantity}</span>
                                    <span className="text-sm text-slate-400">
                                        {order.avg_price ? `@ $${parseFloat(order.avg_price).toFixed(2)}` : ""}
                                    </span>
                                    <span className="text-xs text-slate-500 ml-auto">
                                        {order.created_at ? new Date(order.created_at).toLocaleString() : ""}
                                    </span>
                                </div>
                            ))}
                        </>
                    )}
                    {signals.length === 0 && tradeHistory.filter(o => o.status === "Filled").length === 0 && (
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

function formatDollar(value: number): string {
    const abs = Math.abs(value);
    const formatted = abs.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
    return value < 0 ? `-$${formatted}` : `$${formatted}`;
}

function AccountOverview({ account }: { account: AccountSummary }) {
    const netLiq = parseFloat(account.net_liquidation);
    const cash = parseFloat(account.cash_balance);
    const mktValue = parseFloat(account.total_market_value);
    const unrealizedPnl = parseFloat(account.unrealized_pnl);
    const initialCapital = parseFloat(account.initial_capital);
    const costBasis = parseFloat(account.total_cost_basis);
    const commissions = parseFloat(account.total_commissions);

    const realizedPnl = parseFloat(account.realized_pnl);
    const totalReturn = unrealizedPnl + realizedPnl;
    const returnPct = initialCapital > 0 ? (totalReturn / initialCapital) * 100 : 0;
    const deployedPct = netLiq > 0 ? (costBasis / netLiq) * 100 : 0;

    return (
        <div className="space-y-3">
            {/* Account Label Row */}
            <div className="flex items-center gap-2">
                <span className="text-lg font-semibold text-white">{account.label}</span>
                <span className="text-sm text-slate-400">{account.broker_account}</span>
                <span
                    className={cn(
                        "text-xs font-bold px-2 py-0.5 rounded-full border",
                        account.is_enabled
                            ? "bg-emerald-500/20 text-emerald-400 border-emerald-500/30"
                            : "bg-red-500/20 text-red-400 border-red-500/30"
                    )}
                >
                    {account.is_enabled ? "Enabled" : "Disabled"}
                </span>
                {account.cache_updated_at && (
                    <span className="text-xs text-slate-500 ml-auto">
                        IBKR sync: {new Date(account.cache_updated_at).toLocaleString()}
                    </span>
                )}
            </div>

            {/* Row 1: Portfolio Overview */}
            <div className="grid grid-cols-4 gap-3">
                <OverviewCard label="Net Liquidation" value={formatDollar(netLiq)} />
                <OverviewCard label="Cash Balance" value={formatDollar(cash)} />
                <OverviewCard label="Stock Market Value" value={formatDollar(mktValue)} />
                <OverviewCard
                    label="Unrealized PnL"
                    value={`${unrealizedPnl >= 0 ? "+" : ""}${formatDollar(unrealizedPnl)}${costBasis > 0 ? ` (${unrealizedPnl >= 0 ? "+" : ""}${((unrealizedPnl / costBasis) * 100).toFixed(1)}%)` : ""}`}
                    valueColor={unrealizedPnl >= 0 ? "text-emerald-400" : "text-red-400"}
                />
            </div>

            {/* Row 2: Performance & Limits */}
            <div className="grid grid-cols-4 gap-3">
                <OverviewCard
                    label="Total Return"
                    value={`${totalReturn >= 0 ? "+" : ""}${formatDollar(totalReturn)} (${totalReturn >= 0 ? "+" : ""}${returnPct.toFixed(1)}%)`}
                    valueColor={totalReturn >= 0 ? "text-emerald-400" : "text-red-400"}
                />
                <OverviewCard
                    label="Capital Deployed"
                    value={`${deployedPct.toFixed(1)}%`}
                    subtitle={`${formatDollar(costBasis)} of net liq`}
                />
                <OverviewCard
                    label="Commissions"
                    value={formatDollar(commissions)}
                    subtitle={`${account.total_trades} trade${account.total_trades !== 1 ? "s" : ""}`}
                />
                <OverviewCard
                    label="Positions"
                    value={`${account.position_count} / ${account.max_positions}`}
                    subtitle={`Max order $${account.max_order_value} | Max loss $${account.max_daily_loss}`}
                />
            </div>
        </div>
    );
}

function OverviewCard({
    label,
    value,
    subtitle,
    valueColor,
}: {
    label: string;
    value: string;
    subtitle?: string;
    valueColor?: string;
}) {
    return (
        <div className="bg-slate-900/50 border border-white/10 rounded-xl px-4 py-3">
            <p className="text-xs text-slate-500 uppercase tracking-wider mb-1">{label}</p>
            <p className={cn("text-sm font-semibold", valueColor || "text-white")}>{value}</p>
            {subtitle && <p className="text-xs text-slate-500 mt-0.5">{subtitle}</p>}
        </div>
    );
}

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
