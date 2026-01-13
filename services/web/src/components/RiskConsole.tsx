import React, { useState } from 'react';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, BarChart, Bar, Cell } from 'recharts';

const Card: React.FC<{ children: React.ReactNode; className?: string }> = ({ children, className }) => (
  <div className={`bg-surface-900 border border-surface-800 rounded-md ${className}`}>
    {children}
  </div>
);

// Mock Data for Drawdown
const DRAWDOWN_DATA = Array.from({ length: 50 }, (_, i) => ({
  time: i,
  val: -Math.abs(Math.sin(i / 5) * 2 + Math.random()) // Always negative
}));

// Mock Data for Correlation Matrix
const STRATEGY_CORRELATION = [
  { name: 'Alpha', alpha: 1.0, sentiment: 0.2, mean: -0.4, opt: 0.1 },
  { name: 'Sent.', alpha: 0.2, sentiment: 1.0, mean: 0.1, opt: 0.05 },
  { name: 'Mean', alpha: -0.4, sentiment: 0.1, mean: 1.0, opt: -0.1 },
  { name: 'Opt.', alpha: 0.1, sentiment: 0.05, mean: -0.1, opt: 1.0 },
];

export const RiskConsole: React.FC = () => {
  const [killSwitchActive, setKillSwitchActive] = useState(false);

  return (
    <div className="h-full overflow-y-auto p-4 bg-surface-950 flex flex-col gap-4">
      
      {/* 1. Global Safety Header & Kill Switch */}
      <div className="flex justify-between items-center bg-surface-900 p-4 rounded-md border border-surface-800 shadow-sm flex-shrink-0">
         <div className="flex items-center gap-6">
            <h2 className="text-xl font-bold text-surface-100 flex items-center gap-2">
               <span className="material-symbols-outlined text-hermes-500">shield</span> 
               全域风控中心 (Global Risk)
            </h2>
            <div className="h-8 w-px bg-surface-700"></div>
            <div className="flex gap-6">
               <div className="flex flex-col">
                  <span className="text-[10px] text-surface-500 font-bold uppercase tracking-wider">总体 VaR (95%, 1日)</span>
                  <span className="text-sm font-mono text-surface-200 font-bold">$12,450 <span className="text-trade-warn text-[10px]">(1.2%)</span></span>
               </div>
               <div className="flex flex-col">
                  <span className="text-[10px] text-surface-500 font-bold uppercase tracking-wider">总杠杆率 (Gross Lev)</span>
                  <span className="text-sm font-mono text-surface-200 font-bold">1.45x <span className="text-surface-500 text-[10px]">/ 3.0x Cap</span></span>
               </div>
               <div className="flex flex-col">
                  <span className="text-[10px] text-surface-500 font-bold uppercase tracking-wider">保证金占用</span>
                  <span className="text-sm font-mono text-surface-200 font-bold">42.5% <span className="text-hermes-500 text-[10px]">Safe</span></span>
               </div>
            </div>
         </div>

         {/* Emergency Controls */}
         <div className="flex items-center gap-3">
             <div className="flex items-center gap-2 px-3 py-1.5 bg-surface-950 rounded border border-surface-800">
                <div className="w-2 h-2 rounded-full bg-hermes-500 animate-pulse"></div>
                <span className="text-xs font-bold text-surface-300">系统状态: 监控中</span>
             </div>
             <button 
               onClick={() => setKillSwitchActive(!killSwitchActive)}
               className={`px-5 py-2 rounded font-bold text-sm uppercase tracking-wide flex items-center gap-2 transition-all shadow-lg ${killSwitchActive ? 'bg-surface-800 text-surface-400 border border-surface-600' : 'bg-trade-down hover:bg-red-600 text-white shadow-trade-down/30 animate-pulse'}`}
             >
                <span className="material-symbols-outlined icon-filled">power_settings_new</span>
                {killSwitchActive ? '系统已熔断 (HALTED)' : '紧急熔断 (KILL SWITCH)'}
             </button>
         </div>
      </div>

      <div className="grid grid-cols-12 gap-4 flex-1 min-h-0">
         
         {/* LEFT COLUMN: HFT & System Health (High Frequency Focus) */}
         <div className="col-span-5 flex flex-col gap-4">
            <Card className="flex flex-col p-0 overflow-hidden">
               <div className="px-4 py-3 border-b border-surface-800 bg-surface-950/30 flex justify-between items-center">
                  <h3 className="text-sm font-bold text-surface-200 flex items-center gap-2">
                     <span className="material-symbols-outlined text-blue-400">speed</span>
                     高频策略卫士 (HFT Guard)
                  </h3>
                  <span className="text-[10px] bg-blue-500/10 text-blue-400 px-2 py-0.5 rounded border border-blue-500/20 font-bold">舆情系统专用</span>
               </div>
               
               <div className="p-4 grid grid-cols-2 gap-4">
                  {/* Metric 1 */}
                  <div className="bg-surface-950 p-3 rounded border border-surface-800">
                     <div className="flex justify-between mb-1">
                        <span className="text-[10px] text-surface-500 font-bold uppercase">API 延迟 (Latency)</span>
                        <span className="text-[10px] text-hermes-500 font-bold">Excellent</span>
                     </div>
                     <div className="text-xl font-mono text-surface-100 font-bold">12<span className="text-xs text-surface-500 ml-1">ms</span></div>
                     <div className="w-full h-1 bg-surface-800 rounded-full mt-2 overflow-hidden">
                        <div className="h-full bg-hermes-500 w-[15%]"></div>
                     </div>
                  </div>
                   {/* Metric 2 */}
                   <div className="bg-surface-950 p-3 rounded border border-surface-800">
                     <div className="flex justify-between mb-1">
                        <span className="text-[10px] text-surface-500 font-bold uppercase">拒单率 (Rejections)</span>
                        <span className="text-[10px] text-surface-300 font-bold">0/1000</span>
                     </div>
                     <div className="text-xl font-mono text-surface-100 font-bold">0.00<span className="text-xs text-surface-500 ml-1">%</span></div>
                     <div className="w-full h-1 bg-surface-800 rounded-full mt-2 overflow-hidden">
                        <div className="h-full bg-trade-up w-[0%]"></div>
                     </div>
                  </div>
                  {/* Metric 3 */}
                   <div className="bg-surface-950 p-3 rounded border border-surface-800">
                     <div className="flex justify-between mb-1">
                        <span className="text-[10px] text-surface-500 font-bold uppercase">订单流速 (OPS)</span>
                        <span className="text-[10px] text-trade-warn font-bold">Peak: 45</span>
                     </div>
                     <div className="text-xl font-mono text-surface-100 font-bold">8.5<span className="text-xs text-surface-500 ml-1">/sec</span></div>
                     <div className="w-full h-1 bg-surface-800 rounded-full mt-2 overflow-hidden">
                        <div className="h-full bg-trade-warn w-[30%]"></div>
                     </div>
                  </div>
                  {/* Metric 4 */}
                   <div className="bg-surface-950 p-3 rounded border border-surface-800 relative overflow-hidden">
                     <div className="absolute right-0 top-0 p-1">
                        <span className="material-symbols-outlined text-surface-700 text-[32px]">warning_amber</span>
                     </div>
                     <div className="flex justify-between mb-1 relative z-10">
                        <span className="text-[10px] text-surface-500 font-bold uppercase">滑点监控 (Slippage)</span>
                     </div>
                     <div className="text-xl font-mono text-surface-100 font-bold relative z-10">1.2<span className="text-xs text-surface-500 ml-1">bp</span></div>
                     <div className="text-[10px] text-surface-400 mt-1 relative z-10">正常范围 (&lt;5bp)</div>
                  </div>
               </div>

               {/* Active Alerts Log */}
               <div className="flex-1 border-t border-surface-800 flex flex-col min-h-0">
                  <div className="px-4 py-2 text-[10px] font-bold text-surface-500 uppercase bg-surface-950">实时风控日志 (Risk Log)</div>
                  <div className="flex-1 overflow-y-auto p-2 font-mono text-xs space-y-1">
                     <div className="flex gap-2 text-surface-400"><span className="text-surface-600">[14:32:01]</span> <span className="text-hermes-500">INFO</span> Order #8821 passed pre-trade check. Size: 0.5 BTC.</div>
                     <div className="flex gap-2 text-surface-400"><span className="text-surface-600">[14:31:55]</span> <span className="text-trade-warn">WARN</span> High volatility detected on SOL-USDT. Reducing size multiplier to 0.8x.</div>
                     <div className="flex gap-2 text-surface-400"><span className="text-surface-600">[14:30:12]</span> <span className="text-hermes-500">INFO</span> Strategy 'Sentiment-v2' active. OPS stable.</div>
                     <div className="flex gap-2 text-surface-400"><span className="text-surface-600">[14:28:45]</span> <span className="text-trade-down">BLOCK</span> Order #8819 rejected. Exceeds max notional ($50k).</div>
                     <div className="flex gap-2 text-surface-400"><span className="text-surface-600">[14:28:45]</span> <span className="text-hermes-500">INFO</span> Position Check: BTC Exposure 18% (Limit 20%).</div>
                  </div>
               </div>
            </Card>

            {/* Systematic Rules Status */}
            <Card className="p-4">
               <h3 className="text-sm font-bold text-surface-200 mb-3 flex items-center gap-2">
                  <span className="material-symbols-outlined text-surface-400">checklist</span> 系统风控规则状态
               </h3>
               <div className="space-y-2">
                  <div className="flex justify-between items-center p-2 bg-surface-950 border border-surface-800 rounded">
                     <span className="text-xs text-surface-300">单笔最大下单金额 (Max Order)</span>
                     <span className="text-xs font-mono font-bold text-surface-200">$50,000 <span className="text-hermes-500 ml-2">ACTIVE</span></span>
                  </div>
                  <div className="flex justify-between items-center p-2 bg-surface-950 border border-surface-800 rounded">
                     <span className="text-xs text-surface-300">日内最大亏损停机 (Max Daily Loss)</span>
                     <span className="text-xs font-mono font-bold text-surface-200">-3.0% <span className="text-hermes-500 ml-2">ACTIVE</span></span>
                  </div>
                  <div className="flex justify-between items-center p-2 bg-surface-950 border border-surface-800 rounded">
                     <span className="text-xs text-surface-300">胖手指检测 (Fat Finger)</span>
                     <span className="text-xs font-mono font-bold text-surface-200">Price Deviation &gt; 5% <span className="text-hermes-500 ml-2">ACTIVE</span></span>
                  </div>
               </div>
            </Card>
         </div>

         {/* RIGHT COLUMN: Mid/Low Freq Portfolio Risk */}
         <div className="col-span-7 flex flex-col gap-4">
            <div className="grid grid-cols-2 gap-4 h-64">
               {/* Drawdown Monitor */}
               <Card className="flex flex-col p-4">
                  <div className="flex justify-between items-center mb-2">
                     <h3 className="text-xs font-bold text-surface-400 uppercase tracking-wide">最大回撤监控 (Max Drawdown)</h3>
                     <span className="text-xs font-bold text-trade-down">-1.24% <span className="text-surface-500 font-normal">/ Limit -5.0%</span></span>
                  </div>
                  <div className="flex-1 min-h-0">
                     <ResponsiveContainer width="100%" height="100%">
                        <AreaChart data={DRAWDOWN_DATA}>
                           <defs>
                              <linearGradient id="colorDd" x1="0" y1="0" x2="0" y2="1">
                                 <stop offset="5%" stopColor="#FF4560" stopOpacity={0.2}/>
                                 <stop offset="95%" stopColor="#FF4560" stopOpacity={0}/>
                              </linearGradient>
                           </defs>
                           <CartesianGrid strokeDasharray="3 3" stroke="#27272a" vertical={false} />
                           <XAxis dataKey="time" hide />
                           <YAxis hide domain={[-5, 0]} />
                           <Tooltip contentStyle={{backgroundColor: '#18181b', borderColor: '#27272a', fontSize: '12px'}} />
                           <Area type="monotone" dataKey="val" stroke="#FF4560" fill="url(#colorDd)" strokeWidth={2} />
                        </AreaChart>
                     </ResponsiveContainer>
                  </div>
               </Card>

               {/* Correlation Matrix */}
               <Card className="flex flex-col p-4">
                  <div className="flex justify-between items-center mb-4">
                     <h3 className="text-xs font-bold text-surface-400 uppercase tracking-wide">策略相关性矩阵 (Correlation)</h3>
                     <span className="text-[10px] text-surface-500">Diversification Check</span>
                  </div>
                  <div className="flex-1 grid grid-cols-5 gap-1 text-[10px] font-mono">
                     <div className="col-span-1"></div>
                     {STRATEGY_CORRELATION.map(s => <div key={s.name} className="flex items-end justify-center text-surface-500 pb-1">{s.name}</div>)}
                     
                     {STRATEGY_CORRELATION.map((row, rIdx) => (
                        <React.Fragment key={rIdx}>
                           <div className="flex items-center justify-end pr-2 text-surface-500">{row.name}</div>
                           {[row.alpha, row.sentiment, row.mean, row.opt].map((val, cIdx) => (
                              <div 
                                 key={cIdx} 
                                 className={`rounded flex items-center justify-center font-bold border border-transparent hover:border-white/20 transition-all cursor-default
                                    ${val === 1 ? 'bg-surface-800 text-surface-600' : 
                                      val > 0.5 ? 'bg-trade-down/80 text-white' : 
                                      val > 0.2 ? 'bg-trade-down/40 text-surface-200' :
                                      val < -0.2 ? 'bg-blue-500/40 text-surface-200' : 
                                      'bg-surface-800 text-surface-400'}
                                 `}
                              >
                                 {val === 1 ? '-' : val.toFixed(1)}
                              </div>
                           ))}
                        </React.Fragment>
                     ))}
                  </div>
               </Card>
            </div>

            {/* Exposure & Concentration */}
            <Card className="flex-1 p-0 flex flex-col">
               <div className="px-4 py-3 border-b border-surface-800 bg-surface-950/30">
                  <h3 className="text-sm font-bold text-surface-200 flex items-center gap-2">
                     <span className="material-symbols-outlined text-purple-400">pie_chart</span>
                     敞口集中度 (Concentration Risk)
                  </h3>
               </div>
               <div className="flex-1 p-4 grid grid-cols-2 gap-8">
                  <div className="flex flex-col gap-3">
                     <div className="flex justify-between text-xs font-bold text-surface-400 uppercase">
                        <span>按资产 (By Asset)</span>
                        <span>Limit: 25%</span>
                     </div>
                     <div className="space-y-3">
                        <div>
                           <div className="flex justify-between text-xs mb-1"><span className="text-surface-200">BTC + BTC-Perp</span> <span className="text-trade-warn">22.4%</span></div>
                           <div className="w-full bg-surface-950 h-2 rounded-full overflow-hidden"><div className="h-full bg-trade-warn w-[22.4%]"></div></div>
                        </div>
                        <div>
                           <div className="flex justify-between text-xs mb-1"><span className="text-surface-200">ETH + ETH-Perp</span> <span className="text-hermes-500">14.2%</span></div>
                           <div className="w-full bg-surface-950 h-2 rounded-full overflow-hidden"><div className="h-full bg-hermes-500 w-[14.2%]"></div></div>
                        </div>
                        <div>
                           <div className="flex justify-between text-xs mb-1"><span className="text-surface-200">NVIDIA (NVDA)</span> <span className="text-hermes-500">8.5%</span></div>
                           <div className="w-full bg-surface-950 h-2 rounded-full overflow-hidden"><div className="h-full bg-hermes-500 w-[8.5%]"></div></div>
                        </div>
                        <div>
                           <div className="flex justify-between text-xs mb-1"><span className="text-surface-200">USDT (Cash)</span> <span className="text-surface-400">35.0%</span></div>
                           <div className="w-full bg-surface-950 h-2 rounded-full overflow-hidden"><div className="h-full bg-surface-600 w-[35%]"></div></div>
                        </div>
                     </div>
                  </div>

                  <div className="flex flex-col gap-3">
                     <div className="flex justify-between text-xs font-bold text-surface-400 uppercase">
                        <span>按策略类型 (By Strategy)</span>
                        <span>Limit: 50%</span>
                     </div>
                     <div className="space-y-3">
                        <div>
                           <div className="flex justify-between text-xs mb-1"><span className="text-surface-200">Trend Following</span> <span className="text-hermes-500">35%</span></div>
                           <div className="w-full bg-surface-950 h-2 rounded-full overflow-hidden"><div className="h-full bg-blue-500 w-[35%]"></div></div>
                        </div>
                        <div>
                           <div className="flex justify-between text-xs mb-1"><span className="text-surface-200">Sentiment HFT</span> <span className="text-hermes-500">20%</span></div>
                           <div className="w-full bg-surface-950 h-2 rounded-full overflow-hidden"><div className="h-full bg-purple-500 w-[20%]"></div></div>
                        </div>
                        <div>
                           <div className="flex justify-between text-xs mb-1"><span className="text-surface-200">Statistical Arb</span> <span className="text-hermes-500">15%</span></div>
                           <div className="w-full bg-surface-950 h-2 rounded-full overflow-hidden"><div className="h-full bg-orange-500 w-[15%]"></div></div>
                        </div>
                     </div>
                  </div>
               </div>
            </Card>
         </div>
      </div>
    </div>
  );
};