import React, { useState } from "react";
import { Eye, RefreshCw, CheckCircle, XCircle } from "lucide-react";
import Panel, { Field } from "../components/Panel";
import { audit, AuditResult, getConfig } from "../lib/api";
import * as stellar from "../lib/stellar";
import { useUi } from "../ui";

export default function Audit() {
  const { playBeep } = useUi();
  const [busy, setBusy] = useState(false);
  const [res, setRes] = useState<AuditResult | null>(null);
  const [onchain, setOnchain] = useState<string | null>(null);

  const run = async () => {
    setBusy(true); setRes(null); setOnchain(null);
    playBeep(500, 0.1);
    try {
      const r = await audit();
      setRes(r);
      try {
        const cfg = await getConfig();
        setOnchain(await stellar.disclosure(cfg));
      } catch { /* escrow disclosure read is best-effort */ }
      playBeep(880, 0.2);
    } catch (e) {
      setRes({ ok: false, log: String(e) });
    } finally {
      setBusy(false);
    }
  };

  const match = res?.ok && onchain && res.disclosureCommitment?.toLowerCase() === onchain.toLowerCase();

  return (
    <section className="max-w-3xl mx-auto space-y-8">
      <div className="text-center">
        <h1 className="font-display text-4xl md:text-5xl font-bold tracking-tighter mb-3">
          Auditor <span className="text-transparent bg-clip-text bg-gradient-to-r from-[#BDF589] to-[#636EB4]">View Key</span>
        </h1>
        <p className="text-gray-400 max-w-xl mx-auto">
          The chain stores only a blinded commitment. An auditor holding the view key can decrypt the disclosure off-chain and prove it matches exactly what settled.
        </p>
      </div>

      <Panel title="Open Disclosure" icon={<Eye className="w-4 h-4 text-[#BDF589]" />}>
        <button
          onClick={run}
          disabled={busy}
          className="w-full py-4 bg-gradient-to-r from-[#BDF589] to-[#636EB4] text-black font-mono font-bold tracking-wider uppercase rounded-xl hover:opacity-90 disabled:opacity-50 flex items-center justify-center space-x-2"
        >
          {busy ? <><RefreshCw className="w-4 h-4 animate-spin text-black" /><span>Decrypting…</span></>
                : <><Eye className="w-4 h-4 text-black" /><span>Decrypt with View Key</span></>}
        </button>
      </Panel>

      {res?.ok && (
        <Panel
          title="Disclosed Figures"
          icon={match ? <CheckCircle className="w-4 h-4 text-emerald-400" /> : <Eye className="w-4 h-4 text-[#BDF589]" />}
          right={
            onchain == null ? null
            : match ? <span className="font-mono text-[10px] text-emerald-400">MATCHES ON-CHAIN</span>
            : <span className="font-mono text-[10px] text-rose-400">MISMATCH</span>
          }
        >
          <div className="space-y-4">
            <div className="grid grid-cols-3 gap-4">
              <Field label="Invoice amount" value={res.invoiceAmount} accent />
              <Field label="Buyer balance" value={res.buyerBalance} />
              <Field label="Ship date (unix)" value={res.shipDate} />
            </div>
            <Field label="Recomputed commitment" value={res.disclosureCommitment} accent />
            <Field label="On-chain escrow.disclosure()" value={onchain ?? "(escrow not yet released / unavailable)"} />
            <div className="flex items-center space-x-2 text-sm font-mono">
              {match ? (
                <><CheckCircle className="w-4 h-4 text-emerald-400" /><span className="text-emerald-400">sha256(preimage) equals the settled commitment.</span></>
              ) : onchain ? (
                <><XCircle className="w-4 h-4 text-rose-400" /><span className="text-rose-400">Commitments differ.</span></>
              ) : (
                <span className="text-gray-500">Release the escrow to compare against the on-chain commitment.</span>
              )}
            </div>
          </div>
        </Panel>
      )}

      {res && !res.ok && (
        <Panel title="Audit Failed" icon={<XCircle className="w-4 h-4 text-rose-400" />}>
          <p className="text-xs font-mono text-gray-400 break-all">{res.log}</p>
          <p className="text-xs text-gray-500 mt-2">Generate a proof on the Prove page first — it writes the encrypted disclosure that this page opens.</p>
        </Panel>
      )}
    </section>
  );
}
