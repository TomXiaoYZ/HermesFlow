import React from 'react';
import { EQUITY_DATA, ACTIVE_STRATEGIES, LIVE_EXECUTIONS } from '../constants';
import { AreaChart, Area, XAxis, Tooltip, ResponsiveContainer, PieChart, Pie, Cell } from 'recharts';

const Card: React.FC<{ children: React.ReactNode; className?: string }> = ({ children, className }) => (
  <div className={`bg-surface-900 border border-surface-800 rounded-md shadow-sm ${className}`}>
    {children}
  </div>
);

const SectionHeader: React.FC<{ title: string; action?: React.ReactNode; icon?: string }> = ({ title, action, icon }) => (
  <div className="px-4 py-3 border-b border-surface-800 flex justify-between items-center bg-surface-950/30">
    <h3 className="text-sm font-bold text-surface-200 uppercase tracking-wide flex items-center gap-2">
      {icon && <span className="material-symbols-outlined text-[18px] text-surface-400">{icon}</span>}
      {title}
    </h3>
    {action}
  </div>
);

export const Dashboard: React.FC = () => {
  return (
    <div className="h-full overflow-hidden flex flex-col p-4 gap-4 bg-surface-950">
      
      {/* Top Row: KPIs (Increased Size) */}
      <div className="grid grid-cols-5 gap-4 h-28 flex-shrink-0">
         {[
           { label: '账户净值 (NAV)', val: '$1,240,500', sub: 'USD', change: '+1.24%', trend: 'up' },
           { label: '今日盈亏 (PnL)', val: '+$12,400', sub: 'USD', change: '+1.02%', trend: 'up' },
           { label: '可用购买力', val: '$682,275', sub: 'USD', change: '55% 使用率', trend: 'neutral' },
           { label: '总敞口 (Exposure)', val: '$1.85M', sub: '名义价值', change: '1.5x 杠杆', trend: 'warn' },
           { label: '夏普比率 (YTD)', val: '2.45', sub: 'Ratio', change: '前 5%', trend: 'up' },
         ].map((kpi, i) => (
           <Card key={i} className="px-5 py-3 flex flex-col justify-center relative overflow-hidden hover:border-surface-600 transition-colors group">
              <div className="flex justify-between items-start mb-2">
                 <span className="text-xs text-surface-400 font-bold uppercase tracking-wider group-hover:text-surface-200 transition-colors">{kpi.label}</span>
                 <span className={`text-xs font-bold px-1.5 py-0.5 rounded ${kpi.trend === 'up' ? 'text-trade-up bg-trade-up/10' : kpi.trend === 'down' ? 'text-trade-down bg-trade-down/10' : 'text-trade-warn bg-trade-warn/10'}`}>{kpi.change}</span>
              </div>
              <div className="flex items-baseline gap-2 mt-1">
                 <span className={`text-2xl font-bold font-mono tracking-tight ${kpi.trend === 'up' ? 'text-surface-100' : 'text-surface-100'}`}>{kpi.val}</span>
                 <span className="text-xs text-surface-500 font-medium">{kpi.sub}</span>
              </div>
              {/* Background Graph Simulation */}
              <div className="absolute right-0 bottom-0 opacity-5 group-hover:opacity-10 transition-opacity">
                 <span className="material-symbols-outlined text-[64px]">show_chart</span>
              </div>
           </Card>
         ))}
      </div>

      {/* Middle Row: Equity & Allocation */}
      <div className="grid grid-cols-12 gap-4 flex-[2] min-h-0">
        {/* Equity Chart */}
        <Card className="col-span-8 flex flex-col">
          <SectionHeader 
            title="账户权益曲线 (Equity)" 
            icon="monitoring"
            action={
              <div className="flex gap-1 bg-surface-950 p-1 rounded border border-surface-800">
                {['1日', '1周', '1月', '3月', '今年'].map((period, i) => (
                  <button key={period} className={`text-xs px-3 py-1 rounded transition font-medium ${i === 2 ? 'bg-surface-700 text-surface-100 shadow-sm' : 'text-surface-400 hover:text-surface-200 hover:bg-surface-800'}`}>
                    {period}
                  </button>
                ))}
              </div>
            }
          />
          <div className="flex-1 w-full min-h-0 p-2 bg-surface-950/20">
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={EQUITY_DATA} margin={{ top: 10, right: 0, left: 0, bottom: 0 }}>
                <defs>
                  <linearGradient id="colorValue" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="#00E396" stopOpacity={0.1}/>
                    <stop offset="95%" stopColor="#00E396" stopOpacity={0}/>
                  </linearGradient>
                </defs>
                <Tooltip 
                  contentStyle={{backgroundColor: '#18181b', borderColor: '#27272a', color: '#e4e4e7', fontSize: '12px', padding: '8px 12px', borderRadius: '4px'}}
                  itemStyle={{color: '#00E396', fontWeight: 'bold'}}
                  cursor={{stroke: '#27272a', strokeWidth: 1}}
                />
                <XAxis dataKey="time" hide />
                <Area type="monotone" dataKey="value" stroke="#00E396" strokeWidth={2} fillOpacity={1} fill="url(#colorValue)" isAnimationActive={false} />
                <Area type="monotone" dataKey="benchmark" stroke="#3f3f46" strokeWidth={2} strokeDasharray="4 4" fill="none" isAnimationActive={false} />
              </AreaChart>
            </ResponsiveContainer>
          </div>
          <div className="h-10 border-t border-surface-800 flex items-center px-4 gap-6 bg-surface-950/50">
             <div className="flex items-center gap-2 text-xs font-medium text-surface-300">
                <div className="w-3 h-0.5 bg-hermes-500"></div> 投资组合
             </div>
             <div className="flex items-center gap-2 text-xs font-medium text-surface-400">
                <div className="w-3 h-0.5 bg-surface-500"></div> 基准 (BTC)
             </div>
          </div>
        </Card>

        {/* Allocation Compact */}
        <Card className="col-span-4 flex flex-col">
          <SectionHeader title="资产配置 (Allocation)" icon="pie_chart" />
          <div className="flex-1 flex flex-col p-2">
             <div className="flex-1 relative min-h-[160px]">
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie
                      data={[
                        { name: 'Crypto', value: 45 },
                        { name: 'Stocks', value: 30 },
                        { name: 'Options', value: 25 },
                      ]}
                      cx="50%"
                      cy="50%"
                      innerRadius={60}
                      outerRadius={80}
                      paddingAngle={4}
                      dataKey="value"
                      stroke="none"
                    >
                      <Cell fill="#00E396" />
                      <Cell fill="#3b82f6" />
                      <Cell fill="#8b5cf6" />
                    </Pie>
                  </PieChart>
                </ResponsiveContainer>
                <div className="absolute inset-0 flex flex-col items-center justify-center pointer-events-none">
                   <span className="text-surface-100 font-bold font-mono text-2xl">100%</span>
                   <span className="text-xs text-surface-400 font-medium uppercase tracking-wide">总敞口</span>
                </div>
             </div>
             <div className="px-4 pb-4 grid grid-cols-3 gap-3">
                {[
                   { label: '数字货币', val: '45%', color: 'bg-hermes-500' },
                   { label: '股票', val: '30%', color: 'bg-blue-500' },
                   { label: '期权衍生品', val: '25%', color: 'bg-purple-500' },
                ].map((item, i) => (
                   <div key={i} className="bg-surface-950 border border-surface-800 rounded-md p-2 text-center hover:border-surface-600 transition-colors">
                      <div className={`w-4 h-1 ${item.color} mx-auto mb-1.5 rounded-full`}></div>
                      <div className="text-xs text-surface-400 font-medium">{item.label}</div>
                      <div className="text-sm font-bold text-surface-100 mt-0.5">{item.val}</div>
                   </div>
                ))}
             </div>
          </div>
        </Card>
      </div>

      {/* Bottom Row: Strategies & Live Feed */}
      <div className="grid grid-cols-12 gap-4 flex-[3] min-h-0">
        
        {/* Active Strategies */}
        <Card className="col-span-7 flex flex-col">
          <SectionHeader 
             title="实盘策略监控 (Active Strategies)" 
             icon="memory"
             action={
                <div className="flex gap-3 text-xs font-mono text-surface-400 bg-surface-950 px-2 py-1 rounded border border-surface-800">
                   <span>总盈亏: <span className="text-trade-up font-bold">+$56,800</span></span>
                   <span className="text-surface-700">|</span>
                   <span>运行中: <span className="text-surface-200 font-bold">4</span></span>
                </div>
             }
          />
          <div className="flex-1 overflow-auto bg-surface-950/20">
            <table className="w-full text-left text-xs">
              <thead className="bg-surface-950 text-surface-500 font-semibold border-b border-surface-800 sticky top-0 z-10">
                <tr>
                  <th className="px-4 py-2.5">策略名称</th>
                  <th className="px-4 py-2.5">标的</th>
                  <th className="px-4 py-2.5 text-center">状态</th>
                  <th className="px-4 py-2.5 text-right">杠杆</th>
                  <th className="px-4 py-2.5 text-right">日盈亏</th>
                  <th className="px-4 py-2.5 text-right">夏普</th>
                  <th className="px-4 py-2.5 text-right">操作</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-surface-800 font-mono text-surface-300">
                {ACTIVE_STRATEGIES.map((strategy) => (
                  <tr key={strategy.id} className="hover:bg-surface-800 transition-colors group">
                    <td className="px-4 py-2.5 font-bold text-surface-200 font-sans border-l-2 border-transparent hover:border-hermes-500 transition-all text-sm">{strategy.name}</td>
                    <td className="px-4 py-2.5 text-surface-400 font-medium">{strategy.asset}</td>
                    <td className="px-4 py-2.5 text-center">
                      <span className={`inline-flex items-center gap-1.5 px-2 py-1 rounded text-[10px] uppercase font-bold tracking-wider ${strategy.status === '运行中' ? 'bg-hermes-500/10 text-hermes-500' : 'bg-surface-700 text-surface-400'}`}>
                        {strategy.status === '运行中' && <span className="w-1.5 h-1.5 rounded-full bg-hermes-500 animate-pulse"></span>}
                        {strategy.status}
                      </span>
                    </td>
                    <td className="px-4 py-2.5 text-right text-surface-400">{strategy.leverage}</td>
                    <td className={`px-4 py-2.5 text-right font-bold text-sm ${strategy.dailyPnL >= 0 ? 'text-trade-up' : 'text-trade-down'}`}>
                      {strategy.dailyPnL >= 0 ? '+' : ''}{strategy.dailyPnL.toLocaleString()}
                    </td>
                    <td className="px-4 py-2.5 text-right">{strategy.sharpe}</td>
                    <td className="px-4 py-2.5 text-right">
                       <button className="text-surface-500 hover:text-hermes-500 p-1 rounded hover:bg-surface-700 transition"><span className="material-symbols-outlined text-[16px]">tune</span></button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>

        {/* LIVE EXECUTION FEEDS (New Module) */}
        <Card className="col-span-5 flex flex-col">
           <SectionHeader 
             title="实时成交 (Live Executions)" 
             icon="terminal"
             action={<span className="flex items-center gap-1.5 text-[10px] text-hermes-500 font-bold uppercase tracking-wider bg-hermes-500/10 px-2 py-1 rounded border border-hermes-500/20"><span className="w-2 h-2 rounded-full bg-hermes-500 animate-pulse"></span> 实时</span>}
           />
           <div className="flex-1 overflow-auto flex flex-col bg-[#0c0c0e]">
              <div className="grid grid-cols-12 gap-2 px-4 py-2 border-b border-surface-800 text-[11px] font-bold text-surface-500 bg-surface-900 sticky top-0 z-10 uppercase tracking-wide">
                 <div className="col-span-2">时间</div>
                 <div className="col-span-3">代码</div>
                 <div className="col-span-1">方向</div>
                 <div className="col-span-2 text-right">价格</div>
                 <div className="col-span-2 text-right">数量</div>
                 <div className="col-span-2 text-right">交易所</div>
              </div>
              <div className="flex-1 font-mono text-xs divide-y divide-surface-800/50">
                 {LIVE_EXECUTIONS.map((exec) => (
                    <div key={exec.id} className="grid grid-cols-12 gap-2 px-4 py-1.5 hover:bg-surface-800/50 cursor-pointer group transition-colors items-center">
                       <div className="col-span-2 text-surface-400 text-[11px]">{exec.time}</div>
                       <div className="col-span-3 font-bold text-surface-200 flex items-center gap-1.5">
                          {exec.ticker}
                          {exec.ticker.includes('BTC') && <span className="text-[9px] bg-surface-800 px-1 rounded text-surface-400 font-normal">永续</span>}
                       </div>
                       <div className={`col-span-1 font-bold text-[11px] ${exec.side === '买入' ? 'text-trade-up' : 'text-trade-down'}`}>{exec.side}</div>
                       <div className="col-span-2 text-right text-surface-200 font-medium">{exec.price.toLocaleString()}</div>
                       <div className="col-span-2 text-right text-surface-400">{exec.size}</div>
                       <div className="col-span-2 text-right text-surface-500 text-[10px] group-hover:text-hermes-500 transition-colors">{exec.venue}</div>
                    </div>
                 ))}
                 {/* Fill empty space lines to look like a terminal */}
                 {Array.from({length: 5}).map((_, i) => (
                    <div key={`empty-${i}`} className="grid grid-cols-12 gap-2 px-4 py-1.5 opacity-20">
                       <div className="col-span-2 text-surface-700 font-mono">--:--:--</div>
                       <div className="col-span-10 text-surface-800 border-b border-surface-700/50 border-dashed relative top-[-4px]"></div>
                    </div>
                 ))}
              </div>
           </div>
           {/* Terminal Status Bar */}
           <div className="h-8 bg-surface-900 border-t border-surface-800 flex items-center px-4 justify-between text-[10px] font-mono text-surface-400">
              <div className="flex gap-4">
                 <span className="flex items-center gap-1"><span className="w-1.5 h-1.5 bg-hermes-500 rounded-full"></span> 延迟: <span className="text-surface-200">12ms</span></span>
                 <span className="flex items-center gap-1"><span className="w-1.5 h-1.5 bg-hermes-500 rounded-full"></span> FIX引擎: <span className="text-surface-200">已连接</span></span>
              </div>
              <div className="uppercase font-bold tracking-wider">已处理: <span className="text-surface-200">8,421</span> 条消息</div>
           </div>
        </Card>
      </div>
    </div>
  );
};