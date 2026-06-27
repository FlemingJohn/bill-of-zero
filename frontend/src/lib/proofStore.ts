// Persists the most recent proof across pages (Prove -> Escrow) via localStorage.
import { ProveResult } from "./api";

const KEY = "boz.proof";

export function saveProof(p: ProveResult) {
  localStorage.setItem(KEY, JSON.stringify(p));
}

export function loadProof(): ProveResult | null {
  const raw = localStorage.getItem(KEY);
  if (!raw) return null;
  try {
    return JSON.parse(raw) as ProveResult;
  } catch {
    return null;
  }
}

export function clearProof() {
  localStorage.removeItem(KEY);
}
