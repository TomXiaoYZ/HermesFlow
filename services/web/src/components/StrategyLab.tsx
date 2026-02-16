"use client";

import React, { useState, useEffect, useMemo } from 'react';
import { loadFactorConfig } from '@/utils/genome';
import { FACTOR_DATA, SENTIMENT_FEED, FACTOR_LIBRARY, SENTIMENT_MOVERS, HOT_TOPICS } from '../constants';
import { BarChart, Bar, Cell, XAxis, YAxis, CartesianGrid, Tooltip as RechartsTooltip, ResponsiveContainer, LineChart, Line, AreaChart, Area, Legend, ComposedChart, ReferenceLine, ScatterChart, Scatter } from 'recharts';
import EvolutionExplorer from '@/components/EvolutionExplorer';

// --- Quick Trade Modal Component ---
const QuickTradeModal = ({ item, onClose }: { item: typeof SENTIMENT_FEED[0], onClose: () => void }) => {
   const isBullish = item.score > 0;
   const [side, setSide] = useState<'buy' | 'sell'>(isBullish ? 'buy' : 'sell');
   const [price, setPrice] = useState(item.entity === 'BTC' ? '64235.50' : item.entity === 'ETH' ? '3450.20' : '185.40');
   const [amount, setAmount] = useState('1.0');

   return (
      <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/60 backdrop-blur-[2px] animate-in fade-in duration-200">
         <div className="bg-surface-900 border border-surface-700 w-[420px] rounded-lg shadow-2xl flex flex-col overflow-hidden animate-in zoom-in-95 duration-200">
            {/* Header */}
            <div className="px-5 py-4 border-b border-surface-800 flex justify-between items-center bg-surface-950">
               <div>
                  <h3 className="text-lg font-bold text-surface-100 flex items-center gap-2">
                     {item.entity}/USDT <span className="text-xs bg-surface-800 border border-surface-700 px-1.5 py-0.5 rounded text-surface-400 font-normal">永续合约</span>
                  </h3>
               </div>
               <button onClick={onClose} className="text-surface-500 hover:text-surface-300 transition-colors"><span className="material-symbols-outlined">close</span></button>
            </div>

            {/* News Context */}
            <div className="px-5 py-3 bg-surface-950/50 border-b border-surface-800">
               <div className="flex items-center gap-2 mb-1">
                  <span className="text-[10px] font-bold uppercase tracking-wide text-surface-500">信号来源 (Signal)</span>
                  <span className={`text-[10px] px-1.5 rounded font-bold border ${item.score > 0 ? 'bg-trade-up/10 text-trade-up border-trade-up/20' : 'bg-trade-down/10 text-trade-down border-trade-down/20'}`}>
                     {item.source} • 情绪 {item.score > 0 ? '+' : ''}{item.score}
                  </span>
               </div>
               <p className="text-xs text-surface-300 leading-relaxed line-clamp-2 border-l-2 border-surface-700 pl-2 italic">
                  "{item.content}"
               </p>
            </div>

            {/* Trading Form */}
            <div className="p-5 flex flex-col gap-4">
               {/* Side Toggle */}
               <div className="grid grid-cols-2 gap-2 bg-surface-950 p-1 rounded-md border border-surface-800">
                  <button onClick={() => setSide('buy')} className={`py-2 rounded font-bold text-sm transition-all ${side === 'buy' ? 'bg-trade-up text-surface-950 shadow-sm' : 'text-surface-400 hover:text-surface-200'}`}>买入 / 做多</button>
                  <button onClick={() => setSide('sell')} className={`py-2 rounded font-bold text-sm transition-all ${side === 'sell' ? 'bg-trade-down text-white shadow-sm' : 'text-surface-400 hover:text-surface-200'}`}>卖出 / 做空</button>
               </div>

               {/* Inputs */}
               <div className="space-y-3">
                  <div>
                     <label className="text-xs font-bold text-surface-500 uppercase tracking-wide mb-1.5 block">价格 (USDT)</label>
                     <div className="relative">
                        <input type="text" value={price} onChange={e => setPrice(e.target.value)} className="w-full bg-surface-950 border border-surface-700 rounded px-3 py-2 text-sm text-surface-200 font-mono focus:border-hermes-500 outline-none transition-colors" />
                        <span className="absolute right-3 top-2 text-xs text-surface-500">最新</span>
                     </div>
                  </div>
                  <div>
                     <label className="text-xs font-bold text-surface-500 uppercase tracking-wide mb-1.5 block">数量 ({item.entity})</label>
                     <div className="relative">
                        <input type="text" value={amount} onChange={e => setAmount(e.target.value)} className="w-full bg-surface-950 border border-surface-700 rounded px-3 py-2 text-sm text-surface-200 font-mono focus:border-hermes-500 outline-none transition-colors" />
                        <div className="absolute right-2 top-1.5 flex gap-1">
                           <button className="px-1.5 py-0.5 bg-surface-800 hover:bg-surface-700 rounded text-[10px] text-surface-400 border border-surface-700 transition-colors">25%</button>
                           <button className="px-1.5 py-0.5 bg-surface-800 hover:bg-surface-700 rounded text-[10px] text-surface-400 border border-surface-700 transition-colors">50%</button>
                        </div>
                     </div>
                  </div>
               </div>

               {/* Summary */}
               <div className="flex justify-between items-center text-xs text-surface-400 mt-2 px-1">
                  <span>预估价值: <span className="text-surface-200 font-mono">{(parseFloat(price) * parseFloat(amount)).toLocaleString()} USDT</span></span>
                  <span>杠杆: <span className="text-hermes-500 font-bold">5x</span></span>
               </div>

               {/* Action */}
               <button onClick={onClose} className={`w-full py-3 rounded-md font-bold text-sm uppercase tracking-wide shadow-lg transition-all active:scale-[0.98] flex items-center justify-center gap-2 ${side === 'buy' ? 'bg-trade-up hover:bg-trade-up/90 text-surface-950 shadow-trade-up/20' : 'bg-trade-down hover:bg-trade-down/90 text-white shadow-trade-down/20'}`}>
                  <span className="material-symbols-outlined text-[18px]">bolt</span>
                  {side === 'buy' ? '确认买入 (Long)' : '确认卖出 (Short)'}
               </button>
            </div>
         </div>
      </div>
   );
}

