//! Thin orchestration over the tools that already work:
//!   - the `host` binary  → real Groth16 proving and the auditor disclosure
//!   - the `stellar` CLI  → on-chain fund / release / refund / balance / is_released
//!
//! Every function here blocks; the app runs them on a worker thread (see
//! `app.rs`) so the UI never freezes. `prove` additionally streams progress
//! lines back through a callback while it runs.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::thread;

use anyhow::{anyhow, bail, Context, Result};

use crate::config::Config;

/// Document inputs. The seller owns the trade-document fields; `buyer_balance`
/// is the buyer's own private figure (entered on the Buyer tab). `amount` is
/// derived as quantity * unit_price.
#[derive(Debug, Clone)]
pub struct DocInput {
    pub quantity: u64,
    pub unit_price: u64,
    pub ship_date_unix: u64,
    pub currency: String,
    pub origin: String,
    pub bol_number: u64,
    pub carrier: String,
    pub buyer_balance_usdc: u64,
}

impl DocInput {
    pub fn amount(&self) -> u64 {
        self.quantity.saturating_mul(self.unit_price)
    }
}

/// Parsed result of a successful proof.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)] // some fields are retained for reference / future panels
pub struct Proof {
    pub lc_id: String,
    pub terms_digest: String,
    pub disclosure_cmt: String,
    pub journal: String,
    pub seal: String,
}

/// Result of a state-changing on-chain call.
#[derive(Debug, Clone, Default)]
pub struct TxResult {
    pub hash: Option<String>,
}

/// Decoded auditor disclosure (from `host audit`). Hidden fields carry the
/// literal "hidden (committed)" so the UI can show what the profile withholds.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)] // commitment kept for reference; UI shows the match result
pub struct Disclosure {
    pub profile: String,
    pub amount: String,
    pub quantity: String,
    pub unit_price: String,
    pub currency: String,
    pub buyer_id: String,
    pub seller_id: String,
    pub ship_date: String,
    pub origin_id: String,
    pub bol_number: String,
    pub carrier_id: String,
    pub balance: String,
    pub commitment: String,
    /// "yes" / "no" / "n/a …"
    pub commitment_match: String,
}

