//! Loads the two on-disk facts the TUI needs: the deployed contract addresses
//! (`deployment.json`) and the human-readable LC terms (`sample_data/lc_terms.json`).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

/// Deployed-contract config. Extra fields in deployment.json (imageId, sample*,
/// …) are ignored — we only declare what the TUI uses.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // some fields are retained for reference / future use
pub struct Deployment {
    pub network: String,
    pub rpc_url: String,
    pub network_passphrase: String,
    pub router: String,
    pub groth16_verifier: String,
    pub selector: String,
    pub token: String,
    pub seller: String,
    pub deployer: String,
    pub escrow: String,
    pub lc_id: u64,
}

/// The LC terms the bank fixed: credit limit, deadline, and the two parties.
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct LcTerms {
    pub lc_id: u64,
    pub credit_limit_usdc: u64,
    pub shipment_deadline_unix: u64,
    pub buyer: String,
    pub seller: String,
}

/// Everything the TUI loads at startup, plus the project root it was found in.
#[derive(Debug, Clone)]
pub struct Config {
    pub root: PathBuf,
    pub deployment: Deployment,
    pub terms: LcTerms,
}

impl Config {
    /// Find the project root (the dir holding deployment.json), starting at the
    /// current dir and walking up, then load both files.
    pub fn load() -> Result<Self> {
        let root = find_root().context(
            "could not find deployment.json — run the TUI from within the bill-of-zero repo",
        )?;
        let deployment = read_json(root.join("deployment.json"))?;
        let terms = read_json(root.join("sample_data/lc_terms.json"))?;
        Ok(Self { root, deployment, terms })
    }
}

fn find_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join("deployment.json").is_file() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let path = path.as_ref();
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parsing {}", path.display()))
}
