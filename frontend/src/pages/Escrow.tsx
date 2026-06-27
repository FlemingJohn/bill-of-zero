import React, { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { Cpu, Shield, RefreshCw, CheckCircle, ExternalLink, AlertTriangle } from "lucide-react";
import Panel, { Field } from "../components/Panel";
import { getConfig, DeployConfig } from "../lib/api";
import { loadProof } from "../lib/proofStore";
import * as stellar from "../lib/stellar";
import { useUi } from "../ui";

const expertTx = (h: string) => `https://stellar.expert/explorer/testnet/tx/${h}`;
const expertC = (c: string) => `https://stellar.expert/explorer/testnet/contract/${c}`;

export default function Escrow() {
  const { playBeep } = useUi();
  const [cfg, setCfg] = useState<DeployConfig | null>(null);
  const [released, setReleased] = useState<boolean | null>(null);
  const [escrowBal, setEscrowBal] = useState<string>("…");
  const [sellerBal, setSellerBal] = useState<string>("…");
  const [busy, setBusy] = useState<"" | "fund" | "release">("");
  const [txHash, setTxHash] = useState("");
  const [err, setErr] = useState("");
  const proof = loadProof();

  const refresh = async (c: DeployConfig) => {
    try {
      setReleased(await stellar.isReleased(c));
      setEscrowBal((await stellar.balance(c, c.escrow)).toString());
      setSellerBal((await stellar.balance(c, c.seller)).toString());
    } catch (e) {
      setErr(String(e));
    }
  };

  useEffect(() => {
    getConfig().then((c) => { setCfg(c); refresh(c); }).catch((e) => setErr(String(e)));
  }, []);

  const onFund = async () => {
    if (!cfg) return;
    setBusy("fund"); setErr(""); setTxHash("");
    playBeep(600, 0.1);
    try {
      const h = await stellar.fund(cfg, 100000n);
      setTxHash(h); await refresh(cfg); playBeep(1000, 0.2);
    } catch (e) { setErr(String(e)); } finally { setBusy(""); }
  };

  const onRelease = async () => {
    if (!cfg || !proof?.seal || !proof?.journal) return;
    setBusy("release"); setErr(""); setTxHash("");
    playBeep(700, 0.12);
    try {
      const h = await stellar.release(cfg, proof.seal, proof.journal);
      setTxHash(h); await refresh(cfg); playBeep(1000, 0.3);
    } catch (e) { setErr(String(e)); } finally { setBusy(""); }
  };

  return (
    <section className="max-w-3xl mx-auto space-y-8">
      <div className="text-center">
        <h1 className="font-display text-4xl md:text-5xl font-bold tracking-tighter mb-3">
          Escrow <span className="text-transparent bg-clip-text bg-gradient-to-r from-[#BDF589] to-[#636EB4]">Control</span>
        </h1>
        <p className="text-gray-400 max-w-xl mx-auto">
          A live escrow on Stellar testnet, bound to LC #{cfg?.lcId ?? "1001"}. Releasing it runs genuine Groth16 verification through the Nethermind router; funds move only if the proof checks out.
        </p>
      </div>

      <Panel
        title="Deployed Contracts"
        icon={<Cpu className="w-4 h-4 text-[#BDF589]" />}
        right={
          released === null ? <span className="font-mono text-[10px] text-gray-500">reading…</span>
          : released ? <span className="font-mono text-[10px] text-emerald-400">RELEASED</span>
          : <span className="font-mono text-[10px] text-[#BDF589]">FUNDED · AWAITING PROOF</span>
        }
      >
        <div className="space-y-4">
          <Field label="Escrow" value={cfg?.escrow} accent />
          <Field label="VerifierRouter (Nethermind)" value={cfg?.router} />
          <Field label="Groth16 verifier · selector 73c457ba" value={cfg?.groth16Verifier} />
          <div className="grid grid-cols-2 gap-4 pt-2">
            <Field label="Escrow balance (stroops)" value={escrowBal} accent />
            <Field label="Seller balance (stroops)" value={sellerBal} />
          </div>
          {cfg && (
            <a href={expertC(cfg.escrow)} target="_blank" rel="noreferrer"
              className="inline-flex items-center space-x-1.5 text-xs font-mono text-gray-400 hover:text-[#BDF589]">
              <span>View escrow on Stellar Expert</span><ExternalLink className="w-3 h-3" />
            </a>
          )}
        </div>
      </Panel>

      <Panel title="Settle" icon={<Shield className="w-4 h-4 text-[#BDF589]" />}>
        <div className="space-y-4">
          {!proof?.seal ? (
            <div className="text-sm text-gray-400">
              No proof loaded.{" "}
              <Link to="/prove" className="text-[#BDF589] hover:underline">Generate one on the Prove page</Link>{" "}
              first, then return here to release.
            </div>
          ) : (
            <Field label="Loaded seal" value={proof.seal} accent />
          )}

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <button
              onClick={onFund}
              disabled={busy !== ""}
              className="py-3 rounded-xl border border-white/10 text-gray-200 font-mono text-xs uppercase tracking-wider hover:border-white/30 disabled:opacity-50 flex items-center justify-center space-x-2"
            >
              {busy === "fund" ? <RefreshCw className="w-4 h-4 animate-spin" /> : null}
              <span>Fund +100,000</span>
            </button>
            <button
              onClick={onRelease}
              disabled={busy !== "" || !proof?.seal || released === true}
              className="py-3 rounded-xl bg-[#BDF589] text-black font-mono text-xs font-bold uppercase tracking-wider hover:opacity-90 disabled:opacity-40 flex items-center justify-center space-x-2"
            >
              {busy === "release" ? <RefreshCw className="w-4 h-4 animate-spin text-black" /> : <Shield className="w-4 h-4 text-black" />}
              <span>{released ? "Already Released" : "Release with Proof"}</span>
            </button>
          </div>

          <p className="text-[10px] text-gray-500 font-mono">
            Fund and Release are signed by your connected Freighter wallet (Testnet). Release submits seal + journal to the escrow, which calls the verifier router on-chain.
          </p>

          {txHash && (
            <div className="flex items-center space-x-2 text-sm font-mono text-emerald-400">
              <CheckCircle className="w-4 h-4" />
              <a href={expertTx(txHash)} target="_blank" rel="noreferrer" className="hover:underline flex items-center space-x-1">
                <span>tx {txHash.slice(0, 10)}…</span><ExternalLink className="w-3 h-3" />
              </a>
            </div>
          )}
          {err && (
            <div className="flex items-start space-x-2 text-xs font-mono text-rose-400">
              <AlertTriangle className="w-4 h-4 shrink-0" /><span className="break-all">{err}</span>
            </div>
          )}
        </div>
      </Panel>
    </section>
  );
}
