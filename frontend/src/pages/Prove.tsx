import React, { useState } from "react";
import { Link } from "react-router-dom";
import { Terminal, RefreshCw, Key, CheckCircle, AlertTriangle, ArrowRight } from "lucide-react";
import Panel, { Field } from "../components/Panel";
import { prove, ProveResult } from "../lib/api";
import { saveProof } from "../lib/proofStore";
import { useUi } from "../ui";

export default function Prove() {
  const { playBeep } = useUi();
  const [docs, setDocs] = useState<"valid" | "tampered">("valid");
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<ProveResult | null>(null);

  const run = async () => {
    setBusy(true);
    setResult(null);
    playBeep(350, 0.1);
    try {
      const r = await prove(docs);
      setResult(r);
      if (r.ok) {
        saveProof(r);
        playBeep(880, 0.25);
      } else {
        playBeep(200, 0.2, "triangle");
      }
    } catch (e) {
      setResult({ ok: false, message: String(e) });
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="max-w-3xl mx-auto space-y-8">
      <div className="text-center">
        <h1 className="font-display text-4xl md:text-5xl font-bold tracking-tighter mb-3">
          ZK Prover <span className="text-transparent bg-clip-text bg-gradient-to-r from-[#BDF589] to-[#636EB4]">Console</span>
        </h1>
        <p className="text-gray-400 max-w-xl mx-auto">
          Generate a real Groth16 proof that a private document set satisfies the Letter-of-Credit terms. The documents never leave the prover — only the seal and the 80-byte journal do.
        </p>
      </div>

      <Panel title="Initialize ZK Prover" icon={<Terminal className="w-4 h-4 text-[#BDF589]" />}>
        <div className="space-y-5">
          <div className="space-y-1.5">
            <label className="block text-xs font-mono text-gray-500 uppercase">Document Set</label>
            <div className="grid grid-cols-2 gap-3">
              {(["valid", "tampered"] as const).map((d) => (
                <button
                  key={d}
                  onClick={() => { setDocs(d); playBeep(400, 0.05); }}
                  className={`py-3 rounded-xl border text-xs font-mono uppercase tracking-wider transition-all ${
                    docs === d ? "border-[#BDF589] bg-[#BDF589]/10 text-[#BDF589]" : "border-white/10 text-gray-400 hover:text-white"
                  }`}
                >
                  {d === "valid" ? "Compliant invoice (95,000)" : "Over-limit invoice (250,000)"}
                </button>
              ))}
            </div>
          </div>

          <button
            onClick={run}
            disabled={busy}
            className="w-full py-4 bg-gradient-to-r from-[#BDF589] to-[#636EB4] text-black font-mono font-bold tracking-wider uppercase rounded-xl hover:opacity-90 disabled:opacity-50 flex items-center justify-center space-x-2"
          >
            {busy ? (
              <><RefreshCw className="w-4 h-4 animate-spin text-black" /><span>Proving in zkVM… (a few minutes)</span></>
            ) : (
              <><Key className="w-4 h-4 text-black" /><span>Generate Groth16 Proof</span></>
            )}
          </button>

          {busy && (
            <p className="text-[10px] text-gray-500 font-mono text-center">
              Running the RISC Zero guest, then the STARK→SNARK wrap in Docker. This is a genuine proof, not a mock.
            </p>
          )}
        </div>
      </Panel>

      {result && result.ok && (
        <Panel title="Proof Generated" icon={<CheckCircle className="w-4 h-4 text-emerald-400" />}
          right={<span className="font-mono text-[10px] text-emerald-400">SELECTOR 73c457ba</span>}>
          <div className="space-y-4">
            <Field label="LC ID" value={String(result.lcId)} accent />
            <Field label="image_id (pinned guest)" value={result.imageId} />
            <Field label="terms_digest" value={result.termsDigest} />
            <Field label="disclosure_commitment" value={result.disclosureCommitment} />
            <Field label="journal (80 bytes)" value={result.journal} />
            <Field label="journal_digest = sha256(journal)" value={result.journalDigest} />
            <Field label="seal (Groth16)" value={result.seal} accent />
            <Link
              to="/escrow"
              onClick={() => playBeep(1000, 0.15)}
              className="mt-2 inline-flex items-center space-x-2 text-sm font-mono text-[#BDF589] hover:translate-x-1 transition-transform"
            >
              <span>Settle this proof on the escrow</span>
              <ArrowRight className="w-4 h-4" />
            </Link>
          </div>
        </Panel>
      )}

      {result && !result.ok && (
        <Panel title="Proof Rejected" icon={<AlertTriangle className="w-4 h-4 text-rose-400" />}>
          <div className="space-y-3">
            <p className="text-sm font-mono text-rose-400">{result.message}</p>
            <p className="text-xs text-gray-400">
              The guest enforces the LC rules with assertions. Non-compliant documents make it panic, so no proof can exist — the escrow can never be unlocked with them. That is the security property, working as intended.
            </p>
          </div>
        </Panel>
      )}
    </section>
  );
}
