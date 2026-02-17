"use client";

import React, { useEffect } from 'react';
import { Dna } from 'lucide-react';
import { loadFactorConfig } from '@/utils/genome';
import EvolutionExplorer from '@/components/EvolutionExplorer';

const StrategyLab: React.FC = () => {
   useEffect(() => {
      loadFactorConfig().catch(() => {/* factor config load failed */});
   }, []);

   return (
      <div className="h-full flex flex-col overflow-hidden bg-[#030305]">
         {/* Header */}
         <div className="h-11 bg-slate-950/80 backdrop-blur-md border-b border-white/5 flex items-center px-5 shrink-0">
            <Dna className="w-4 h-4 text-indigo-400 mr-2.5" />
            <h2 className="text-sm font-bold text-slate-100">Strategy Lab</h2>
            <span className="text-[10px] text-slate-600 ml-3 font-mono">Per-Symbol Genetic Evolution</span>
         </div>

         {/* Content */}
         <div className="flex-1 min-h-0">
            <EvolutionExplorer />
         </div>
      </div>
   );
};

export default StrategyLab;
