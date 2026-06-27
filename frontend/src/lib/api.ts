// Calls to the Bill of Zero backend (prover + auditor + config).

export interface DeployConfig {
  network: string;
  rpcUrl: string;
  networkPassphrase: string;
  router: string;
  groth16Verifier: string;
  selector: string;
  token: string;
  seller: string;
  deployer: string;
  escrow: string;
  lcId?: number;
  imageId?: string;
  termsDigest?: string;
}

export interface ProveResult {
  ok: boolean;
  rejected?: boolean;
  message?: string;
  log?: string;
  lcId?: number;
  imageId?: string;
  termsDigest?: string;
  approvedRoot?: string;
  disclosureCommitment?: string;
  journal?: string;
  journalDigest?: string;
  seal?: string;
}

export interface AuditResult {
  ok: boolean;
  invoiceAmount?: string;
  buyerBalance?: string;
  shipDate?: string;
  buyerId?: string;
  sellerId?: string;
  disclosureCommitment?: string;
  log?: string;
}

let cachedConfig: DeployConfig | null = null;

export async function getConfig(): Promise<DeployConfig> {
  if (cachedConfig) return cachedConfig;
  const res = await fetch("/api/config");
  if (!res.ok) throw new Error("Failed to load deployment config");
  cachedConfig = await res.json();
  return cachedConfig!;
}

export async function prove(docs: "valid" | "tampered"): Promise<ProveResult> {
  const res = await fetch("/api/prove", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ docs }),
  });
  return res.json();
}

export async function audit(): Promise<AuditResult> {
  const res = await fetch("/api/audit", { method: "POST" });
  return res.json();
}
