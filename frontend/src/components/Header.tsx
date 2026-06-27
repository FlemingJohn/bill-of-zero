import React from "react";
import { Shield, Languages, Volume2, VolumeX, Menu, X } from "lucide-react";
import { TranslationDict } from "../types";

interface HeaderProps {
  lang: "en" | "fr";
  setLang: React.Dispatch<React.SetStateAction<"en" | "fr">>;
  muted: boolean;
  setMuted: React.Dispatch<React.SetStateAction<boolean>>;
  mobileMenuOpen: boolean;
  setMenuOpen: React.Dispatch<React.SetStateAction<boolean>>;
  setContactModalOpen: React.Dispatch<React.SetStateAction<boolean>>;
  playBeep: (freq?: number, duration?: number, type?: OscillatorType) => void;
  t: TranslationDict;
}

export default function Header({
  lang,
  setLang,
  muted,
  setMuted,
  mobileMenuOpen,
  setMenuOpen,
  setContactModalOpen,
  playBeep,
  t,
}: HeaderProps) {
  return (
    <header id="header-navbar" className="relative z-20 border-b border-white/10 bg-[#0F0F0F]/80 backdrop-blur-md">
      <div className="max-w-7xl mx-auto px-6 h-20 flex items-center justify-between">
        
        <div 
          id="nav-logo" 
          className="flex items-center space-x-3 cursor-pointer group"
          onClick={() => { playBeep(523.25, 0.1, "sine"); }}
        >
          <div className="relative w-8 h-8 rounded-lg bg-gradient-to-tr from-[#BDF589] to-[#636EB4] p-[1.5px] transition-transform group-hover:scale-110">
            <div className="w-full h-full bg-[#0F0F0F] rounded-[7px] flex items-center justify-center">
              <Shield className="w-4 h-4 text-[#BDF589]" />
            </div>
          </div>
          <span className="font-display font-bold tracking-tight text-lg bg-gradient-to-r from-white to-white/70 bg-clip-text text-transparent group-hover:text-white transition-colors">
            Bill of <span className="text-[#BDF589]">Zero</span>
          </span>
        </div>

        <nav id="desktop-nav" className="hidden md:flex items-center space-x-1">
          {[
            { label: t.navProducts, id: "products" },
            { label: t.navDocuments, id: "documents" },
            { label: t.navIllustrations, id: "illustrations" },
            { label: t.navPartners, id: "partners" }
          ].map((link, idx) => (
            <a
              key={link.id}
              href={`#${link.id}`}
              id={`nav-link-${link.id}`}
              className="px-4 py-2 text-sm text-gray-400 hover:text-[#BDF589] font-medium tracking-wide transition-colors duration-200"
              onMouseEnter={() => playBeep(800 + idx * 50, 0.03, "sine")}
            >
              {link.label}
            </a>
          ))}
        </nav>

        <div id="nav-actions" className="hidden md:flex items-center space-x-4">
          
          <button
            id="lang-selector-btn"
            onClick={() => {
              setLang(prev => prev === "en" ? "fr" : "en");
              playBeep(700, 0.1, "sine");
            }}
            className="p-2 rounded-lg border border-white/10 hover:border-[#BDF589]/50 text-gray-400 hover:text-white transition-all flex items-center space-x-1.5 text-xs font-mono"
            title="Change Language"
          >
            <Languages className="w-3.5 h-3.5" />
            <span>{lang.toUpperCase()}</span>
          </button>

          <button
            id="audio-toggle-btn"
            onClick={() => {
              setMuted(prev => !prev);
              if (muted) {
                setTimeout(() => playBeep(600, 0.1, "sine"), 50);
              }
            }}
            className={`p-2 rounded-lg border transition-all ${
              !muted 
                ? "border-[#BDF589]/30 text-[#BDF589] bg-[#BDF589]/5" 
                : "border-white/10 text-gray-400 hover:text-white"
            }`}
            title={muted ? "Unmute sound effects" : "Mute sound effects"}
          >
            {muted ? <VolumeX className="w-4 h-4" /> : <Volume2 className="w-4 h-4" />}
          </button>

          <button
            id="nav-contact-btn"
            onClick={() => {
              setContactModalOpen(true);
              playBeep(650, 0.08, "sine");
            }}
            className="px-5 py-2 rounded-lg border border-white/10 text-sm font-medium tracking-wide hover:border-[#BDF589] hover:text-[#BDF589] transition-all duration-200"
          >
            {t.navContact}
          </button>
        </div>

        <div className="flex md:hidden items-center space-x-3">
          <button
            onClick={() => {
              setLang(prev => prev === "en" ? "fr" : "en");
              playBeep(700, 0.1, "sine");
            }}
            className="p-1.5 rounded border border-white/10 text-gray-400 text-xs font-mono"
          >
            {lang.toUpperCase()}
          </button>
          <button
            onClick={() => {
              setMenuOpen(!mobileMenuOpen);
              playBeep(500, 0.1, "sine");
            }}
            className="p-2 text-gray-400 hover:text-white"
            id="mobile-menu-toggle"
          >
            {mobileMenuOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
          </button>
        </div>

      </div>

      {mobileMenuOpen && (
        <div id="mobile-menu-drawer" className="md:hidden border-t border-white/5 bg-[#0F0F0F] px-6 py-4 space-y-3 z-30 relative">
          <nav className="flex flex-col space-y-2">
            <a href="#products" className="py-2 text-gray-400 text-sm">{t.navProducts}</a>
            <a href="#documents" className="py-2 text-gray-400 text-sm">{t.navDocuments}</a>
            <a href="#illustrations" className="py-2 text-gray-400 text-sm">{t.navIllustrations}</a>
            <a href="#partners" className="py-2 text-gray-400 text-sm">{t.navPartners}</a>
            <button
              onClick={() => {
                setMenuOpen(false);
                setContactModalOpen(true);
                playBeep(600, 0.1, "sine");
              }}
              className="w-full text-left py-2 text-[#BDF589] text-sm"
            >
              {t.navContact}
            </button>
          </nav>
          <div className="pt-3 border-t border-white/10 flex items-center justify-between">
            <span className="text-xs text-gray-500 font-mono">SOUND FEEDBACK</span>
            <button
              onClick={() => setMuted(!muted)}
              className="px-3 py-1 text-xs rounded border border-white/10 text-gray-300"
            >
              {muted ? "MUTED" : "ACTIVE"}
            </button>
          </div>
        </div>
      )}
    </header>
  );
}