// --- Alpha Lens Component (Institutional Factor Analysis) ---
const AlphaLens = () => {
   // Mock Quantile Data (Layered Returns)
   const QUANTILE_DATA = [
      { bucket: 'Top 10%', return: 18.5, color: '#00E396' },
      { bucket: 'Q2', return: 8.2, color: '#00E396' },
      { bucket: 'Q3', return: 2.1, color: '#71717a' },
      { bucket: 'Q4', return: -1.5, color: '#71717a' },
      { bucket: 'Bottom 10%', return: -12.4, color: '#FF4560' },
   ];

   // Mock IC Series Data
   const IC_DATA = Array.from({ length: 30 }, (_, i) => ({
      date: i,
      ic: (Math.random() * 0.15 - 0.02).toFixed(3),
      ma: 0.05
   }));

   return (
      <div className="flex flex-col h-full bg-[#0c0c0e]">
         {/* Top Stats Bar */}
         <div className="grid grid-cols-4 gap-4 p-4 border-b border-surface-800 bg-surface-950">
            <div>
               <div className="text-[10px] font-bold text-surface-500 uppercase">Information Coeff (IC)</div>
               <div className="text-xl font-mono font-bold text-surface-100 mt-1">0.058 <span className="text-[10px] text-trade-up">(Strong)</span></div>
            </div>
            <div>
               <div className="text-[10px] font-bold text-surface-500 uppercase">Information Ratio (IR)</div>
               <div className="text-xl font-mono font-bold text-surface-100 mt-1">2.14</div>
            </div>
            <div>
               <div className="text-[10px] font-bold text-surface-500 uppercase">Turnover Rate</div>
               <div className="text-xl font-mono font-bold text-surface-100 mt-1">45% <span className="text-[10px] text-surface-400">/ Day</span></div>
            </div>
            <div>
               <div className="text-[10px] font-bold text-surface-500 uppercase">Long-Short Spread</div>
               <div className="text-xl font-mono font-bold text-hermes-500 mt-1">+30.9%</div>
            </div>
         </div>

         {/* Charts Layout */}
         <div className="flex-1 p-4 grid grid-cols-2 gap-4 overflow-y-auto">

            {/* 1. Quantile Returns */}
            <div className="bg-surface-900 border border-surface-800 rounded-md p-4 flex flex-col h-64">
               <h4 className="text-xs font-bold text-surface-300 uppercase mb-4 flex items-center gap-2">
                  <span className="material-symbols-outlined text-[16px]">bar_chart</span>
                  分层收益分析 (Quantile Returns)
               </h4>
               <div className="flex-1 min-h-0">
                  <ResponsiveContainer width="100%" height="100%">
                     <BarChart data={QUANTILE_DATA} layout="vertical">
                        <CartesianGrid strokeDasharray="3 3" stroke="#27272a" horizontal={false} />
                        <XAxis type="number" tick={{ fontSize: 10, fill: '#71717a' }} axisLine={false} tickLine={false} />
                        <YAxis type="category" dataKey="bucket" tick={{ fontSize: 11, fill: '#a1a1aa' }} width={70} axisLine={false} tickLine={false} />
                        <RechartsTooltip cursor={{ fill: '#27272a' }} contentStyle={{ backgroundColor: '#18181b', borderColor: '#27272a', fontSize: '12px' }} />
                        <Bar dataKey="return" barSize={18} radius={[0, 4, 4, 0]}>
                           {QUANTILE_DATA.map((entry, index) => (
                              <Cell key={`cell-${index}`} fill={entry.return > 0 ? '#00E396' : '#FF4560'} />
                           ))}
                        </Bar>
                     </BarChart>
                  </ResponsiveContainer>
               </div>
               <div className="mt-2 text-[10px] text-surface-500 text-center">
                  *Top 10% 组对比 Bottom 10% 组表现出显著的单调性 (Monotonicity)，因子有效。
               </div>
            </div>

            {/* 2. IC Time Series */}
            <div className="bg-surface-900 border border-surface-800 rounded-md p-4 flex flex-col h-64">
               <h4 className="text-xs font-bold text-surface-300 uppercase mb-4 flex items-center gap-2">
                  <span className="material-symbols-outlined text-[16px]">timeline</span>
                  IC 序列 (IC Time Series)
               </h4>
               <div className="flex-1 min-h-0">
                  <ResponsiveContainer width="100%" height="100%">
                     <BarChart data={IC_DATA}>
                        <CartesianGrid strokeDasharray="3 3" stroke="#27272a" vertical={false} />
                        <XAxis dataKey="date" hide />
                        <YAxis tick={{ fontSize: 10, fill: '#71717a' }} axisLine={false} tickLine={false} />
                        <ReferenceLine y={0} stroke="#52525b" />
                        <RechartsTooltip contentStyle={{ backgroundColor: '#18181b', borderColor: '#27272a', fontSize: '12px' }} />
                        <Bar dataKey="ic" fill="#8884d8" name="Daily IC">
                           {IC_DATA.map((entry, index) => (
                              <Cell key={`cell-${index}`} fill={parseFloat(entry.ic) > 0 ? '#3b82f6' : '#FF4560'} fillOpacity={0.6} />
                           ))}
                        </Bar>
                     </BarChart>
                  </ResponsiveContainer>
               </div>
            </div>

            {/* 3. Cumulative Long/Short Return */}
            <div className="col-span-2 bg-surface-900 border border-surface-800 rounded-md p-4 flex flex-col h-64">
               <h4 className="text-xs font-bold text-surface-300 uppercase mb-4 flex items-center gap-2">
                  <span className="material-symbols-outlined text-[16px]">show_chart</span>
                  多空对冲净值 (Long-Short Equity)
               </h4>
               <div className="flex-1 min-h-0">
                  <ResponsiveContainer width="100%" height="100%">
                     <AreaChart data={Array.from({ length: 50 }, (_, i) => ({
                        step: i,
                        val: 1000 + (i * 15) + (Math.random() * 50 - 25)
                     }))}>
                        <defs>
                           <linearGradient id="colorLSE" x1="0" y1="0" x2="0" y2="1">
                              <stop offset="5%" stopColor="#8b5cf6" stopOpacity={0.2} />
                              <stop offset="95%" stopColor="#8b5cf6" stopOpacity={0} />
                           </linearGradient>
                        </defs>
                        <CartesianGrid strokeDasharray="3 3" stroke="#27272a" vertical={false} />
                        <XAxis dataKey="step" hide />
                        <YAxis tick={{ fontSize: 10, fill: '#71717a' }} axisLine={false} tickLine={false} domain={['auto', 'auto']} />
                        <RechartsTooltip contentStyle={{ backgroundColor: '#18181b', borderColor: '#27272a', fontSize: '12px' }} />
                        <Area type="monotone" dataKey="val" stroke="#8b5cf6" strokeWidth={2} fill="url(#colorLSE)" />
                     </AreaChart>
                  </ResponsiveContainer>
               </div>
            </div>

         </div>
      </div>
   );
};

