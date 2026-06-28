//! Thin orchestration over the tools that already work:
//!   - the `host` binary  → real/dev Groth16 proving and the auditor disclosure
//!   - the `stellar` CLI  → on-chain fund / release / balance / is_released
//!
//! Every function here blocks; the app runs them on a worker thread (see
//! `app.rs`) so the UI never freezes.

use std::collections::HashMap;
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};

use crate::config::Config;

/// Seller's document inputs (what the user types on the Seller tab).
#[derive(Debug, Clone)]
pub struct DocInput {
    pub amount_usdc: u64,
    pub ship_date_unix: u64,
    pub buyer_balance_usdc: u64,
}

/// Parsed result of a successful proof.
#[derive(Debug, Clone, Default)]
pub struct Proof {
    pub lc_id: String,
    pub terms_digest: String,
    pub disclosure_cmt: String,
    pub journal: String,
    pub seal: String,
}

/// Decoded auditor disclosure (from `host audit`).
#[derive(Debug, Clone, Default)]
pub struct Disclosure {
    pub amount: String,
    pub balance: String,
    pub ship_date: String,
    pub buyer_id: String,
    pub seller_id: String,
    pub commitment: String,
}

/// Generate a real Groth16 proof for the given documents (selector 73c457ba).
/// Takes minutes. On a guest panic (non-compliant docs) this returns an error
/// carrying the panic message — no proof can exist, which is the whole point.
pub fn prove(cfg: &Config, input: &DocInput) -> Result<Proof> {
    let docs_path = cfg.root.join("sample_data/.tui_docs.json");
    let docs_json = build_docs_json(cfg, input);
    std::fs::write(&docs_path, docs_json).context("writing temp docs file")?;

    // Always a real proof — never RISC0_DEV_MODE. The seal must verify on-chain.
    let out = Command::new("cargo")
        .current_dir(&cfg.root)
        .env_remove("RISC0_DEV_MODE")
        .args(["run", "--release", "--quiet", "--bin", "host", "--"])
        .arg(cfg.root.join("sample_data/lc_terms.json"))
        .arg(&docs_path)
        .arg(cfg.root.join("sample_data/approved_sellers.json"))
        .output()
        .context("failed to launch `cargo run --bin host`")?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    if !out.status.success() {
        // Tampered / non-compliant docs → the guest panics on purpose.
        if let Some(p) = stderr.lines().find(|l| l.contains("Guest panicked:")) {
            bail!("{}", p.trim());
        }
        bail!("proving failed:\n{}", tail(&stderr, 800));
    }

    let f = parse_kv(&stdout);
    let get = |k: &str| f.get(k).cloned().unwrap_or_default();
    let seal = get("seal");
    if seal.is_empty() {
        bail!("proof produced no seal:\n{}", tail(&stdout, 800));
    }
    Ok(Proof {
        lc_id: get("lc_id"),
        terms_digest: get("terms_digest"),
        disclosure_cmt: get("disclosure_cmt"),
        journal: get("journal"),
        seal,
    })
}

/// Decrypt and recompute the auditor disclosure written by the last proof.
pub fn audit(cfg: &Config) -> Result<Disclosure> {
    let out = Command::new("cargo")
        .current_dir(&cfg.root)
        .args(["run", "--release", "--quiet", "--bin", "host", "--", "audit"])
        .arg(cfg.root.join("sample_data/disclosure.bin"))
        .output()
        .context("failed to launch `host audit`")?;
    if !out.status.success() {
        bail!("audit failed:\n{}", tail(&String::from_utf8_lossy(&out.stderr), 800));
    }
    let f = parse_kv(&String::from_utf8_lossy(&out.stdout));
    let get = |k: &str| f.get(k).cloned().unwrap_or_default();
    Ok(Disclosure {
        amount: get("invoice amount"),
        balance: get("buyer escrow balance"),
        ship_date: get("shipment date (unix)"),
        buyer_id: get("invoice buyer_id"),
        seller_id: get("invoice seller_id"),
        // value trails with "(must equal …)" — keep just the hex token.
        commitment: get("disclosure_commitment").split_whitespace().next().unwrap_or("").to_string(),
    })
}

