import React from "react";
import { Shield } from "lucide-react";

export default function Footer() {
  return (
    <footer id="footer-section" className="border-t border-white/10 bg-[#0A0A0A] py-12 relative z-10">
      <div className="max-w-7xl mx-auto px-6 flex flex-col md:flex-row items-center justify-between space-y-6 md:space-y-0">
        <div className="flex items-center space-x-3 text-gray-400">
          <Shield className="w-5 h-5 text-[#BDF589]" />
          <span className="font-mono text-xs tracking-wider">
            © {new Date().getFullYear()} Bill of Zero. ZK Letter-of-Credit settlement on Stellar.
          </span>
        </div>
        <div className="flex items-center space-x-6 text-xs font-mono text-gray-500">
          <a href="#architecture" className="hover:text-[#BDF589]">Whitepaper</a>
          <a href="#contracts" className="hover:text-[#BDF589]">GitHub</a>
          <a href="#status" className="hover:text-emerald-400 flex items-center space-x-1.5">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-ping" />
            <span>Testnet Live</span>
          </a>
        </div>
      </div>
    </footer>
  );
}
