"use client";

import React, { useEffect } from 'react';
import { loadFactorConfig } from '@/utils/genome';
import EvolutionExplorer from '@/components/EvolutionExplorer';

const StrategyLab: React.FC = () => {
   useEffect(() => {
      loadFactorConfig().catch(() => {/* factor config load failed */});
   }, []);

   return (
      <div className="h-full flex flex-col overflow-hidden bg-slate-950">
         {/* Header */}
         <div className="h-12 bg-slate-950 border-b border-white/5 flex items-center px-6">
            <h2 className="text-sm font-bold text-slate-100">Strategy Lab</h2>
            <span className="text-xs text-slate-600 ml-3">Per-Symbol Genetic Evolution</span>
         </div>

         {/* Content */}
         <div className="flex-1 min-h-0">
            <EvolutionExplorer />
         </div>
      </div>
   );
};

export default StrategyLab;
