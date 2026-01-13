import React, { useState, useEffect } from 'react';
import { Dashboard } from './components/Dashboard';
import { StrategyLab } from './components/StrategyLab';
import { TradingTerminal } from './components/TradingTerminal';
import { RiskConsole } from './components/RiskConsole';
import { SystemConfig } from './components/SystemConfig';
import { KnowledgeBase } from './components/KnowledgeBase';
import { UserManagement } from './components/UserManagement';
import { PortfolioManager } from './components/PortfolioManager';
import { AlertCenter } from './components/AlertCenter';

const App: React.FC = () => {
  const [activeView, setActiveView] = useState('dashboard');
  const [currentTime, setCurrentTime] = useState(new Date());

  useEffect(() => {
    const timer = setInterval(() => setCurrentTime(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);

  const renderView = () => {
    switch (activeView) {
      case 'dashboard': return <Dashboard />;
      case 'strategy': return <StrategyLab />;
      case 'terminal': return <TradingTerminal />;
      case 'risk': return <RiskConsole />;
      case 'knowledge': return <KnowledgeBase />;
      case 'config': return <SystemConfig />;
      case 'users': return <UserManagement />;
      case 'portfolio': return <PortfolioManager />;
      case 'alerts': return <AlertCenter />;
      default: return <Dashboard />;
    }
  };

  const NavButton = ({ id, icon, label }: { id: string, icon: string, label: string }) => (
    <button 
      onClick={() => setActiveView(id)}
      className={`relative w-full flex items-center gap-3 px-4 py-3 rounded-md transition-all group mb-1 ${
        activeView === id 
          ? 'bg-surface-800 text-hermes-500 shadow-sm border border-surface-700' 
          : 'text-surface-400 hover:text-surface-200 hover:bg-surface-800/50'
      }`}
    >
      <span className={`material-symbols-outlined text-[20px] ${activeView === id ? 'icon-filled' : ''}`}>{icon}</span>
      <span className="text-sm font-medium tracking-wide">{label}</span>
      {activeView === id && <div className="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-5 bg-hermes-500 rounded-r"></div>}
    </button>
  );

  return (
    <div className="flex h-screen bg-surface-950 text-surface-200 font-sans overflow-hidden">
      {/* Sidebar - Expanded fonts */}
      <aside className="w-64 bg-surface-950 border-r border-surface-800 flex-shrink-0 flex flex-col justify-between z-20">
        <div>
          {/* Brand */}
          <div className="h-14 flex items-center gap-3 px-5 border-b border-surface-800">
            <div className="w-6 h-6 bg-hermes-500 rounded-sm flex items-center justify-center shadow-lg shadow-hermes-500/20">
              <span className="material-symbols-outlined text-surface-950 text-[18px] font-bold">query_stats</span>
            </div>
            <div className="flex flex-col">
              <span className="text-lg font-bold text-surface-100 tracking-tight leading-none">Hermes<span className="font-normal text-surface-400">Flow</span></span>
              <span className="text-[10px] text-hermes-500 font-bold uppercase tracking-widest mt-0.5">PRO版</span>
            </div>
          </div>

          {/* Navigation */}
          <nav className="flex flex-col gap-1 p-3 mt-2">
            <p className="px-4 py-2 text-xs uppercase font-bold text-surface-500 tracking-wider">工作台 (Workspace)</p>
            <NavButton id="dashboard" icon="dashboard" label="仪表盘" />
            <NavButton id="terminal" icon="terminal" label="市场行情" />
            
            <p className="px-4 py-2 text-xs uppercase font-bold text-surface-500 tracking-wider mt-4">量化投研 (Research)</p>
            <NavButton id="strategy" icon="science" label="策略实验室" />
            <NavButton id="knowledge" icon="auto_stories" label="AI 知识库" />
            <NavButton id="risk" icon="shield" label="风控中心" />
            <NavButton id="portfolio" icon="pie_chart" label="投资组合" />
            
            <p className="px-4 py-2 text-xs uppercase font-bold text-surface-500 tracking-wider mt-4">系统管理 (System)</p>
            <NavButton id="alerts" icon="campaign" label="告警配置" />
            <NavButton id="users" icon="admin_panel_settings" label="用户管理" />
            <NavButton id="config" icon="settings" label="系统配置" />
          </nav>
        </div>

        {/* User Status */}
        <div className="p-4 border-t border-surface-800">
          <div className="flex items-center gap-3 px-3 py-2.5 rounded-md hover:bg-surface-800 cursor-pointer transition-colors border border-transparent hover:border-surface-700">
            <div className="w-9 h-9 rounded-full bg-surface-700 flex items-center justify-center text-sm font-bold text-surface-300 border border-surface-600">AC</div>
            <div className="flex flex-col overflow-hidden">
              <span className="text-sm font-bold text-surface-200 truncate">Alex Chen</span>
              <span className="text-xs text-surface-500 font-mono flex items-center gap-1.5">
                量化研究员
              </span>
            </div>
          </div>
        </div>
      </aside>

      {/* Main Layout */}
      <div className="flex-1 flex flex-col min-w-0 bg-surface-950">
        {/* Top Status Bar - Increased Size */}
        <header className="h-12 bg-surface-950 border-b border-surface-800 flex items-center justify-between px-6 text-sm select-none">
          <div className="flex items-center gap-6">
             <div className="flex items-center gap-2 text-surface-400">
                <span className="material-symbols-outlined text-[18px]">dns</span>
                <span className="font-mono text-hermes-500 font-bold">12ms</span>
             </div>
             <div className="w-px h-4 bg-surface-700"></div>
             <div className="flex items-center gap-3">
                <span className="text-surface-300 font-medium">BTC</span>
                <span className="font-mono text-trade-up font-bold text-base">64,235.50</span>
                <span className="font-mono text-trade-up text-xs bg-trade-up/10 px-1.5 py-0.5 rounded">+1.24%</span>
             </div>
             <div className="w-px h-4 bg-surface-700"></div>
             <div className="flex items-center gap-3">
                <span className="text-surface-300 font-medium">ETH</span>
                <span className="font-mono text-trade-down font-bold text-base">3,450.20</span>
                <span className="font-mono text-trade-down text-xs bg-trade-down/10 px-1.5 py-0.5 rounded">-0.85%</span>
             </div>
          </div>

          {/* Right Action Area */}
          <div className="flex items-center gap-5">
             <div className="flex items-center gap-2 text-surface-300 bg-surface-900 px-3 py-1 rounded border border-surface-800">
                <span className="material-symbols-outlined text-[16px]">schedule</span>
                <span className="font-mono font-medium">{currentTime.toLocaleTimeString('en-US', {hour12: false})}</span>
             </div>
             <div className="flex items-center gap-2 text-surface-400 hover:text-surface-200 cursor-pointer">
                <span className="material-symbols-outlined text-[20px]">notifications</span>
                <span className="bg-trade-down w-2 h-2 rounded-full relative -ml-3 -mt-3 border-2 border-surface-950"></span>
             </div>
             <div className="flex items-center gap-2 text-surface-400 hover:text-surface-200 cursor-pointer">
                <span className="material-symbols-outlined text-[20px]">help</span>
             </div>
          </div>
        </header>

        {/* Viewport */}
        <main className="flex-1 relative overflow-hidden bg-surface-950">
          {renderView()}
        </main>
      </div>
    </div>
  );
};

export default App;