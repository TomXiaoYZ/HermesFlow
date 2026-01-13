import React, { useState } from 'react';
import { AreaChart, Area, XAxis, Tooltip as RechartsTooltip, ResponsiveContainer } from 'recharts';

// Mock Documents Data
const MOCK_DOCS = [
  { id: 1, name: 'Goldman_Sachs_Crypto_Outlook_2024.pdf', type: 'PDF', size: '2.4MB', date: '2023-12-15', status: 'ready', category: 'Crypto' },
  { id: 2, name: 'Fed_FOMC_Minutes_Jan.txt', type: 'TXT', size: '156KB', date: '2024-01-31', status: 'ready', category: 'Macro' },
  { id: 3, name: 'NVDA_Q4_Earnings_Transcript.pdf', type: 'PDF', size: '1.8MB', date: '2024-02-21', status: 'processing', category: 'Equities' },
  { id: 4, name: 'Internal_Alpha_Strategy_Review.docx', type: 'DOC', size: '850KB', date: '2024-03-01', status: 'ready', category: 'Internal' },
];

const MOCK_CHAT = [
  { role: 'ai', content: '已加载《Goldman_Sachs_Crypto_Outlook_2024》。该报告主要分析了机构资金流入对比特币价格周期的影响。有什么具体问题需要我解答吗？' },
];

const MOCK_SIGNALS = [
  { asset: 'BTC', sentiment: 'Bullish', confidence: 0.85, reason: 'ETF批准带来的结构性资金流入' },
  { asset: 'ETH', sentiment: 'Neutral', confidence: 0.50, reason: '坎昆升级预期已部分计价' },
  { asset: 'COIN', sentiment: 'Bullish', confidence: 0.72, reason: '交易量预期随波动率回升' },
];

const Card: React.FC<{ children: React.ReactNode; className?: string }> = ({ children, className }) => (
  <div className={`bg-surface-900 border border-surface-800 rounded-md ${className}`}>
    {children}
  </div>
);

