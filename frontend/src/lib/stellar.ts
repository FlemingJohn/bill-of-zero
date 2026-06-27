// Client-side Stellar/Soroban access via @stellar/stellar-sdk + Freighter.
// The backend never holds keys: the connected wallet signs fund/release.

import { contract } from "@stellar/stellar-sdk";
import * as freighter from "@stellar/freighter-api";
import { Buffer } from "buffer";
import { DeployConfig } from "./api";

function hexToBuf(hex: string): Buffer {
  return Buffer.from(hex.replace(/^0x/, "").replace(/…$/, ""), "hex");
}

export async function connectWallet(): Promise<string> {
  const r: any = await freighter.requestAccess();
  if (r?.error) throw new Error(String(r.error));
  return r.address as string;
}

export async function getConnectedAddress(): Promise<string> {
  try {
    const r: any = await freighter.getAddress();
    return (r?.address as string) || "";
  } catch {
    return "";
  }
}

export async function isFreighterInstalled(): Promise<boolean> {
  try {
    const r: any = await freighter.isConnected();
    return !!(r === true || r?.isConnected);
  } catch {
    return false;
  }
}

async function client(cfg: DeployConfig, contractId: string, withSigner: boolean) {
  const opts: any = {
    contractId,
    networkPassphrase: cfg.networkPassphrase,
    rpcUrl: cfg.rpcUrl,
    allowHttp: cfg.rpcUrl.startsWith("http://"),
  };
  if (withSigner) {
    const address = await getConnectedAddress();
    opts.publicKey = address;
    opts.signTransaction = async (xdr: string) => {
      const r: any = await freighter.signTransaction(xdr, {
        networkPassphrase: cfg.networkPassphrase,
        address,
      });
      if (r?.error) throw new Error(String(r.error));
      return { signedTxXdr: r.signedTxXdr, signerAddress: r.signerAddress ?? address };
    };
  }
  return contract.Client.from(opts);
}

export async function isReleased(cfg: DeployConfig): Promise<boolean> {
  const c: any = await client(cfg, cfg.escrow, false);
  const tx = await c.is_released();
  return tx.result as boolean;
}

export async function disclosure(cfg: DeployConfig): Promise<string | null> {
  const c: any = await client(cfg, cfg.escrow, false);
  const tx = await c.disclosure();
  const v = tx.result;
  if (!v) return null;
  return Buffer.from(v).toString("hex");
}

export async function balance(cfg: DeployConfig, id: string): Promise<bigint> {
  const c: any = await client(cfg, cfg.token, false);
  const tx = await c.balance({ id });
  return tx.result as bigint;
}

export async function fund(cfg: DeployConfig, amount: bigint): Promise<string> {
  const from = await getConnectedAddress();
  const c: any = await client(cfg, cfg.escrow, true);
  const tx = await c.fund({ from, amount });
  const sent = await tx.signAndSend();
  return sent?.sendTransactionResponse?.hash ?? sent?.getTransactionResponse?.txHash ?? "";
}

export async function release(cfg: DeployConfig, sealHex: string, journalHex: string): Promise<string> {
  const c: any = await client(cfg, cfg.escrow, true);
  const tx = await c.release({ seal: hexToBuf(sealHex), journal: hexToBuf(journalHex) });
  const sent = await tx.signAndSend();
  return sent?.sendTransactionResponse?.hash ?? sent?.getTransactionResponse?.txHash ?? "";
}
