import React, { useEffect, useRef, useState } from "react";
import { Link, NavLink, Outlet } from "react-router-dom";
import { Shield, Volume2, VolumeX, Wallet } from "lucide-react";
import Footer from "./Footer";
import { useUi } from "../ui";
import { connectWallet, getConnectedAddress } from "../lib/stellar";

const short = (a: string) => (a ? `${a.slice(0, 4)}…${a.slice(-4)}` : "");

const navItems = [
  { to: "/", label: "Home", end: true },
  { to: "/prove", label: "Prove" },
  { to: "/escrow", label: "Escrow" },
  { to: "/audit", label: "Audit" },
];

export default function Layout() {
  const { muted, setMuted, playBeep } = useUi();
  const [mousePos, setMousePos] = useState({ x: 0, y: 0 });
  const [addr, setAddr] = useState("");
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    getConnectedAddress().then(setAddr).catch(() => {});
    const onMove = (e: MouseEvent) => {
      const rect = containerRef.current?.getBoundingClientRect();
      if (rect) setMousePos({ x: e.clientX - rect.left, y: e.clientY - rect.top });
    };
    window.addEventListener("mousemove", onMove);
    return () => window.removeEventListener("mousemove", onMove);
  }, []);

  const onConnect = async () => {
    playBeep(900, 0.12, "sine");
    try {
      setAddr(await connectWallet());
    } catch (e) {
      alert("Could not connect Freighter. Is the extension installed and set to Testnet?");
    }
  };

  return (
    <div
      ref={containerRef}
      className="relative min-h-screen bg-[#0F0F0F] text-white font-sans overflow-x-hidden cyber-grid selection:bg-[#BDF589] selection:text-black"
    >
      {/* Ambient glow blobs */}
      <div className="pointer-events-none absolute inset-0 overflow-hidden">
        <div
          className="absolute w-[450px] h-[450px] rounded-full bg-[#BDF589] opacity-[0.06] blur-[120px] transition-transform duration-300 ease-out animate-pulse-glow"
          style={{ transform: `translate(${mousePos.x - 225}px, ${mousePos.y - 225}px)` }}
        />
        <div className="absolute top-[10%] right-[5%] w-[400px] h-[400px] rounded-full bg-[#636EB4] opacity-[0.12] blur-[100px]" />
        <div className="absolute bottom-[20%] left-[-5%] w-[450px] h-[450px] rounded-full bg-[#E43D3D] opacity-[0.08] blur-[130px] animate-pulse-glow" />
      </div>

      <div className="pointer-events-none absolute inset-0 z-0">
        <div className="absolute left-20 top-0 bottom-0 w-[1px] bg-white/5" />
        <div className="absolute right-20 top-0 bottom-0 w-[1px] bg-white/5" />
        <div className="absolute top-20 left-0 right-0 h-[1px] bg-white/5" />
      </div>

      {/* Header */}
      <header className="relative z-20 border-b border-white/10 bg-[#0F0F0F]/80 backdrop-blur-md">
        <div className="max-w-7xl mx-auto px-6 h-20 flex items-center justify-between">
          <Link to="/" className="flex items-center space-x-3 cursor-pointer group" onClick={() => playBeep(523.25, 0.1)}>
            <div className="relative w-8 h-8 rounded-lg bg-gradient-to-tr from-[#BDF589] to-[#636EB4] p-[1.5px] transition-transform group-hover:scale-110">
              <div className="w-full h-full bg-[#0F0F0F] rounded-[7px] flex items-center justify-center">
                <Shield className="w-4 h-4 text-[#BDF589]" />
              </div>
            </div>
            <span className="font-display font-bold tracking-tight text-lg text-white">
              Bill of <span className="text-[#BDF589]">Zero</span>
            </span>
          </Link>

          <nav className="hidden md:flex items-center space-x-1">
            {navItems.map((item, idx) => (
              <NavLink
                key={item.to}
                to={item.to}
                end={item.end}
                onMouseEnter={() => playBeep(800 + idx * 50, 0.03)}
                className={({ isActive }) =>
                  `px-4 py-2 text-sm font-medium tracking-wide transition-colors duration-200 ${
                    isActive ? "text-[#BDF589]" : "text-gray-400 hover:text-[#BDF589]"
                  }`
                }
              >
                {item.label}
              </NavLink>
            ))}
          </nav>

          <div className="flex items-center space-x-3">
            <button
              onClick={() => {
                setMuted((m) => !m);
                if (muted) setTimeout(() => playBeep(600, 0.1), 50);
              }}
              className={`p-2 rounded-lg border transition-all ${
                !muted ? "border-[#BDF589]/30 text-[#BDF589] bg-[#BDF589]/5" : "border-white/10 text-gray-400 hover:text-white"
              }`}
              title={muted ? "Unmute" : "Mute"}
            >
              {muted ? <VolumeX className="w-4 h-4" /> : <Volume2 className="w-4 h-4" />}
            </button>
            <button
              onClick={onConnect}
              className="px-4 py-2 rounded-lg border border-white/10 text-sm font-mono tracking-wide hover:border-[#BDF589] hover:text-[#BDF589] transition-all duration-200 flex items-center space-x-2"
            >
              <Wallet className="w-3.5 h-3.5" />
              <span>{addr ? short(addr) : "Connect Wallet"}</span>
            </button>
          </div>
        </div>
      </header>

      <main className="relative z-10 max-w-7xl mx-auto px-6 pt-12 pb-24">
        <Outlet context={{ addr, setAddr }} />
      </main>

      <Footer />
    </div>
  );
}
