import React, { useState } from 'react';

const Card: React.FC<{ children: React.ReactNode; className?: string }> = ({ children, className }) => (
  <div className={`bg-surface-900 border border-surface-800 rounded-md overflow-hidden ${className}`}>
    {children}
  </div>
);

// Mock Data for Rules
const INITIAL_RULES = [
  { id: 1, name: 'BTC 暴涨急跌预警', condition: 'BTC/USDT 1m Change > 1.5% OR < -1.5%', channels: ['Discord', 'Telegram'], priority: 'High', status: 'active' },
  { id: 2, name: '全域风控熔断通知', condition: 'Portfolio Drawdown > 3.0%', channels: ['Discord', 'Slack', 'Telegram'], priority: 'Critical', status: 'active' },
  { id: 3, name: '鲸鱼转账监控 (Whale Alert)', condition: 'On-Chain TX Value > $10,000,000', channels: ['Telegram'], priority: 'Medium', status: 'paused' },
  { id: 4, name: 'AI 舆情强信号推送', condition: 'Sentiment Score > 90 (Bullish)', channels: ['Slack'], priority: 'Low', status: 'active' },
];

// Mock Data for Logs
const ALERT_LOGS = [
  { id: 101, time: '14:32:05', channel: 'Discord', type: 'Price Alert', message: 'BTC/USDT broke above 64,500 (+2.1%)', status: 'success' },
  { id: 102, time: '14:30:11', channel: 'Telegram', type: 'Risk Warning', message: 'Margin utilization > 40% [Warning]', status: 'success' },
  { id: 103, time: '14:15:00', channel: 'Slack', type: 'System', message: 'Daily Strategy Report Generated', status: 'failed' },
  { id: 104, time: '13:58:22', channel: 'Discord', type: 'Sentiment', message: 'New Bullish Signal: SOL (Score: 92)', status: 'success' },
  { id: 105, time: '13:45:10', channel: 'Telegram', type: 'Whale Alert', message: '10,000 ETH transferred to Binance', status: 'success' },
];

