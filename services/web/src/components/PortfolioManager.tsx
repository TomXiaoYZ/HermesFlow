import React, { useState } from 'react';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, BarChart, Bar, Cell, PieChart, Pie, Legend } from 'recharts';

const Card: React.FC<{ children: React.ReactNode; className?: string }> = ({ children, className }) => (
  <div className={`bg-surface-900 border border-surface-800 rounded-md overflow-hidden ${className}`}>
    {children}
  </div>
);

// Mock Data: Cumulative Performance vs Benchmark
const PERFORMANCE_DATA = [
  { date: 'Jan', portfolio: 0, benchmark: 0 },
  { date: 'Feb', portfolio: 5.2, benchmark: 3.1 },
  { date: 'Mar', portfolio: 4.8, benchmark: 4.5 },
  { date: 'Apr', portfolio: 12.5, benchmark: 6.2 },
  { date: 'May', portfolio: 15.8, benchmark: 5.8 },
  { date: 'Jun', portfolio: 14.2, benchmark: 8.4 },
  { date: 'Jul', portfolio: 22.5, benchmark: 10.1 },
  { date: 'Aug', portfolio: 28.4, benchmark: 12.5 },
  { date: 'Sep', portfolio: 26.1, benchmark: 11.2 },
  { date: 'Oct', portfolio: 35.5, benchmark: 15.8 },
  { date: 'Nov', portfolio: 42.1, benchmark: 18.2 },
  { date: 'Dec', portfolio: 45.8, benchmark: 21.5 },
];

// Mock Data: PnL Attribution by Strategy Category
const ATTRIBUTION_DATA = [
  { name: '趋势跟踪 (Trend)', value: 45200 },
  { name: '均值回归 (Rev)', value: 12500 },
  { name: '套利 (Arb)', value: 8400 },
  { name: '做市 (MM)', value: 3200 },
  { name: '主观交易 (Manual)', value: -5600 },
];

// Mock Data: Asset Allocation
const ALLOCATION_DATA = [
  { name: 'BTC & Altcoins', value: 55, color: '#00E396' },
  { name: 'US Tech Stocks', value: 25, color: '#008FFB' },
  { name: 'Cash (USDT/USD)', value: 15, color: '#FEB019' },
  { name: 'Derivatives', value: 5, color: '#775DD0' },
];

// Mock PnL Calendar Data (Last 30 days)
const PNL_CALENDAR = Array.from({ length: 28 }, (_, i) => {
    const val = (Math.random() - 0.4) * 2000;
    return { day: i + 1, val };
});

