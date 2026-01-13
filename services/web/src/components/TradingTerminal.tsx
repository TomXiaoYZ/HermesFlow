import React, { useState } from 'react';
import { ORDER_BOOK_ASKS, ORDER_BOOK_BIDS } from '../constants';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

const chartData = Array.from({ length: 100 }, (_, i) => ({
  time: i,
  price: 64000 + Math.random() * 500 - 250 + (i * 2),
}));

const EXCHANGES = ['Binance', 'OKX', 'Bybit', 'Coinbase Pro', 'Kraken'];
const SYMBOLS = ['BTC/USDT', 'ETH/USDT', 'SOL/USDT', 'BNB/USDT', 'DOGE/USDT'];

export const TradingTerminal: React.FC = () => {
  const [orderSide, setOrderSide] = useState<'buy' | 'sell'>('buy');
  const [exchange, setExchange] = useState('Binance');
  const [symbol, setSymbol] = useState('BTC/USDT');

  const baseAsset = symbol.split('/')[0];
  const quoteAsset = symbol.split('/')[1];

  return (
    <div className="h-full flex gap-1 p-2 overflow-hidden bg-surface-950">
      
      {/* Left Panel: Charts & Position */}
      <div className="flex-1 flex flex-col gap-1 min-w-0">
         {/* Top Info Bar */}
         <div className="bg-surface-900 border border-surface-800 p-2.5 flex items-center justify-between rounded-sm">
            <div className="flex items-center gap-4">
               {/* Selectors */}
               <div className="flex items-center gap-3">
                  <div className="flex items-center bg-surface-950 border border-surface-700 hover:border-hermes-500 transition-colors rounded px-2 relative group">
                     <span className="material-symbols-outlined text-surface-500 text-[16px] mr-2 group-hover:text-hermes-500 transition-colors">hub</span>
                     <select 
                        value={exchange}
                        onChange={(e) => setExchange(e.target.value)}
                        className="bg-transparent text-sm font-bold text-surface-200 outline-none appearance-none pr-6 py-1 cursor-pointer w-[100px]"
                     >
                        {EXCHANGES.map(e => <option key={e} value={e}>{e}</option>)}
                     </select>
                     <span className="absolute right-1 top-1/2 -translate-y-1/2 pointer-events-none material-symbols-outlined text-[16px] text-surface-500">arrow_drop_down</span>
                  </div>

                  <span className="h-4 w-px bg-surface-700"></span>

                  <div className="flex items-center bg-surface-950 border border-surface-700 hover:border-hermes-500 transition-colors rounded px-2 relative group">
                     <select 
                        value={symbol}
                        onChange={(e) => setSymbol(e.target.value)}
                        className="bg-transparent text-lg font-bold text-surface-100 outline-none appearance-none pr-6 py-0.5 cursor-pointer w-[140px]"
                     >
                        {SYMBOLS.map(s => <option key={s} value={s}>{s}</option>)}
                     </select>
                     <span className="absolute right-1 top-1/2 -translate-y-1/2 pointer-events-none material-symbols-outlined text-[16px] text-surface-500">arrow_drop_down</span>
                  </div>
                  
                  <span className="text-xs bg-surface-800 text-surface-400 px-1.5 py-0.5 rounded border border-surface-700 font-medium">永续合约</span>
               </div>

               <div className="h-6 w-px bg-surface-700"></div>
               <div className="flex gap-4 text-sm font-mono">
                  <span className="text-trade-up font-bold text-lg">64,235.50</span>
                  <span className="text-surface-400 flex items-center">标记价格: <span className="text-surface-200 ml-1">64,236.00</span></span>
                  <span className="text-surface-400 flex items-center">资金费率: <span className="text-trade-warn ml-1">0.0100%</span></span>
               </div>
            </div>
            <div className="flex gap-1 bg-surface-950 p-1 rounded border border-surface-800">
               {['1分','5分','15分','1时','4时','1日'].map(t => (
                  <button key={t} className="text-xs px-2.5 py-1 hover:bg-surface-700 text-surface-400 hover:text-surface-100 rounded transition font-medium">{t}</button>
               ))}
            </div>
         </div>

         {/* Chart Area */}
         <div className="flex-1 bg-surface-900 border border-surface-800 relative rounded-sm min-h-0">
            <ResponsiveContainer width="100%" height="100%">
               <AreaChart data={chartData}>
                  <defs>
                  <linearGradient id="colorPrice" x1="0" y1="0" x2="0" y2="1">
                     <stop offset="5%" stopColor="#00E396" stopOpacity={0.1}/>
                     <stop offset="95%" stopColor="#00E396" stopOpacity={0}/>
                  </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="#27272a" vertical={false} />
                  <XAxis dataKey="time" hide />
                  <YAxis domain={['auto', 'auto']} orientation="right" tick={{fontSize: 11, fill: '#71717a', fontWeight: 500}} axisLine={false} tickLine={false} width={50} />
                  <Tooltip 
                     contentStyle={{backgroundColor: '#18181b', borderColor: '#27272a', fontSize: '12px'}}
                     itemStyle={{color: '#00E396'}}
                  />
                  <Area type="monotone" dataKey="price" stroke="#00E396" strokeWidth={2} fillOpacity={1} fill="url(#colorPrice)" isAnimationActive={false} />
               </AreaChart>
            </ResponsiveContainer>
            <div className="absolute top-4 left-4 flex flex-col pointer-events-none">
               <span className="text-xs text-surface-500 font-bold">{exchange} Data Feed</span>
               <span className="text-sm text-surface-300 font-mono">Volume: 1.25B</span>
            </div>
         </div>

         {/* Positions Panel */}
         <div className="h-64 bg-surface-900 border border-surface-800 rounded-sm flex flex-col">
            <div className="flex border-b border-surface-800">
               {['当前持仓 (4)', '当前挂单 (2)', '历史委托', '成交记录'].map((tab, i) => (
                  <button key={tab} className={`px-5 py-2 text-xs font-bold uppercase tracking-wide border-r border-surface-800 hover:bg-surface-800 transition-colors ${i === 0 ? 'bg-surface-800 text-hermes-500 border-b-2 border-b-hermes-500' : 'text-surface-400'}`}>
                     {tab}
                  </button>
               ))}
            </div>
            <div className="flex-1 overflow-auto bg-surface-950/30">
               <table className="w-full text-left text-xs font-mono">
                  <thead className="bg-surface-950 text-surface-500 sticky top-0 font-sans border-b border-surface-800">
                  <tr>
                     <th className="px-4 py-2 font-medium">合约</th>
                     <th className="px-4 py-2 font-medium text-right">持仓数量</th>
                     <th className="px-4 py-2 font-medium text-right">开仓均价</th>
                     <th className="px-4 py-2 font-medium text-right">标记价格</th>
                     <th className="px-4 py-2 font-medium text-right">强平价格</th>
                     <th className="px-4 py-2 font-medium text-right">未实现盈亏 (ROE%)</th>
                  </tr>
                  </thead>
                  <tbody className="divide-y divide-surface-800/50">
                  <tr className="hover:bg-surface-800/50 transition-colors">
                     <td className="px-4 py-2 text-trade-up font-bold text-sm">BTCUSDT</td>
                     <td className="px-4 py-2 text-right text-surface-200">0.520</td>
                     <td className="px-4 py-2 text-right text-surface-400">62,150.0</td>
                     <td className="px-4 py-2 text-right text-surface-200">64,235.5</td>
                     <td className="px-4 py-2 text-right text-trade-warn">58,400.0</td>
                     <td className="px-4 py-2 text-right text-trade-up font-bold">+1,084.46 (+3.2%)</td>
                  </tr>
                  <tr className="hover:bg-surface-800/50 transition-colors">
                     <td className="px-4 py-2 text-trade-up font-bold text-sm">ETH-29SEP</td>
                     <td className="px-4 py-2 text-right text-surface-200">10.000</td>
                     <td className="px-4 py-2 text-right text-surface-400">145.00</td>
                     <td className="px-4 py-2 text-right text-surface-200">120.50</td>
                     <td className="px-4 py-2 text-right text-trade-warn">85.00</td>
                     <td className="px-4 py-2 text-right text-trade-down font-bold">-245.00 (-12.4%)</td>
                  </tr>
                  </tbody>
               </table>
            </div>
         </div>
      </div>

      {/* Right Panel: Order Book & Entry */}
      <div className="w-[340px] flex flex-col gap-1 min-w-[340px]">
         {/* Order Book */}
         <div className="flex-1 bg-surface-900 border border-surface-800 rounded-sm flex flex-col min-h-0">
            <div className="px-3 py-2 border-b border-surface-800 flex justify-between items-center bg-surface-900">
               <span className="text-sm font-bold text-surface-200">订单簿 (Order Book)</span>
               <div className="flex gap-1">
                  <span className="material-symbols-outlined text-[18px] text-surface-500 cursor-pointer hover:text-surface-200">more_vert</span>
               </div>
            </div>
            <div className="grid grid-cols-3 px-3 py-1.5 text-[11px] text-surface-500 uppercase font-bold tracking-wider bg-surface-950/50">
               <span>价格</span>
               <span className="text-right">数量</span>
               <span className="text-right">累计</span>
            </div>
            
            <div className="flex-1 overflow-hidden flex flex-col font-mono text-xs">
               <div className="flex-1 overflow-hidden flex flex-col-reverse justify-end pb-1">
                  {ORDER_BOOK_ASKS.map((ask, i) => (
                  <div key={i} className="grid grid-cols-3 px-3 py-[2px] hover:bg-surface-800 cursor-pointer relative group items-center">
                     <span className="text-trade-down relative z-10 font-medium">{ask.price.toFixed(1)}</span>
                     <span className="text-right text-surface-300 relative z-10">{ask.size.toFixed(3)}</span>
                     <span className="text-right text-surface-500 relative z-10">{ask.total.toFixed(1)}</span>
                     <div className="absolute right-0 top-0 bottom-0 bg-trade-down/10 transition-all duration-300" style={{width: `${Math.random() * 80}%`}}></div>
                  </div>
                  ))}
               </div>
               
               <div className="py-2.5 border-y border-surface-800 bg-surface-950 flex items-center justify-center gap-2 shadow-inner">
                  <span className="text-xl font-bold text-trade-up tracking-tight">64,235.50</span>
                  <span className="material-symbols-outlined text-[16px] text-trade-up">arrow_upward</span>
               </div>

               <div className="flex-1 overflow-hidden pt-1">
                  {ORDER_BOOK_BIDS.map((bid, i) => (
                  <div key={i} className="grid grid-cols-3 px-3 py-[2px] hover:bg-surface-800 cursor-pointer relative group items-center">
                     <span className="text-trade-up relative z-10 font-medium">{bid.price.toFixed(1)}</span>
                     <span className="text-right text-surface-300 relative z-10">{bid.size.toFixed(3)}</span>
                     <span className="text-right text-surface-500 relative z-10">{bid.total.toFixed(1)}</span>
                     <div className="absolute right-0 top-0 bottom-0 bg-trade-up/10 transition-all duration-300" style={{width: `${Math.random() * 80}%`}}></div>
                  </div>
                  ))}
               </div>
            </div>
         </div>

         {/* Order Entry */}
         <div className="bg-surface-900 border border-surface-800 rounded-sm p-4 flex flex-col gap-4">
            <div className="flex bg-surface-950 p-1 rounded-md border border-surface-800">
               <button 
                  onClick={() => setOrderSide('buy')}
                  className={`flex-1 py-1.5 text-sm font-bold rounded transition-all ${orderSide === 'buy' ? 'bg-trade-up text-surface-950 shadow-md' : 'text-surface-500 hover:text-surface-300'}`}
               >买入 (Buy)</button>
               <button 
                  onClick={() => setOrderSide('sell')}
                  className={`flex-1 py-1.5 text-sm font-bold rounded transition-all ${orderSide === 'sell' ? 'bg-trade-down text-white shadow-md' : 'text-surface-500 hover:text-surface-300'}`}
               >卖出 (Sell)</button>
            </div>

            <div className="flex justify-between items-center text-xs text-surface-400 font-medium">
               <span>可用: <span className="text-surface-200">45,230 {quoteAsset}</span></span>
               <span className="bg-surface-800 px-2 py-0.5 rounded border border-surface-700 text-[10px] uppercase font-bold tracking-wider text-surface-300">全仓 20x</span>
            </div>

            <div className="space-y-3">
               <div className="relative group">
                  <span className="absolute left-3 top-2.5 text-xs text-surface-500 font-medium">价格</span>
                  <input type="text" className="w-full bg-surface-950 border border-surface-700 rounded-md px-3 py-2.5 text-right text-sm font-mono text-surface-100 focus:border-surface-500 outline-none transition-colors group-hover:border-surface-600" defaultValue="64235.5" />
                  <span className="absolute right-10 top-3 text-[10px] text-surface-600">{quoteAsset}</span>
               </div>
               <div className="relative group">
                  <span className="absolute left-3 top-2.5 text-xs text-surface-500 font-medium">数量</span>
                  <input type="text" className="w-full bg-surface-950 border border-surface-700 rounded-md px-3 py-2.5 text-right text-sm font-mono text-surface-100 focus:border-surface-500 outline-none transition-colors group-hover:border-surface-600" placeholder="0.00" />
                  <span className="absolute right-10 top-3 text-[10px] text-surface-600">{baseAsset}</span>
               </div>
            </div>

            <div className="grid grid-cols-4 gap-2">
               {[10, 25, 50, 100].map(p => (
                  <button key={p} className="bg-surface-800 hover:bg-surface-700 border border-surface-700 rounded-xs py-1.5 text-surface-300 font-medium transition-colors">{p}%</button>
               ))}
            </div>

            <button className={`w-full py-3 rounded-md font-bold text-sm uppercase tracking-wide transition-all active:scale-[0.98] shadow-lg ${orderSide === 'buy' ? 'bg-trade-up text-surface-900 shadow-trade-up/20 hover:bg-trade-up/90' : 'bg-trade-down text-white shadow-trade-down/20 hover:bg-trade-down/90'}`}>
               {orderSide === 'buy' ? `买入 / 做多 ${baseAsset}` : `卖出 / 做空 ${baseAsset}`}
            </button>
         </div>
      </div>
    </div>
  );
};