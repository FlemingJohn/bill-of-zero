import React from "react";
import { ArrowRight } from "lucide-react";
import { TranslationDict } from "../types";

interface HeroProps {
  t: TranslationDict;
  setRegisterModalOpen: (val: boolean) => void;
  setStartedModalOpen: (val: boolean) => void;
  playBeep: (freq?: number, duration?: number, type?: OscillatorType) => void;
}

export default function Hero({
  t,
  setRegisterModalOpen,
  setStartedModalOpen,
  playBeep,
}: HeroProps) {
  return (
    <section id="hero-presentation-section" className="text-center max-w-4xl mx-auto mt-6 md:mt-16">
      <div 
        id="hero-tagline-badge"
        className="inline-flex items-center space-x-2 px-3 py-1.5 rounded-full bg-white/5 border border-white/10 mb-8 animate-pulse-glow"
      >
        <span className="w-1.5 h-1.5 rounded-full bg-[#BDF589] shadow-[0_0_8px_#BDF589]" />
        <span className="text-xs font-mono tracking-wider text-gray-300 uppercase">
          {t.badge}
        </span>
      </div>

      <h1 
        id="hero-main-title"
        className="font-display text-5xl md:text-8xl font-bold tracking-tighter leading-none mb-6"
      >
        {t.titleMain}
        <span className="relative inline-block px-4 text-transparent bg-clip-text bg-gradient-to-r from-[#BDF589] to-[#636EB4] ml-1">
          {t.titleSub}
          <span className="absolute -top-2 -right-1 text-[10px] text-white/20 font-mono select-none">+</span>
        </span>
      </h1>

      <p 
        id="hero-desc-paragraph"
        className="text-base md:text-lg text-gray-400 font-normal leading-relaxed max-w-2xl mx-auto mb-10"
      >
        {t.desc}
      </p>

      <div id="hero-actions-container" className="flex flex-col sm:flex-row items-center justify-center space-y-4 sm:space-y-0 sm:space-x-6 mb-16">
        <button
          id="hero-register-btn"
          onClick={() => {
            setRegisterModalOpen(true);
            playBeep(900, 0.12, "sine");
          }}
          className="w-full sm:w-auto px-8 py-4 bg-white/5 hover:bg-white/10 text-white border border-white/10 hover:border-white/30 rounded-xl font-mono text-sm font-semibold tracking-wider uppercase transition-all duration-200"
        >
          {t.btnRegister}
        </button>

        <button
          id="hero-get-started-btn"
          onClick={() => {
            setStartedModalOpen(true);
            playBeep(1000, 0.15, "sine");
          }}
          className="relative w-full sm:w-auto group"
        >
          <div className="absolute -inset-1 rounded-xl bg-gradient-to-r from-[#BDF589] to-[#636EB4] opacity-70 blur-md group-hover:opacity-100 transition duration-300" />
          <div className="relative px-8 py-4 bg-[#0F0F0F] rounded-xl flex items-center justify-center space-x-2 border border-white/10 group-hover:border-[#BDF589]/50 transition-colors">
            <span className="text-[#BDF589] group-hover:text-white font-mono text-sm font-bold tracking-wider uppercase transition-colors">
              {t.btnGetStarted}
            </span>
            <ArrowRight className="w-4 h-4 text-[#BDF589] group-hover:translate-x-1 transition-transform" />
          </div>
        </button>
      </div>
    </section>
  );
}