// --- Visual Node Editor for Factor Research (Original + Updated) ---
// --- STITCH REDESIGNED: Factor Research (Node Editor) ---
const FactorResearch = () => {
   // Mock Data for Library
   const library = [
      { category: 'FILTER_ALT 资产池 (UNIVERSE)', items: ['TOP 100 Market Cap', 'Vol > 2.5%', 'Liquid > 10M'] },
      { category: 'SHOW_CHART 技术指标 (TECHNICAL)', items: ['RSI 相对强弱', 'MACD 指数平滑', 'KDJ 随机指标', 'Bollinger 布林带', 'ATR 平均真实波幅'] }
   ];

   return (
      <div className="h-full flex bg-[#030305]">
         {/* 1. Left Sidebar: Component Library */}
         <div className="w-64 bg-slate-950/80 backdrop-blur-md border-r border-white/5 flex flex-col z-20">
            {/* Search */}
            <div className="p-4 border-b border-white/5">
               <div className="relative">
                  <span className="material-symbols-outlined absolute left-3 top-2.5 text-slate-500 text-[18px]">search</span>
                  <div className="absolute top-2.5 right-3 text-[10px] text-slate-600 border border-slate-700 rounded px-1.5 py-0.5">⌘K</div>
                  <input
                     type="text"
                     placeholder="Search nodes..."
                     className="w-full bg-slate-900/50 border border-slate-700/50 rounded-lg pl-10 pr-10 py-2 text-sm text-slate-300 focus:outline-none focus:border-cyan-500/50 focus:ring-1 focus:ring-cyan-500/20 transition-all placeholder:text-slate-600"
                  />
               </div>
            </div>

            <div className="flex-1 overflow-y-auto custom-scrollbar p-2">
               {library.map((g, i) => (
                  <div key={i} className="mb-6 px-2">
                     <h3 className="text-[10px] font-bold text-slate-500 uppercase tracking-widest mb-3 pl-2 flex items-center gap-2">
                        {g.category}
                     </h3>
                     <div className="space-y-1">
                        {g.items.map((item, j) => (
                           <div key={j} className="text-xs text-slate-400 hover:text-cyan-400 hover:bg-cyan-500/5 px-3 py-2 rounded-md transition-all cursor-move flex items-center justify-between group border border-transparent hover:border-cyan-500/10">
                              <div className="flex items-center gap-2">
                                 <div className={`w-1.5 h-1.5 rounded-full ${i === 0 ? 'bg-amber-500' : 'bg-cyan-500'} group-hover:shadow-[0_0_8px_currentColor]`}></div>
                                 {item}
                              </div>
                              <span className="opacity-0 group-hover:opacity-100 material-symbols-outlined text-[14px]">drag_indicator</span>
                           </div>
                        ))}
                     </div>
                  </div>
               ))}
            </div>
         </div>

         {/* 2. Main Canvas (Node Graph) */}
         <div className="flex-1 relative overflow-hidden bg-[#050507]">
            {/* Grid Background */}
            <div className="absolute inset-0 opacity-20"
               style={{
                  backgroundImage: 'radial-gradient(circle at 1px 1px, #334155 1px, transparent 0)',
                  backgroundSize: '24px 24px'
               }}>
            </div>

            {/* Canvas Controls */}
            <div className="absolute top-4 right-4 flex gap-2 z-10">
               <button className="p-2 bg-slate-800/80 backdrop-blur hover:bg-slate-700 rounded-lg text-slate-400 border border-white/5 transition-colors"><span className="material-symbols-outlined text-[18px]">undo</span></button>
               <button className="p-2 bg-slate-800/80 backdrop-blur hover:bg-slate-700 rounded-lg text-slate-400 border border-white/5 transition-colors"><span className="material-symbols-outlined text-[18px]">redo</span></button>
               <div className="w-px h-8 bg-slate-700/50 mx-1"></div>
               <button className="p-2 bg-slate-800/80 backdrop-blur hover:bg-slate-700 rounded-lg text-slate-400 border border-white/5 transition-colors"><span className="material-symbols-outlined text-[18px]">fit_screen</span></button>
            </div>

            {/* Nodes Container - Mocking a layout */}
            <div className="w-full h-full relative">
               {/* Connection Lines (SVG) */}
               <svg className="absolute inset-0 w-full h-full pointer-events-none z-0">
                  <path d="M 400 300 C 480 300, 480 400, 560 400" stroke="#475569" strokeWidth="2" fill="none" className="opacity-50" />
                  <path d="M 400 500 C 480 500, 480 420, 560 420" stroke="#475569" strokeWidth="2" fill="none" className="opacity-50" />
                  <path d="M 760 410 C 840 410, 840 410, 920 410" stroke="#475569" strokeWidth="2" fill="none" className="opacity-50" />
               </svg>

               {/* Node 1: RSI */}
               <div className="absolute top-[250px] left-[200px] w-52 bg-slate-900/90 backdrop-blur-md rounded-xl border border-slate-700/50 shadow-2xl z-10 group hover:border-cyan-500/50 transition-all">
                  <div className="h-1 w-full bg-cyan-500 rounded-t-xl shadow-[0_0_10px_#06b6d4]"></div>
                  <div className="p-3">
                     <div className="flex justify-between items-start mb-3">
                        <div className="flex gap-2 items-center">
                           <span className="material-symbols-outlined text-cyan-400 text-[18px]">show_chart</span>
                           <span className="text-xs font-bold text-slate-200">RSI (14) • Reversal</span>
                        </div>
                        <span className="material-symbols-outlined text-slate-600 text-[16px]">more_horiz</span>
                     </div>
                     <div className="space-y-2">
                        <div className="flex justify-between items-center text-[10px] text-slate-400 bg-slate-950/50 p-1.5 rounded">
                           <span>Current Value</span>
                           <span className="font-mono text-cyan-400">32.8</span>
                        </div>
                        <div className="flex justify-between items-center bg-slate-800/50 rounded p-2 border border-slate-700/50">
                           <span className="text-[10px] font-bold text-slate-300">Z-SCORE</span>
                           <span className="text-lg font-mono font-bold text-white">+1.72</span>
                        </div>
                     </div>
                  </div>
                  {/* Port */}
                  <div className="absolute -right-1.5 top-1/2 w-3 h-3 bg-cyan-500 rounded-full border-2 border-slate-900 shadow-[0_0_8px_currentColor]"></div>
               </div>

               {/* Node 2: AI Sentiment */}
               <div className="absolute top-[450px] left-[200px] w-52 bg-slate-900/90 backdrop-blur-md rounded-xl border border-slate-700/50 shadow-2xl z-10 group hover:border-purple-500/50 transition-all">
                  <div className="h-1 w-full bg-purple-500 rounded-t-xl shadow-[0_0_10px_#a855f7]"></div>
                  <div className="p-3">
                     <div className="flex justify-between items-start mb-3">
                        <div className="flex gap-2 items-center">
                           <span className="material-symbols-outlined text-purple-400 text-[18px]">psychology</span>
                           <span className="text-xs font-bold text-slate-200">AI Sentiment</span>
                        </div>
                        <span className="material-symbols-outlined text-slate-600 text-[16px]">more_horiz</span>
                     </div>
                     <div className="space-y-2">
                        <div className="flex justify-between items-center text-[10px] text-slate-400 bg-slate-950/50 p-1.5 rounded">
                           <span>Score (0-100)</span>
                           <span className="font-mono text-purple-400">75.1</span>
                        </div>
                        <div className="flex justify-between items-center bg-slate-800/50 rounded p-2 border border-slate-700/50">
                           <span className="text-[10px] font-bold text-slate-300">Z-SCORE</span>
                           <span className="text-lg font-mono font-bold text-white">+1.67</span>
                        </div>
                     </div>
                  </div>
                  {/* Port */}
                  <div className="absolute -right-1.5 top-1/2 w-3 h-3 bg-purple-500 rounded-full border-2 border-slate-900 shadow-[0_0_8px_currentColor]"></div>
               </div>

               {/* Node 3: Multi-Factor Model */}
               <div className="absolute top-[350px] left-[560px] w-64 bg-slate-900/90 backdrop-blur-md rounded-xl border border-slate-600/60 shadow-2xl z-20 ring-1 ring-slate-700/50">
                  <div className="px-3 py-2 border-b border-white/5 bg-slate-800/50 flex justify-between items-center rounded-t-xl">
                     <div className="flex items-center gap-2">
                        <span className="text-[10px] bg-indigo-500/20 text-indigo-400 px-1.5 py-0.5 rounded border border-indigo-500/30">hub</span>
                        <span className="text-xs font-bold text-white">Multi-Factor Model</span>
                     </div>
                     <div className="flex gap-1">
                        <div className="w-2 h-2 rounded-full bg-red-500/20"></div>
                        <div className="w-2 h-2 rounded-full bg-yellow-500/20"></div>
                        <div className="w-2 h-2 rounded-full bg-green-500"></div>
                     </div>
                  </div>
                  <div className="p-3 space-y-3">
                     {/* Input Slots */}
                     <div className="space-y-2">
                        <div className="flex justify-between items-center relative">
                           <span className="text-[10px] text-slate-400">Tech (RSI)</span>
                           <div className="w-24 h-1.5 bg-slate-950 rounded-full overflow-hidden">
                              <div className="h-full bg-cyan-500 w-[48%] shadow-[0_0_5px_cyan]"></div>
                           </div>
                           <span className="text-[10px] font-mono text-slate-500">48%</span>
                           <div className="absolute -left-4 w-2 h-2 bg-slate-600 rounded-full hover:bg-cyan-500 transition-colors cursor-pointer border border-slate-900"></div>
                        </div>
                        <div className="flex justify-between items-center relative">
                           <span className="text-[10px] text-slate-400">Sentiment</span>
                           <div className="w-24 h-1.5 bg-slate-950 rounded-full overflow-hidden">
                              <div className="h-full bg-purple-500 w-[68%] shadow-[0_0_5px_purple]"></div>
                           </div>
                           <span className="text-[10px] font-mono text-slate-500">68%</span>
                           <div className="absolute -left-4 w-2 h-2 bg-slate-600 rounded-full hover:bg-purple-500 transition-colors cursor-pointer border border-slate-900"></div>
                        </div>
                     </div>
                  </div>
                  {/* Output Port */}
                  <div className="absolute -right-1.5 top-1/2 w-3 h-3 bg-indigo-500 rounded-full border-2 border-slate-900 shadow-[0_0_8px_currentColor]"></div>
               </div>

               {/* Node 4: Final Signal */}
               <div className="absolute top-[375px] left-[920px] w-40 bg-black/40 backdrop-blur-xl rounded-lg border-2 border-white/90 shadow-[0_0_30px_rgba(255,255,255,0.1)] z-10 flex flex-col items-center justify-center p-4">
                  <span className="text-[9px] font-bold text-slate-400 mb-2 tracking-widest uppercase">Final Signal</span>
                  <div className="text-3xl font-mono font-bold text-transparent bg-clip-text bg-gradient-to-r from-white to-slate-400 tracking-tighter">
                     +1.63
                  </div>
                  <span className="text-[9px] text-slate-600 mt-1">COMPOSITE Z-SCORE</span>
                  <div className="absolute -left-1.5 top-1/2 w-3 h-3 bg-white rounded-full border-2 border-black shadow-[0_0_10px_white]"></div>
               </div>

            </div>
         </div>

         {/* 3. Right Sidebar: Backtest Config */}
         <div className="w-72 bg-slate-950/80 backdrop-blur-md border-l border-white/5 flex flex-col flex-shrink-0 z-20">
            <div className="p-4 border-b border-white/5 flex justify-between items-center">
               <h3 className="text-xs font-bold text-slate-100 uppercase tracking-widest">回测配置 (Backtest)</h3>
               <span className="material-symbols-outlined text-slate-500 hover:text-white cursor-pointer transition-colors text-[18px]">settings</span>
            </div>

            <div className="flex-1 overflow-y-auto custom-scrollbar p-5 space-y-8">
               {/* Time Range */}
               <div className="space-y-3">
                  <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block">回测区间 (Time Range)</label>
                  <div className="grid grid-cols-2 gap-3">
                     <div className="bg-slate-900 border border-slate-800 hover:border-slate-600 rounded p-2 transition-colors group">
                        <div className="text-[9px] text-slate-500 group-hover:text-slate-400">开始时间</div>
                        <div className="text-xs font-mono text-slate-200 mt-0.5">2023-01-01</div>
                     </div>
                     <div className="bg-slate-900 border border-slate-800 hover:border-slate-600 rounded p-2 transition-colors group">
                        <div className="text-[9px] text-slate-500 group-hover:text-slate-400">结束时间</div>
                        <div className="text-xs font-mono text-slate-200 mt-0.5">2023-12-31</div>
                     </div>
                  </div>
               </div>

               {/* Benchmark */}
               <div className="space-y-3">
                  <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block">基准指数 (Benchmark)</label>
                  <div className="relative">
                     <select className="w-full appearance-none bg-slate-900 border border-slate-800 rounded px-3 py-2.5 text-xs text-slate-200 outline-none focus:border-indigo-500 transition-colors">
                        <option>HS300 (沪深300)</option>
                        <option>ZZ500 (中证500)</option>
                        <option>BTC (比特币)</option>
                     </select>
                     <span className="material-symbols-outlined absolute right-2.5 top-2.5 text-slate-500 pointer-events-none text-[16px]">expand_more</span>
                  </div>
               </div>

               {/* Capital */}
               <div className="space-y-3">
                  <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block">初始资金 (Capital)</label>
                  <div className="relative group">
                     <span className="absolute left-3 top-2.5 text-slate-500 group-focus-within:text-indigo-400 transition-colors font-mono">¥</span>
                     <input type="text" defaultValue="1,000,000" className="w-full bg-slate-900 border border-slate-800 rounded px-3 py-2 pl-7 text-sm font-mono text-slate-200 focus:border-indigo-500 outline-none transition-colors" />
                  </div>
               </div>

               <div className="pt-4 space-y-4 border-t border-white/5">
                  <div className="flex justify-between items-center group cursor-pointer">
                     <span className="text-xs text-slate-400 group-hover:text-slate-200 transition-colors">包含交易费率</span>
                     <div className="w-8 h-4 bg-slate-800 rounded-full relative transition-colors group-hover:bg-slate-700">
                        <div className="w-2.5 h-2.5 bg-slate-400 rounded-full absolute right-1 top-0.5"></div>
                     </div>
                  </div>
                  <div className="flex justify-between items-center group cursor-pointer">
                     <span className="text-xs text-slate-400 group-hover:text-slate-200 transition-colors">滑点控制 (Slippage)</span>
                     <div className="w-8 h-4 bg-slate-800 rounded-full relative transition-colors group-hover:bg-slate-700">
                        <div className="w-2.5 h-2.5 bg-slate-400 rounded-full absolute left-1 top-0.5"></div>
                     </div>
                  </div>
               </div>
            </div>

            <div className="p-5 border-t border-white/5 bg-slate-950/50 backdrop-blur-xl">
               <button className="relative w-full py-3.5 bg-gradient-to-r from-emerald-500 to-teal-500 hover:from-emerald-400 hover:to-teal-400 text-white font-bold rounded-lg shadow-[0_0_20px_rgba(16,185,129,0.3)] transition-all transform hover:scale-[1.02] flex items-center justify-between px-4 group overflow-hidden">
                  <div className="absolute inset-0 bg-white/20 translate-y-full group-hover:translate-y-0 transition-transform duration-300"></div>
                  <span className="material-symbols-outlined relative z-10">play_circle</span>
                  <span className="relative z-10 text-xs uppercase tracking-widest">开始回测 (Run Backtest)</span>
               </button>
               <div className="text-center mt-3 text-[9px] text-slate-500 font-mono">预计耗时: ~45s • 消耗算力点: 12</div>
            </div>
         </div>
      </div>
   );
};

