import React from "react";

// Terminal-style card matching the live-feed / modal aesthetic.
export default function Panel({
  title,
  icon,
  right,
  children,
}: {
  title: string;
  icon?: React.ReactNode;
  right?: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <div className="border border-white/10 rounded-2xl bg-[#0F0F0F]/90 backdrop-blur-md overflow-hidden">
      <div className="px-6 py-4 bg-white/[0.02] border-b border-white/5 flex items-center justify-between">
        <div className="flex items-center space-x-2">
          {icon}
          <span className="font-mono text-xs font-semibold tracking-widest uppercase text-gray-300">{title}</span>
        </div>
        {right}
      </div>
      <div className="p-6">{children}</div>
    </div>
  );
}

// A labelled monospace key/value row used to display hashes/addresses.
export function Field({ label, value, accent }: { label: string; value?: string; accent?: boolean }) {
  return (
    <div className="space-y-1">
      <div className="text-[10px] font-mono uppercase tracking-wider text-gray-500">{label}</div>
      <div className={`font-mono text-xs break-all ${accent ? "text-[#BDF589]" : "text-gray-300"}`}>{value || "—"}</div>
    </div>
  );
}
