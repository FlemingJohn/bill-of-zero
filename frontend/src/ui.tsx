import React, { createContext, useContext, useRef, useState } from "react";

type Ui = {
  muted: boolean;
  setMuted: React.Dispatch<React.SetStateAction<boolean>>;
  playBeep: (freq?: number, duration?: number, type?: OscillatorType) => void;
};

const UiContext = createContext<Ui | null>(null);

export function UiProvider({ children }: { children: React.ReactNode }) {
  const [muted, setMuted] = useState(true);
  const mutedRef = useRef(true);
  mutedRef.current = muted;

  const playBeep = (freq = 800, duration = 0.08, type: OscillatorType = "sine") => {
    if (mutedRef.current) return;
    try {
      const ctx = new (window.AudioContext || (window as any).webkitAudioContext)();
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();
      osc.type = type;
      osc.frequency.setValueAtTime(freq, ctx.currentTime);
      gain.gain.setValueAtTime(0.04, ctx.currentTime);
      gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + duration);
      osc.connect(gain);
      gain.connect(ctx.destination);
      osc.start();
      osc.stop(ctx.currentTime + duration);
    } catch {
      /* AudioContext needs a user gesture; ignore until then */
    }
  };

  return <UiContext.Provider value={{ muted, setMuted, playBeep }}>{children}</UiContext.Provider>;
}

export function useUi(): Ui {
  const ctx = useContext(UiContext);
  if (!ctx) throw new Error("useUi must be used within UiProvider");
  return ctx;
}