export const AlertCenter: React.FC = () => {
  const [rules, setRules] = useState(INITIAL_RULES);
  const [channels, setChannels] = useState({
     discord: { enabled: true, url: 'https://discord.com/api/webhooks/9812...', status: 'connected' },
     slack: { enabled: false, url: '', status: 'disconnected' },
     telegram: { enabled: true, token: '123456:ABC-DEF1234ghIkl...', chatId: '@hermes_alerts', status: 'connected' }
  });

  const toggleRule = (id: number) => {
     setRules(prev => prev.map(r => r.id === id ? { ...r, status: r.status === 'active' ? 'paused' : 'active' } : r));
  };

  const toggleChannel = (key: keyof typeof channels) => {
     setChannels(prev => ({
        ...prev,
        [key]: { ...prev[key], enabled: !prev[key].enabled }
     }));
  };

  return (
    <div className="h-full overflow-y-auto p-6 flex flex-col gap-6 bg-surface-950">
       {/* Header */}
       <div className="flex justify-between items-end">
          <div>
             <h2 className="text-2xl font-bold text-surface-100 mb-1 flex items-center gap-2">
                <span className="material-symbols-outlined text-trade-warn">campaign</span>
                告警配置中心 (Alert Center)
             </h2>
             <p className="text-xs text-surface-500">配置多渠道消息推送与风险预警规则</p>
          </div>
          <button className="px-4 py-2 bg-trade-warn hover:bg-yellow-500 text-surface-950 font-bold text-sm rounded shadow-lg shadow-yellow-500/20 transition-colors flex items-center gap-2">
              <span className="material-symbols-outlined text-[18px]">add_alert</span>
              新增告警规则
          </button>
       </div>

       {/* Channel Configuration Grid */}
       <div className="grid grid-cols-3 gap-6">
          {/* Discord Card */}
          <Card className="border-t-4 border-t-[#5865F2]">
             <div className="p-4 border-b border-surface-800 flex justify-between items-center bg-surface-950/30">
                <div className="flex items-center gap-2">
                   <div className="w-8 h-8 rounded bg-[#5865F2]/10 flex items-center justify-center">
                      <i className="material-symbols-outlined text-[#5865F2]" style={{fontStyle:'normal'}}>forum</i>
                   </div>
                   <span className="font-bold text-surface-200">Discord</span>
                </div>
                <div className="relative inline-block w-10 h-5 align-middle select-none transition duration-200 ease-in">
                    <input type="checkbox" checked={channels.discord.enabled} onChange={() => toggleChannel('discord')} className="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none cursor-pointer checked:right-0 checked:border-[#5865F2] right-5 border-surface-600"/>
                    <label className={`toggle-label block overflow-hidden h-5 rounded-full cursor-pointer ${channels.discord.enabled ? 'bg-[#5865F2]' : 'bg-surface-700'}`}></label>
                </div>
             </div>
             <div className="p-4 space-y-4">
                <div>
                   <label className="text-[10px] font-bold text-surface-500 uppercase mb-1.5 block">Webhook URL</label>
                   <input type="password" value={channels.discord.url} readOnly className="w-full bg-surface-950 border border-surface-700 rounded p-2 text-xs text-surface-400 font-mono focus:border-[#5865F2] outline-none" />
                </div>
                <div className="flex justify-between items-center pt-2">
                   <span className="flex items-center gap-1.5 text-xs text-[#5865F2] bg-[#5865F2]/10 px-2 py-1 rounded border border-[#5865F2]/20 font-bold">
                      <span className="w-1.5 h-1.5 rounded-full bg-[#5865F2]"></span> Connected
                   </span>
                   <button className="text-xs hover:text-white text-surface-400 hover:bg-surface-800 px-3 py-1.5 rounded transition-colors border border-transparent hover:border-surface-700">测试推送</button>
                </div>
             </div>
          </Card>

          {/* Slack Card */}
          <Card className="border-t-4 border-t-[#E01E5A]">
             <div className="p-4 border-b border-surface-800 flex justify-between items-center bg-surface-950/30">
                <div className="flex items-center gap-2">
                   <div className="w-8 h-8 rounded bg-[#E01E5A]/10 flex items-center justify-center">
                      <i className="material-symbols-outlined text-[#E01E5A]" style={{fontStyle:'normal'}}>workspaces</i>
                   </div>
                   <span className="font-bold text-surface-200">Slack</span>
                </div>
                <div className="relative inline-block w-10 h-5 align-middle select-none transition duration-200 ease-in">
                    <input type="checkbox" checked={channels.slack.enabled} onChange={() => toggleChannel('slack')} className="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none cursor-pointer checked:right-0 checked:border-[#E01E5A] right-5 border-surface-600"/>
                    <label className={`toggle-label block overflow-hidden h-5 rounded-full cursor-pointer ${channels.slack.enabled ? 'bg-[#E01E5A]' : 'bg-surface-700'}`}></label>
                </div>
             </div>
             <div className="p-4 space-y-4">
                <div>
                   <label className="text-[10px] font-bold text-surface-500 uppercase mb-1.5 block">Webhook URL</label>
                   <input type="text" placeholder="https://hooks.slack.com/..." className="w-full bg-surface-950 border border-surface-700 rounded p-2 text-xs text-surface-400 font-mono focus:border-[#E01E5A] outline-none" />
                </div>
                <div className="flex justify-between items-center pt-2">
                   <span className="flex items-center gap-1.5 text-xs text-surface-500 bg-surface-950 px-2 py-1 rounded border border-surface-800 font-bold">
                      <span className="w-1.5 h-1.5 rounded-full bg-surface-500"></span> Disconnected
                   </span>
                   <button disabled className="text-xs text-surface-600 px-3 py-1.5 rounded cursor-not-allowed">测试推送</button>
                </div>
             </div>
          </Card>

          {/* Telegram Card */}
          <Card className="border-t-4 border-t-[#24A1DE]">
             <div className="p-4 border-b border-surface-800 flex justify-between items-center bg-surface-950/30">
                <div className="flex items-center gap-2">
                   <div className="w-8 h-8 rounded bg-[#24A1DE]/10 flex items-center justify-center">
                      <i className="material-symbols-outlined text-[#24A1DE]" style={{fontStyle:'normal'}}>send</i>
                   </div>
                   <span className="font-bold text-surface-200">Telegram</span>
                </div>
                <div className="relative inline-block w-10 h-5 align-middle select-none transition duration-200 ease-in">
                    <input type="checkbox" checked={channels.telegram.enabled} onChange={() => toggleChannel('telegram')} className="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none cursor-pointer checked:right-0 checked:border-[#24A1DE] right-5 border-surface-600"/>
                    <label className={`toggle-label block overflow-hidden h-5 rounded-full cursor-pointer ${channels.telegram.enabled ? 'bg-[#24A1DE]' : 'bg-surface-700'}`}></label>
                </div>
             </div>
             <div className="p-4 space-y-3">
                <div>
                   <label className="text-[10px] font-bold text-surface-500 uppercase mb-1 block">Bot Token</label>
                   <input type="password" value={channels.telegram.token} readOnly className="w-full bg-surface-950 border border-surface-700 rounded p-2 text-xs text-surface-400 font-mono focus:border-[#24A1DE] outline-none" />
                </div>
                <div>
                   <label className="text-[10px] font-bold text-surface-500 uppercase mb-1 block">Chat ID / Channel</label>
                   <input type="text" value={channels.telegram.chatId} readOnly className="w-full bg-surface-950 border border-surface-700 rounded p-2 text-xs text-surface-400 font-mono focus:border-[#24A1DE] outline-none" />
                </div>
                <div className="flex justify-between items-center pt-1">
                   <span className="flex items-center gap-1.5 text-xs text-[#24A1DE] bg-[#24A1DE]/10 px-2 py-1 rounded border border-[#24A1DE]/20 font-bold">
                      <span className="w-1.5 h-1.5 rounded-full bg-[#24A1DE]"></span> Connected
                   </span>
                   <button className="text-xs hover:text-white text-surface-400 hover:bg-surface-800 px-3 py-1.5 rounded transition-colors border border-transparent hover:border-surface-700">测试推送</button>
                </div>
             </div>
          </Card>
       </div>

       {/* Rules and Logs */}
       <div className="grid grid-cols-12 gap-6 flex-1 min-h-0">
          
          {/* Rules List */}
          <Card className="col-span-8 flex flex-col">
             <div className="px-4 py-3 border-b border-surface-800 flex justify-between items-center bg-surface-950/30">
                <h3 className="font-bold text-surface-200 text-sm flex items-center gap-2">
                   <span className="material-symbols-outlined text-surface-400 text-[18px]">rule</span> 告警规则 (Active Rules)
                </h3>
             </div>
             <div className="flex-1 overflow-auto">
                <table className="w-full text-left text-xs">
                   <thead className="bg-surface-950 text-surface-500 font-bold uppercase sticky top-0 border-b border-surface-800">
                      <tr>
                         <th className="px-4 py-3">规则名称</th>
                         <th className="px-4 py-3">触发条件</th>
                         <th className="px-4 py-3">推送渠道</th>
                         <th className="px-4 py-3">优先级</th>
                         <th className="px-4 py-3 text-right">状态</th>
                         <th className="px-4 py-3 text-right">操作</th>
                      </tr>
                   </thead>
                   <tbody className="divide-y divide-surface-800 bg-surface-900/50">
                      {rules.map(rule => (
                         <tr key={rule.id} className="hover:bg-surface-800 transition-colors group">
                            <td className="px-4 py-3 font-bold text-surface-200">{rule.name}</td>
                            <td className="px-4 py-3 font-mono text-surface-400">{rule.condition}</td>
                            <td className="px-4 py-3">
                               <div className="flex gap-1">
                                  {rule.channels.map(c => (
                                     <span key={c} className={`w-5 h-5 rounded flex items-center justify-center text-[12px] ${
                                        c === 'Discord' ? 'bg-[#5865F2]/20 text-[#5865F2]' : 
                                        c === 'Slack' ? 'bg-[#E01E5A]/20 text-[#E01E5A]' : 
                                        'bg-[#24A1DE]/20 text-[#24A1DE]'
                                     }`} title={c}>
                                        {c[0]}
                                     </span>
                                  ))}
                               </div>
                            </td>
                            <td className="px-4 py-3">
                               <span className={`px-2 py-0.5 rounded text-[10px] font-bold uppercase ${
                                  rule.priority === 'Critical' ? 'bg-trade-down/20 text-trade-down border border-trade-down/30' :
                                  rule.priority === 'High' ? 'bg-trade-warn/20 text-trade-warn border border-trade-warn/30' :
                                  'bg-surface-700 text-surface-400'
                               }`}>
                                  {rule.priority}
                               </span>
                            </td>
                            <td className="px-4 py-3 text-right">
                               <div className="flex items-center justify-end gap-2">
                                  <span className={`w-1.5 h-1.5 rounded-full ${rule.status === 'active' ? 'bg-trade-up' : 'bg-surface-600'}`}></span>
                                  <span className={`text-[10px] font-bold uppercase ${rule.status === 'active' ? 'text-surface-300' : 'text-surface-500'}`}>{rule.status}</span>
                               </div>
                            </td>
                            <td className="px-4 py-3 text-right">
                               <button 
                                  onClick={() => toggleRule(rule.id)}
                                  className={`p-1 rounded hover:bg-surface-700 transition-colors ${rule.status === 'active' ? 'text-trade-warn' : 'text-trade-up'}`}
                               >
                                  <span className="material-symbols-outlined text-[18px]">{rule.status === 'active' ? 'pause_circle' : 'play_circle'}</span>
                               </button>
                            </td>
                         </tr>
                      ))}
                   </tbody>
                </table>
             </div>
          </Card>

          {/* Logs Console */}
          <Card className="col-span-4 flex flex-col bg-[#0c0c0e]">
             <div className="px-4 py-3 border-b border-surface-800 flex justify-between items-center bg-surface-900">
                <h3 className="font-bold text-surface-200 text-sm flex items-center gap-2">
                   <span className="material-symbols-outlined text-surface-400 text-[18px]">history</span> 推送日志 (Logs)
                </h3>
                <span className="text-[10px] text-surface-500 font-mono">Real-time</span>
             </div>
             <div className="flex-1 overflow-y-auto p-0 font-mono text-xs">
                {ALERT_LOGS.map(log => (
                   <div key={log.id} className="p-3 border-b border-surface-800/50 hover:bg-surface-900 transition-colors flex gap-3">
                      <div className="text-surface-500 flex-shrink-0 w-14">{log.time}</div>
                      <div className="flex-1 min-w-0">
                         <div className="flex items-center gap-2 mb-1">
                            <span className={`font-bold ${log.status === 'success' ? 'text-trade-up' : 'text-trade-down'}`}>
                               {log.status === 'success' ? 'SENT' : 'FAIL'}
                            </span>
                            <span className="text-surface-400">to</span>
                            <span className={`font-bold ${
                               log.channel === 'Discord' ? 'text-[#5865F2]' : 
                               log.channel === 'Slack' ? 'text-[#E01E5A]' : 
                               'text-[#24A1DE]'
                            }`}>{log.channel}</span>
                            <span className="px-1.5 py-0.5 rounded bg-surface-800 text-surface-400 text-[10px] border border-surface-700">{log.type}</span>
                         </div>
                         <div className="text-surface-300 break-words leading-tight">{log.message}</div>
                      </div>
                   </div>
                ))}
             </div>
          </Card>
       </div>
    </div>
  );
};