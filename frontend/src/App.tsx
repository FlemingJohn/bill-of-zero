import React, { useState, useEffect, useRef, FormEvent } from "react";
import { ThreatLog } from "./types";
import { translations, threatTypes, targetClusters, countriesList } from "./translations";

import Header from "./components/Header";
import Hero from "./components/Hero";
import Stats from "./components/Stats";
import ThreatFeed from "./components/ThreatFeed";
import Footer from "./components/Footer";
import { RegisterModal, DeployModal, ContactModal } from "./components/Modals";

export default function App() {
  const [lang, setLang] = useState<"en" | "fr">("en");
  const [muted, setMuted] = useState(true);
  const [mobileMenuOpen, setMenuOpen] = useState(false);
  const [mousePos, setMousePos] = useState({ x: 0, y: 0 });
  const containerRef = useRef<HTMLDivElement>(null);

  const [registerModalOpen, setRegisterModalOpen] = useState(false);
  const [startedModalOpen, setStartedModalOpen] = useState(false);
  const [contactModalOpen, setContactModalOpen] = useState(false);

  const [regName, setRegName] = useState("");
  const [regEmail, setRegEmail] = useState("");
  const [regSuccess, setRegSuccess] = useState(false);
  const [generatedKey, setGeneratedKey] = useState("");
  const [isGenerating, setIsGenerating] = useState(false);
  const [copied, setCopied] = useState(false);

  const [deployZone, setDeployZone] = useState("Stellar Testnet");
  const [deployLevel, setDeployLevel] = useState("USDC");
  const [isDeploying, setIsDeploying] = useState(false);
  const [deployProgress, setDeployProgress] = useState(0);
  const [deployLogs, setDeployLogs] = useState<string[]>([]);

  const [contactEmail, setContactEmail] = useState("");
  const [contactMsg, setContactMsg] = useState("");
  const [contactSuccess, setContactSuccess] = useState(false);

  const [clients, setClients] = useState(0);
  const [countriesCount, setCountriesCount] = useState(0);
  const [websitesCount, setWebsitesCount] = useState(0);
  const [attacksCount, setAttacksCount] = useState(0);

  const [threatLogs, setThreatLogs] = useState<ThreatLog[]>([
    { id: "1", time: "11:27:01", type: "release", target: "LC-1001", status: "FUNDS SETTLED", country: "SG" },
    { id: "2", time: "11:26:48", type: "verify", target: "LC-1042-ACME", status: "VERIFIER OK", country: "DE" },
    { id: "3", time: "11:26:30", type: "proof", target: "BoL-SH-2207", status: "SEAL 73c457ba", country: "JP" }
  ]);

  const playBeep = (freq = 800, duration = 0.08, type: OscillatorType = "sine") => {
    if (muted) return;
    try {
      const audioCtx = new (window.AudioContext || (window as any).webkitAudioContext)();
      const osc = audioCtx.createOscillator();
      const gainNode = audioCtx.createGain();
      
      osc.type = type;
      osc.frequency.setValueAtTime(freq, audioCtx.currentTime);
      
      gainNode.gain.setValueAtTime(0.04, audioCtx.currentTime);
      gainNode.gain.exponentialRampToValueAtTime(0.001, audioCtx.currentTime + duration);
      
      osc.connect(gainNode);
      gainNode.connect(audioCtx.destination);
      
      osc.start();
      osc.stop(audioCtx.currentTime + duration);
    } catch (e) {
    }
  };

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (containerRef.current) {
        const rect = containerRef.current.getBoundingClientRect();
        setMousePos({
          x: e.clientX - rect.left,
          y: e.clientY - rect.top,
        });
      }
    };
    window.addEventListener("mousemove", handleMouseMove);
    return () => window.removeEventListener("mousemove", handleMouseMove);
  }, []);

  useEffect(() => {
    let step = 0;
    const duration = 2000;
    const intervalTime = 25;
    const totalSteps = duration / intervalTime;
    
    const counterTimer = setInterval(() => {
      step += 1;
      setClients(Math.min(Math.floor((step / totalSteps) * 8), 8));
      setCountriesCount(Math.min(Math.floor((step / totalSteps) * 80), 80));
      setWebsitesCount(Math.min(Math.floor((step / totalSteps) * 100), 100));
      setAttacksCount(Math.min(Math.floor((step / totalSteps) * 3), 3));
      
      if (step >= totalSteps) {
        clearInterval(counterTimer);
      }
    }, intervalTime);
    
    return () => clearInterval(counterTimer);
  }, []);

  useEffect(() => {
    const logInterval = setInterval(() => {
      const now = new Date();
      const timeStr = now.toTimeString().split(" ")[0];
      const randomThreat = threatTypes[Math.floor(Math.random() * threatTypes.length)];
      const randomTarget = targetClusters[Math.floor(Math.random() * targetClusters.length)];
      const randomCountry = countriesList[Math.floor(Math.random() * countriesList.length)];

      const newLog: ThreatLog = {
        id: Math.random().toString(),
        time: timeStr,
        type: randomThreat.type,
        target: randomTarget,
        status: randomThreat.status,
        country: randomCountry
      };

      setThreatLogs(prev => [newLog, ...prev.slice(0, 11)]);
      
      if (randomThreat.type === "release" || randomThreat.type === "verify") {
        playBeep(900, 0.05, "sine");
      } else {
        playBeep(450, 0.07, "triangle");
      }
    }, 4000);

    return () => clearInterval(logInterval);
  }, [muted]);

  const copyKeyToClipboard = () => {
    navigator.clipboard.writeText(generatedKey);
    setCopied(true);
    playBeep(1200, 0.15, "sine");
    setTimeout(() => setCopied(false), 2000);
  };

  const handleRegister = (e: FormEvent) => {
    e.preventDefault();
    if (!regName || !regEmail) return;

    setIsGenerating(true);
    playBeep(350, 0.1, "sine");
    
    let currentBeep = 0;
    const beepTimer = setInterval(() => {
      currentBeep++;
      playBeep(400 + (currentBeep * 100), 0.04, "square");
      if (currentBeep > 5) clearInterval(beepTimer);
    }, 150);

    setTimeout(() => {
      // Mock a Groth16 seal: the real on-chain selector (73c457ba) + hex bytes.
      const hex = "0123456789abcdef";
      let seal = "73c457ba";
      for (let i = 0; i < 24; i++) {
        seal += hex.charAt(Math.floor(Math.random() * hex.length));
      }
      setGeneratedKey("0x" + seal + "…");
      setIsGenerating(false);
      setRegSuccess(true);
      playBeep(880, 0.25, "sine");
    }, 1800);
  };

  const handleStartDeployment = () => {
    setIsDeploying(true);
    setDeployProgress(0);
    setDeployLogs(["[INFO] Connecting to " + deployZone + " RPC..."]);
    playBeep(600, 0.1, "sine");
  };

  useEffect(() => {
    if (!isDeploying) return;

    const deploymentSteps = [
      { prg: 15, msg: "[INFO] Building escrow.wasm (soroban-sdk 25, wasm32v1-none)..." },
      { prg: 30, msg: `[OK] Uploaded Wasm hash to ${deployZone}.` },
      { prg: 45, msg: "[INFO] Instantiating escrow bound to LC terms_digest..." },
      { prg: 65, msg: "[INFO] Pinning RISC Zero image_id + VerifierRouter address..." },
      { prg: 80, msg: "[OK] init(lc_id, terms_digest, image_id, router, token, seller)" },
      { prg: 95, msg: `[INFO] Funding escrow with ${deployLevel}...` },
      { prg: 100, msg: `[COMPLETE] Escrow live on ${deployZone}. Awaiting proof to release.` }
    ];

    let currentStep = 0;
    const timer = setInterval(() => {
      if (currentStep < deploymentSteps.length) {
        const stepData = deploymentSteps[currentStep];
        setDeployProgress(stepData.prg);
        setDeployLogs(prev => [...prev, stepData.msg]);
        playBeep(500 + (stepData.prg * 4), 0.08, "triangle");
        currentStep++;
      } else {
        clearInterval(timer);
        setTimeout(() => {
          setIsDeploying(false);
          setStartedModalOpen(false);
          playBeep(1000, 0.3, "sine");
        }, 1500);
      }
    }, 900);

    return () => clearInterval(timer);
  }, [isDeploying, deployZone, deployLevel]);

  const handleContactSubmit = (e: FormEvent) => {
    e.preventDefault();
    if (!contactEmail || !contactMsg) return;
    playBeep(800, 0.1, "sine");
    setContactSuccess(true);
    setTimeout(() => {
      setContactSuccess(false);
      setContactEmail("");
      setContactMsg("");
      setContactModalOpen(false);
    }, 3000);
  };

  const t = translations[lang];

  return (
    <div 
      id="main-cyber-root"
      ref={containerRef}
      className="relative min-h-screen bg-[#0F0F0F] text-white font-sans overflow-x-hidden cyber-grid selection:bg-[#BDF589] selection:text-black"
    >
      <div 
        id="bg-ambient-glowing-blobs"
        className="pointer-events-none absolute inset-0 overflow-hidden"
      >
        <div 
          className="absolute w-[450px] h-[450px] rounded-full bg-[#BDF589] opacity-[0.06] blur-[120px] transition-transform duration-300 ease-out animate-pulse-glow"
          style={{
            transform: `translate(${mousePos.x - 225}px, ${mousePos.y - 225}px)`,
          }}
        />
        <div className="absolute top-[10%] right-[5%] w-[400px] h-[400px] rounded-full bg-[#636EB4] opacity-[0.12] blur-[100px]" />
        <div className="absolute bottom-[20%] left-[-5%] w-[450px] h-[450px] rounded-full bg-[#E43D3D] opacity-[0.08] blur-[130px] animate-pulse-glow" />
      </div>

      <div id="grid-crosshairs" className="pointer-events-none absolute inset-0 z-0">
        <div className="absolute left-20 top-0 bottom-0 w-[1px] bg-white/5 border-dashed" />
        <div className="absolute right-20 top-0 bottom-0 w-[1px] bg-white/5 border-dashed" />
        <div className="absolute top-20 left-0 right-0 h-[1px] bg-white/5 border-dashed" />
      </div>

      <Header 
        lang={lang}
        setLang={setLang}
        muted={muted}
        setMuted={setMuted}
        mobileMenuOpen={mobileMenuOpen}
        setMenuOpen={setMenuOpen}
        setContactModalOpen={setContactModalOpen}
        playBeep={playBeep}
        t={t}
      />

      <main id="main-content-area" className="relative z-10 max-w-7xl mx-auto px-6 pt-12 pb-24">
        <Hero 
          t={t}
          setRegisterModalOpen={setRegisterModalOpen}
          setStartedModalOpen={setStartedModalOpen}
          playBeep={playBeep}
        />

        <Stats 
          clients={clients}
          countriesCount={countriesCount}
          websitesCount={websitesCount}
          attacksCount={attacksCount}
          t={t}
        />

        <ThreatFeed 
          threatLogs={threatLogs}
          t={t}
        />
      </main>

      <Footer />

      {registerModalOpen && (
        <RegisterModal 
          regName={regName}
          setRegName={setRegName}
          regEmail={regEmail}
          setRegEmail={setRegEmail}
          regSuccess={regSuccess}
          setRegSuccess={setRegSuccess}
          generatedKey={generatedKey}
          isGenerating={isGenerating}
          copied={copied}
          handleRegister={handleRegister}
          copyKeyToClipboard={copyKeyToClipboard}
          setRegisterModalOpen={setRegisterModalOpen}
          playBeep={playBeep}
          t={t}
        />
      )}

      {startedModalOpen && (
        <DeployModal 
          deployZone={deployZone}
          setDeployZone={setDeployZone}
          deployLevel={deployLevel}
          setDeployLevel={setDeployLevel}
          isDeploying={isDeploying}
          deployProgress={deployProgress}
          deployLogs={deployLogs}
          handleStartDeployment={handleStartDeployment}
          setStartedModalOpen={setStartedModalOpen}
          setIsDeploying={setIsDeploying}
          playBeep={playBeep}
          t={t}
        />
      )}

      {contactModalOpen && (
        <ContactModal 
          contactEmail={contactEmail}
          setContactEmail={setContactEmail}
          contactMsg={contactMsg}
          setContactMsg={setContactMsg}
          contactSuccess={contactSuccess}
          handleContactSubmit={handleContactSubmit}
          setContactModalOpen={setContactModalOpen}
          playBeep={playBeep}
        />
      )}
    </div>
  );
}
