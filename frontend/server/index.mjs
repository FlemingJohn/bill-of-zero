// Bill of Zero — backend API.
//
// Bridges the browser to the parts that can't run in a browser:
//   - the RISC Zero prover (native Rust binary + Docker for the Groth16 step)
//   - the auditor disclosure decrypt
//   - the deployed-contract config
//
// On-chain fund/release is done client-side with @stellar/stellar-sdk + Freighter,
// so this server never holds keys.

import express from "express";
import cors from "cors";
import { execFile } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PROJECT_ROOT = path.resolve(__dirname, "..", ".."); // ~/bill-of-zero
const SAMPLE = path.join(PROJECT_ROOT, "sample_data");

const app = express();
app.use(cors());
app.use(express.json());

// Run a command, capturing stdout/stderr, with a generous timeout for proving.
function run(cmd, args, opts = {}) {
  return new Promise((resolve) => {
    const child = execFile(
      cmd,
      args,
      { cwd: PROJECT_ROOT, timeout: opts.timeout ?? 300000, maxBuffer: 1024 * 1024 * 16, env: { ...process.env, ...opts.env } },
      (err, stdout, stderr) => resolve({ code: err?.code ?? 0, stdout: stdout || "", stderr: stderr || "", err })
    );
    void child;
  });
}

// Parse the host's "key : value" output lines into an object.
function parseHostOutput(text) {
  const fields = {};
  for (const line of text.split("\n")) {
    const m = line.match(/^(\w[\w ]*?)\s*:\s*(.+?)\s*$/);
    if (m) fields[m[1].trim()] = m[2].trim();
  }
  return fields;
}

// GET /api/config — deployed contract addresses for the frontend.
app.get("/api/config", async (_req, res) => {
  try {
    const raw = await readFile(path.join(PROJECT_ROOT, "deployment.json"), "utf8");
    res.json(JSON.parse(raw));
  } catch (e) {
    res.status(500).json({ error: "deployment.json not found", detail: String(e) });
  }
});

// POST /api/prove — generate a REAL Groth16 proof.
// body: { docs: "valid" | "tampered" }  (uses the bundled sample data)
app.post("/api/prove", async (req, res) => {
  const which = req.body?.docs === "tampered" ? "docs_tampered.json" : "docs_valid.json";
  const args = [
    "run", "--release", "--quiet", "--bin", "host", "--",
    path.join(SAMPLE, "lc_terms.json"),
    path.join(SAMPLE, which),
    path.join(SAMPLE, "approved_sellers.json"),
  ];
  const { code, stdout, stderr } = await run("cargo", args, { timeout: 300000 });
  const combined = stdout + "\n" + stderr;

  // Tampered docs: the guest panics on purpose -> no proof. Report it as a
  // successful "rejected" outcome, not a server error.
  if (code !== 0) {
    const panic = combined.match(/Guest panicked:.*$/m)?.[0] || "Proof generation failed.";
    return res.json({ ok: false, rejected: which.includes("tampered"), message: panic, log: combined.slice(-2000) });
  }

  const f = parseHostOutput(stdout);
  res.json({
    ok: true,
    lcId: Number(f.lc_id),
    imageId: f.image_id,
    termsDigest: f.terms_digest,
    approvedRoot: f.approved_root,
    disclosureCommitment: f.disclosure_cmt,
    journal: f.journal,
    journalDigest: f.journal_digest,
    seal: f.seal,
  });
});

// POST /api/audit — decrypt the auditor disclosure and recompute the commitment.
app.post("/api/audit", async (_req, res) => {
  const args = ["run", "--release", "--quiet", "--bin", "host", "--", "audit", path.join(SAMPLE, "disclosure.bin")];
  const { code, stdout, stderr } = await run("cargo", args, { timeout: 120000 });
  if (code !== 0) return res.status(500).json({ ok: false, log: (stdout + stderr).slice(-2000) });
  const f = parseHostOutput(stdout);
  res.json({
    ok: true,
    invoiceAmount: f["invoice amount"],
    buyerBalance: f["buyer escrow balance"],
    shipDate: f["shipment date (unix)"],
    buyerId: f["invoice buyer_id"],
    sellerId: f["invoice seller_id"],
    disclosureCommitment: f["disclosure_commitment"]?.split(/\s+/)[0],
  });
});

const PORT = process.env.PORT || 8787;
app.listen(PORT, () => console.log(`Bill of Zero API on :${PORT}`));
