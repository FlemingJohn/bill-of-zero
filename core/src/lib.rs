#![no_std]
//! Shared data model for Bill of Zero.
//!
//! These types are the single source of truth shared by the zkVM guest
//! (which enforces the Letter-of-Credit rules) and the host (which feeds
//! documents in and submits the proof). Keeping them here guarantees the
//! guest and host can never disagree on serialization.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Depth of the approved-seller Merkle tree (2^DEPTH leaves). Depth 4 = up to
/// 16 approved exporters, which is plenty for the demo.
pub const TREE_DEPTH: usize = 4;

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
    /// Feature 1 (Merkle membership): root of the bank's approved-exporter
    /// allowlist. The seller must prove membership without revealing which
    /// approved exporter it is.
    pub approved_root: [u8; 32],
    /// Feature 4 (issuer signature): the ed25519 public key of the trusted
    /// document issuer (e.g. the carrier / issuing bank). The guest proves the
    /// presented documents carry this issuer's signature.
    pub issuer_pubkey: [u8; 32],
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

/// A Merkle inclusion proof: the sibling hashes along the path from a leaf to
/// the root, plus the position bits (bit i: 0 => node is the left child at
/// level i, 1 => right child).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerkleProof {
    pub siblings: [[u8; 32]; TREE_DEPTH],
    pub index_bits: u32,
}

/// The full private document set presented for an LC drawdown.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentSet {
    pub invoice: Invoice,
    pub bill_of_lading: BillOfLading,
    /// Feature 3 (range proof): the buyer's available escrow balance. The guest
    /// proves this covers the LC credit line WITHOUT revealing the exact figure.
    pub buyer_balance: u64,
    /// Feature 1 (Merkle membership): proof that `invoice.seller_id` is a leaf
    /// of the LC's `approved_root` tree.
    pub seller_merkle: MerkleProof,
    /// Feature 4 (issuer signature): the issuer's ed25519 signature over
    /// `doc_digest(invoice, bill_of_lading)`. Stored as two 32-byte halves
    /// because serde only derives arrays up to length 32.
    pub issuer_sig: [[u8; 32]; 2],
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

/// Hash two 32-byte nodes into their parent. Used to walk a Merkle path.
pub fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(left);
    h.update(right);
    h.finalize().into()
}

/// Recompute the Merkle root implied by `leaf` and `proof`. The guest compares
/// this against the LC's `approved_root` to prove membership.
pub fn merkle_root(leaf: &[u8; 32], proof: &MerkleProof) -> [u8; 32] {
    let mut cur = *leaf;
    let mut idx = proof.index_bits;
    let mut i = 0;
    while i < TREE_DEPTH {
        let sib = &proof.siblings[i];
        cur = if idx & 1 == 0 {
            hash_pair(&cur, sib)
        } else {
            hash_pair(sib, &cur)
        };
        idx >>= 1;
        i += 1;
    }
    cur
}

/// Canonical digest of the trade documents. This is the message the issuer
/// signs (feature 4) and that the guest verifies the signature against. Covers
/// every field the LC rules care about, so the signature pins the exact docs.
pub fn doc_digest(inv: &Invoice, bol: &BillOfLading) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(inv.amount.to_le_bytes());
    h.update(inv.buyer_id);
    h.update(inv.seller_id);
    h.update(bol.ship_date.to_le_bytes());
    h.update(bol.buyer_id);
    h.update(bol.seller_id);
    h.finalize().into()
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
    h.update(t.approved_root);
    h.update(t.issuer_pubkey);
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