/// `is_released()` on the escrow (read-only).
pub fn is_released(cfg: &Config, source: &str) -> Result<bool> {
    let v = invoke(cfg, source, &cfg.deployment.escrow, true, &["is_released"])?;
    Ok(v.trim().trim_matches('"') == "true")
}

/// Token balance of an account/contract, in stroops (read-only).
pub fn balance(cfg: &Config, source: &str, id: &str) -> Result<String> {
    let v = invoke(cfg, source, &cfg.deployment.token, true, &["balance", "--id", id])?;
    Ok(v.trim().trim_matches('"').to_string())
}

/// Fund the escrow from the connected source account.
pub fn fund(cfg: &Config, source: &str, amount: u64) -> Result<String> {
    let from = key_address(source)?;
    let amount = amount.to_string();
    invoke(cfg, source, &cfg.deployment.escrow, false, &["fund", "--from", &from, "--amount", &amount])
}

/// Release the escrow with a proof (seal + journal).
pub fn release(cfg: &Config, source: &str, seal_hex: &str, journal_hex: &str) -> Result<String> {
    invoke(
        cfg,
        source,
        &cfg.deployment.escrow,
        false,
        &["release", "--seal", seal_hex, "--journal", journal_hex],
    )
}

// --- internals ------------------------------------------------------------

/// Resolve a `stellar keys` identity alias to its G… address.
pub fn key_address(source: &str) -> Result<String> {
    let out = Command::new("stellar")
        .args(["keys", "address", source])
        .output()
        .context("failed to run `stellar keys address`")?;
    if !out.status.success() {
        bail!(
            "no Stellar key '{}'. Create one with `stellar keys generate {} --network testnet --fund`",
            source, source
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Run `stellar contract invoke` against a contract, passing rpc-url +
/// passphrase explicitly so we don't depend on a named network being configured.
/// `view` adds `--send=no` so read-only calls only simulate (no tx, no fee).
fn invoke(cfg: &Config, source: &str, contract: &str, view: bool, fn_args: &[&str]) -> Result<String> {
    let d = &cfg.deployment;
    let mut cmd = Command::new("stellar");
    cmd.args(["contract", "invoke", "--id", contract, "--source-account", source])
        .args(["--rpc-url", &d.rpc_url])
        .args(["--network-passphrase", &d.network_passphrase]);
    if view {
        cmd.arg("--send=no");
    }
    cmd.arg("--").args(fn_args);

    let out = cmd.output().context("failed to launch `stellar contract invoke`")?;
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    if !out.status.success() {
        return Err(anyhow!("{}", first_error_line(&stderr)));
    }
    // Successful invokes print the return value on stdout; tx logs go to stderr.
    Ok(if stdout.trim().is_empty() { stderr.trim().to_string() } else { stdout })
}

fn build_docs_json(cfg: &Config, i: &DocInput) -> String {
    let buyer = &cfg.terms.buyer;
    let seller = &cfg.terms.seller;
    format!(
        r#"{{
  "invoice": {{ "amount_usdc": {amount}, "buyer": {buyer:?}, "seller": {seller:?}, "currency": "USDC" }},
  "bill_of_lading": {{ "ship_date_unix": {ship}, "buyer": {buyer:?}, "seller": {seller:?}, "goods": "5000x precision optical lenses" }},
  "buyer_balance_usdc": {balance}
}}"#,
        amount = i.amount_usdc,
        ship = i.ship_date_unix,
        balance = i.buyer_balance_usdc,
    )
}

/// Parse the host's "key : value" lines (keys may contain spaces).
fn parse_kv(text: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    for line in text.lines() {
        if let Some(idx) = line.find(':') {
            let key = line[..idx].trim();
            let val = line[idx + 1..].trim();
            if !key.is_empty() && !val.is_empty() {
                m.insert(key.to_string(), val.to_string());
            }
        }
    }
    m
}

fn first_error_line(stderr: &str) -> String {
    stderr
        .lines()
        .find(|l| l.to_lowercase().contains("error"))
        .map(|l| l.trim().to_string())
        .unwrap_or_else(|| tail(stderr, 400))
}

fn tail(s: &str, n: usize) -> String {
    let s = s.trim();
    if s.len() <= n {
        s.to_string()
    } else {
        format!("…{}", &s[s.len() - n..])
    }
}
