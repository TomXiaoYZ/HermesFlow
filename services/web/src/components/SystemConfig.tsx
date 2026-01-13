import React, { useState } from 'react';

const Card: React.FC<{ children: React.ReactNode; className?: string }> = ({ children, className }) => (
  <div className={`bg-surface-900 border border-surface-800 rounded-md overflow-hidden ${className}`}>
    {children}
  </div>
);

// Mock Data for Whitelist
const INITIAL_SYMBOLS = [
   { id: 1, exchange: 'Binance', symbol: 'BTC/USDT', type: 'Perpetual', status: 'active', cost: 'High (L2)' },
   { id: 2, exchange: 'Binance', symbol: 'ETH/USDT', type: 'Perpetual', status: 'active', cost: 'High (L2)' },
   { id: 3, exchange: 'OKX', symbol: 'SOL/USDT', type: 'Perpetual', status: 'paused', cost: 'Med (L1)' },
   { id: 4, exchange: 'Interactive Brokers', symbol: 'NVDA', type: 'Stock', status: 'active', cost: 'Low (Snapshot)' },
];

export const SystemConfig: React.FC = () => {
  const [symbols, setSymbols] = useState(INITIAL_SYMBOLS);

  const toggleStatus = (id: number) => {
     setSymbols(prev => prev.map(s => s.id === id ? { ...s, status: s.status === 'active' ? 'paused' : 'active' } : s));
  };

  return (
    <div className="h-full overflow-y-auto p-6 flex flex-col gap-6 bg-surface-950">
       <h2 className="text-2xl font-bold text-surface-100 mb-2">系统配置 (System Configuration)</h2>
       
       <div className="grid grid-cols-2 gap-6">
          {/* API Connections */}
          <Card>
             <div className="p-4 border-b border-surface-800 bg-surface-950/50">
                <h3 className="font-bold text-surface-200 flex items-center gap-2 text-base">
                   <span className="material-symbols-outlined text-hermes-500 text-[20px]">hub</span> 交易所连接 (Exchange Connections)
                </h3>
             </div>
             <div className="p-5 space-y-4">
                <div className="p-4 bg-surface-950 rounded border border-surface-800 flex justify-between items-center shadow-sm hover:border-surface-600 transition-colors">
                   <div className="flex items-center gap-4">
                      <div className="w-10 h-10 rounded bg-[#FCD535]/10 flex items-center justify-center text-[#FCD535] font-bold text-sm">BN</div>
                      <div>
                         <h4 className="text-surface-200 font-bold text-sm">Binance (币安)</h4>
                         <div className="flex items-center gap-2 text-xs mt-0.5">
                            <span className="w-2 h-2 rounded-full bg-hermes-500 shadow-[0_0_5px_#00E396]"></span>
                            <span className="text-surface-400 font-mono">已连接 (32ms)</span>
                         </div>
                      </div>
                   </div>
                   <button className="text-xs border border-surface-700 rounded px-4 py-1.5 text-surface-400 hover:text-surface-100 hover:bg-surface-800 transition font-medium">配置</button>
                </div>

                <div className="p-4 bg-surface-950 rounded border border-surface-800 flex justify-between items-center shadow-sm hover:border-surface-600 transition-colors">
                   <div className="flex items-center gap-4">
                      <div className="w-10 h-10 rounded bg-[#D62828]/10 flex items-center justify-center text-[#D62828] font-bold text-sm">IB</div>
                      <div>
                         <h4 className="text-surface-200 font-bold text-sm">Interactive Brokers (盈透)</h4>
                         <div className="flex items-center gap-2 text-xs mt-0.5">
                            <span className="w-2 h-2 rounded-full bg-trade-down animate-pulse"></span>
                            <span className="text-surface-400 font-mono">网关离线</span>
                         </div>
                      </div>
                   </div>
                   <button className="text-xs bg-hermes-500/10 text-hermes-500 border border-hermes-500/20 rounded px-4 py-1.5 hover:bg-hermes-500/20 transition font-bold">重连</button>
                </div>
             </div>
          </Card>

          {/* Data Subscriptions */}
          <Card>
             <div className="p-4 border-b border-surface-800 bg-surface-950/50">
                <h3 className="font-bold text-surface-200 flex items-center gap-2 text-base">
                   <span className="material-symbols-outlined text-blue-400 text-[20px]">subscriptions</span> 数据订阅 (Data Feeds)
                </h3>
             </div>
             <div className="p-5 space-y-3">
                <label className="flex items-center justify-between p-3 rounded-md hover:bg-surface-800 cursor-pointer border border-transparent hover:border-surface-700 transition-colors">
                   <div className="flex items-center gap-4">
                      <input type="radio" name="sub" className="w-4 h-4 text-hermes-500 bg-surface-950 border-surface-600 focus:ring-hermes-500 accent-hermes-500" />
                      <div>
                         <p className="text-sm font-bold text-surface-300">Level 1 (基础)</p>
                         <p className="text-xs text-surface-500 mt-0.5">实时快照, BBO (最优买卖)</p>
                      </div>
                   </div>
                </label>
                <label className="flex items-center justify-between p-3 rounded-md bg-hermes-500/5 border border-hermes-500/20 cursor-pointer shadow-sm">
                   <div className="flex items-center gap-4">
                      <input type="radio" name="sub" defaultChecked className="w-4 h-4 text-hermes-500 bg-surface-950 border-surface-600 focus:ring-hermes-500 accent-hermes-500" />
                      <div>
                         <p className="text-sm font-bold text-surface-100">Level 2 (专业)</p>
                         <p className="text-xs text-surface-400 mt-0.5">10档深度, 逐笔成交</p>
                      </div>
                   </div>
                   <span className="text-[10px] bg-hermes-500 text-surface-950 px-2 py-0.5 rounded font-bold uppercase tracking-wide">使用中</span>
                </label>
                <label className="flex items-center justify-between p-3 rounded-md hover:bg-surface-800 cursor-pointer border border-transparent hover:border-surface-700 transition-colors">
                   <div className="flex items-center gap-4">
                      <input type="radio" name="sub" className="w-4 h-4 text-hermes-500 bg-surface-950 border-surface-600 focus:ring-hermes-500 accent-hermes-500" />
                      <div>
                         <p className="text-sm font-bold text-surface-300">Full Order Book (机构)</p>
                         <p className="text-xs text-surface-500 mt-0.5">MBO, 原始订单流</p>
                      </div>
                   </div>
                </label>
             </div>
          </Card>

          {/* AI Model Management */}
          <Card className="col-span-2">
             <div className="p-4 border-b border-surface-800 bg-surface-950/50 flex justify-between items-center">
                <h3 className="font-bold text-surface-200 flex items-center gap-2 text-base">
                   <span className="material-symbols-outlined text-purple-500 text-[20px]">psychology</span> AI 模型管理 (Model Management)
                </h3>
                <span className="text-xs text-surface-500 bg-surface-900 px-2 py-1 rounded border border-surface-800">当前消耗: $12.45 / Month</span>
             </div>
             <div className="p-5 grid grid-cols-2 gap-8">
                {/* Left: Model Selection */}
                <div className="space-y-4">
                   <div>
                      <label className="text-xs font-bold text-surface-400 uppercase tracking-wider mb-2 block">默认推理模型 (Default Inference)</label>
                      <div className="flex items-center gap-2 bg-surface-950 border border-surface-700 p-3 rounded-md">
                         <div className="w-8 h-8 rounded-full bg-blue-500/20 text-blue-400 flex items-center justify-center font-bold text-xs border border-blue-500/30">G</div>
                         <div className="flex-1">
                            <div className="text-sm font-bold text-surface-200">Google Gemini 1.5 Pro</div>
                            <div className="text-[10px] text-surface-500">2M Context • Multimodal • 知识库分析首选</div>
                         </div>
                         <button className="text-xs text-hermes-500 font-bold hover:underline">更改</button>
                      </div>
                   </div>

                   <div>
                      <label className="text-xs font-bold text-surface-400 uppercase tracking-wider mb-2 block">备用/快速模型 (Fast/Fallback)</label>
                      <div className="flex items-center gap-2 bg-surface-950 border border-surface-700 p-3 rounded-md">
                         <div className="w-8 h-8 rounded-full bg-yellow-500/20 text-yellow-400 flex items-center justify-center font-bold text-xs border border-yellow-500/30">F</div>
                         <div className="flex-1">
                            <div className="text-sm font-bold text-surface-200">Gemini 1.5 Flash</div>
                            <div className="text-[10px] text-surface-500">Low Latency • High Throughput • 实时舆情首选</div>
                         </div>
                         <button className="text-xs text-hermes-500 font-bold hover:underline">更改</button>
                      </div>
                   </div>
                </div>

                {/* Right: API & Parameters */}
                <div className="space-y-4">
                   <div>
                      <label className="text-xs font-bold text-surface-400 uppercase tracking-wider mb-2 block">API Key Configuration</label>
                      <div className="relative">
                         <input type="password" value="AIzaSy...MockKey...12345" readOnly className="w-full bg-surface-950 border border-surface-700 rounded p-2.5 text-sm font-mono text-surface-400 outline-none focus:border-hermes-500 transition-colors" />
                         <div className="absolute right-2 top-2 flex gap-1">
                            <button className="p-1 hover:text-white text-surface-500"><span className="material-symbols-outlined text-[16px]">visibility</span></button>
                            <button className="p-1 hover:text-white text-surface-500"><span className="material-symbols-outlined text-[16px]">refresh</span></button>
                         </div>
                      </div>
                   </div>

                   <div className="grid grid-cols-2 gap-4">
                      <div>
                         <label className="text-xs font-bold text-surface-400 uppercase tracking-wider mb-2 block">Temperature</label>
                         <div className="flex items-center gap-2">
                            <input type="range" min="0" max="1" step="0.1" defaultValue="0.2" className="flex-1 accent-hermes-500 h-1 bg-surface-700 rounded-lg appearance-none cursor-pointer" />
                            <span className="text-xs font-mono text-surface-200 w-8 text-right">0.2</span>
                         </div>
                         <div className="text-[10px] text-surface-500 mt-1">低数值 = 更严谨的金融分析</div>
                      </div>
                      <div>
                         <label className="text-xs font-bold text-surface-400 uppercase tracking-wider mb-2 block">Max Output</label>
                         <select className="w-full bg-surface-950 border border-surface-700 rounded p-1 text-xs text-surface-300 outline-none">
                            <option>4096 Tokens</option>
                            <option>8192 Tokens</option>
                            <option>Unlimited</option>
                         </select>
                      </div>
                   </div>

                   <div className="pt-2">
                       <button className="w-full py-2 bg-surface-800 hover:bg-surface-700 border border-surface-700 rounded text-xs font-bold text-surface-300 flex items-center justify-center gap-2 transition-colors">
                          <span className="material-symbols-outlined text-[16px]">check_circle</span> 测试连接 (Test Connection)
                       </button>
                   </div>
                </div>
             </div>
          </Card>

          {/* NEW: Symbol Whitelist Configuration */}
          <Card className="col-span-2">
             <div className="p-4 border-b border-surface-800 bg-surface-950/50 flex justify-between items-center">
                <h3 className="font-bold text-surface-200 flex items-center gap-2 text-base">
                   <span className="material-symbols-outlined text-orange-400 text-[20px]">list_alt</span> 标的白名单配置 (Symbol Whitelist)
                </h3>
                <span className="text-[10px] text-trade-warn bg-trade-warn/5 px-3 py-1.5 rounded border border-trade-warn/20 flex items-center gap-2">
                    <span className="material-symbols-outlined text-[14px]">warning</span>
                    <span className="font-bold">成本控制:</span> 仅订阅白名单内的实时行情数据
                </span>
             </div>
             
             <div className="p-5 flex flex-col gap-6">
                {/* Add New Bar */}
                <div className="flex gap-4 items-end bg-surface-950 p-4 rounded border border-surface-800">
                    <div className="flex-1">
                        <label className="text-[10px] font-bold text-surface-500 uppercase mb-2 block">交易所 (Exchange)</label>
                        <select className="w-full bg-surface-900 border border-surface-700 rounded p-2.5 text-sm text-surface-200 outline-none focus:border-hermes-500">
                            <option>Binance</option>
                            <option>OKX</option>
                            <option>Interactive Brokers</option>
                            <option>Bybit</option>
                        </select>
                    </div>
                    <div className="flex-1">
                        <label className="text-[10px] font-bold text-surface-500 uppercase mb-2 block">交易对代码 (Symbol)</label>
                        <input type="text" placeholder="e.g. BTC/USDT" className="w-full bg-surface-900 border border-surface-700 rounded p-2.5 text-sm text-surface-200 outline-none focus:border-hermes-500 font-mono" />
                    </div>
                    <div className="flex-1">
                         <label className="text-[10px] font-bold text-surface-500 uppercase mb-2 block">资产类型 (Asset Type)</label>
                         <select className="w-full bg-surface-900 border border-surface-700 rounded p-2.5 text-sm text-surface-200 outline-none focus:border-hermes-500">
                            <option>Perpetual (永续合约)</option>
                            <option>Spot (现货)</option>
                            <option>Stock (股票)</option>
                            <option>Option (期权)</option>
                        </select>
                    </div>
                    <button className="px-6 py-2.5 bg-surface-100 hover:bg-white text-surface-950 font-bold text-sm rounded transition-colors shadow-lg flex items-center gap-2 h-[42px]">
                        <span className="material-symbols-outlined text-[18px]">add</span>
                        添加监控
                    </button>
                </div>

                {/* List */}
                <div className="border border-surface-800 rounded-md overflow-hidden shadow-sm">
                    <table className="w-full text-left text-xs">
                        <thead className="bg-surface-950 text-surface-500 font-bold uppercase tracking-wider border-b border-surface-800">
                            <tr>
                                <th className="px-5 py-3.5">交易所</th>
                                <th className="px-5 py-3.5">标的代码</th>
                                <th className="px-5 py-3.5">类型</th>
                                <th className="px-5 py-3.5">数据成本等级 (Cost)</th>
                                <th className="px-5 py-3.5">状态</th>
                                <th className="px-5 py-3.5 text-right">操作</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-surface-800 bg-surface-900/30 font-mono text-sm">
                            {symbols.map((item) => (
                                <tr key={item.id} className="hover:bg-surface-800/50 transition-colors group">
                                    <td className="px-5 py-3 font-bold text-surface-300 font-sans">{item.exchange}</td>
                                    <td className="px-5 py-3 font-bold text-surface-100">{item.symbol}</td>
                                    <td className="px-5 py-3 text-surface-400 text-xs font-sans">{item.type}</td>
                                    <td className="px-5 py-3">
                                        <span className={`text-[10px] px-2 py-0.5 rounded font-bold border ${
                                            item.cost.includes('High') ? 'bg-trade-down/10 text-trade-down border-trade-down/20' :
                                            item.cost.includes('Med') ? 'bg-trade-warn/10 text-trade-warn border-trade-warn/20' :
                                            'bg-surface-700 text-surface-400 border-surface-600'
                                        }`}>
                                            {item.cost}
                                        </span>
                                    </td>
                                    <td className="px-5 py-3">
                                        <div className="flex items-center gap-2">
                                            <div className={`w-2 h-2 rounded-full ${item.status === 'active' ? 'bg-hermes-500 shadow-[0_0_5px_#00E396]' : 'bg-surface-600'}`}></div>
                                            <span className={`text-xs font-bold uppercase ${item.status === 'active' ? 'text-surface-200' : 'text-surface-500'}`}>{item.status}</span>
                                        </div>
                                    </td>
                                    <td className="px-5 py-3 text-right">
                                        <div className="flex justify-end gap-2 opacity-60 group-hover:opacity-100 transition-opacity">
                                            <button 
                                                onClick={() => toggleStatus(item.id)}
                                                className={`p-1.5 rounded transition-colors ${item.status === 'active' ? 'hover:bg-trade-warn/20 text-trade-warn' : 'hover:bg-hermes-500/20 text-hermes-500'}`} 
                                                title={item.status === 'active' ? "暂停订阅" : "恢复订阅"}
                                            >
                                                <span className="material-symbols-outlined text-[18px]">{item.status === 'active' ? 'pause_circle' : 'play_circle'}</span>
                                            </button>
                                            <button className="p-1.5 hover:bg-trade-down/20 text-surface-500 hover:text-trade-down rounded transition-colors" title="删除">
                                                <span className="material-symbols-outlined text-[18px]">delete</span>
                                            </button>
                                        </div>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                    <div className="bg-surface-950/50 p-2 border-t border-surface-800 text-center text-[10px] text-surface-500">
                        当前预估月度数据费用: <span className="text-surface-300 font-bold">$45.00</span> (3 Active Streams)
                    </div>
                </div>
             </div>
          </Card>
       </div>
    </div>
  );
};