// --- REDESIGNED: Parameter Optimization Module (Heatmap Grid) ---
const Optimization = () => {
   const [running, setRunning] = useState(false);
   const [progress, setProgress] = useState(0);
   const [selectedCell, setSelectedCell] = useState<{ x: number, y: number, val: number } | null>(null);

   // Mock Heatmap Data (Sharpe Ratios)
   const xAxisLabels = [5, 10, 15, 20, 25, 30];
   const yAxisLabels = [30, 40, 50, 60, 70, 80, 90, 100];

   // Generate grid data with a "peak"
   const gridData = yAxisLabels.map((y, yIdx) => {
      return xAxisLabels.map((x, xIdx) => {
         const dist = Math.sqrt(Math.pow(xIdx - 2, 2) + Math.pow(yIdx - 3, 2));
         let val = 2.8 - (dist * 0.4) + (Math.random() * 0.3);
         return parseFloat(val.toFixed(2));
      });
   });

   const handleRun = () => {
      setRunning(true);
      setProgress(0);
      const interval = window.setInterval(() => {
         setProgress(prev => {
            if (prev >= 100) {
               clearInterval(interval);
               setRunning(false);
               return 100;
            }
            return prev + 5;
         });
      }, 100);
   };

   // Helper for heat color with neon effect
   const getColor = (val: number) => {
      if (val >= 2.5) return 'bg-emerald-500 shadow-[0_0_10px_#10b981] z-10'; // High Green
      if (val >= 2.0) return 'bg-emerald-500/70';
      if (val >= 1.5) return 'bg-amber-500/60'; // Med Yellow
      if (val >= 1.0) return 'bg-rose-500/50'; // Low Red
      return 'bg-slate-800/50';
   };

   return (
      <div className="h-full flex bg-[#030305]">
         {/* 1. Left Config Panel */}
         <div className="w-80 bg-slate-950/80 backdrop-blur-md border-r border-white/5 flex flex-col p-6 overflow-y-auto z-20">
            <h2 className="text-lg font-bold text-white mb-1 tracking-tight">参数寻优 (Optimization)</h2>
            <p className="text-[10px] text-slate-500 mb-8 uppercase tracking-widest font-bold">Grid Search & Sensitivity Analysis</p>

            <div className="space-y-8">
               {/* Strategy Select */}
               <div className="space-y-3">
                  <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block">Target Strategy</label>
                  <div className="relative">
                     <select className="w-full appearance-none bg-slate-900/50 border border-slate-700/50 rounded px-3 py-2.5 text-xs text-slate-200 outline-none focus:border-cyan-500 transition-colors">
                        <option>Alpha-Trend-v1 (BTC/USDT)</option>
                        <option>Mean-Reversion-ETH</option>
                     </select>
                     <span className="material-symbols-outlined absolute right-2.5 top-2.5 text-slate-500 pointer-events-none text-[16px]">expand_more</span>
                  </div>
               </div>

               {/* Parameter 1 Config */}
               <div className="bg-slate-900/40 border border-white/5 rounded-lg p-4 space-y-3">
                  <div className="flex justify-between items-center border-b border-white/5 pb-2">
                     <span className="text-xs font-bold text-cyan-400 flex items-center gap-1.5">
                        <span className="w-1.5 h-1.5 rounded-full bg-cyan-400"></span> PARAM X
                     </span>
                     <span className="text-[10px] font-mono text-slate-500">Fast MA</span>
                  </div>
                  <div className="grid grid-cols-3 gap-2">
                     {['Start', 'End', 'Step'].map((label, i) => (
                        <div key={label}>
                           <label className="text-[9px] text-slate-600 uppercase block mb-1 text-center">{label}</label>
                           <input type="number" defaultValue={[5, 30, 5][i]} className="w-full bg-black/20 border border-slate-800 rounded p-1.5 text-xs font-mono text-center text-slate-300 focus:border-cyan-500/50 outline-none transition-colors" />
                        </div>
                     ))}
                  </div>
               </div>

               {/* Parameter 2 Config */}
               <div className="bg-slate-900/40 border border-white/5 rounded-lg p-4 space-y-3">
                  <div className="flex justify-between items-center border-b border-white/5 pb-2">
                     <span className="text-xs font-bold text-purple-400 flex items-center gap-1.5">
                        <span className="w-1.5 h-1.5 rounded-full bg-purple-400"></span> PARAM Y
                     </span>
                     <span className="text-[10px] font-mono text-slate-500">Slow MA</span>
                  </div>
                  <div className="grid grid-cols-3 gap-2">
                     {['Start', 'End', 'Step'].map((label, i) => (
                        <div key={label}>
                           <label className="text-[9px] text-slate-600 uppercase block mb-1 text-center">{label}</label>
                           <input type="number" defaultValue={[30, 100, 10][i]} className="w-full bg-black/20 border border-slate-800 rounded p-1.5 text-xs font-mono text-center text-slate-300 focus:border-purple-500/50 outline-none transition-colors" />
                        </div>
                     ))}
                  </div>
               </div>

               {/* Objective */}
               <div className="space-y-3">
                  <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wider block">Optimization Target</label>
                  <div className="grid grid-cols-2 gap-2">
                     <button className="py-2.5 bg-cyan-500/10 border border-cyan-500/30 text-cyan-400 rounded text-[10px] font-bold uppercase tracking-wider hover:bg-cyan-500/20 transition-all">Max Sharpe</button>
                     <button className="py-2.5 bg-slate-800/50 border border-slate-700/50 text-slate-400 rounded text-[10px] font-bold uppercase tracking-wider hover:text-slate-200 transition-all">Max Return</button>
                  </div>
               </div>
            </div>

            <div className="mt-auto pt-6">
               <button
                  onClick={handleRun}
                  disabled={running}
                  className={`w-full py-3.5 rounded-lg font-bold text-xs uppercase tracking-widest shadow-lg transition-all flex items-center justify-center gap-2 relative overflow-hidden group ${running ? 'bg-slate-800 text-slate-500 cursor-not-allowed' : 'bg-white text-black hover:bg-slate-200'}`}
               >
                  {running ? (
                     <>
                        <span className="w-3 h-3 border-2 border-slate-500 border-t-transparent rounded-full animate-spin"></span>
                        Processing {progress}%
                     </>
                  ) : (
                     <>
                        <span className="material-symbols-outlined text-[16px]">play_arrow</span>
                        Start Computation
                     </>
                  )}
                  {!running && <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/50 to-transparent -translate-x-full group-hover:translate-x-full transition-transform duration-500"></div>}
               </button>
            </div>
         </div>

         {/* 2. Main Visualization Area */}
         <div className="flex-1 flex flex-col p-6 overflow-hidden bg-[#050507] relative">
            {/* Background Grid */}
            <div className="absolute inset-0 bg-[url('https://grainy-gradients.vercel.app/noise.svg')] opacity-20 pointer-events-none"></div>

            {/* Toolbar */}
            <div className="flex justify-between items-center mb-8 relative z-10">
               <h3 className="text-sm font-bold text-white flex items-center gap-2 uppercase tracking-wider">
                  <span className="material-symbols-outlined text-cyan-500 text-[20px]">grid_on</span>
                  Sharpe Ratio Heatmap
               </h3>
               <div className="flex items-center gap-3 bg-slate-900/50 border border-white/5 rounded-full px-4 py-1.5 backdrop-blur-sm">
                  <span className="text-[10px] font-bold text-slate-500 uppercase">Index</span>
                  <div className="w-24 h-1.5 bg-gradient-to-r from-rose-500/50 via-amber-500/60 to-emerald-500 rounded-full"></div>
               </div>
            </div>

            {/* Heatmap Grid */}
            <div className="flex-1 bg-slate-900/20 border border-white/5 rounded-2xl p-8 relative flex flex-col items-center justify-center backdrop-blur-sm">

               <div className="relative">
                  {/* Y-Axis Label */}
                  <div className="absolute -left-12 top-1/2 -translate-y-1/2 -rotate-90 text-[10px] font-bold text-purple-400 tracking-[0.2em] origin-center whitespace-nowrap">
                     SLOW MA PERIOD
                  </div>

                  <div className="flex">
                     {/* Y-Axis Ticks */}
                     <div className="flex flex-col justify-between pr-4 py-3 text-[10px] font-mono text-slate-500 text-right h-[400px]">
                        {yAxisLabels.map(l => <span key={l}>{l}</span>)}
                     </div>

                     {/* The Grid */}
                     <div className="grid grid-rows-8 grid-cols-6 gap-1.5 h-[400px] w-[500px]">
                        {gridData.map((row, yIdx) =>
                           row.map((val, xIdx) => (
                              <div
                                 key={`${xIdx}-${yIdx}`}
                                 onClick={() => setSelectedCell({ x: xAxisLabels[xIdx], y: yAxisLabels[yIdx], val })}
                                 className={`rounded-md cursor-pointer transition-all duration-300 hover:scale-125 hover:z-20 border border-white/0 hover:border-white/20 relative group ${getColor(val)}`}
                              >
                                 <div className="opacity-0 group-hover:opacity-100 absolute bottom-full left-1/2 -translate-x-1/2 mb-2 bg-black/90 border border-white/10 text-white text-[10px] px-2 py-1.5 rounded whitespace-nowrap z-30 pointer-events-none font-mono shadow-xl backdrop-blur-md">
                                    SR: <span className="text-cyan-400 font-bold">{val}</span>
                                 </div>
                              </div>
                           ))
                        )}
                     </div>
                  </div>

                  {/* X-Axis Labels */}
                  <div className="w-[500px] flex justify-between pl-10 mt-4 text-[10px] font-mono text-slate-500">
                     {xAxisLabels.map(l => <span key={l} className="w-8 text-center">{l}</span>)}
                  </div>
                  {/* X-Axis Label */}
                  <div className="mt-4 text-[10px] font-bold text-cyan-400 tracking-[0.2em] text-center pl-10">
                     FAST MA PERIOD
                  </div>
               </div>
            </div>

            {/* 3. Selected Iteration Details (Bottom Panel) */}
            <div className="h-40 mt-6 bg-slate-900/40 border border-white/5 rounded-2xl p-5 flex gap-8 backdrop-blur-md z-10 transition-all">
               {selectedCell ? (
                  <>
                     <div className="w-56 border-r border-white/5 pr-8 flex flex-col justify-center gap-1">
                        <span className="text-[10px] font-bold text-slate-500 uppercase tracking-widest">Selected Params</span>
                        <div className="text-3xl font-mono text-white font-bold tracking-tighter">
                           {selectedCell.x} <span className="text-slate-700 text-xl font-light">/</span> {selectedCell.y}
                        </div>
                        <div className="flex items-center gap-2 mt-2">
                           <span className="text-[10px] bg-cyan-500/10 border border-cyan-500/20 text-cyan-400 px-2 py-0.5 rounded font-bold font-mono">SR: {selectedCell.val}</span>
                        </div>
                     </div>
                     <div className="flex-1 grid grid-cols-4 gap-6 items-center">
                        <div className="group">
                           <div className="text-[10px] text-slate-500 mb-1 uppercase tracking-wider group-hover:text-emerald-400 transition-colors">Ann. Return</div>
                           <div className="text-xl font-mono font-bold text-emerald-400 group-hover:shadow-[0_0_10px_rgba(52,211,153,0.3)] transition-shadow inline-block">+{Math.floor(selectedCell.val * 15.2)}%</div>
                        </div>
                        <div className="group">
                           <div className="text-[10px] text-slate-500 mb-1 uppercase tracking-wider group-hover:text-rose-400 transition-colors">Max Drawdown</div>
                           <div className="text-xl font-mono font-bold text-rose-400 group-hover:shadow-[0_0_10px_rgba(251,113,133,0.3)] transition-shadow inline-block">-{Math.abs(selectedCell.val * -4.2).toFixed(1)}%</div>
                        </div>
                        <div className="group">
                           <div className="text-[10px] text-slate-500 mb-1 uppercase tracking-wider group-hover:text-amber-400 transition-colors">Win Rate</div>
                           <div className="text-xl font-mono font-bold text-amber-400">{(45 + selectedCell.val * 8).toFixed(1)}%</div>
                        </div>
                        <div>
                           <button className="w-full py-2.5 bg-white text-black hover:bg-slate-200 rounded text-xs font-bold uppercase tracking-wider transition-colors shadow-[0_0_15px_rgba(255,255,255,0.1)]">
                              Apply Params
                           </button>
                        </div>
                     </div>
                  </>
               ) : (
                  <div className="w-full h-full flex flex-col items-center justify-center text-slate-600 gap-3">
                     <span className="material-symbols-outlined text-[24px] opacity-50">touch_app</span>
                     <span className="text-xs uppercase tracking-widest opacity-70">Select a cell in the heatmap to view details</span>
                  </div>
               )}
            </div>
         </div>
      </div>
   );
};

// --- STITCH REDESIGNED: Code Editor ---
const CodeEditor = () => (
   <div className="h-full flex bg-[#030305]">
      {/* 1. File Explorer */}
      <div className="w-64 bg-slate-950/80 backdrop-blur-md border-r border-white/5 flex flex-col transition-all z-20">
         <div className="p-4 border-b border-white/5 flex items-center justify-between">
            <span className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">Explorer</span>
            <div className="flex gap-2">
               <span className="material-symbols-outlined text-[14px] text-slate-500 cursor-pointer hover:text-white transition-colors">create_new_folder</span>
               <span className="material-symbols-outlined text-[14px] text-slate-500 cursor-pointer hover:text-white transition-colors">note_add</span>
            </div>
         </div>
         <div className="flex-1 overflow-y-auto py-3 text-xs font-mono">
            <div className="px-4 py-2 text-slate-400 hover:bg-slate-800/50 cursor-pointer flex items-center gap-2 group">
               <span className="material-symbols-outlined text-[16px] text-amber-500/80 group-hover:text-amber-400">folder_open</span>
               <span className="group-hover:text-slate-200 transition-colors">strategies</span>
            </div>
            <div className="px-8 py-2 text-cyan-400 bg-cyan-500/10 cursor-pointer flex items-center gap-2 border-l-2 border-cyan-500 font-medium">
               <span className="material-symbols-outlined text-[16px]">code</span> alpha_v1.py
            </div>
            <div className="px-8 py-2 text-slate-500 hover:text-slate-300 hover:bg-slate-800/30 cursor-pointer flex items-center gap-2 transition-colors">
               <span className="material-symbols-outlined text-[16px]">code</span> momentum.py
            </div>
            <div className="px-8 py-2 text-slate-500 hover:text-slate-300 hover:bg-slate-800/30 cursor-pointer flex items-center gap-2 transition-colors">
               <span className="material-symbols-outlined text-[16px]">code</span> mean_revert.py
            </div>
            <div className="px-4 py-2 text-slate-400 hover:bg-slate-800/50 cursor-pointer flex items-center gap-2 mt-2 group">
               <span className="material-symbols-outlined text-[16px] text-amber-500/80 group-hover:text-amber-400">folder</span>
               <span className="group-hover:text-slate-200 transition-colors">backtests</span>
            </div>
            <div className="px-8 py-2 text-slate-500 hover:text-slate-300 hover:bg-slate-800/30 cursor-pointer flex items-center gap-2 transition-colors">
               <span className="material-symbols-outlined text-[16px]">description</span> bt_202401.log
            </div>
         </div>
         {/* Bottom Action */}
         <div className="p-3 border-t border-white/5 bg-black/20">
            <div className="flex items-center gap-2 text-xs text-slate-500">
               <div className="w-2 h-2 rounded-full bg-emerald-500"></div> Connected to Kernel
            </div>
         </div>
      </div>

      {/* 2. Main Editor Area */}
      <div className="flex-1 flex flex-col bg-[#050507] relative">
         {/* Editor Toolbar */}
         <div className="h-10 bg-slate-900/50 backdrop-blur border-b border-white/5 flex items-center px-4 gap-4 z-10">
            <span className="text-xs text-slate-300 font-mono flex items-center gap-2">
               <span className="text-cyan-500">strategies</span> <span className="text-slate-600">/</span> alpha_v1.py
            </span>
            <span className="ml-auto flex items-center gap-2">
               <div className="h-4 w-px bg-white/10 mx-2"></div>
               <button className="flex items-center gap-1.5 px-3 py-1 bg-emerald-500/10 text-emerald-500 rounded border border-emerald-500/20 text-[10px] font-bold uppercase tracking-wider hover:bg-emerald-500/20 transition-colors shadow-[0_0_10px_rgba(16,185,129,0.1)]">
                  <span className="material-symbols-outlined text-[14px]">play_arrow</span> Run Strategy
               </button>
               <button className="p-1.5 hover:bg-slate-800 rounded text-slate-400 transition-colors tooltip" title="Save"><span className="material-symbols-outlined text-[16px]">save</span></button>
               <button className="p-1.5 hover:bg-slate-800 rounded text-slate-400 transition-colors tooltip" title="Settings"><span className="material-symbols-outlined text-[16px]">settings</span></button>
            </span>
         </div>

         {/* Code Content */}
         <div className="flex-1 p-0 font-mono text-sm overflow-auto leading-7 relative bg-[#050507] custom-scrollbar">
            {/* Line Numbers */}
            <div className="absolute left-0 top-0 bottom-0 w-12 text-right pr-4 pt-4 text-slate-700 select-none border-r border-white/5 bg-[#050507] text-xs font-mono leading-7 z-10">
               {Array.from({ length: 20 }, (_, i) => <div key={i}>{i + 1}</div>)}
            </div>
            {/* Code */}
            <div className="pl-16 pt-4 text-slate-300 text-xs">
               <div><span className="text-purple-400">import</span> hermes_api <span className="text-purple-400">as</span> api</div>
               <div><span className="text-purple-400">from</span> strategies.base <span className="text-purple-400">import</span> Strategy</div>
               <br />
               <div><span className="text-purple-400">class</span> <span className="text-yellow-300">MyAlpha</span>(Strategy):</div>
               <div>&nbsp;&nbsp;<span className="text-slate-500 italic"># Initialize implementation parameters</span></div>
               <div>&nbsp;&nbsp;<span className="text-purple-400">def</span> <span className="text-blue-400">initialize</span>(self):</div>
               <div>&nbsp;&nbsp;&nbsp;&nbsp;self.symbol = <span className="text-emerald-400">&apos;BTC/USDT&apos;</span></div>
               <div>&nbsp;&nbsp;&nbsp;&nbsp;self.lookback = <span className="text-orange-400">20</span></div>
               <div>&nbsp;&nbsp;&nbsp;&nbsp;self.log(<span className="text-emerald-400">&quot;Strategy Started - Alpha V1&quot;</span>)</div>
               <br />
               <div>&nbsp;&nbsp;<span className="text-purple-400">def</span> <span className="text-blue-400">on_data</span>(self, data):</div>
               <div>&nbsp;&nbsp;&nbsp;&nbsp;<span className="text-slate-500 italic"># Calculate simple moving average</span></div>
               <div>&nbsp;&nbsp;&nbsp;&nbsp;price = data.close</div>
               <div>&nbsp;&nbsp;&nbsp;&nbsp;ma = api.sma(data.close, self.lookback)</div>
               <div>&nbsp;&nbsp;&nbsp;&nbsp;rsi = api.rsi(data.close, <span className="text-orange-400">14</span>)</div>
               <br />
               <div>&nbsp;&nbsp;&nbsp;&nbsp;<span className="text-slate-500 italic"># Trading Logic</span></div>
               <div>&nbsp;&nbsp;&nbsp;&nbsp;<span className="text-purple-400">if</span> price &gt; ma <span className="text-purple-400">and</span> rsi &lt; <span className="text-orange-400">30</span>:</div>
               <div>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;self.buy(self.symbol, <span className="text-orange-400">1.0</span>)</div>
               <div>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;self.log(<span className="text-emerald-400">{'f"BUY SIGNAL @ {price}"'}</span>)</div>
               <div>&nbsp;&nbsp;&nbsp;&nbsp;<span className="text-purple-400">elif</span> price &lt; ma <span className="text-purple-400">or</span> rsi &gt; <span className="text-orange-400">70</span>:</div>
               <div>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;self.sell(self.symbol, <span className="text-orange-400">1.0</span>)</div>
               <div className="bg-slate-800/50 w-full h-[1px] my-2"></div>
            </div>
         </div>

         {/* Terminal / Logs */}
         <div className="h-44 bg-[#030305] border-t border-white/5 flex flex-col z-20">
            <div className="flex items-center px-4 py-1.5 bg-slate-900/50 border-b border-white/5 gap-4">
               <span className="text-[10px] uppercase font-bold text-slate-300 border-b-2 border-cyan-500 pb-1.5 -mb-2">Output</span>
               <span className="text-[10px] uppercase font-bold text-slate-600 hover:text-slate-300 cursor-pointer pb-1.5 -mb-2">Terminal</span>
               <span className="text-[10px] uppercase font-bold text-slate-600 hover:text-slate-300 cursor-pointer pb-1.5 -mb-2">Debug Console</span>
               <div className="ml-auto flex gap-2">
                  <span className="material-symbols-outlined text-[14px] text-slate-500 cursor-pointer">delete</span>
                  <span className="material-symbols-outlined text-[14px] text-slate-500 cursor-pointer">keyboard_arrow_down</span>
               </div>
            </div>
            <div className="flex-1 p-3 font-mono text-xs overflow-y-auto space-y-1">
               <div className="text-slate-500">[10:02:15] <span className="text-emerald-500">INFO</span> System initialized successfully.</div>
               <div className="text-slate-500">[10:02:16] <span className="text-blue-400">LOAD</span> Loading strategy configuration &apos;alpha_v1.py&apos;...</div>
               <div className="text-slate-500">[10:02:17] <span className="text-emerald-500">SUCCESS</span> Compilation completed (156ms).</div>
               <div className="text-slate-400">[10:02:18] <span className="text-amber-500">WARN</span> Data gap detected at 2023-01-05 (2 bars missing). Interpolating...</div>
               <div className="text-slate-300 mt-2 border-t border-slate-800/50 pt-2 flex items-center gap-2">
                  <span className="text-cyan-500">➜</span>
                  <span>Starting backtest (2023-01-01 to 2023-12-31)...</span>
               </div>
               <div className="text-slate-300 animate-pulse font-bold ml-4">_</div>
            </div>
         </div>
      </div>
   </div>
);

// --- STITCH REDESIGNED: Signal Lifecycle Component ---
const SignalLifecycle = () => (
   <div className="bg-slate-900/30 border border-white/5 p-4 rounded-xl mt-4 relative overflow-hidden backdrop-blur-sm">
      <div className="absolute top-0 right-0 w-32 h-32 bg-cyan-500/5 rounded-full blur-3xl -mr-16 -mt-16 pointer-events-none"></div>
      <h4 className="text-[10px] font-bold text-slate-400 uppercase mb-5 flex items-center gap-2 tracking-widest">
         <span className="material-symbols-outlined text-[16px] text-cyan-500">history_toggle_off</span>
         Signal Lifecycle Reconstruction - Event ID #89921
      </h4>
      <div className="relative h-24 flex items-center">
         {/* Timeline Line */}
         <div className="absolute left-4 right-4 top-1/2 h-0.5 bg-slate-800/50"></div>
         <div className="absolute left-4 right-4 top-1/2 h-0.5 bg-gradient-to-r from-blue-500/50 via-purple-500/50 to-emerald-500/50 opacity-30"></div>

         {/* Steps */}
         <div className="relative z-10 flex justify-between w-full px-8">
            {/* Step 1: Ingestion */}
            <div className="flex flex-col items-center gap-2 group">
               <div className="w-8 h-8 rounded-full bg-slate-950 border border-blue-500/50 text-blue-400 flex items-center justify-center font-bold text-[10px] shadow-[0_0_15px_rgba(59,130,246,0.3)] group-hover:scale-110 transition-transform cursor-crosshair">
                  IN
               </div>
               <div className="text-center">
                  <div className="text-[9px] text-slate-500 font-mono mb-0.5">T+0ms</div>
                  <div className="text-[10px] font-bold text-slate-200 group-hover:text-blue-400 transition-colors">Ingested</div>
                  <div className="text-[9px] text-slate-600">Bloomberg API</div>
               </div>
            </div>

            {/* Step 2: NLP */}
            <div className="flex flex-col items-center gap-2 group">
               <div className="w-8 h-8 rounded-full bg-slate-950 border border-purple-500/50 text-purple-400 flex items-center justify-center font-bold text-[10px] shadow-[0_0_15px_rgba(168,85,247,0.3)] group-hover:scale-110 transition-transform cursor-crosshair">
                  AI
               </div>
               <div className="text-center">
                  <div className="text-[9px] text-slate-500 font-mono mb-0.5">T+12ms</div>
                  <div className="text-[10px] font-bold text-slate-200 group-hover:text-purple-400 transition-colors">Sentiment</div>
                  <div className="text-[9px] text-slate-600">Score: +92</div>
               </div>
            </div>

            {/* Step 3: Signal Logic */}
            <div className="flex flex-col items-center gap-2 group">
               <div className="w-8 h-8 rounded-full bg-slate-950 border border-cyan-500/50 text-cyan-400 flex items-center justify-center font-bold text-[10px] shadow-[0_0_15px_rgba(6,182,212,0.3)] group-hover:scale-110 transition-transform cursor-crosshair">
                  SIG
               </div>
               <div className="text-center">
                  <div className="text-[9px] text-slate-500 font-mono mb-0.5">T+15ms</div>
                  <div className="text-[10px] font-bold text-slate-200 group-hover:text-cyan-400 transition-colors">Trigger</div>
                  <div className="text-[9px] text-slate-600">Threshold &gt; 85</div>
               </div>
            </div>

            {/* Step 4: Execution */}
            <div className="flex flex-col items-center gap-2 group">
               <div className="w-8 h-8 rounded-full bg-slate-950 border border-emerald-500/50 text-emerald-400 flex items-center justify-center font-bold text-[10px] shadow-[0_0_15px_rgba(16,185,129,0.3)] group-hover:scale-110 transition-transform cursor-crosshair">
                  EX
               </div>
               <div className="text-center">
                  <div className="text-[9px] text-slate-500 font-mono mb-0.5">T+18ms</div>
                  <div className="text-[10px] font-bold text-slate-200 group-hover:text-emerald-400 transition-colors">Order Sent</div>
                  <div className="text-[9px] text-slate-600">Binance Limit</div>
               </div>
            </div>
         </div>
      </div>
   </div>
);

// --- UPDATED: Sentiment Analysis with Event Reconstruction ---
// --- STITCH REDESIGNED: Sentiment Analysis with Event Reconstruction ---
const SentimentAnalysis = () => {
   // Mock chart data combining price and sentiment volume
   const [sentimentChartData, setSentimentChartData] = useState<{ time: string, price: number, sentiment: number, volume: number }[]>([]);

   useEffect(() => {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setSentimentChartData(Array.from({ length: 24 }, (_, i) => ({
         time: `${i}:00`,
         price: 64000 + Math.random() * 800 + i * 50,
         sentiment: 50 + Math.sin(i / 3) * 30 + Math.random() * 10,
         volume: Math.floor(Math.random() * 5000) + 2000
      })));
   }, []);

   const [activeTradeItem, setActiveTradeItem] = useState<typeof SENTIMENT_FEED[0] | null>(null);

   // New Component: Signal Lifecycle Reconstruction for HFT visualization


   return (
      <div className="h-full p-6 flex flex-col gap-6 overflow-hidden bg-[#030305] relative">
         {/* Background Noise */}
         <div className="absolute inset-0 bg-[url('https://grainy-gradients.vercel.app/noise.svg')] opacity-20 pointer-events-none"></div>

         {/* Quick Trade Modal Overlay */}
         {activeTradeItem && <QuickTradeModal item={activeTradeItem} onClose={() => setActiveTradeItem(null)} />}

         {/* Top Row: Market Overview Cards */}
         <div className="grid grid-cols-4 gap-6 h-36 flex-shrink-0 z-10">
            {/* Market Mood Gauge */}
            <div className="col-span-1 bg-slate-900/40 rounded-xl border border-white/5 p-5 relative overflow-hidden backdrop-blur-md group hover:border-white/10 transition-colors">
               <div className="flex justify-between items-center mb-4">
                  <span className="text-[10px] font-bold text-slate-400 uppercase tracking-widest">Market Mood</span>
                  <span className="text-[10px] text-slate-600 font-mono">Global AI</span>
               </div>
               <div className="flex items-end gap-3 mt-1">
                  <span className="text-4xl font-bold text-white tracking-tighter">76</span>
                  <span className="text-xs font-bold text-emerald-400 mb-1.5 uppercase tracking-wider">Extreme Greed</span>
               </div>
               {/* Visual Gauge Bar */}
               <div className="w-full h-1.5 bg-slate-800 rounded-full mt-4 flex gap-0.5 overflow-hidden">
                  <div className="w-[20%] bg-rose-500/30"></div>
                  <div className="w-[30%] bg-amber-500/30"></div>
                  <div className="w-[50%] bg-emerald-500 shadow-[0_0_10px_#10b981]"></div>
               </div>
               <div className="absolute bottom-4 right-4 text-[10px] text-slate-500 font-mono">
                  Vol <span className="text-emerald-400">+5%</span>
               </div>
            </div>

            {/* Key Metrics */}
            {[
               { label: 'Bull/Bear Ratio', val: '2.45', change: '+12%', color: 'text-emerald-400', colorRaw: 'emerald' },
               { label: 'AI Confidence', val: '94%', change: 'High', color: 'text-cyan-400', colorRaw: 'cyan' },
               { label: 'Social Volume (24h)', val: '1.2M', change: '+15%', color: 'text-white', colorRaw: 'slate' },
            ].map((m, i) => (
               <div key={i} className="col-span-1 bg-slate-900/40 rounded-xl border border-white/5 p-5 flex flex-col justify-center backdrop-blur-md hover:border-white/10 transition-all group relative overflow-hidden">
                  <div className={`absolute top-0 right-0 w-20 h-20 bg-${m.colorRaw}-500/5 rounded-full blur-2xl -mr-8 -mt-8 transition-opacity opacity-50 group-hover:opacity-100`}></div>
                  <span className="text-[10px] font-bold text-slate-400 uppercase tracking-widest z-10">{m.label}</span>
                  <div className={`text-3xl font-bold mt-2 font-mono ${m.color} tracking-tight z-10`}>{m.val}</div>
                  <div className="text-[10px] text-slate-500 mt-2 flex items-center gap-1 z-10 font-mono">
                     vs yest. <span className={m.change.includes('+') ? 'text-emerald-400' : 'text-slate-400'}>{m.change}</span>
                  </div>
               </div>
            ))}
         </div>

         {/* Main Content Grid */}
         <div className="flex-1 min-h-0 grid grid-cols-12 gap-6 z-10">

            {/* Left Column: Charts & Analysis (8 cols) */}
            <div className="col-span-8 flex flex-col gap-6">
               {/* Main Chart */}
               <div className="flex-1 bg-slate-900/40 rounded-xl border border-white/5 p-5 flex flex-col backdrop-blur-md relative">
                  <div className="flex justify-between items-center mb-6">
                     <h3 className="text-xs font-bold text-white flex items-center gap-2 uppercase tracking-wider">
                        <span className="material-symbols-outlined text-cyan-500 text-[18px]">monitoring</span>
                        Price vs Sentiment Multi-D
                     </h3>
                     <div className="flex gap-2">
                        {['BTC', 'ETH', 'SOL', 'NVDA'].map(t => (
                           <button key={t} className={`text-[10px] px-3 py-1 rounded-md font-bold transition-all ${t === 'BTC' ? 'bg-cyan-500/10 border border-cyan-500/50 text-cyan-400 shadow-[0_0_10px_rgba(6,182,212,0.2)]' : 'border border-transparent hover:bg-slate-800 text-slate-500 hover:text-slate-300'}`}>{t}</button>
                        ))}
                     </div>
                  </div>
                  <div className="flex-1 w-full min-h-0">
                     <ResponsiveContainer width="100%" height="100%">
                        <ComposedChart data={sentimentChartData}>
                           <defs>
                              <linearGradient id="colorVol" x1="0" y1="0" x2="0" y2="1">
                                 <stop offset={0.05} stopColor="#3b82f6" stopOpacity={0.1} />
                                 <stop offset={0.95} stopColor="#3b82f6" stopOpacity={0} />
                              </linearGradient>
                           </defs>
                           <CartesianGrid strokeDasharray="3 3" stroke="#ffffff" strokeOpacity={0.05} vertical={false} />
                           <XAxis dataKey="time" tick={{ fontSize: 9, fill: '#52525b', fontFamily: 'monospace' }} axisLine={false} tickLine={false} dy={10} />
                           <YAxis yAxisId="right" orientation="right" tick={{ fontSize: 9, fill: '#52525b', fontFamily: 'monospace' }} axisLine={false} tickLine={false} domain={['auto', 'auto']} dx={10} />
                           <YAxis yAxisId="left" hide />
                           <RechartsTooltip
                              contentStyle={{ backgroundColor: 'rgba(2, 6, 23, 0.9)', borderColor: 'rgba(255,255,255,0.1)', fontSize: '12px', borderRadius: '8px', backdropFilter: 'blur(4px)' }}
                              itemStyle={{ color: '#e2e8f0' }}
                           />
                           <Legend verticalAlign="top" height={36} iconSize={8} wrapperStyle={{ fontSize: '10px', color: '#71717a', textTransform: 'uppercase', letterSpacing: '1px' }} />
                           <Area yAxisId="left" type="monotone" dataKey="volume" name="Social Vol" fill="url(#colorVol)" stroke="#3b82f6" strokeOpacity={0.5} />
                           <Line yAxisId="right" type="monotone" dataKey="price" name="Price" stroke="#10b981" strokeWidth={2} dot={false} />
                           <Line yAxisId="left" type="monotone" dataKey="sentiment" name="Sentiment Score" stroke="#f59e0b" strokeWidth={2} dot={false} strokeDasharray="4 4" />
                        </ComposedChart>
                     </ResponsiveContainer>
                  </div>

                  {/* Inserted Signal Lifecycle Here */}
                  <SignalLifecycle />
               </div>

               {/* Bottom: Asset Ranking & Word Cloud */}
               <div className="h-64 grid grid-cols-2 gap-6">
                  {/* Sentiment Movers Table */}
                  <div className="bg-slate-900/40 rounded-xl border border-white/5 flex flex-col overflow-hidden backdrop-blur-md">
                     <div className="px-5 py-3 border-b border-white/5 flex justify-between items-center bg-black/20">
                        <h3 className="text-xs font-bold text-white uppercase tracking-wider">Sentiment Movers</h3>
                        <span className="text-[9px] text-slate-500 font-mono uppercase">Real-time</span>
                     </div>
                     <div className="flex-1 overflow-auto custom-scrollbar">
                        <table className="w-full text-left">
                           <thead className="bg-black/20 text-slate-500 sticky top-0 backdrop-blur-sm">
                              <tr>
                                 <th className="px-5 py-2 text-[10px] font-bold uppercase tracking-wider">Asset</th>
                                 <th className="px-5 py-2 text-right text-[10px] font-bold uppercase tracking-wider">Score</th>
                                 <th className="px-5 py-2 text-right text-[10px] font-bold uppercase tracking-wider">24h Chg</th>
                                 <th className="px-5 py-2 text-right text-[10px] font-bold uppercase tracking-wider">Signal</th>
                              </tr>
                           </thead>
                           <tbody className="divide-y divide-white/5 text-[11px] font-mono">
                              {SENTIMENT_MOVERS.map((m, i) => (
                                 <tr key={i} className="hover:bg-white/5 transition-colors cursor-pointer group">
                                    <td className="px-5 py-2.5 font-bold text-slate-300 group-hover:text-white">{m.symbol}</td>
                                    <td className="px-5 py-2.5 text-right">
                                       <span className={`px-1.5 py-0.5 rounded ${m.score > 0 ? 'bg-emerald-500/10 text-emerald-400' : 'bg-rose-500/10 text-rose-400'}`}>{m.score}</span>
                                    </td>
                                    <td className="px-5 py-2.5 text-right text-slate-400">{m.change}</td>
                                    <td className="px-5 py-2.5 text-right font-bold">
                                       <span className={m.signal === 'Buy' ? 'text-emerald-400' : m.signal === 'Sell' ? 'text-rose-400' : 'text-slate-500'}>{m.signal}</span>
                                    </td>
                                 </tr>
                              ))}
                           </tbody>
                        </table>
                     </div>
                  </div>

                  {/* Narrative Cloud */}
                  <div className="bg-slate-900/40 rounded-xl border border-white/5 flex flex-col backdrop-blur-md">
                     <div className="px-5 py-3 border-b border-white/5 flex justify-between items-center bg-black/20">
                        <h3 className="text-xs font-bold text-white uppercase tracking-wider">Narrative Cloud</h3>
                     </div>
                     <div className="flex-1 p-5 flex flex-wrap content-center justify-center gap-3">
                        {HOT_TOPICS.map((topic, i) => (
                           <span
                              key={i}
                              className={`px-3 py-1.5 rounded-full border transition-all cursor-pointer hover:scale-105 hover:shadow-lg backdrop-blur-sm font-medium ${topic.sentiment === 'up' ? 'bg-emerald-500/5 border-emerald-500/20 text-emerald-400 hover:border-emerald-500/50' :
                                 topic.sentiment === 'down' ? 'bg-rose-500/5 border-rose-500/20 text-rose-400 hover:border-rose-500/50' :
                                    topic.sentiment === 'warn' ? 'bg-amber-500/5 border-amber-500/20 text-amber-400 hover:border-amber-500/50' :
                                       'bg-slate-800/50 border-slate-700/50 text-slate-400'
                                 }`}
                              style={{
                                 fontSize: `${Math.max(10, topic.weight / 6)}px`,
                                 opacity: Math.max(0.6, topic.weight / 100)
                              }}
                           >
                              {topic.text}
                           </span>
                        ))}
                     </div>
                  </div>
               </div>
            </div>

            {/* Right Column: Institutional Feed (4 cols) */}
            <div className="col-span-4 bg-slate-900/40 rounded-xl border border-white/5 flex flex-col shadow-xl overflow-hidden backdrop-blur-md">
               <div className="px-5 py-3 border-b border-white/5 flex justify-between items-center bg-black/20">
                  <h3 className="text-xs font-bold text-white flex items-center gap-2 uppercase tracking-wider">
                     <span className="material-symbols-outlined text-cyan-500 animate-pulse text-[18px]">cell_tower</span>
                     Institutional Stream
                  </h3>
                  <div className="flex gap-1">
                     <button className="p-1 hover:bg-white/5 rounded text-slate-500 hover:text-white transition-colors"><span className="material-symbols-outlined text-[16px]">filter_list</span></button>
                     <button className="p-1 hover:bg-white/5 rounded text-slate-500 hover:text-white transition-colors"><span className="material-symbols-outlined text-[16px]">settings</span></button>
                  </div>
               </div>

               <div className="flex-1 overflow-y-auto custom-scrollbar">
                  {SENTIMENT_FEED.map((item) => (
                     <div key={item.id} className="p-4 border-b border-white/5 hover:bg-white/5 transition-colors group cursor-pointer relative">
                        <div className="absolute left-0 top-0 bottom-0 w-0.5 bg-gradient-to-b from-transparent via-cyan-500/0 to-transparent group-hover:via-cyan-500/50 transition-all"></div>
                        <div className="flex justify-between items-start mb-2">
                           <div className="flex items-center gap-2">
                              <span className={`text-[9px] font-bold px-1.5 py-0.5 rounded uppercase tracking-wide border ${item.type === 'News' ? 'border-blue-500/20 text-blue-400 bg-blue-500/5' :
                                 item.type === 'Social' ? 'border-purple-500/20 text-purple-400 bg-purple-500/5' :
                                    'border-slate-700/50 text-slate-400 bg-slate-800/30'
                                 }`}>{item.type}</span>
                              <span className="text-[10px] text-slate-600 font-mono group-hover:text-slate-500 transition-colors">{item.time}</span>
                           </div>
                           <span className={`text-[10px] font-bold font-mono ${item.score > 0 ? 'text-emerald-400' : 'text-rose-400'}`}>
                              {item.score > 0 ? `+${item.score}` : item.score} Impact
                           </span>
                        </div>

                        <h4 className="text-sm text-slate-300 font-medium leading-normal mb-2 group-hover:text-white transition-colors">
                           <span className="text-cyan-500 font-bold mr-1.5">[{item.entity}]</span>
                           {item.content}
                        </h4>

                        <div className="flex justify-between items-center mt-3">
                           <div className="flex items-center gap-2 text-[10px] text-slate-600 group-hover:text-slate-500">
                              <span className="flex items-center gap-1.5"><span className="material-symbols-outlined text-[14px]">account_circle</span> {item.source}</span>
                           </div>
                           <button
                              onClick={(e) => {
                                 e.stopPropagation();
                                 setActiveTradeItem(item);
                              }}
                              className="opacity-0 translate-y-2 group-hover:translate-y-0 group-hover:opacity-100 transition-all duration-300 bg-cyan-500 hover:bg-cyan-400 text-black font-bold text-[10px] px-3 py-1.5 rounded shadow-[0_0_15px_rgba(6,182,212,0.3)] uppercase tracking-wide"
                           >
                              Execute Trade
                           </button>
                        </div>
                     </div>
                  ))}
               </div>
            </div>
         </div>
      </div>
   );
};

const TabButton = ({ label, icon, isActive, onClick }: { label: string, icon: string, isActive: boolean, onClick: () => void }) => (
   <button
      onClick={onClick}
      className={`flex items-center gap-2 h-full border-b-2 px-5 transition-all ${isActive
         ? 'border-hermes-500 text-surface-100'
         : 'border-transparent text-surface-400 hover:text-surface-200'
         }`}
   >
      <span className="material-symbols-outlined text-[20px]">{icon}</span>
      <span className="text-sm font-medium uppercase tracking-wide">{label}</span>
   </button>
);

const StrategyLab: React.FC = () => {
   const [activeTab, setActiveTab] = useState('evolution');

   useEffect(() => {
      loadFactorConfig().catch(console.error);
   }, []);



   return (
      <div className="h-full flex flex-col overflow-hidden bg-surface-950">
         {/* Header */}
         <div className="h-14 bg-surface-950 border-b border-surface-800 flex items-center justify-between px-6">
            <div>
               <h2 className="text-lg font-bold text-surface-100">策略实验室 (Strategy Lab)</h2>
               <p className="text-xs text-surface-500">低代码因子组合与逻辑构建</p>
            </div>
            <div className="flex gap-2">
               <button className="px-3 py-1.5 border border-surface-700 bg-surface-900 rounded text-xs text-surface-300 hover:text-white flex items-center gap-2">
                  <span className="material-symbols-outlined text-[16px]">save</span> 保存草稿
               </button>
               <button className="px-3 py-1.5 border border-surface-700 bg-surface-900 rounded text-xs text-surface-300 hover:text-white flex items-center gap-2">
                  <span className="material-symbols-outlined text-[16px]">history</span> 历史版本
               </button>
            </div>
         </div>

         {/* Tab Navigation */}
         <div className="h-10 bg-surface-900 border-b border-surface-800 flex items-center px-4">
            <div className="flex gap-2 h-full">
               <TabButton label="进化迭代 (Evolution)" icon="genetics" isActive={activeTab === 'evolution'} onClick={() => setActiveTab('evolution')} />
               <TabButton label="因子研究 (Alpha)" icon="science" isActive={activeTab === 'factor'} onClick={() => setActiveTab('factor')} />
               <TabButton label="舆情情报" icon="psychology" isActive={activeTab === 'sentiment'} onClick={() => setActiveTab('sentiment')} />
               <TabButton label="代码编辑" icon="code" isActive={activeTab === 'editor'} onClick={() => setActiveTab('editor')} />
               <TabButton label="参数寻优" icon="tune" isActive={activeTab === 'optimization'} onClick={() => setActiveTab('optimization')} />
            </div>
         </div>

         {/* Content Area */}
         <div className="flex-1 min-h-0 relative">
            {activeTab === 'evolution' && <div className="p-6 overflow-y-auto h-full"><EvolutionExplorer /></div>}
            {activeTab === 'factor' && <FactorResearch />}
            {activeTab === 'sentiment' && <SentimentAnalysis />}
            {activeTab === 'editor' && <CodeEditor />}
            {activeTab === 'optimization' && <Optimization />}
         </div>
      </div >
   );
};
export default StrategyLab;