export const PortfolioManager: React.FC = () => {
  const [timeRange, setTimeRange] = useState('YTD');

  return (
    <div className="h-full overflow-y-auto p-6 flex flex-col gap-6 bg-surface-950">
       {/* Header with Filters */}
       <div className="flex justify-between items-end">
          <div>
             <h2 className="text-2xl font-bold text-surface-100 mb-1 flex items-center gap-2">
                <span className="material-symbols-outlined text-purple-500">pie_chart</span>
                投资组合分析 (Portfolio Analytics)
             </h2>
             <p className="text-xs text-surface-500">历史业绩归因、资产配置与盈亏日历</p>
          </div>
          <div className="flex bg-surface-900 p-1 rounded border border-surface-800">
             {['1M', '3M', '6M', 'YTD', '1Y', 'ALL'].map(range => (
                <button 
                   key={range}
                   onClick={() => setTimeRange(range)}
                   className={`px-3 py-1 text-xs font-bold rounded transition-colors ${timeRange === range ? 'bg-surface-700 text-surface-100 shadow' : 'text-surface-400 hover:text-surface-200'}`}
                >
                   {range}
                </button>
             ))}
          </div>
       </div>

       {/* Top Stats Cards - Analytical Focus (Not Realtime) */}
       <div className="grid grid-cols-4 gap-4">
          <Card className="p-5 flex flex-col justify-between h-32 relative overflow-hidden group">
             <div className="z-10">
                <div className="text-xs text-surface-400 font-bold uppercase tracking-wider mb-1">累计收益率 (Cum. Return)</div>
                <div className="text-3xl font-mono font-bold text-trade-up">+45.8%</div>
                <div className="text-xs text-surface-500 mt-1">vs Benchmark: <span className="text-surface-300">+21.5%</span></div>
             </div>
             <div className="absolute right-0 bottom-0 opacity-10 group-hover:opacity-20 transition-opacity">
                <span className="material-symbols-outlined text-[80px] text-trade-up">trending_up</span>
             </div>
          </Card>
          <Card className="p-5 flex flex-col justify-between h-32">
             <div>
                <div className="text-xs text-surface-400 font-bold uppercase tracking-wider mb-1">夏普比率 (Sharpe)</div>
                <div className="text-3xl font-mono font-bold text-hermes-500">2.84</div>
                <div className="text-xs text-surface-500 mt-1">Sortino: <span className="text-surface-300">3.12</span></div>
             </div>
          </Card>
          <Card className="p-5 flex flex-col justify-between h-32">
             <div>
                <div className="text-xs text-surface-400 font-bold uppercase tracking-wider mb-1">最大回撤 (Max Drawdown)</div>
                <div className="text-3xl font-mono font-bold text-trade-down">-8.4%</div>
                <div className="text-xs text-surface-500 mt-1">Recovery Days: <span className="text-surface-300">12 Days</span></div>
             </div>
          </Card>
          <Card className="p-5 flex flex-col justify-between h-32">
             <div>
                <div className="text-xs text-surface-400 font-bold uppercase tracking-wider mb-1">胜率 / 盈亏比 (Win Rate)</div>
                <div className="text-3xl font-mono font-bold text-blue-400">58.2%</div>
                <div className="text-xs text-surface-500 mt-1">Profit Factor: <span className="text-surface-300">1.65</span></div>
             </div>
          </Card>
       </div>

       <div className="grid grid-cols-12 gap-6 min-h-[400px]">
          {/* Main Chart: Cumulative Performance */}
          <Card className="col-span-8 p-5 flex flex-col">
             <div className="flex justify-between items-center mb-6">
                <h3 className="text-sm font-bold text-surface-200">累计收益走势 (Equity Curve)</h3>
                <div className="flex items-center gap-4 text-xs">
                   <div className="flex items-center gap-2"><span className="w-3 h-0.5 bg-hermes-500"></span> Portfolio</div>
                   <div className="flex items-center gap-2"><span className="w-3 h-0.5 bg-surface-500 border-dashed border-t border-surface-500"></span> Benchmark (BTC)</div>
                </div>
             </div>
             <div className="flex-1 w-full min-h-0">
                <ResponsiveContainer width="100%" height="100%">
                   <AreaChart data={PERFORMANCE_DATA}>
                      <defs>
                         <linearGradient id="colorEq" x1="0" y1="0" x2="0" y2="1">
                            <stop offset="5%" stopColor="#00E396" stopOpacity={0.1}/>
                            <stop offset="95%" stopColor="#00E396" stopOpacity={0}/>
                         </linearGradient>
                      </defs>
                      <CartesianGrid strokeDasharray="3 3" stroke="#27272a" vertical={false} />
                      <XAxis dataKey="date" tick={{fontSize: 12, fill: '#71717a'}} axisLine={false} tickLine={false} />
                      <YAxis tick={{fontSize: 12, fill: '#71717a'}} axisLine={false} tickLine={false} tickFormatter={(v) => `${v}%`} />
                      <Tooltip 
                         contentStyle={{backgroundColor: '#18181b', borderColor: '#27272a', fontSize: '12px'}}
                         itemStyle={{fontWeight: 'bold'}}
                         formatter={(value: any) => [`${value}%`]}
                      />
                      <Area type="monotone" dataKey="portfolio" name="Portfolio" stroke="#00E396" strokeWidth={3} fill="url(#colorEq)" />
                      <Area type="monotone" dataKey="benchmark" name="Benchmark" stroke="#71717a" strokeWidth={2} strokeDasharray="4 4" fill="none" />
                   </AreaChart>
                </ResponsiveContainer>
             </div>
          </Card>

          {/* Asset Allocation */}
          <Card className="col-span-4 p-5 flex flex-col">
             <h3 className="text-sm font-bold text-surface-200 mb-4">资产分布 (Allocation)</h3>
             <div className="flex-1 min-h-0 relative">
                <ResponsiveContainer width="100%" height="100%">
                   <PieChart>
                      <Pie
                         data={ALLOCATION_DATA}
                         cx="50%"
                         cy="50%"
                         innerRadius={60}
                         outerRadius={80}
                         paddingAngle={5}
                         dataKey="value"
                      >
                         {ALLOCATION_DATA.map((entry, index) => (
                            <Cell key={`cell-${index}`} fill={entry.color} stroke="none" />
                         ))}
                      </Pie>
                      <Tooltip contentStyle={{backgroundColor: '#18181b', borderColor: '#27272a', fontSize: '12px'}} />
                   </PieChart>
                </ResponsiveContainer>
                {/* Center Text */}
                <div className="absolute inset-0 flex flex-col items-center justify-center pointer-events-none">
                   <span className="text-2xl font-bold text-surface-100">$1.8M</span>
                   <span className="text-xs text-surface-500 uppercase">Total AUM</span>
                </div>
             </div>
             <div className="space-y-3 mt-4">
                {ALLOCATION_DATA.map((item, i) => (
                   <div key={i} className="flex justify-between items-center text-xs">
                      <div className="flex items-center gap-2">
                         <div className="w-2 h-2 rounded-full" style={{backgroundColor: item.color}}></div>
                         <span className="text-surface-300">{item.name}</span>
                      </div>
                      <span className="font-bold text-surface-200">{item.value}%</span>
                   </div>
                ))}
             </div>
          </Card>
       </div>

       <div className="grid grid-cols-12 gap-6 h-[350px]">
          {/* PnL Calendar (Heatmap style) */}
          <Card className="col-span-5 p-5 flex flex-col">
             <div className="flex justify-between items-center mb-4">
                <h3 className="text-sm font-bold text-surface-200">盈亏日历 (PnL Calendar)</h3>
                <div className="flex items-center gap-2 text-[10px] text-surface-500">
                   <span className="w-2 h-2 bg-trade-down rounded-sm"></span> Loss
                   <span className="w-2 h-2 bg-trade-up rounded-sm"></span> Profit
                </div>
             </div>
             <div className="flex-1 grid grid-cols-7 gap-2 content-start">
                {['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'].map(d => (
                   <div key={d} className="text-center text-[10px] text-surface-500 font-bold uppercase mb-1">{d}</div>
                ))}
                {/* Empty slots for offset if needed */}
                <div className="aspect-square"></div>
                <div className="aspect-square"></div>
                
                {PNL_CALENDAR.map((day) => (
                   <div 
                      key={day.day} 
                      className={`aspect-square rounded-sm flex items-center justify-center text-[10px] font-mono border border-transparent hover:border-white/20 transition-all cursor-pointer relative group
                         ${day.val > 1000 ? 'bg-trade-up text-surface-950 font-bold' : 
                           day.val > 0 ? 'bg-trade-up/20 text-trade-up' : 
                           day.val < -1000 ? 'bg-trade-down text-white font-bold' : 
                           'bg-trade-down/20 text-trade-down'}
                      `}
                   >
                      {day.day}
                      {/* Tooltip */}
                      <div className="opacity-0 group-hover:opacity-100 absolute bottom-full mb-2 bg-surface-950 border border-surface-700 text-xs px-2 py-1 rounded whitespace-nowrap z-20 pointer-events-none">
                         {day.val > 0 ? '+' : ''}{day.val.toFixed(0)} USD
                      </div>
                   </div>
                ))}
             </div>
          </Card>

          {/* Attribution Chart */}
          <Card className="col-span-7 p-5 flex flex-col">
             <h3 className="text-sm font-bold text-surface-200 mb-4">收益归因分析 (PnL Attribution)</h3>
             <div className="flex-1 min-h-0">
                <ResponsiveContainer width="100%" height="100%">
                   <BarChart data={ATTRIBUTION_DATA} layout="vertical" margin={{left: 20}}>
                      <CartesianGrid strokeDasharray="3 3" stroke="#27272a" horizontal={false} />
                      <XAxis type="number" hide />
                      <YAxis dataKey="name" type="category" tick={{fontSize: 11, fill: '#a1a1aa'}} width={80} axisLine={false} tickLine={false} />
                      <Tooltip 
                         cursor={{fill: '#27272a'}}
                         contentStyle={{backgroundColor: '#18181b', borderColor: '#27272a', fontSize: '12px'}}
                      />
                      <Bar dataKey="value" barSize={20} radius={[0, 4, 4, 0]}>
                         {ATTRIBUTION_DATA.map((entry, index) => (
                            <Cell key={`cell-${index}`} fill={entry.value > 0 ? '#00E396' : '#FF4560'} />
                         ))}
                      </Bar>
                   </BarChart>
                </ResponsiveContainer>
             </div>
          </Card>
       </div>
    </div>
  );
};