export const KnowledgeBase: React.FC = () => {
  const [activeCategory, setActiveCategory] = useState('All');
  const [selectedDoc, setSelectedDoc] = useState<number | null>(1);
  const [chatInput, setChatInput] = useState('');
  const [chatHistory, setChatHistory] = useState(MOCK_CHAT);

  const handleSend = () => {
    if (!chatInput.trim()) return;
    const newMsg = { role: 'user', content: chatInput };
    setChatHistory([...chatHistory, newMsg]);
    setChatInput('');
    
    // Simulating AI Response
    setTimeout(() => {
      setChatHistory(prev => [...prev, { 
        role: 'ai', 
        content: '根据第 14 页图表数据，高盛预测如果比特币现货 ETF 获得批准，首年将带来约 50-100 亿美元的净流入，这可能推动 BTC 价格突破前高。风险点在于监管的不确定性仍未完全消除。' 
      }]);
    }, 1000);
  };

  return (
    <div className="h-full flex flex-col bg-surface-950 overflow-hidden">
      {/* Header */}
      <div className="h-14 bg-surface-950 border-b border-surface-800 flex items-center justify-between px-6 flex-shrink-0">
         <div>
            <h2 className="text-lg font-bold text-surface-100 flex items-center gap-2">
               <span className="material-symbols-outlined text-hermes-500">auto_stories</span>
               AI 投研知识库 (Knowledge Center)
            </h2>
            <p className="text-xs text-surface-500">基于 Gemini 1.5 Pro (2M Context) 的深度研报分析与信号提取</p>
         </div>
         <div className="flex gap-3">
             <button className="px-4 py-2 bg-hermes-500 hover:bg-hermes-400 text-surface-950 font-bold text-sm rounded transition-colors flex items-center gap-2">
                <span className="material-symbols-outlined text-[18px]">upload_file</span> 上传文档
             </button>
         </div>
      </div>

      <div className="flex-1 flex min-h-0">
         {/* 1. Left Sidebar: Document Library */}
         <div className="w-72 bg-surface-950 border-r border-surface-800 flex flex-col flex-shrink-0">
            {/* Search & Filter */}
            <div className="p-3 border-b border-surface-800 space-y-3">
               <div className="relative">
                  <span className="absolute left-2.5 top-2.5 material-symbols-outlined text-surface-500 text-[18px]">search</span>
                  <input type="text" placeholder="搜索研报/纪要..." className="w-full bg-surface-900 border border-surface-700 rounded-md py-2 pl-9 pr-3 text-sm text-surface-200 outline-none focus:border-hermes-500" />
               </div>
               <div className="flex gap-2 overflow-x-auto pb-1 no-scrollbar">
                  {['All', 'Crypto', 'Macro', 'Equities'].map(cat => (
                     <button 
                        key={cat}
                        onClick={() => setActiveCategory(cat)}
                        className={`text-[10px] px-2.5 py-1 rounded-full border whitespace-nowrap transition-colors ${activeCategory === cat ? 'bg-surface-100 text-surface-950 border-surface-100 font-bold' : 'bg-surface-900 border-surface-700 text-surface-400 hover:border-surface-500'}`}
                     >
                        {cat}
                     </button>
                  ))}
               </div>
            </div>

            {/* File List */}
            <div className="flex-1 overflow-y-auto p-2 space-y-1">
               {MOCK_DOCS.map(doc => (
                  <div 
                     key={doc.id}
                     onClick={() => setSelectedDoc(doc.id)}
                     className={`p-3 rounded-md cursor-pointer border transition-all group relative ${selectedDoc === doc.id ? 'bg-surface-800 border-surface-600' : 'bg-transparent border-transparent hover:bg-surface-900'}`}
                  >
                     <div className="flex items-start gap-3">
                        <div className={`w-8 h-8 rounded flex items-center justify-center text-[10px] font-bold border ${
                           doc.type === 'PDF' ? 'bg-red-500/10 text-red-500 border-red-500/20' : 
                           doc.type === 'TXT' ? 'bg-blue-500/10 text-blue-500 border-blue-500/20' : 
                           'bg-surface-700 text-surface-400 border-surface-600'
                        }`}>
                           {doc.type}
                        </div>
                        <div className="flex-1 min-w-0">
                           <h4 className={`text-sm font-medium truncate mb-1 ${selectedDoc === doc.id ? 'text-surface-100' : 'text-surface-300'}`}>{doc.name}</h4>
                           <div className="flex justify-between items-center text-[10px] text-surface-500">
                              <span>{doc.date}</span>
                              <span>{doc.size}</span>
                           </div>
                        </div>
                     </div>
                     {doc.status === 'processing' && (
                        <div className="absolute inset-x-0 bottom-0 h-0.5 bg-surface-700 overflow-hidden rounded-b-md">
                           <div className="h-full bg-hermes-500 w-1/3 animate-[shimmer_1s_infinite]"></div>
                        </div>
                     )}
                  </div>
               ))}
            </div>

            {/* Storage Status */}
            <div className="p-3 border-t border-surface-800 bg-surface-950/50">
               <div className="flex justify-between text-[10px] text-surface-400 mb-1">
                  <span>知识库容量</span>
                  <span>1.2GB / 5GB</span>
               </div>
               <div className="w-full h-1 bg-surface-800 rounded-full overflow-hidden">
                  <div className="h-full bg-surface-500 w-[24%]"></div>
               </div>
            </div>
         </div>

         {/* 2. Middle: Analysis Interface (Chat & Preview) */}
         <div className="flex-1 flex flex-col bg-[#0c0c0e] relative min-w-0">
            {/* Document Context Header */}
            <div className="h-10 border-b border-surface-800 flex items-center px-4 bg-surface-950 justify-between">
               <span className="text-xs text-surface-400 font-mono flex items-center gap-2">
                  <span className="w-2 h-2 rounded-full bg-hermes-500"></span>
                  正在分析: <span className="text-surface-200 font-bold">Goldman_Sachs_Crypto_Outlook_2024.pdf</span>
               </span>
               <div className="flex gap-2">
                  <button className="p-1 text-surface-400 hover:text-white"><span className="material-symbols-outlined text-[16px]">visibility</span></button>
                  <button className="p-1 text-surface-400 hover:text-white"><span className="material-symbols-outlined text-[16px]">share</span></button>
               </div>
            </div>

            {/* Chat Area */}
            <div className="flex-1 overflow-y-auto p-4 space-y-6">
               {chatHistory.map((msg, i) => (
                  <div key={i} className={`flex gap-4 ${msg.role === 'user' ? 'flex-row-reverse' : ''}`}>
                     <div className={`w-8 h-8 rounded-full flex-shrink-0 flex items-center justify-center ${msg.role === 'ai' ? 'bg-hermes-500 text-surface-950' : 'bg-surface-700 text-surface-300'}`}>
                        <span className="material-symbols-outlined text-[18px]">{msg.role === 'ai' ? 'smart_toy' : 'person'}</span>
                     </div>
                     <div className={`max-w-[80%] rounded-lg p-3 text-sm leading-relaxed shadow-sm ${msg.role === 'ai' ? 'bg-surface-900 border border-surface-800 text-surface-200' : 'bg-surface-800 text-surface-100'}`}>
                        {msg.content}
                        {msg.role === 'ai' && (
                           <div className="mt-3 pt-3 border-t border-surface-800 flex gap-2">
                              <button className="text-[10px] bg-surface-950 border border-surface-700 px-2 py-1 rounded text-surface-400 hover:text-hermes-500 transition-colors flex items-center gap-1">
                                 <span className="material-symbols-outlined text-[12px]">find_in_page</span> 查看引用源 P.14
                              </button>
                              <button className="text-[10px] bg-surface-950 border border-surface-700 px-2 py-1 rounded text-surface-400 hover:text-hermes-500 transition-colors flex items-center gap-1">
                                 <span className="material-symbols-outlined text-[12px]">add_task</span> 生成交易信号
                              </button>
                           </div>
                        )}
                     </div>
                  </div>
               ))}
            </div>

            {/* Input Area */}
            <div className="p-4 bg-surface-950 border-t border-surface-800">
               <div className="relative">
                  <input 
                     type="text" 
                     value={chatInput}
                     onChange={(e) => setChatInput(e.target.value)}
                     onKeyDown={(e) => e.key === 'Enter' && handleSend()}
                     placeholder="询问关于该文档的细节，或要求提取交易策略..." 
                     className="w-full bg-surface-900 border border-surface-700 rounded-lg pl-4 pr-12 py-3 text-sm text-surface-100 placeholder-surface-500 focus:border-hermes-500 outline-none shadow-inner"
                  />
                  <button 
                     onClick={handleSend}
                     className="absolute right-2 top-1.5 p-1.5 bg-hermes-500 hover:bg-hermes-400 text-surface-950 rounded transition-colors"
                  >
                     <span className="material-symbols-outlined text-[20px]">arrow_upward</span>
                  </button>
               </div>
               <div className="text-center mt-2 text-[10px] text-surface-600">
                  Model: <span className="text-hermes-500 font-bold">Gemini 1.5 Pro</span> • Context Used: 45K / 2000K Tokens
               </div>
            </div>
         </div>

         {/* 3. Right Sidebar: Structured Insights (The "Quant" Value Add) */}
         <div className="w-80 bg-surface-950 border-l border-surface-800 flex flex-col flex-shrink-0">
            <div className="px-4 py-3 border-b border-surface-800 bg-surface-950/30">
               <h3 className="text-sm font-bold text-surface-200 flex items-center gap-2">
                  <span className="material-symbols-outlined text-purple-400">psychology_alt</span>
                  Alpha 信号提取
               </h3>
            </div>
            
            <div className="flex-1 overflow-y-auto p-4 space-y-6">
               {/* Extracted Signals */}
               <div className="space-y-3">
                  <div className="text-xs font-bold text-surface-500 uppercase tracking-wide">识别到的交易机会</div>
                  {MOCK_SIGNALS.map((sig, i) => (
                     <Card key={i} className="p-3 hover:border-surface-600 transition-colors cursor-pointer group">
                        <div className="flex justify-between items-center mb-2">
                           <span className="font-bold text-surface-200">{sig.asset}</span>
                           <span className={`text-[10px] px-1.5 py-0.5 rounded font-bold uppercase ${
                              sig.sentiment === 'Bullish' ? 'bg-trade-up/10 text-trade-up' : 
                              sig.sentiment === 'Bearish' ? 'bg-trade-down/10 text-trade-down' : 
                              'bg-surface-700 text-surface-400'
                           }`}>{sig.sentiment}</span>
                        </div>
                        <p className="text-xs text-surface-400 leading-snug mb-2">{sig.reason}</p>
                        <div className="flex items-center gap-2">
                           <div className="flex-1 h-1 bg-surface-800 rounded-full overflow-hidden">
                              <div className="h-full bg-hermes-500" style={{width: `${sig.confidence * 100}%`}}></div>
                           </div>
                           <span className="text-[10px] text-hermes-500 font-mono">{(sig.confidence * 100).toFixed(0)}% Conf.</span>
                        </div>
                        {/* Hidden Action */}
                        <div className="h-0 overflow-hidden group-hover:h-auto group-hover:mt-2 transition-all">
                           <button className="w-full py-1.5 bg-surface-800 hover:bg-surface-700 text-xs font-bold text-surface-200 rounded border border-surface-700 flex items-center justify-center gap-1">
                              <span className="material-symbols-outlined text-[14px]">add_chart</span> 发送到策略室
                           </button>
                        </div>
                     </Card>
                  ))}
               </div>

               {/* Key Risks */}
               <div>
                  <div className="text-xs font-bold text-surface-500 uppercase tracking-wide mb-3">关键风险因子 (Risk Factors)</div>
                  <ul className="space-y-2">
                     <li className="flex gap-2 text-xs text-surface-300">
                        <span className="material-symbols-outlined text-[14px] text-trade-warn mt-0.5">warning</span>
                        <span>监管不确定性可能导致短期波动率上升 (VIX > 20)</span>
                     </li>
                     <li className="flex gap-2 text-xs text-surface-300">
                        <span className="material-symbols-outlined text-[14px] text-surface-500 mt-0.5">info</span>
                        <span>宏观流动性紧缩对长尾资产的压制</span>
                     </li>
                  </ul>
               </div>

               {/* Summary Stats */}
               <div className="bg-surface-900 rounded p-3 border border-surface-800">
                  <div className="text-xs font-bold text-surface-500 uppercase mb-2">文档情感综述</div>
                  <div className="flex items-center justify-center py-2">
                     <div className="text-center">
                        <div className="text-2xl font-bold text-trade-up">68/100</div>
                        <div className="text-[10px] text-surface-400">偏向乐观</div>
                     </div>
                  </div>
               </div>
            </div>
         </div>
      </div>
    </div>
  );
};