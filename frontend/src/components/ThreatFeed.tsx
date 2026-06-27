import React from "react";
import { Activity } from "lucide-react";
import { ThreatLog, TranslationDict } from "../types";
import { threatTypes } from "../translations";

interface ThreatFeedProps {
  threatLogs: ThreatLog[];
  t: TranslationDict;
}

export default function ThreatFeed({ threatLogs, t }: ThreatFeedProps) {
  return (
    <section id="live-threat-feed" className="mt-16 max-w-4xl mx-auto">
      <div className="border border-white/10 rounded-2xl bg-[#0F0F0F]/90 backdrop-blur-md overflow-hidden">
        
        <div className="px-6 py-4 bg-white/[0.02] border-b border-white/5 flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <div className="w-2.5 h-2.5 rounded-full bg-red-500 animate-pulse" />
            <span className="font-mono text-xs font-semibold tracking-widest uppercase text-gray-300">
              {t.liveFeedTitle}
            </span>
          </div>
          <div className="flex items-center space-x-1.5 text-gray-500">
            <Activity className="w-3.5 h-3.5 text-[#BDF589]" />
            <span className="font-mono text-[10px]">LIVE BROADCAST</span>
          </div>
        </div>

        <div className="p-4 bg-[#0F0F0F] border-b border-white/5 text-xs text-gray-400 font-sans">
          {t.liveFeedDesc}
        </div>

        <div className="p-6 font-mono text-xs h-[240px] overflow-y-auto space-y-2.5 scrollbar-thin scrollbar-thumb-white/10">
          {threatLogs.map((log) => {
            const config = threatTypes.find(item => item.type === log.type) || threatTypes[4];
            return (
              <div key={log.id} className="flex items-start justify-between border-b border-white/[0.02] pb-1.5 last:border-0 hover:bg-white/[0.01] px-2 py-1 rounded transition-colors">
                <div className="flex items-center space-x-3">
                  <span className="text-gray-600">[{log.time}]</span>
                  <span className="px-1.5 py-0.5 rounded bg-white/5 text-gray-400 text-[10px]">{log.country}</span>
                  <span className={`${config.color} font-medium`}>{config.label}</span>
                </div>
                <div className="flex items-center space-x-4">
                  <span className="text-gray-500 text-[11px] hidden sm:inline">{log.target}</span>
                  <span className={`text-[10px] px-2 py-0.5 rounded font-semibold ${
                    log.type === "reject" ? "bg-red-500/10 text-red-400" : "bg-emerald-500/10 text-emerald-400"
                  }`}>
                    {log.status}
                  </span>
                </div>
              </div>
            );
          })}
        </div>

      </div>
    </section>
  );
}
