import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import Hero from "../components/Hero";
import Stats from "../components/Stats";
import ThreatFeed from "../components/ThreatFeed";
import { translations, threatTypes, targetClusters, countriesList } from "../translations";
import { ThreatLog } from "../types";
import { useUi } from "../ui";

export default function Landing() {
  const navigate = useNavigate();
  const { playBeep } = useUi();
  const t = translations.en;

  const [clients, setClients] = useState(0);
  const [countriesCount, setCountriesCount] = useState(0);
  const [websitesCount, setWebsitesCount] = useState(0);
  const [attacksCount, setAttacksCount] = useState(0);

  const [logs, setLogs] = useState<ThreatLog[]>([
    { id: "1", time: "11:27:01", type: "release", target: "LC-1001", status: "FUNDS SETTLED", country: "SG" },
    { id: "2", time: "11:26:48", type: "verify", target: "LC-1042-ACME", status: "VERIFIER OK", country: "DE" },
    { id: "3", time: "11:26:30", type: "proof", target: "BoL-SH-2207", status: "SEAL 73c457ba", country: "JP" },
  ]);

  useEffect(() => {
    let step = 0;
    const totalSteps = 2000 / 25;
    const timer = setInterval(() => {
      step += 1;
      setClients(Math.min(Math.floor((step / totalSteps) * 8), 8));
      setCountriesCount(Math.min(Math.floor((step / totalSteps) * 80), 80));
      setWebsitesCount(Math.min(Math.floor((step / totalSteps) * 100), 100));
      setAttacksCount(Math.min(Math.floor((step / totalSteps) * 3), 3));
      if (step >= totalSteps) clearInterval(timer);
    }, 25);
    return () => clearInterval(timer);
  }, []);

  useEffect(() => {
    const timer = setInterval(() => {
      const now = new Date().toTimeString().split(" ")[0];
      const ev = threatTypes[Math.floor(Math.random() * threatTypes.length)];
      const log: ThreatLog = {
        id: Math.random().toString(),
        time: now,
        type: ev.type,
        target: targetClusters[Math.floor(Math.random() * targetClusters.length)],
        status: ev.status,
        country: countriesList[Math.floor(Math.random() * countriesList.length)],
      };
      setLogs((prev) => [log, ...prev.slice(0, 11)]);
    }, 4000);
    return () => clearInterval(timer);
  }, []);

  return (
    <>
      <Hero
        t={t}
        setRegisterModalOpen={() => navigate("/prove")}
        setStartedModalOpen={() => navigate("/escrow")}
        playBeep={playBeep}
      />
      <Stats clients={clients} countriesCount={countriesCount} websitesCount={websitesCount} attacksCount={attacksCount} t={t} />
      <ThreatFeed threatLogs={logs} t={t} />
    </>
  );
}