/// Generate a real Groth16 proof (selector 73c457ba). Takes minutes; `progress`
/// is called with each non-empty stderr line as the zkVM runs. On a guest panic
/// (non-compliant docs) this returns an error carrying the panic message — no
/// proof can exist, which is the whole point.
pub fn prove(cfg: &Config, input: &DocInput, progress: &dyn Fn(String)) -> Result<Proof> {
    let docs_path = cfg.root.join("sample_data/.tui_docs.json");
    std::fs::write(&docs_path, build_docs_json(cfg, input)).context("writing temp docs file")?;

    // Always a real proof — never RISC0_DEV_MODE. The seal must verify on-chain.
    let mut child = Command::new("cargo")
        .current_dir(&cfg.root)
        .env_remove("RISC0_DEV_MODE")
        .args(["run", "--release", "--quiet", "--bin", "host", "--"])
        .arg(cfg.root.join("sample_data/lc_terms.json"))
        .arg(&docs_path)
        .arg(cfg.root.join("sample_data/approved_sellers.json"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to launch `cargo run --bin host`")?;

    // Collect stdout (the result lines) on a thread while we stream stderr.
    let mut stdout = child.stdout.take().unwrap();
    let stdout_handle = thread::spawn(move || {
        let mut s = String::new();
        let _ = stdout.read_to_string(&mut s);
        s
    });

    let mut err_lines: Vec<String> = Vec::new();
    for line in BufReader::new(child.stderr.take().unwrap()).lines() {
        let line = line.unwrap_or_default();
        if !line.trim().is_empty() {
            progress(line.clone());
            err_lines.push(line);
        }
    }

    let status = child.wait().context("waiting for prover")?;
    let stdout = stdout_handle.join().unwrap_or_default();

    if !status.success() {
        if let Some(p) = err_lines.iter().find(|l| l.contains("Guest panicked:")) {
            bail!("{}", p.trim());
        }
        let tail: Vec<String> = err_lines.iter().rev().take(6).rev().cloned().collect();
        bail!("proving failed:\n{}", tail.join("\n"));
    }

    let f = parse_kv(&stdout);
    let get = |k: &str| f.get(k).cloned().unwrap_or_default();
    let seal = get("seal");
    if seal.is_empty() {
        bail!("proof produced no seal");
    }
    Ok(Proof {
        lc_id: get("lc_id"),
        terms_digest: get("terms_digest"),
        disclosure_cmt: get("disclosure_cmt"),
        journal: get("journal"),
        seal,
    })
}

/// Decrypt the disclosure for a given auditor `profile` (tax / regulator / full)
/// and, if `expected` is the on-chain commitment, verify the match.
pub fn audit(cfg: &Config, profile: &str, expected: Option<&str>) -> Result<Disclosure> {
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&cfg.root)
        .args(["run", "--release", "--quiet", "--bin", "host", "--", "audit"])
        .arg(cfg.root.join("sample_data/disclosure.bin"))
        .arg(profile);
    if let Some(e) = expected {
        cmd.arg(e);
    }
    let out = cmd.output().context("failed to launch `host audit`")?;
    if !out.status.success() {
        bail!("audit failed:\n{}", tail(&String::from_utf8_lossy(&out.stderr), 800));
    }
    let f = parse_kv(&String::from_utf8_lossy(&out.stdout));
    let get = |k: &str| f.get(k).cloned().unwrap_or_default();
    Ok(Disclosure {
        profile: profile.to_string(),
        amount: get("invoice amount"),
        quantity: get("quantity"),
        unit_price: get("unit price"),
        currency: get("currency"),
        buyer_id: get("invoice buyer_id"),
        seller_id: get("invoice seller_id"),
        ship_date: get("shipment date (unix)"),
        origin_id: get("country of origin id"),
        bol_number: get("bill-of-lading number"),
        carrier_id: get("carrier id"),
        balance: get("buyer escrow balance"),
        commitment: get("disclosure_commitment").split_whitespace().next().unwrap_or("").to_string(),
        commitment_match: get("commitment match"),
    })
}

/// Read the escrow's recorded disclosure commitment (hex), if any (read-only).
pub fn escrow_disclosure(cfg: &Config, source: &str) -> Option<String> {
    let (out, err) = invoke_raw(cfg, source, &cfg.deployment.escrow, true, &["disclosure"]).ok()?;
    find_hash(&format!("{out}\n{err}"))
}

/// `is_released()` on the escrow (read-only).
pub fn is_released(cfg: &Config, source: &str) -> Result<bool> {
    let (out, _) = invoke_raw(cfg, source, &cfg.deployment.escrow, true, &["is_released"])?;
    Ok(out.trim().trim_matches('"') == "true")
}

/// Token balance of an account/contract, in stroops (read-only).
pub fn balance(cfg: &Config, source: &str, id: &str) -> Result<String> {
    let (out, _) = invoke_raw(cfg, source, &cfg.deployment.token, true, &["balance", "--id", id])?;
    Ok(out.trim().trim_matches('"').to_string())
}

/// Fund the escrow from the connected source account.
pub fn fund(cfg: &Config, source: &str, amount: u64) -> Result<TxResult> {
    let from = key_address(source)?;
    let amount = amount.to_string();
    let (out, err) = invoke_raw(cfg, source, &cfg.deployment.escrow, false, &["fund", "--from", &from, "--amount", &amount])?;
    Ok(TxResult { hash: find_hash(&format!("{out}\n{err}")) })
}

/// Release the escrow with a proof (seal + journal).
pub fn release(cfg: &Config, source: &str, seal_hex: &str, journal_hex: &str) -> Result<TxResult> {
    let (out, err) = invoke_raw(cfg, source, &cfg.deployment.escrow, false, &["release", "--seal", seal_hex, "--journal", journal_hex])?;
    Ok(TxResult { hash: find_hash(&format!("{out}\n{err}")) })
}

/// Refund the escrow remainder (or cancel after expiry) back to the buyer.
pub fn refund(cfg: &Config, source: &str) -> Result<TxResult> {
    let (out, err) = invoke_raw(cfg, source, &cfg.deployment.escrow, false, &["refund"])?;
    Ok(TxResult { hash: find_hash(&format!("{out}\n{err}")) })
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

/// Run `stellar contract invoke`, returning (stdout, stderr). `view` adds
/// `--send=no` so reads only simulate (no tx, no fee).
fn invoke_raw(cfg: &Config, source: &str, contract: &str, view: bool, fn_args: &[&str]) -> Result<(String, String)> {
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
    Ok((stdout, stderr))
}

fn build_docs_json(cfg: &Config, i: &DocInput) -> String {
    let buyer = &cfg.terms.buyer;
    let seller = &cfg.terms.seller;
    format!(
        r#"{{
  "invoice": {{ "amount_usdc": {amount}, "quantity": {qty}, "unit_price": {price}, "buyer": {buyer:?}, "seller": {seller:?}, "currency": {currency:?} }},
  "bill_of_lading": {{ "ship_date_unix": {ship}, "buyer": {buyer:?}, "seller": {seller:?}, "origin": {origin:?}, "bol_number": {bol}, "carrier": {carrier:?} }},
  "buyer_balance_usdc": {balance}
}}"#,
        amount = i.amount(),
        qty = i.quantity,
        price = i.unit_price,
        currency = i.currency,
        ship = i.ship_date_unix,
        origin = i.origin,
        bol = i.bol_number,
        carrier = i.carrier,
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

/// Find the first 64-hex-char run (a Stellar transaction hash) in CLI output.
fn find_hash(s: &str) -> Option<String> {
    let mut run = 0usize;
    let mut start = 0usize;
    for (i, c) in s.char_indices() {
        if c.is_ascii_hexdigit() {
            if run == 0 {
                start = i;
            }
            run += 1;
            if run == 64 {
                return Some(s[start..start + 64].to_lowercase());
            }
        } else {
            run = 0;
        }
    }
    None
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
