import React from "react";
import { TranslationDict } from "../types";

interface StatsProps {
  clients: number;
  countriesCount: number;
  websitesCount: number;
  attacksCount: number;
  t: TranslationDict;
}

export default function Stats({
  clients,
  countriesCount,
  websitesCount,
  attacksCount,
  t,
}: StatsProps) {
  return (
    <section 
      id="stats-metrics-section" 
      className="relative z-10 grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-8 py-10 border-t border-b border-white/10 bg-[#0F0F0F]/50 rounded-2xl px-8 max-w-6xl mx-auto"
    >
      <div id="stat-block-clients" className="relative pr-4 space-y-1">
        <div className="text-4xl md:text-5xl font-display font-bold text-white tracking-tight">
          {clients}
        </div>
        <div className="text-xs font-mono uppercase text-gray-500 tracking-wider">
          {t.statClients}
        </div>
        <div className="hidden lg:block absolute right-0 top-2 bottom-2 w-[1px] bg-white/10" />
      </div>

      <div id="stat-block-countries" className="relative lg:px-4 space-y-1">
        <div className="text-4xl md:text-5xl font-display font-bold text-[#BDF589] tracking-tight">
          {countriesCount}B
        </div>
        <div className="text-xs font-mono uppercase text-gray-500 tracking-wider">
          {t.statCountries}
        </div>
        <div className="hidden lg:block absolute right-0 top-2 bottom-2 w-[1px] bg-white/10" />
      </div>

      <div id="stat-block-websites" className="relative lg:px-4 space-y-1">
        <div className="text-4xl md:text-5xl font-display font-bold text-white tracking-tight">
          {websitesCount}%
        </div>
        <div className="text-xs font-mono uppercase text-gray-500 tracking-wider">
          {t.statWebsites}
        </div>
        <div className="hidden lg:block absolute right-0 top-2 bottom-2 w-[1px] bg-white/10" />
      </div>

      <div id="stat-block-attacks" className="lg:pl-4 space-y-1">
        <div className="text-4xl md:text-5xl font-display font-bold text-[#636EB4] tracking-tight">
          {attacksCount}
        </div>
        <div className="text-xs font-mono uppercase text-gray-500 tracking-wider">
          {t.statAttacks}
        </div>
      </div>
    </section>
  );
}
