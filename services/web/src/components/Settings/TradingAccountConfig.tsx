"use client";

import React, { useState, useEffect } from "react";
import { Shield, Wallet, Save, ChevronDown, ChevronUp } from "lucide-react";
import { cn } from "@/lib/utils";
import { authFetch } from "@/lib/api";

interface TradingAccount {
  account_id: string;
  label: string;
  broker: string;
  broker_account: string | null;
  mode: string;
  is_enabled: boolean;
  max_order_value: string;
  max_positions: number;
  max_daily_loss: string;
  updated_at: string | null;
}

interface AccountDraft {
  label: string;
  broker_account: string;
  is_enabled: boolean;
  max_order_value: string;
  max_positions: string;
  max_daily_loss: string;
}

const GATEWAY_BASE = "http://localhost:8080";

export default function TradingAccountConfig() {
  const [accounts, setAccounts] = useState<TradingAccount[]>([]);
  const [drafts, setDrafts] = useState<Record<string, AccountDraft>>({});
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [saving, setSaving] = useState<string | null>(null);

  const fetchAccounts = async () => {
    try {
      const res = await authFetch(`${GATEWAY_BASE}/api/v1/config/accounts`);
      if (res.ok) {
        const data = await res.json();
        if (Array.isArray(data)) {
          setAccounts(data);
          const newDrafts: Record<string, AccountDraft> = {};
          for (const a of data) {
            newDrafts[a.account_id] = {
              label: a.label,
              broker_account: a.broker_account || "",
              is_enabled: a.is_enabled,
              max_order_value: a.max_order_value,
              max_positions: String(a.max_positions),
              max_daily_loss: a.max_daily_loss,
            };
          }
          setDrafts(newDrafts);
        }
      }
    } catch {
      /* fetch failed */
    }
  };

  useEffect(() => {
    fetchAccounts();
  }, []);

  const updateDraft = (accountId: string, field: keyof AccountDraft, value: string | boolean) => {
    setDrafts((prev) => ({
      ...prev,
      [accountId]: { ...prev[accountId], [field]: value },
    }));
  };

  const handleSave = async (accountId: string) => {
    const draft = drafts[accountId];
    if (!draft) return;

    setSaving(accountId);
    try {
      const body: Record<string, unknown> = {
        label: draft.label,
        broker_account: draft.broker_account || null,
        is_enabled: draft.is_enabled,
        max_order_value: parseFloat(draft.max_order_value) || 2000,
        max_positions: parseInt(draft.max_positions, 10) || 5,
        max_daily_loss: parseFloat(draft.max_daily_loss) || 500,
      };

      const res = await authFetch(`${GATEWAY_BASE}/api/v1/config/accounts/${encodeURIComponent(accountId)}`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      });

      if (res.ok) {
        fetchAccounts();
      }
    } catch {
      /* save failed */
    } finally {
      setSaving(null);
    }
  };

  if (accounts.length === 0) {
    return (
      <div className="space-y-4">
        <h2 className="text-lg font-semibold flex items-center gap-2 text-orange-300">
          <Wallet className="w-4 h-4" />
          Trading Accounts
        </h2>
        <div className="text-slate-500 text-sm p-4 border border-dashed border-slate-700 rounded-lg">
          No trading accounts configured. Run migration 025_trading_accounts.sql to seed accounts.
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <h2 className="text-lg font-semibold flex items-center gap-2 text-orange-300">
        <Wallet className="w-4 h-4" />
        Trading Accounts
      </h2>

      <div className="space-y-4">
        {accounts.map((account) => {
          const draft = drafts[account.account_id];
          if (!draft) return null;
          const isExpanded = expandedId === account.account_id;

          return (
            <div key={account.account_id} className="bg-slate-900/50 rounded-xl border border-white/5 overflow-hidden">
              {/* Header — always visible */}
              <button
                onClick={() => setExpandedId(isExpanded ? null : account.account_id)}
                className="w-full px-5 py-4 flex items-center justify-between hover:bg-white/5 transition-colors"
              >
                <div className="flex items-center gap-3">
                  <span className="font-bold text-white">{account.label}</span>
                  <span className="text-xs text-slate-500">{account.account_id}</span>
                  <span className="text-xs text-slate-500 capitalize">{account.mode.replace("_", " ")}</span>
                  <span
                    className={cn(
                      "px-2 py-0.5 rounded text-xs",
                      draft.is_enabled
                        ? "bg-emerald-500/10 text-emerald-400"
                        : "bg-red-500/10 text-red-400"
                    )}
                  >
                    {draft.is_enabled ? "Active" : "Disabled"}
                  </span>
                </div>
                {isExpanded ? (
                  <ChevronUp className="w-4 h-4 text-slate-400" />
                ) : (
                  <ChevronDown className="w-4 h-4 text-slate-400" />
                )}
              </button>

              {/* Expanded Content */}
              {isExpanded && (
                <div className="px-5 pb-5 space-y-5 border-t border-white/5">
                  {/* Account Info */}
                  <div className="grid grid-cols-2 gap-4 pt-4">
                    <div className="space-y-1">
                      <label className="text-xs text-slate-500 uppercase">Label</label>
                      <input
                        type="text"
                        value={draft.label}
                        onChange={(e) => updateDraft(account.account_id, "label", e.target.value)}
                        className="w-full bg-slate-950 border border-white/10 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-orange-500/50"
                      />
                    </div>
                    <div className="space-y-1">
                      <label className="text-xs text-slate-500 uppercase">Broker Account</label>
                      <input
                        type="text"
                        value={draft.broker_account}
                        onChange={(e) => updateDraft(account.account_id, "broker_account", e.target.value)}
                        className="w-full bg-slate-950 border border-white/10 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-orange-500/50"
                      />
                    </div>
                  </div>

                  {/* Mode (read-only) */}
                  <div className="space-y-1">
                    <label className="text-xs text-slate-500 uppercase">Mode</label>
                    <div className="text-sm text-slate-300 capitalize bg-slate-950/50 border border-white/5 rounded-lg px-3 py-2">
                      {account.mode.replace("_", " ")}
                    </div>
                  </div>

                  {/* Enabled Toggle */}
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-slate-300">Trading Enabled</span>
                    <button
                      onClick={() => updateDraft(account.account_id, "is_enabled", !draft.is_enabled)}
                      className={cn(
                        "relative inline-flex h-6 w-11 items-center rounded-full transition-colors",
                        draft.is_enabled ? "bg-emerald-600" : "bg-slate-700"
                      )}
                    >
                      <span
                        className={cn(
                          "inline-block h-4 w-4 transform rounded-full bg-white transition-transform",
                          draft.is_enabled ? "translate-x-6" : "translate-x-1"
                        )}
                      />
                    </button>
                  </div>

                  {/* Risk Limits */}
                  <div className="space-y-3">
                    <h4 className="text-xs font-semibold text-slate-400 uppercase tracking-wider flex items-center gap-1.5">
                      <Shield className="w-3.5 h-3.5" />
                      Risk Limits
                    </h4>
                    <div className="grid grid-cols-3 gap-4">
                      <div className="space-y-1">
                        <label className="text-xs text-slate-500">Max Order Value ($)</label>
                        <input
                          type="number"
                          value={draft.max_order_value}
                          onChange={(e) => updateDraft(account.account_id, "max_order_value", e.target.value)}
                          className="w-full bg-slate-950 border border-white/10 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-orange-500/50"
                        />
                      </div>
                      <div className="space-y-1">
                        <label className="text-xs text-slate-500">Max Positions</label>
                        <input
                          type="number"
                          value={draft.max_positions}
                          onChange={(e) => updateDraft(account.account_id, "max_positions", e.target.value)}
                          className="w-full bg-slate-950 border border-white/10 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-orange-500/50"
                        />
                      </div>
                      <div className="space-y-1">
                        <label className="text-xs text-slate-500">Max Daily Loss ($)</label>
                        <input
                          type="number"
                          value={draft.max_daily_loss}
                          onChange={(e) => updateDraft(account.account_id, "max_daily_loss", e.target.value)}
                          className="w-full bg-slate-950 border border-white/10 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-orange-500/50"
                        />
                      </div>
                    </div>
                  </div>

                  {/* Save */}
                  <div className="flex items-center justify-between pt-2">
                    {account.updated_at && (
                      <span className="text-xs text-slate-600">
                        Last updated: {new Date(account.updated_at).toLocaleString()}
                      </span>
                    )}
                    <button
                      onClick={() => handleSave(account.account_id)}
                      disabled={saving === account.account_id}
                      className="ml-auto px-4 py-2 bg-orange-600 hover:bg-orange-500 rounded-lg text-white text-sm font-medium flex items-center gap-2 transition-colors disabled:opacity-50"
                    >
                      <Save className="w-4 h-4" />
                      {saving === account.account_id ? "Saving..." : "Save"}
                    </button>
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
