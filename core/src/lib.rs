#![no_std]
//! Shared data model for Bill of Zero.
//!
//! These types are the single source of truth shared by the zkVM guest
//! (which enforces the Letter-of-Credit rules) and the host (which feeds
//! documents in and submits the proof). Keeping them here guarantees the
//! guest and host can never disagree on serialization.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Public Letter-of-Credit terms. These are known to the escrow contract
/// (the LC is what the escrow was funded against). They are passed to the
/// guest as input and bound into the journal via `terms_digest`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LcTerms {
    pub lc_id: u64,
    /// Maximum payable amount (token base units).
    pub credit_limit: u64,
    /// Latest valid shipment date (unix seconds).
    pub deadline: u64,
    /// sha256(buyer legal name).
    pub buyer_id: [u8; 32],
    /// sha256(seller legal name).
    pub seller_id: [u8; 32],
}

/// Private commercial invoice. Never leaves the off-chain prover.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Invoice {
    pub amount: u64,
    pub buyer_id: [u8; 32],
    pub seller_id: [u8; 32],
}

/// Private bill of lading (shipping document). Never leaves the prover.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BillOfLading {
    pub ship_date: u64,
    pub buyer_id: [u8; 32],
    pub seller_id: [u8; 32],
}

/// The full private document set presented for an LC drawdown.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentSet {
    pub invoice: Invoice,
    pub bill_of_lading: BillOfLading,
}

/// Length of the journal the guest commits.
///
/// Fixed 48-byte layout so the Soroban escrow can parse it with plain slicing
/// (no risc0 serde on-chain):
///   [0..8]   lc_id          (little-endian u64)
///   [8..16]  release_amount (little-endian u64)
///   [16..48] terms_digest   (sha256 of the canonical LcTerms)
pub const JOURNAL_LEN: usize = 48;

/// Derive a 32-byte party id from a human-readable legal name.
pub fn id_from_name(name: &str) -> [u8; 32] {
    Sha256::digest(name.as_bytes()).into()
}

/// Canonical digest of the LC terms. Computed identically in the guest and by
/// the deployer (via the host), so the escrow can prove the proof was checked
/// against the *correct* LC terms and not attacker-chosen lenient ones.
pub fn terms_digest(t: &LcTerms) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(t.lc_id.to_le_bytes());
    h.update(t.credit_limit.to_le_bytes());
    h.update(t.deadline.to_le_bytes());
    h.update(t.buyer_id);
    h.update(t.seller_id);
    h.finalize().into()
}

/// Pack the public journal into its fixed 48-byte on-chain layout.
pub fn pack_journal(lc_id: u64, release_amount: u64, terms_digest: &[u8; 32]) -> [u8; JOURNAL_LEN] {
    let mut out = [0u8; JOURNAL_LEN];
    out[0..8].copy_from_slice(&lc_id.to_le_bytes());
    out[8..16].copy_from_slice(&release_amount.to_le_bytes());
    out[16..48].copy_from_slice(terms_digest);
    out
}
