import React, { FormEvent } from "react";
import { 
  X, 
  Terminal, 
  RefreshCw, 
  Key, 
  CheckCircle, 
  Copy, 
  Cpu, 
  Shield, 
  Database, 
  Send 
} from "lucide-react";
import { TranslationDict } from "../types";

interface RegisterModalProps {
  regName: string;
  setRegName: (val: string) => void;
  regEmail: string;
  setRegEmail: (val: string) => void;
  regSuccess: boolean;
  setRegSuccess: (val: boolean) => void;
  generatedKey: string;
  isGenerating: boolean;
  copied: boolean;
  handleRegister: (e: FormEvent) => void;
  copyKeyToClipboard: () => void;
  setRegisterModalOpen: (val: boolean) => void;
  playBeep: (freq?: number, duration?: number, type?: OscillatorType) => void;
  t: TranslationDict;
}

export function RegisterModal({
  regName,
  setRegName,
  regEmail,
  setRegEmail,
  regSuccess,
  setRegSuccess,
  generatedKey,
  isGenerating,
  copied,
  handleRegister,
  copyKeyToClipboard,
  setRegisterModalOpen,
  playBeep,
  t,
}: RegisterModalProps) {
  return (
    <div id="register-access-modal" className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm">
      <div className="w-full max-w-lg border border-white/10 rounded-2xl bg-[#0F0F0F] overflow-hidden shadow-2xl relative">
        
        <div className="px-6 py-4 bg-white/[0.02] border-b border-white/5 flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <Terminal className="w-4 h-4 text-[#BDF589]" />
            <span className="font-mono text-xs font-semibold tracking-wider text-gray-300">
              {t.terminalTitle}
            </span>
          </div>
          <button 
            onClick={() => {
              setRegisterModalOpen(false);
              setRegSuccess(false);
              playBeep(400, 0.1, "sine");
            }}
            className="p-1 hover:bg-white/5 rounded text-gray-400 hover:text-white"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="p-6 space-y-4">
          <p className="text-xs text-gray-400 leading-relaxed font-mono">
            {t.terminalSub}
          </p>

          {!regSuccess ? (
            <form onSubmit={handleRegister} className="space-y-4 pt-2">
              <div className="space-y-1.5">
                <label className="block text-xs font-mono text-gray-500 uppercase">LC Reference</label>
                <input
                  type="text"
                  required
                  value={regName}
                  onChange={(e) => setRegName(e.target.value)}
                  placeholder="e.g. LC-1001"
                  className="w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-[#BDF589] transition-all font-mono"
                />
              </div>
              <div className="space-y-1.5">
                <label className="block text-xs font-mono text-gray-500 uppercase">Seller / Beneficiary</label>
                <input
                  type="text"
                  required
                  value={regEmail}
                  onChange={(e) => setRegEmail(e.target.value)}
                  placeholder="e.g. Shenzhen Optics Co"
                  className="w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-[#BDF589] transition-all font-mono"
                />
              </div>

              <button
                type="submit"
                disabled={isGenerating}
                className="w-full py-4 bg-gradient-to-r from-[#BDF589] to-[#636EB4] text-black font-mono font-bold tracking-wider uppercase rounded-xl hover:opacity-90 disabled:opacity-50 flex items-center justify-center space-x-2"
              >
                {isGenerating ? (
                  <>
                    <RefreshCw className="w-4 h-4 animate-spin text-black" />
                    <span>PROVING IN ZKVM...</span>
                  </>
                ) : (
                  <>
                    <Key className="w-4 h-4 text-black" />
                    <span>GENERATE GROTH16 PROOF</span>
                  </>
                )}
              </button>
            </form>
          ) : (
            <div className="space-y-4 pt-2 text-center">
              <div className="inline-flex p-3 rounded-full bg-emerald-500/10 mb-2">
                <CheckCircle className="w-8 h-8 text-emerald-400" />
              </div>
              <p className="text-sm font-mono text-emerald-400">
                {t.registerSuccess}
              </p>

              <div className="bg-white/5 border border-white/10 rounded-xl p-4 flex items-center justify-between font-mono text-sm">
                <span className="text-gray-300 font-bold select-all tracking-wider">{generatedKey}</span>
                <button
                  onClick={copyKeyToClipboard}
                  className="p-2 hover:bg-white/5 rounded-lg text-gray-400 hover:text-white"
                  title="Copy Key"
                >
                  {copied ? (
                    <span className="text-xs text-emerald-400 font-semibold">COPIED</span>
                  ) : (
                    <Copy className="w-4 h-4" />
                  )}
                </button>
              </div>

              <p className="text-[10px] text-gray-500 font-mono italic">
                This seal is verified on-chain by the RISC Zero VerifierRouter before the escrow releases any funds.
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

interface DeployModalProps {
  deployZone: string;
  setDeployZone: (val: string) => void;
  deployLevel: string;
  setDeployLevel: (val: string) => void;
  isDeploying: boolean;
  deployProgress: number;
  deployLogs: string[];
  handleStartDeployment: () => void;
  setStartedModalOpen: (val: boolean) => void;
  setIsDeploying: (val: boolean) => void;
  playBeep: (freq?: number, duration?: number, type?: OscillatorType) => void;
  t: TranslationDict;
}

export function DeployModal({
  deployZone,
  setDeployZone,
  deployLevel,
  setDeployLevel,
  isDeploying,
  deployProgress,
  deployLogs,
  handleStartDeployment,
  setStartedModalOpen,
  setIsDeploying,
  playBeep,
  t,
}: DeployModalProps) {
  return (
    <div id="get-started-deploy-modal" className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm">
      <div className="w-full max-w-lg border border-white/10 rounded-2xl bg-[#0F0F0F] overflow-hidden shadow-2xl relative">
        
        <div className="px-6 py-4 bg-white/[0.02] border-b border-white/5 flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <Cpu className="w-4 h-4 text-[#BDF589]" />
            <span className="font-mono text-xs font-semibold tracking-wider text-gray-300">
              {t.provisionTitle}
            </span>
          </div>
          <button 
            onClick={() => {
              setStartedModalOpen(false);
              setIsDeploying(false);
              playBeep(400, 0.1, "sine");
            }}
            className="p-1 hover:bg-white/5 rounded text-gray-400 hover:text-white"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="p-6 space-y-4">
          <p className="text-xs text-gray-400 leading-relaxed font-mono">
            {t.provisionDesc}
          </p>

          {!isDeploying ? (
            <div className="space-y-4">
              <div className="space-y-1.5">
                <label className="block text-xs font-mono text-gray-500 uppercase">Target Network</label>
                <select
                  value={deployZone}
                  onChange={(e) => setDeployZone(e.target.value)}
                  className="w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-[#BDF589] font-mono text-white [&>option]:bg-[#0F0F0F]"
                >
                  <option value="Stellar Testnet">Stellar Testnet</option>
                  <option value="Futurenet">Futurenet</option>
                  <option value="Localnet">Localnet (Quickstart)</option>
                  <option value="Stellar Mainnet">Stellar Mainnet</option>
                </select>
              </div>

              <div className="space-y-1.5">
                <label className="block text-xs font-mono text-gray-500 uppercase">Settlement Asset</label>
                <div className="grid grid-cols-3 gap-3">
                  {["USDC", "EURC", "XLM"].map((level) => (
                    <button
                      key={level}
                      onClick={() => {
                        setDeployLevel(level);
                        playBeep(400, 0.05, "sine");
                      }}
                      className={`py-2 rounded-xl border text-xs font-mono uppercase transition-all ${
                        deployLevel === level
                          ? "border-[#BDF589] bg-[#BDF589]/10 text-[#BDF589]"
                          : "border-white/10 text-gray-400 hover:text-white"
                      }`}
                    >
                      {level}
                    </button>
                  ))}
                </div>
              </div>

              <button
                onClick={handleStartDeployment}
                className="w-full py-4 bg-[#BDF589] text-black font-mono font-bold tracking-wider uppercase rounded-xl hover:bg-opacity-90 flex items-center justify-center space-x-2"
              >
                <Shield className="w-4 h-4 text-black" />
                <span>DEPLOY ESCROW CONTRACT</span>
              </button>
            </div>
          ) : (
            <div className="space-y-4 font-mono">
              <div className="space-y-1.5">
                <div className="flex justify-between text-xs">
                  <span className="text-gray-400">PROVISION PROGRESS</span>
                  <span className="text-[#BDF589] font-bold">{deployProgress}%</span>
                </div>
                <div className="w-full h-2 bg-white/5 rounded-full overflow-hidden">
                  <div 
                    className="h-full bg-gradient-to-r from-[#BDF589] to-[#636EB4] transition-all duration-300"
                    style={{ width: `${deployProgress}%` }}
                  />
                </div>
              </div>

              <div className="h-[150px] bg-black/50 p-4 border border-white/5 rounded-xl text-[10px] space-y-1.5 overflow-y-auto scrollbar-thin">
                {deployLogs.map((logStr, idx) => (
                  <div key={idx} className="text-gray-300">
                    {logStr}
                  </div>
                ))}
              </div>
            </div>
          )}

        </div>
      </div>
    </div>
  );
}

interface ContactModalProps {
  contactEmail: string;
  setContactEmail: (val: string) => void;
  contactMsg: string;
  setContactMsg: (val: string) => void;
  contactSuccess: boolean;
  handleContactSubmit: (e: FormEvent) => void;
  setContactModalOpen: (val: boolean) => void;
  playBeep: (freq?: number, duration?: number, type?: OscillatorType) => void;
}

export function ContactModal({
  contactEmail,
  setContactEmail,
  contactMsg,
  setContactMsg,
  contactSuccess,
  handleContactSubmit,
  setContactModalOpen,
  playBeep,
}: ContactModalProps) {
  return (
    <div id="contact-form-modal" className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-sm">
      <div className="w-full max-w-lg border border-white/10 rounded-2xl bg-[#0F0F0F] overflow-hidden shadow-2xl relative">
        
        <div className="px-6 py-4 bg-white/[0.02] border-b border-white/5 flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <Database className="w-4 h-4 text-[#BDF589]" />
            <span className="font-mono text-xs font-semibold tracking-wider text-gray-300">
              CONTACT THE BUILD TEAM
            </span>
          </div>
          <button 
            onClick={() => {
              setContactModalOpen(false);
              playBeep(400, 0.1, "sine");
            }}
            className="p-1 hover:bg-white/5 rounded text-gray-400 hover:text-white"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        <div className="p-6 space-y-4">
          <p className="text-xs text-gray-400 leading-relaxed font-mono">
            Questions about integrating zero-knowledge Letter-of-Credit settlement? Send a note and the Bill of Zero team will get back to you.
          </p>

          {!contactSuccess ? (
            <form onSubmit={handleContactSubmit} className="space-y-4 pt-2">
              <div className="space-y-1.5">
                <label className="block text-xs font-mono text-gray-500 uppercase">Email</label>
                <input
                  type="email"
                  required
                  value={contactEmail}
                  onChange={(e) => setContactEmail(e.target.value)}
                  placeholder="e.g. treasury@acme.com"
                  className="w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-[#BDF589] transition-all font-mono"
                />
              </div>
              <div className="space-y-1.5">
                <label className="block text-xs font-mono text-gray-500 uppercase">Message</label>
                <textarea
                  required
                  rows={4}
                  value={contactMsg}
                  onChange={(e) => setContactMsg(e.target.value)}
                  placeholder="Tell us about your trade-finance use case, LC volume, and timeline..."
                  className="w-full bg-white/5 border border-white/10 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-[#BDF589] transition-all font-mono resize-none"
                />
              </div>

              <button
                type="submit"
                className="w-full py-4 bg-gradient-to-r from-[#BDF589] to-[#636EB4] text-black font-mono font-bold tracking-wider uppercase rounded-xl hover:opacity-90 flex items-center justify-center space-x-2"
              >
                <Send className="w-4 h-4 text-black" />
                <span>SEND MESSAGE</span>
              </button>
            </form>
          ) : (
            <div className="space-y-4 pt-2 text-center font-mono">
              <div className="inline-flex p-3 rounded-full bg-emerald-500/10 mb-2">
                <CheckCircle className="w-8 h-8 text-emerald-400" />
              </div>
              <p className="text-sm text-emerald-400 uppercase tracking-widest font-bold">
                Message Sent
              </p>
              <p className="text-xs text-gray-400">
                Thanks for reaching out. The Bill of Zero team will reply to your email shortly.
              </p>
            </div>
          )}

        </div>
      </div>
    </div>
  );
}
