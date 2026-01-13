import React, { useState, useEffect } from 'react';
import { FACTOR_DATA, SENTIMENT_FEED, FACTOR_LIBRARY, SENTIMENT_MOVERS, HOT_TOPICS } from '../constants';
import { BarChart, Bar, Cell, XAxis, YAxis, CartesianGrid, Tooltip as RechartsTooltip, ResponsiveContainer, LineChart, Line, AreaChart, Area, Legend, ComposedChart, ReferenceLine, ScatterChart, Scatter } from 'recharts';

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
   const IC_DATA = Array.from({length: 30}, (_, i) => ({
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
                        <XAxis type="number" tick={{fontSize: 10, fill: '#71717a'}} axisLine={false} tickLine={false} />
                        <YAxis type="category" dataKey="bucket" tick={{fontSize: 11, fill: '#a1a1aa'}} width={70} axisLine={false} tickLine={false} />
                        <RechartsTooltip cursor={{fill: '#27272a'}} contentStyle={{backgroundColor: '#18181b', borderColor: '#27272a', fontSize: '12px'}} />
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
                        <YAxis tick={{fontSize: 10, fill: '#71717a'}} axisLine={false} tickLine={false} />
                        <ReferenceLine y={0} stroke="#52525b" />
                        <RechartsTooltip contentStyle={{backgroundColor: '#18181b', borderColor: '#27272a', fontSize: '12px'}} />
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
                     <AreaChart data={Array.from({length: 50}, (_, i) => ({
                        step: i,
                        val: 1000 + (i * 15) + (Math.random() * 50 - 25)
                     }))}>
                        <defs>
                           <linearGradient id="colorLSE" x1="0" y1="0" x2="0" y2="1">
                              <stop offset="5%" stopColor="#8b5cf6" stopOpacity={0.2}/>
                              <stop offset="95%" stopColor="#8b5cf6" stopOpacity={0}/>
                           </linearGradient>
                        </defs>
                        <CartesianGrid strokeDasharray="3 3" stroke="#27272a" vertical={false} />
                        <XAxis dataKey="step" hide />
                        <YAxis tick={{fontSize: 10, fill: '#71717a'}} axisLine={false} tickLine={false} domain={['auto', 'auto']} />
                        <RechartsTooltip contentStyle={{backgroundColor: '#18181b', borderColor: '#27272a', fontSize: '12px'}} />
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
const FactorResearch = () => {
  const [viewMode, setViewMode] = useState<'canvas' | 'analytics'>('canvas');
  
  // State for real-time factor scoring simulation
  const [rsi, setRsi] = useState({ value: 32.4, zScore: 1.15 });
  const [sentiment, setSentiment] = useState({ value: 76, zScore: 2.10 });
  const [composite, setComposite] = useState({ zScore: 1.72, rank: 2 });
  
  // Search State
  const [searchQuery, setSearchQuery] = useState('');

  // Search Filter Helper
  const filterFactors = (factors: {name: string, desc: string}[]) => {
     if (!searchQuery) return factors;
     return factors.filter(f => 
        f.name.toLowerCase().includes(searchQuery.toLowerCase()) || 
        f.desc.toLowerCase().includes(searchQuery.toLowerCase())
     );
  };

  useEffect(() => {
    const timer = setInterval(() => {
      setRsi(prev => {
         const noise = (Math.random() - 0.5) * 4;
         let val = prev.value + noise;
         if (val > 80) val = 78; if (val < 20) val = 22;
         const z = (50 - val) / 10; 
         return { value: parseFloat(val.toFixed(1)), zScore: parseFloat(z.toFixed(2)) };
      });
      setSentiment(prev => {
         const noise = (Math.random() - 0.5) * 2;
         let val = prev.value + noise;
         if (val > 95) val = 94; if (val < 5) val = 6;
         const z = (val - 50) / 15;
         return { value: parseFloat(val.toFixed(1)), zScore: parseFloat(z.toFixed(2)) };
      });
      setComposite(prev => {
          const newZ = (rsi.zScore * 0.4 + sentiment.zScore * 0.6) + (Math.random() - 0.5) * 0.05;
          const rank = newZ > 1.5 ? 1 : newZ > 0.5 ? 2 : newZ > -0.5 ? 3 : 4;
          return { zScore: parseFloat(newZ.toFixed(2)), rank };
      });
    }, 1000);
    return () => clearInterval(timer);
  }, [rsi.zScore, sentiment.zScore]);

  return (
    <div className="h-full flex bg-[#0c0c0e] overflow-hidden">
       {/* 1. Left Sidebar: Factor Library & Universe */}
       <div className="w-64 bg-surface-950 border-r border-surface-800 flex flex-col flex-shrink-0">
          {/* View Switcher */}
          <div className="p-3 border-b border-surface-800 grid grid-cols-2 gap-2 bg-surface-950">
             <button 
                onClick={() => setViewMode('canvas')}
                className={`py-1.5 text-xs font-bold rounded flex items-center justify-center gap-1 transition-colors ${viewMode === 'canvas' ? 'bg-surface-800 text-surface-100 border border-surface-600' : 'text-surface-500 hover:text-surface-300'}`}
             >
                <span className="material-symbols-outlined text-[16px]">hub</span> 构造
             </button>
             <button 
                onClick={() => setViewMode('analytics')}
                className={`py-1.5 text-xs font-bold rounded flex items-center justify-center gap-1 transition-colors ${viewMode === 'analytics' ? 'bg-hermes-500/10 text-hermes-500 border border-hermes-500/20' : 'text-surface-500 hover:text-surface-300'}`}
             >
                <span className="material-symbols-outlined text-[16px]">analytics</span> 透视
             </button>
          </div>

          <div className="p-3 border-b border-surface-800">
             <div className="relative">
                <span className="absolute left-2.5 top-2.5 material-symbols-outlined text-surface-500 text-[18px]">search</span>
                <input 
                  type="text" 
                  placeholder="搜索因子库..." 
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="w-full bg-surface-900 border border-surface-700 rounded-md py-2 pl-9 pr-3 text-sm text-surface-200 placeholder-surface-600 focus:border-hermes-500 outline-none transition-colors" 
                />
             </div>
          </div>

          <div className="flex-1 overflow-y-auto p-2 space-y-4">
             {/* Universe Selection (NEW) */}
             <div>
                <h4 className="px-2 mb-2 text-xs font-bold text-surface-500 uppercase flex items-center gap-2">
                   <span className="material-symbols-outlined text-[14px]">filter_alt</span> 资产池 (Universe)
                </h4>
                <div className="space-y-1.5 px-2">
                   <div className="flex items-center gap-2 text-sm text-surface-300 bg-surface-800/50 p-2 rounded border border-surface-700">
                      <input type="checkbox" defaultChecked className="accent-hermes-500" />
                      <span>TOP 100 Market Cap</span>
                   </div>
                   <div className="flex items-center gap-2 text-sm text-surface-300 bg-surface-800/50 p-2 rounded border border-surface-700">
                      <input type="checkbox" defaultChecked className="accent-hermes-500" />
                      <span>Vol > 2.5%</span>
                   </div>
                </div>
             </div>

             {/* Category: Technical */}
             {filterFactors(FACTOR_LIBRARY.technical).length > 0 && (
                <div>
                   <h4 className="px-2 mb-2 text-xs font-bold text-surface-500 uppercase flex items-center gap-2">
                      <span className="material-symbols-outlined text-[14px]">show_chart</span> 技术指标 (Technical)
                   </h4>
                   <div className="space-y-1">
                      {filterFactors(FACTOR_LIBRARY.technical).map((f, i) => (
                         <div key={i} className="flex flex-col px-3 py-2 bg-surface-900/50 hover:bg-surface-800 border border-transparent hover:border-surface-600 rounded cursor-grab active:cursor-grabbing transition-all group">
                            <span className="text-sm font-bold text-surface-300 group-hover:text-surface-100">{f.name}</span>
                            <span className="text-[10px] text-surface-500">{f.desc}</span>
                         </div>
                      ))}
                   </div>
                </div>
             )}
             {/* ... Other categories ... */}
          </div>
       </div>

       {/* 2. Main Area: Switch between Canvas and Analytics */}
       <div className="flex-1 relative bg-[#09090b] overflow-hidden">
          {viewMode === 'analytics' ? (
             <AlphaLens />
          ) : (
             <>
               {/* Grid Background */}
               <div className="absolute inset-0 opacity-20" style={{backgroundImage: 'radial-gradient(#3f3f46 1px, transparent 1px)', backgroundSize: '20px 20px'}}></div>
               
               {/* Nodes Container */}
               <div className="absolute inset-0">
                  <svg className="absolute inset-0 w-full h-full pointer-events-none z-0">
                     {/* Connection Lines */}
                     <path d="M 450 180 C 500 180, 500 280, 550 280" stroke="#52525b" strokeWidth="2" fill="none" />
                     <path d="M 450 350 C 500 350, 500 280, 550 280" stroke="#52525b" strokeWidth="2" fill="none" />
                     <path d="M 700 280 C 750 280, 750 280, 800 280" stroke="#52525b" strokeWidth="2" fill="none" />
                     {/* NEW: Connection to Execution Node */}
                     <path d="M 960 280 C 1000 280, 1000 280, 1040 280" stroke={composite.zScore > 0.5 || composite.zScore < -0.5 ? "#00E396" : "#52525b"} strokeWidth="2" fill="none" strokeDasharray="4 4" className={composite.zScore > 0.5 || composite.zScore < -0.5 ? "animate-[dash_1s_linear_infinite]" : ""} />
                  </svg>
                  <style>{`@keyframes dash { to { stroke-dashoffset: -8; } }`}</style>

                  {/* Node 1: RSI */}
                  <div className="absolute top-[140px] left-[250px] w-48 bg-surface-900 border border-surface-600 rounded-md shadow-xl flex flex-col z-10">
                     <div className="px-3 py-2 bg-surface-800 border-b border-surface-700 flex justify-between items-center rounded-t-md cursor-move">
                        <div className="flex items-center gap-2">
                           <div className="w-2 h-2 rounded-full bg-blue-500"></div>
                           <span className="text-xs font-bold text-surface-200">RSI (14) - Reversal</span>
                        </div>
                        <span className="material-symbols-outlined text-[14px] text-surface-500">more_horiz</span>
                     </div>
                     <div className="p-3">
                        <div className="flex justify-between items-center mb-2">
                           <span className="text-[10px] text-surface-400">Current Value</span>
                           <span className="text-sm font-mono text-surface-100">{rsi.value}</span>
                        </div>
                        <div className="flex justify-between items-center bg-surface-950 p-2 rounded border border-surface-800/50">
                           <span className="text-[10px] text-surface-500 font-bold uppercase">Z-Score</span>
                           <span className={`text-sm font-bold font-mono ${rsi.zScore > 0 ? 'text-trade-up' : 'text-trade-down'}`}>
                             {rsi.zScore > 0 ? '+' : ''}{rsi.zScore}
                           </span>
                        </div>
                     </div>
                     <div className="absolute -right-1.5 top-1/2 w-3 h-3 bg-surface-400 rounded-full border-2 border-surface-900"></div>
                  </div>

                  {/* Node 2: Sentiment */}
                  <div className="absolute top-[310px] left-[250px] w-48 bg-surface-900 border border-hermes-500/50 rounded-md shadow-xl flex flex-col z-10">
                     <div className="px-3 py-2 bg-surface-800 border-b border-surface-700 flex justify-between items-center rounded-t-md cursor-move">
                        <div className="flex items-center gap-2">
                           <div className="w-2 h-2 rounded-full bg-hermes-500 animate-pulse"></div>
                           <span className="text-xs font-bold text-surface-200">AI Sentiment</span>
                        </div>
                     </div>
                     <div className="p-3">
                        <div className="flex justify-between items-center mb-2">
                           <span className="text-[10px] text-surface-400">Score (0-100)</span>
                           <span className="text-sm font-mono text-hermes-500 font-bold">{sentiment.value}</span>
                        </div>
                        <div className="flex justify-between items-center bg-surface-950 p-2 rounded border border-surface-800/50">
                           <span className="text-[10px] text-surface-500 font-bold uppercase">Z-Score</span>
                           <span className={`text-sm font-bold font-mono ${sentiment.zScore > 0 ? 'text-trade-up' : 'text-trade-down'}`}>
                             {sentiment.zScore > 0 ? '+' : ''}{sentiment.zScore}
                           </span>
                        </div>
                     </div>
                     <div className="absolute -right-1.5 top-1/2 w-3 h-3 bg-hermes-500 rounded-full border-2 border-surface-900"></div>
                  </div>

                  {/* Node 3: Model Weights */}
                  <div className="absolute top-[225px] left-[550px] w-48 bg-surface-900 border border-surface-600 rounded-md shadow-[0_0_15px_rgba(0,0,0,0.2)] flex flex-col z-10">
                     <div className="px-3 py-2 bg-surface-800 border-b border-surface-700 flex justify-between items-center rounded-t-md cursor-move">
                        <div className="flex items-center gap-2">
                           <span className="material-symbols-outlined text-[14px] text-purple-400">hub</span>
                           <span className="text-xs font-bold text-surface-200">Multi-Factor Model</span>
                        </div>
                     </div>
                     <div className="p-3 space-y-2">
                        <div className="flex justify-between items-center">
                           <span className="text-[10px] text-surface-400">Tech (RSI)</span>
                           <div className="flex items-center gap-2">
                              <div className="w-16 h-1.5 bg-surface-950 rounded-full overflow-hidden">
                                 <div className="h-full bg-blue-500 w-[40%]"></div>
                              </div>
                              <span className="text-[10px] font-mono text-surface-300">40%</span>
                           </div>
                        </div>
                        <div className="flex justify-between items-center">
                           <span className="text-[10px] text-surface-400">Sentiment</span>
                           <div className="flex items-center gap-2">
                              <div className="w-16 h-1.5 bg-surface-950 rounded-full overflow-hidden">
                                 <div className="h-full bg-hermes-500 w-[60%]"></div>
                              </div>
                              <span className="text-[10px] font-mono text-surface-300">60%</span>
                           </div>
                        </div>
                     </div>
                     <div className="absolute -left-1.5 top-[30%] w-3 h-3 bg-surface-400 rounded-full border-2 border-surface-900"></div>
                     <div className="absolute -left-1.5 top-[70%] w-3 h-3 bg-surface-400 rounded-full border-2 border-surface-900"></div>
                     <div className="absolute -right-1.5 top-1/2 w-3 h-3 bg-purple-500 rounded-full border-2 border-surface-900"></div>
                  </div>

                  {/* Node 4: Final Score Output */}
                  <div className="absolute top-[240px] left-[800px] w-40 bg-surface-800 border border-surface-600 rounded-md shadow-xl flex flex-col p-0 z-10 border-l-4 border-l-hermes-500 overflow-hidden">
                      <div className="bg-surface-900/50 p-2 border-b border-surface-700/50">
                         <span className="text-[10px] font-bold text-surface-400 uppercase tracking-wider block text-center">Final Signal</span>
                      </div>
                      <div className="p-3 text-center">
                         <div className="text-2xl font-bold text-hermes-500 font-mono tracking-tighter">
                            {composite.zScore > 0 ? '+' : ''}{composite.zScore}
                         </div>
                         <div className="text-[10px] text-surface-500 mt-1 font-medium uppercase">
                            Composite Z-Score
                         </div>
                      </div>
                      <div className="absolute -left-1.5 top-1/2 w-3 h-3 bg-purple-500 rounded-full border-2 border-surface-900"></div>
                      <div className="absolute -right-1.5 top-1/2 w-3 h-3 bg-hermes-500 rounded-full border-2 border-surface-900"></div>
                  </div>

                  {/* NEW Node 5: Trading Execution */}
                  <div className="absolute top-[220px] left-[1040px] w-48 bg-surface-900 border border-hermes-500 rounded-md shadow-[0_0_20px_rgba(0,227,150,0.1)] flex flex-col z-10">
                     <div className="px-3 py-2 bg-hermes-900/30 border-b border-hermes-500/30 flex justify-between items-center rounded-t-md cursor-move">
                        <div className="flex items-center gap-2">
                           <span className="material-symbols-outlined text-[16px] text-hermes-500">token</span>
                           <span className="text-xs font-bold text-hermes-100">Execution Algo</span>
                        </div>
                        <div className="w-2 h-2 rounded-full bg-hermes-500 animate-pulse shadow-[0_0_5px_#00E396]"></div>
                     </div>
                     <div className="p-3 space-y-3">
                        <div>
                           <label className="text-[10px] text-surface-500 font-bold uppercase block mb-1">Target Asset</label>
                           <select className="w-full bg-surface-950 border border-surface-700 rounded text-xs p-1 text-surface-200 outline-none">
                              <option>BTC/USDT Perpetual</option>
                              <option>ETH/USDT Perpetual</option>
                           </select>
                        </div>
                        <div className="flex gap-2">
                           <div className="flex-1">
                              <label className="text-[10px] text-surface-500 font-bold uppercase block mb-1">Size</label>
                              <div className="bg-surface-950 border border-surface-700 rounded text-xs p-1 text-right text-surface-200 font-mono">25%</div>
                           </div>
                           <div className="flex-1">
                              <label className="text-[10px] text-surface-500 font-bold uppercase block mb-1">Algo</label>
                              <div className="bg-surface-950 border border-surface-700 rounded text-xs p-1 text-center text-surface-200">TWAP</div>
                           </div>
                        </div>
                        <div className="pt-1">
                           <button className="w-full py-1.5 bg-hermes-500 hover:bg-hermes-400 text-surface-950 font-bold text-xs rounded transition-colors flex items-center justify-center gap-1">
                              <span className="material-symbols-outlined text-[14px]">play_arrow</span>
                              Auto-Trade: ON
                           </button>
                        </div>
                     </div>
                     <div className="absolute -left-1.5 top-1/2 w-3 h-3 bg-hermes-500 rounded-full border-2 border-surface-900"></div>
                  </div>
               </div>
             </>
          )}
       </div>

       {/* 3. Right Sidebar: Backtest Config */}
       <div className="w-72 bg-surface-950 border-l border-surface-800 flex flex-col flex-shrink-0 z-20 shadow-[-5px_0_15px_rgba(0,0,0,0.5)]">
          <div className="p-3 border-b border-surface-800 flex justify-between items-center bg-surface-950">
             <h3 className="text-sm font-bold text-surface-100">回测配置 (Backtest)</h3>
             <span className="material-symbols-outlined text-surface-500 cursor-pointer text-[18px]">settings</span>
          </div>
          
          <div className="flex-1 overflow-y-auto p-4 space-y-6">
             {/* Time Range */}
             <div>
                <label className="text-xs font-bold text-surface-500 uppercase tracking-wider mb-2 block">回测区间 (Time Range)</label>
                <div className="grid grid-cols-2 gap-2">
                   <div className="bg-surface-900 border border-surface-700 rounded p-2">
                      <div className="text-[10px] text-surface-500">开始时间</div>
                      <div className="text-sm font-mono text-surface-200">2023-01-01</div>
                   </div>
                   <div className="bg-surface-900 border border-surface-700 rounded p-2">
                      <div className="text-[10px] text-surface-500">结束时间</div>
                      <div className="text-sm font-mono text-surface-200">2023-12-31</div>
                   </div>
                </div>
             </div>

             {/* Benchmark */}
             <div>
                <label className="text-xs font-bold text-surface-500 uppercase tracking-wider mb-2 block">基准指数 (Benchmark)</label>
                <select className="w-full bg-surface-900 border border-surface-700 rounded p-2 text-sm text-surface-200 outline-none focus:border-hermes-500">
                   <option>HS300 (沪深300)</option>
                   <option>ZZ500 (中证500)</option>
                   <option>BTC (比特币)</option>
                </select>
             </div>

             {/* Capital */}
             <div>
                <label className="text-xs font-bold text-surface-500 uppercase tracking-wider mb-2 block">初始资金 (Capital)</label>
                <div className="relative">
                   <span className="absolute left-3 top-2 text-surface-400">¥</span>
                   <input type="text" defaultValue="1,000,000" className="w-full bg-surface-900 border border-surface-700 rounded p-2 pl-7 text-sm font-mono text-surface-200 focus:border-hermes-500 outline-none" />
                </div>
             </div>
             
             <div className="border-t border-surface-800 pt-4 space-y-3">
                <div className="flex justify-between items-center">
                   <span className="text-sm text-surface-300">包含交易费率</span>
                   <div className="w-8 h-4 bg-hermes-500 rounded-full relative"><div className="w-2.5 h-2.5 bg-white rounded-full absolute right-1 top-0.5 shadow-sm"></div></div>
                </div>
                <div className="flex justify-between items-center">
                   <span className="text-sm text-surface-300">滑点控制 (Slippage)</span>
                   <div className="w-8 h-4 bg-surface-700 rounded-full relative"><div className="w-2.5 h-2.5 bg-surface-400 rounded-full absolute left-1 top-0.5"></div></div>
                </div>
             </div>
          </div>

          <div className="p-4 border-t border-surface-800 bg-surface-950">
             <button className="w-full py-3 bg-hermes-500 hover:bg-hermes-400 text-surface-950 font-bold rounded shadow-[0_0_15px_rgba(0,227,150,0.3)] transition-all flex items-center justify-center gap-2">
                <span className="material-symbols-outlined">play_circle</span>
                开始回测 (Run Backtest)
             </button>
             <div className="text-center mt-2 text-[10px] text-surface-500">预计耗时: ~45s • 消耗算力点: 12</div>
          </div>
       </div>
    </div>
  );
};

// --- REDESIGNED: Parameter Optimization Module (Heatmap Grid) ---
const Optimization = () => {
   const [running, setRunning] = useState(false);
   const [progress, setProgress] = useState(0);
   const [selectedCell, setSelectedCell] = useState<{x: number, y: number, val: number} | null>(null);

   // Mock Heatmap Data (Sharpe Ratios)
   // X-Axis: Fast MA (5, 10, 15, 20, 25)
   // Y-Axis: Slow MA (20, 30, 40, 50, 60, 70, 80, 90)
   const xAxisLabels = [5, 10, 15, 20, 25, 30];
   const yAxisLabels = [30, 40, 50, 60, 70, 80, 90, 100];
   
   // Generate grid data with a "peak" in the middle
   const gridData = yAxisLabels.map((y, yIdx) => {
      return xAxisLabels.map((x, xIdx) => {
         // Create a fake "peak" around Fast=15, Slow=60
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

   // Helper for heat color
   const getColor = (val: number) => {
      if (val >= 2.5) return 'bg-[#00E396]'; // High Green
      if (val >= 2.0) return 'bg-[#00E396]/70';
      if (val >= 1.5) return 'bg-[#FEB019]/80'; // Med Yellow
      if (val >= 1.0) return 'bg-[#FF4560]/70'; // Low Red
      return 'bg-[#FF4560]/40';
   };

   return (
      <div className="h-full flex bg-[#0c0c0e]">
         {/* 1. Left Config Panel */}
         <div className="w-80 bg-surface-950 border-r border-surface-800 flex flex-col p-5 overflow-y-auto">
            <h2 className="text-lg font-bold text-surface-100 mb-1">参数寻优 (Optimization)</h2>
            <p className="text-xs text-surface-500 mb-6">网格搜索 (Grid Search) 与敏感度分析</p>

            <div className="space-y-6">
               {/* Strategy Select */}
               <div>
                  <label className="text-xs font-bold text-surface-500 uppercase tracking-wider block mb-2">目标策略 (Strategy)</label>
                  <select className="w-full bg-surface-900 border border-surface-700 rounded p-2.5 text-sm text-surface-200 outline-none focus:border-hermes-500">
                     <option>Alpha-Trend-v1 (BTC/USDT)</option>
                     <option>Mean-Reversion-ETH</option>
                  </select>
               </div>

               {/* Parameter 1 Config */}
               <div className="bg-surface-900 border border-surface-800 rounded p-3">
                  <div className="flex justify-between items-center mb-3">
                     <span className="text-sm font-bold text-hermes-500">参数 X: Fast MA</span>
                     <span className="text-xs bg-surface-800 px-1.5 rounded text-surface-400">整数</span>
                  </div>
                  <div className="grid grid-cols-3 gap-2 mb-2">
                     <div>
                        <label className="text-[10px] text-surface-500 uppercase block">Start</label>
                        <input type="number" defaultValue="5" className="w-full bg-surface-950 border border-surface-700 rounded p-1 text-sm font-mono text-center" />
                     </div>
                     <div>
                        <label className="text-[10px] text-surface-500 uppercase block">End</label>
                        <input type="number" defaultValue="30" className="w-full bg-surface-950 border border-surface-700 rounded p-1 text-sm font-mono text-center" />
                     </div>
                     <div>
                        <label className="text-[10px] text-surface-500 uppercase block">Step</label>
                        <input type="number" defaultValue="5" className="w-full bg-surface-950 border border-surface-700 rounded p-1 text-sm font-mono text-center" />
                     </div>
                  </div>
               </div>

               {/* Parameter 2 Config */}
               <div className="bg-surface-900 border border-surface-800 rounded p-3">
                  <div className="flex justify-between items-center mb-3">
                     <span className="text-sm font-bold text-blue-400">参数 Y: Slow MA</span>
                     <span className="text-xs bg-surface-800 px-1.5 rounded text-surface-400">整数</span>
                  </div>
                  <div className="grid grid-cols-3 gap-2 mb-2">
                     <div>
                        <label className="text-[10px] text-surface-500 uppercase block">Start</label>
                        <input type="number" defaultValue="30" className="w-full bg-surface-950 border border-surface-700 rounded p-1 text-sm font-mono text-center" />
                     </div>
                     <div>
                        <label className="text-[10px] text-surface-500 uppercase block">End</label>
                        <input type="number" defaultValue="100" className="w-full bg-surface-950 border border-surface-700 rounded p-1 text-sm font-mono text-center" />
                     </div>
                     <div>
                        <label className="text-[10px] text-surface-500 uppercase block">Step</label>
                        <input type="number" defaultValue="10" className="w-full bg-surface-950 border border-surface-700 rounded p-1 text-sm font-mono text-center" />
                     </div>
                  </div>
               </div>

               {/* Target */}
               <div>
                  <label className="text-xs font-bold text-surface-500 uppercase tracking-wider block mb-2">优化目标 (Objective)</label>
                  <div className="grid grid-cols-2 gap-2">
                     <button className="py-2 bg-hermes-500/10 border border-hermes-500 text-hermes-500 rounded text-xs font-bold">Max Sharpe</button>
                     <button className="py-2 bg-surface-900 border border-surface-700 text-surface-400 hover:text-surface-200 rounded text-xs font-bold transition-colors">Max Return</button>
                  </div>
               </div>
            </div>

            <div className="mt-auto pt-6">
               <button 
                  onClick={handleRun}
                  disabled={running}
                  className={`w-full py-3 rounded-md font-bold text-sm uppercase tracking-wide shadow-lg transition-all flex items-center justify-center gap-2 ${running ? 'bg-surface-800 text-surface-500 cursor-not-allowed' : 'bg-hermes-500 hover:bg-hermes-400 text-surface-950'}`}
               >
                  {running ? (
                     <>
                        <span className="w-4 h-4 border-2 border-surface-500 border-t-transparent rounded-full animate-spin"></span>
                        计算中 {progress}%
                     </>
                  ) : (
                     <>
                        <span className="material-symbols-outlined text-[18px]">play_arrow</span>
                        开始运算 (Run)
                     </>
                  )}
               </button>
            </div>
         </div>

         {/* 2. Main Visualization Area */}
         <div className="flex-1 flex flex-col p-6 overflow-hidden">
            {/* Toolbar */}
            <div className="flex justify-between items-center mb-6">
               <h3 className="text-base font-bold text-surface-200 flex items-center gap-2">
                  <span className="material-symbols-outlined text-hermes-500">grid_on</span>
                  参数热力图分布 (Sharpe Ratio Heatmap)
               </h3>
               <div className="flex gap-2">
                  <div className="flex items-center gap-1 text-xs text-surface-400 px-3">
                     <span className="w-3 h-3 bg-[#FF4560]/40 rounded-sm"></span> Low
                     <span className="w-8 h-1 bg-gradient-to-r from-[#FF4560]/40 via-[#FEB019]/80 to-[#00E396] mx-1 rounded-full"></span>
                     <span className="w-3 h-3 bg-[#00E396] rounded-sm"></span> High
                  </div>
               </div>
            </div>

            {/* Heatmap Grid */}
            <div className="flex-1 bg-surface-900 border border-surface-800 rounded-lg p-6 relative flex flex-col items-center justify-center shadow-inner">
               {/* Y-Axis Label */}
               <div className="absolute left-4 top-1/2 -translate-y-1/2 -rotate-90 text-xs font-bold text-blue-400 tracking-widest origin-center whitespace-nowrap">
                  Slow MA Period (Parameter Y)
               </div>
               
               <div className="flex">
                  {/* Y-Axis Ticks */}
                  <div className="flex flex-col justify-between pr-4 py-2 text-xs font-mono text-surface-400 text-right h-[400px]">
                     {yAxisLabels.map(l => <span key={l}>{l}</span>)}
                  </div>

                  {/* The Grid */}
                  <div className="grid grid-rows-8 grid-cols-6 gap-1 h-[400px] w-[500px]">
                     {gridData.map((row, yIdx) => 
                        row.map((val, xIdx) => (
                           <div 
                              key={`${xIdx}-${yIdx}`}
                              onClick={() => setSelectedCell({x: xAxisLabels[xIdx], y: yAxisLabels[yIdx], val})}
                              className={`rounded-sm cursor-pointer transition-all hover:scale-110 hover:z-10 hover:shadow-lg border border-transparent hover:border-white relative group ${getColor(val)}`}
                           >
                              <div className="opacity-0 group-hover:opacity-100 absolute bottom-full left-1/2 -translate-x-1/2 mb-2 bg-surface-950 border border-surface-700 text-xs px-2 py-1 rounded whitespace-nowrap z-20 pointer-events-none font-mono">
                                 Sharpe: {val}
                              </div>
                           </div>
                        ))
                     )}
                  </div>
               </div>

               {/* X-Axis Labels */}
               <div className="w-[500px] flex justify-between pl-10 mt-3 text-xs font-mono text-surface-400">
                  {xAxisLabels.map(l => <span key={l} className="w-8 text-center">{l}</span>)}
               </div>
               {/* X-Axis Label */}
               <div className="mt-2 text-xs font-bold text-hermes-500 tracking-widest">
                  Fast MA Period (Parameter X)
               </div>
            </div>

            {/* 3. Selected Iteration Details (Bottom Panel) */}
            <div className="h-40 mt-6 bg-surface-900 border border-surface-800 rounded-lg p-4 flex gap-6 animate-in slide-in-from-bottom-4">
               {selectedCell ? (
                  <>
                     <div className="w-48 border-r border-surface-800 pr-6 flex flex-col justify-center">
                        <span className="text-xs font-bold text-surface-500 uppercase mb-1">选中参数组合</span>
                        <div className="text-2xl font-mono text-surface-100 font-bold mb-1">
                           {selectedCell.x} <span className="text-surface-600 text-sm">/</span> {selectedCell.y}
                        </div>
                        <div className="flex items-center gap-2">
                           <span className="text-xs bg-hermes-500/10 text-hermes-500 px-1.5 py-0.5 rounded font-bold">Sharpe: {selectedCell.val}</span>
                        </div>
                     </div>
                     <div className="flex-1 grid grid-cols-4 gap-4 items-center">
                        <div>
                           <div className="text-xs text-surface-500 mb-1">年化收益 (Ann. Return)</div>
                           <div className="text-lg font-bold text-trade-up">+{Math.floor(selectedCell.val * 15.2)}%</div>
                        </div>
                        <div>
                           <div className="text-xs text-surface-500 mb-1">最大回撤 (Max DD)</div>
                           <div className="text-lg font-bold text-trade-down">-{Math.abs(selectedCell.val * -4.2).toFixed(1)}%</div>
                        </div>
                        <div>
                           <div className="text-xs text-surface-500 mb-1">胜率 (Win Rate)</div>
                           <div className="text-lg font-bold text-surface-200">{(45 + selectedCell.val * 8).toFixed(1)}%</div>
                        </div>
                        <div>
                           <button className="w-full py-2 bg-surface-800 hover:bg-surface-700 text-surface-200 border border-surface-600 rounded text-xs font-bold transition-colors">
                              应用此参数
                           </button>
                        </div>
                     </div>
                  </>
               ) : (
                  <div className="w-full h-full flex items-center justify-center text-surface-500 text-sm italic gap-2">
                     <span className="material-symbols-outlined">touch_app</span>
                     点击热力图上的色块查看详细回测指标
                  </div>
               )}
            </div>
         </div>
      </div>
   );
};

const CodeEditor = () => (
  <div className="h-full flex bg-surface-950">
    <div className="w-64 bg-surface-950 border-r border-surface-800 flex flex-col">
       <div className="p-3 border-b border-surface-800 text-xs font-bold text-surface-500 uppercase tracking-wider flex items-center justify-between">
          资源管理器 (Explorer)
          <span className="material-symbols-outlined text-[16px] cursor-pointer hover:text-surface-300">add</span>
       </div>
       <div className="flex-1 overflow-y-auto py-2 text-sm font-mono">
          <div className="px-4 py-1.5 text-surface-300 hover:bg-surface-800 cursor-pointer flex items-center gap-2">
             <span className="material-symbols-outlined text-[16px] text-yellow-600">folder</span> strategies
          </div>
          <div className="px-8 py-1.5 text-hermes-500 bg-surface-800/50 cursor-pointer flex items-center gap-2 border-l-2 border-hermes-500 font-medium">
             <span className="material-symbols-outlined text-[16px]">code</span> alpha_v1.py
          </div>
          <div className="px-8 py-1.5 text-surface-500 hover:text-surface-300 cursor-pointer flex items-center gap-2">
             <span className="material-symbols-outlined text-[16px]">code</span> momentum.py
          </div>
       </div>
    </div>
    
    <div className="flex-1 flex flex-col bg-[#09090b]">
       <div className="h-10 bg-surface-900 border-b border-surface-800 flex items-center px-4 gap-3">
          <span className="text-sm text-surface-400 font-mono font-medium">strategies / alpha_v1.py</span>
          <span className="ml-auto flex items-center gap-2">
             <button className="flex items-center gap-1.5 px-3 py-1 bg-hermes-500/10 text-hermes-500 rounded border border-hermes-500/20 text-xs font-bold hover:bg-hermes-500/20 transition-colors">
                <span className="material-symbols-outlined text-[14px]">play_arrow</span> 运行
             </button>
             <button className="p-1.5 hover:bg-surface-800 rounded text-surface-400 transition-colors"><span className="material-symbols-outlined text-[18px]">save</span></button>
          </span>
       </div>
       <div className="flex-1 p-6 font-mono text-sm overflow-auto text-surface-300 leading-7 relative bg-[#09090b]">
          <div className="absolute left-0 top-6 bottom-0 w-12 text-right pr-4 text-surface-700 select-none border-r border-surface-800/50 bg-[#09090b]">
             1<br/>2<br/>3<br/>4<br/>5<br/>6<br/>7<br/>8<br/>9<br/>10<br/>11<br/>12<br/>13<br/>14<br/>15
          </div>
          <div className="pl-14">
             <span className="text-purple-400 font-bold">import</span> hermes_api <span className="text-purple-400 font-bold">as</span> api<br/>
             <span className="text-purple-400 font-bold">from</span> strategies.base <span className="text-purple-400 font-bold">import</span> Strategy<br/>
             <br/>
             <span className="text-purple-400 font-bold">class</span> <span className="text-yellow-200 font-bold">MyAlpha</span>(Strategy):<br/>
             &nbsp;&nbsp;<span className="text-surface-500 italic"># Init params</span><br/>
             &nbsp;&nbsp;<span className="text-purple-400 font-bold">def</span> <span className="text-blue-400 font-bold">initialize</span>(self):<br/>
             &nbsp;&nbsp;&nbsp;&nbsp;self.symbol = <span className="text-green-400">'BTC/USDT'</span><br/>
             &nbsp;&nbsp;&nbsp;&nbsp;self.lookback = <span className="text-orange-400">20</span><br/>
             &nbsp;&nbsp;&nbsp;&nbsp;self.log(<span className="text-green-400">"Strategy Started"</span>)<br/>
             <br/>
             &nbsp;&nbsp;<span className="text-purple-400 font-bold">def</span> <span className="text-blue-400 font-bold">on_data</span>(self, data):<br/>
             &nbsp;&nbsp;&nbsp;&nbsp;price = data.close<br/>
             &nbsp;&nbsp;&nbsp;&nbsp;ma = api.sma(data.close, self.lookback)<br/>
             &nbsp;&nbsp;&nbsp;&nbsp;<br/>
             &nbsp;&nbsp;&nbsp;&nbsp;<span className="text-purple-400 font-bold">if</span> price > ma:<br/>
             &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;self.buy(self.symbol, <span className="text-orange-400">1.0</span>)<br/>
             &nbsp;&nbsp;&nbsp;&nbsp;<span className="text-purple-400 font-bold">elif</span> price &lt; ma:<br/>
             &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;self.sell(self.symbol, <span className="text-orange-400">1.0</span>)<br/>
          </div>
       </div>
       <div className="h-40 bg-surface-900 border-t border-surface-800 p-3 font-mono text-sm overflow-y-auto">
          <div className="text-surface-500 mb-1">[10:02:15] 系统初始化完毕.</div>
          <div className="text-surface-500 mb-1">[10:02:16] 加载策略 'alpha_v1.py'...</div>
          <div className="text-hermes-500 mb-1">[10:02:17] 编译成功.</div>
          <div className="text-surface-300 mb-1">[10:02:18] > 开始回测 (2023-01-01 to 2023-12-31)</div>
          <div className="text-surface-300 animation-pulse font-bold">_</div>
       </div>
    </div>
  </div>
);

// --- UPDATED: Sentiment Analysis with Event Reconstruction ---
const SentimentAnalysis = () => {
  // Mock chart data combining price and sentiment volume
  const sentimentChartData = Array.from({length: 24}, (_, i) => ({
    time: `${i}:00`,
    price: 64000 + Math.random() * 800 + i * 50,
    sentiment: 50 + Math.sin(i / 3) * 30 + Math.random() * 10,
    volume: Math.floor(Math.random() * 5000) + 2000
  }));

  const [activeTradeItem, setActiveTradeItem] = useState<typeof SENTIMENT_FEED[0] | null>(null);

  // New Component: Signal Lifecycle Reconstruction for HFT visualization
  const SignalLifecycle = () => (
    <div className="bg-surface-900 border border-surface-800 p-4 rounded-md mt-4 shadow-sm">
        <h4 className="text-xs font-bold text-surface-400 uppercase mb-4 flex items-center gap-2">
            <span className="material-symbols-outlined text-[16px]">history_toggle_off</span>
            信号全链路重构 (Signal Lifecycle Reconstruction) - Event ID #89921
        </h4>
        <div className="relative h-24 flex items-center">
            {/* Timeline Line */}
            <div className="absolute left-0 right-0 top-1/2 h-0.5 bg-surface-800"></div>
            
            {/* Steps */}
            <div className="relative z-10 flex justify-between w-full px-4">
                {/* Step 1: Ingestion */}
                <div className="flex flex-col items-center gap-2">
                    <div className="w-8 h-8 rounded-full bg-blue-500/10 border border-blue-500 text-blue-500 flex items-center justify-center font-bold text-xs shadow-[0_0_10px_rgba(59,130,246,0.2)]">
                        IN
                    </div>
                    <div className="text-center">
                        <div className="text-[10px] text-surface-500 font-mono mb-0.5">T+0ms</div>
                        <div className="text-xs font-bold text-surface-200">News Ingested</div>
                        <div className="text-[9px] text-surface-500">Bloomberg API</div>
                    </div>
                </div>

                    {/* Step 2: NLP */}
                    <div className="flex flex-col items-center gap-2">
                    <div className="w-8 h-8 rounded-full bg-purple-500/10 border border-purple-500 text-purple-500 flex items-center justify-center font-bold text-xs shadow-[0_0_10px_rgba(168,85,247,0.2)]">
                        AI
                    </div>
                    <div className="text-center">
                        <div className="text-[10px] text-surface-500 font-mono mb-0.5">T+12ms</div>
                        <div className="text-xs font-bold text-surface-200">Sentiment Engine</div>
                        <div className="text-[9px] text-surface-500">Score: +92 (Bullish)</div>
                    </div>
                </div>

                    {/* Step 3: Signal Logic */}
                    <div className="flex flex-col items-center gap-2">
                    <div className="w-8 h-8 rounded-full bg-hermes-500/10 border border-hermes-500 text-hermes-500 flex items-center justify-center font-bold text-xs shadow-[0_0_10px_rgba(0,227,150,0.2)]">
                        SIG
                    </div>
                    <div className="text-center">
                        <div className="text-[10px] text-surface-500 font-mono mb-0.5">T+15ms</div>
                        <div className="text-xs font-bold text-surface-200">Alpha Trigger</div>
                        <div className="text-[9px] text-surface-500">Threshold > 85</div>
                    </div>
                </div>

                    {/* Step 4: Execution */}
                    <div className="flex flex-col items-center gap-2">
                    <div className="w-8 h-8 rounded-full bg-trade-up/10 border border-trade-up text-trade-up flex items-center justify-center font-bold text-xs shadow-[0_0_10px_rgba(0,227,150,0.2)]">
                        EX
                    </div>
                    <div className="text-center">
                        <div className="text-[10px] text-surface-500 font-mono mb-0.5">T+18ms</div>
                        <div className="text-xs font-bold text-surface-200">Order Sent</div>
                        <div className="text-[9px] text-surface-500">Binance / Buy / Limit</div>
                    </div>
                </div>
            </div>
        </div>
    </div>
  );

  return (
  <div className="h-full p-4 flex flex-col gap-4 overflow-hidden bg-surface-950 relative">
     {/* Quick Trade Modal Overlay */}
     {activeTradeItem && <QuickTradeModal item={activeTradeItem} onClose={() => setActiveTradeItem(null)} />}

     {/* Top Row: Market Overview Cards */}
     <div className="grid grid-cols-4 gap-4 h-32 flex-shrink-0">
        {/* Market Mood Gauge */}
        <div className="col-span-1 bg-surface-900 rounded-md border border-surface-800 p-4 relative overflow-hidden">
           <div className="flex justify-between items-center mb-2">
              <span className="text-xs font-bold text-surface-400 uppercase tracking-wide">市场情绪指数 (Market Mood)</span>
              <span className="text-[10px] text-surface-500">Global AI</span>
           </div>
           <div className="flex items-end gap-2 mt-1">
              <span className="text-3xl font-bold text-hermes-500">76</span>
              <span className="text-xs font-bold text-hermes-500 mb-1">极度贪婪</span>
           </div>
           {/* Visual Gauge Bar */}
           <div className="w-full h-2 bg-surface-800 rounded-full mt-3 flex gap-1">
              <div className="w-[20%] bg-trade-down rounded-l-full opacity-30"></div>
              <div className="w-[30%] bg-trade-warn opacity-30"></div>
              <div className="w-[50%] bg-hermes-500 rounded-r-full shadow-[0_0_10px_#00E396]"></div>
           </div>
           <p className="text-[10px] text-surface-500 mt-2">较昨日 <span className="text-hermes-500 font-bold">+5</span> • 社交声量飙升</p>
        </div>

        {/* Key Metrics */}
        {[
           { label: '看涨/看跌比 (Bull/Bear)', val: '2.45', change: '+12%', color: 'text-hermes-500' },
           { label: 'AI 置信度 (Confidence)', val: '94%', change: 'High', color: 'text-blue-400' },
           { label: '社交总声量 (24h Vol)', val: '1.2M', change: '+15%', color: 'text-surface-200' },
        ].map((m, i) => (
           <div key={i} className="col-span-1 bg-surface-900 rounded-md border border-surface-800 p-4 flex flex-col justify-center">
              <span className="text-xs font-bold text-surface-400 uppercase tracking-wide">{m.label}</span>
              <div className={`text-2xl font-bold mt-1 font-mono ${m.color}`}>{m.val}</div>
              <div className="text-[10px] text-surface-500 mt-1 flex items-center gap-1">
                 环比 <span className={m.change.includes('+') ? 'text-trade-up' : 'text-surface-300'}>{m.change}</span>
              </div>
           </div>
        ))}
     </div>

     {/* Main Content Grid */}
     <div className="flex-1 min-h-0 grid grid-cols-12 gap-4">
        
        {/* Left Column: Charts & Analysis (8 cols) */}
        <div className="col-span-8 flex flex-col gap-4">
           {/* Main Chart */}
           <div className="flex-1 bg-surface-900 rounded-md border border-surface-800 p-4 flex flex-col shadow-sm">
              <div className="flex justify-between items-center mb-4">
                 <h3 className="text-sm font-bold text-surface-200 flex items-center gap-2">
                    <span className="material-symbols-outlined text-hermes-500">monitoring</span>
                    价格 vs 情绪多维分析
                 </h3>
                 <div className="flex gap-2">
                    {['BTC', 'ETH', 'SOL', 'NVDA'].map(t => (
                       <button key={t} className={`text-xs px-2 py-1 rounded border ${t==='BTC' ? 'bg-surface-800 border-hermes-500 text-hermes-500' : 'border-surface-700 text-surface-400 hover:text-surface-200'}`}>{t}</button>
                    ))}
                 </div>
              </div>
              <div className="flex-1 w-full min-h-0">
                 <ResponsiveContainer width="100%" height="100%">
                    <ComposedChart data={sentimentChartData}>
                       <defs>
                          <linearGradient id="colorVol" x1="0" y1="0" x2="0" y2="1">
                             <stop offset={0.05} stopColor="#3b82f6" stopOpacity={0.1}/>
                             <stop offset={0.95} stopColor="#3b82f6" stopOpacity={0}/>
                          </linearGradient>
                       </defs>
                       <CartesianGrid strokeDasharray="3 3" stroke="#27272a" vertical={false} />
                       <XAxis dataKey="time" tick={{fontSize: 10, fill: '#71717a'}} axisLine={false} tickLine={false} />
                       <YAxis yAxisId="right" orientation="right" tick={{fontSize: 10, fill: '#71717a'}} axisLine={false} tickLine={false} domain={['auto', 'auto']} />
                       <YAxis yAxisId="left" hide />
                       <RechartsTooltip contentStyle={{backgroundColor: '#18181b', borderColor: '#27272a', fontSize: '12px'}} />
                       <Legend verticalAlign="top" height={36} iconSize={8} wrapperStyle={{fontSize: '12px', color: '#a1a1aa'}}/>
                       <Area yAxisId="left" type="monotone" dataKey="volume" name="社交声量" fill="url(#colorVol)" stroke="#3b82f6" strokeOpacity={0.5} />
                       <Line yAxisId="right" type="monotone" dataKey="price" name="价格 (Price)" stroke="#00E396" strokeWidth={2} dot={false} />
                       <Line yAxisId="left" type="monotone" dataKey="sentiment" name="情绪分 (Score)" stroke="#ff9800" strokeWidth={2} dot={false} strokeDasharray="4 4" />
                    </ComposedChart>
                 </ResponsiveContainer>
              </div>
              
              {/* Inserted Signal Lifecycle Here */}
              <SignalLifecycle />
           </div>

           {/* Bottom: Asset Ranking & Word Cloud */}
           <div className="h-64 grid grid-cols-2 gap-4">
              {/* Sentiment Movers Table */}
              <div className="bg-surface-900 rounded-md border border-surface-800 flex flex-col overflow-hidden">
                 <div className="px-4 py-3 border-b border-surface-800 flex justify-between items-center bg-surface-950/30">
                    <h3 className="text-sm font-bold text-surface-200">情绪异动榜 (Sentiment Movers)</h3>
                    <span className="text-[10px] text-surface-500">Real-time</span>
                 </div>
                 <div className="flex-1 overflow-auto">
                    <table className="w-full text-left text-xs">
                       <thead className="bg-surface-950 text-surface-500 sticky top-0">
                          <tr>
                             <th className="px-4 py-2">资产</th>
                             <th className="px-4 py-2 text-right">情绪分</th>
                             <th className="px-4 py-2 text-right">24h 变化</th>
                             <th className="px-4 py-2 text-right">信号</th>
                          </tr>
                       </thead>
                       <tbody className="divide-y divide-surface-800">
                          {SENTIMENT_MOVERS.map((m, i) => (
                             <tr key={i} className="hover:bg-surface-800/50 transition-colors">
                                <td className="px-4 py-2 font-bold text-surface-200">{m.symbol}</td>
                                <td className="px-4 py-2 text-right font-mono">
                                   <span className={`px-1.5 py-0.5 rounded ${m.score > 0 ? 'bg-trade-up/10 text-trade-up' : 'bg-trade-down/10 text-trade-down'}`}>{m.score}</span>
                                </td>
                                <td className="px-4 py-2 text-right text-surface-300">{m.change}</td>
                                <td className="px-4 py-2 text-right font-bold">
                                   <span className={m.signal === 'Buy' ? 'text-trade-up' : m.signal === 'Sell' ? 'text-trade-down' : 'text-surface-500'}>{m.signal}</span>
                                </td>
                             </tr>
                          ))}
                       </tbody>
                    </table>
                 </div>
              </div>

              {/* Narrative Cloud */}
              <div className="bg-surface-900 rounded-md border border-surface-800 flex flex-col">
                 <div className="px-4 py-3 border-b border-surface-800 flex justify-between items-center bg-surface-950/30">
                    <h3 className="text-sm font-bold text-surface-200">市场叙事云 (Narrative Cloud)</h3>
                 </div>
                 <div className="flex-1 p-4 flex flex-wrap content-center justify-center gap-3">
                    {HOT_TOPICS.map((topic, i) => (
                       <span 
                          key={i} 
                          className={`px-3 py-1.5 rounded-full border transition-all cursor-pointer hover:scale-105 ${
                             topic.sentiment === 'up' ? 'bg-trade-up/10 border-trade-up/30 text-trade-up' : 
                             topic.sentiment === 'down' ? 'bg-trade-down/10 border-trade-down/30 text-trade-down' : 
                             topic.sentiment === 'warn' ? 'bg-trade-warn/10 border-trade-warn/30 text-trade-warn' :
                             'bg-surface-800 border-surface-700 text-surface-300'
                          }`}
                          style={{
                             fontSize: `${Math.max(11, topic.weight / 5)}px`,
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
        <div className="col-span-4 bg-surface-900 rounded-md border border-surface-800 flex flex-col shadow-sm overflow-hidden">
           <div className="px-4 py-3 border-b border-surface-800 flex justify-between items-center bg-surface-950/30">
              <h3 className="text-sm font-bold text-surface-200 flex items-center gap-2">
                 <span className="material-symbols-outlined text-hermes-500 animate-pulse text-[18px]">cell_tower</span>
                 机构级舆情流
              </h3>
              <div className="flex gap-1">
                 <button className="p-1 hover:bg-surface-800 rounded"><span className="material-symbols-outlined text-[16px] text-surface-400">filter_list</span></button>
                 <button className="p-1 hover:bg-surface-800 rounded"><span className="material-symbols-outlined text-[16px] text-surface-400">settings</span></button>
              </div>
           </div>
           
           <div className="flex-1 overflow-y-auto font-sans">
              {SENTIMENT_FEED.map((item) => (
                 <div key={item.id} className="p-3 border-b border-surface-800 hover:bg-surface-800/50 transition-colors group cursor-pointer relative">
                    <div className="flex justify-between items-start mb-1.5">
                       <div className="flex items-center gap-2">
                          <span className={`text-[10px] font-bold px-1.5 py-0.5 rounded uppercase tracking-wide border ${
                             item.type === 'News' ? 'border-blue-500/30 text-blue-400 bg-blue-500/10' : 
                             item.type === 'Social' ? 'border-purple-500/30 text-purple-400 bg-purple-500/10' :
                             'border-surface-600 text-surface-400 bg-surface-800'
                          }`}>{item.type}</span>
                          <span className="text-[10px] text-surface-500 font-mono">{item.time}</span>
                       </div>
                       <span className={`text-[10px] font-bold ${item.score > 0 ? 'text-trade-up' : 'text-trade-down'}`}>
                          {item.score > 0 ? `+${item.score}` : item.score} Impact
                       </span>
                    </div>
                    
                    <h4 className="text-sm text-surface-200 font-medium leading-snug mb-1 group-hover:text-white transition-colors">
                       <span className="text-hermes-500 font-bold mr-1">[{item.entity}]</span>
                       {item.content}
                    </h4>

                    <div className="flex justify-between items-center mt-2">
                       <div className="flex items-center gap-2 text-[10px] text-surface-500">
                          <span className="flex items-center gap-1"><span className="material-symbols-outlined text-[12px]">account_circle</span> {item.source}</span>
                       </div>
                       <button 
                          onClick={(e) => {
                             e.stopPropagation();
                             setActiveTradeItem(item);
                          }}
                          className="opacity-0 group-hover:opacity-100 transition-all bg-hermes-500 hover:bg-hermes-400 text-surface-950 font-bold text-[10px] px-2.5 py-1 rounded shadow-lg transform active:scale-95"
                       >
                          立即交易
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

export const StrategyLab: React.FC = () => {
  const [activeTab, setActiveTab] = useState('factor');

  const TabButton = ({ id, label, icon }: { id: string, label: string, icon: string }) => (
    <button
      onClick={() => setActiveTab(id)}
      className={`flex items-center gap-2 h-full border-b-2 px-5 transition-all ${
        activeTab === id 
          ? 'border-hermes-500 text-surface-100' 
          : 'border-transparent text-surface-400 hover:text-surface-200'
      }`}
    >
      <span className="material-symbols-outlined text-[20px]">{icon}</span>
      <span className="text-sm font-medium uppercase tracking-wide">{label}</span>
    </button>
  );

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
          <TabButton id="factor" label="因子研究 (Alpha)" icon="science" />
          <TabButton id="sentiment" label="舆情情报" icon="psychology" />
          <TabButton id="editor" label="代码编辑" icon="code" />
          <TabButton id="optimization" label="参数寻优" icon="tune" />
        </div>
      </div>

      {/* Content Area */}
      <div className="flex-1 min-h-0 relative">
        {activeTab === 'factor' && <FactorResearch />}
        {activeTab === 'sentiment' && <SentimentAnalysis />}
        {activeTab === 'editor' && <CodeEditor />}
        {activeTab === 'optimization' && <Optimization />}
      </div>
    </div>
  );
};