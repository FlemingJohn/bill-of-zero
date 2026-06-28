#![no_std]
//! Shared data model for Bill of Zero.
//!
//! These types are the single source of truth shared by the zkVM guest
//! (which enforces the Letter-of-Credit rules) and the host (which feeds
//! documents in and submits the proof). Keeping them here guarantees the
//! guest and host can never disagree on serialization.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Depth of the Merkle trees (2^DEPTH leaves). Depth 4 = up to 16 leaves, used
/// for both the approved-exporter and the allowed-origin allowlists.
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
    /// ISO 4217 numeric currency code the LC settles in (e.g. 840 = USD).
    pub currency: u32,
    /// sha256(buyer legal name).
    pub buyer_id: [u8; 32],
    /// sha256(seller legal name).
    pub seller_id: [u8; 32],
    /// Feature 1 (Merkle membership): root of the bank's approved-exporter
    /// allowlist. The seller must prove membership without revealing which
    /// approved exporter it is.
    pub approved_root: [u8; 32],
    /// Allowed-origin allowlist root: the goods' country of origin must be a
    /// member, proved without revealing which country (Merkle membership).
    pub origins_root: [u8; 32],
    /// Feature 4 (issuer signature): the ed25519 public key of the trusted
    /// document issuer (e.g. the carrier / issuing bank). The guest proves the
    /// presented documents carry this issuer's signature.
    pub issuer_pubkey: [u8; 32],
    /// Feature 5 (selective disclosure): the auditor's X25519 public key. The
    /// documents are encrypted to this key off-chain; the guest commits a
    /// blinded digest of them on-chain so the auditor's later disclosure is
    /// provably the same document set that was settled.
    pub auditor_pubkey: [u8; 32],
}

/// Private commercial invoice. Never leaves the off-chain prover.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Invoice {
    /// Total invoiced amount; must equal quantity * unit_price (rule).
    pub amount: u64,
    /// Number of units shipped.
    pub quantity: u64,
    /// Price per unit.
    pub unit_price: u64,
    /// ISO 4217 numeric currency code; must equal the LC currency (rule).
    pub currency: u32,
    pub buyer_id: [u8; 32],
    pub seller_id: [u8; 32],
}

/// Private bill of lading (shipping document). Never leaves the prover.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BillOfLading {
    pub ship_date: u64,
    pub buyer_id: [u8; 32],
    pub seller_id: [u8; 32],
    /// sha256(country of origin); must be in the LC's allowed-origin set (rule).
    pub origin_id: [u8; 32],
    /// Bill-of-lading document number (disclosed to the auditor, not gated).
    pub bol_number: u64,
    /// sha256(carrier / vessel name) (disclosed to the auditor, not gated).
    pub carrier_id: [u8; 32],
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
    /// Merkle proof that `bill_of_lading.origin_id` is in the LC's origins tree.
    pub origin_merkle: MerkleProof,
    /// Feature 4 (issuer signature): the issuer's ed25519 signature over
    /// `doc_digest`. Stored as two 32-byte halves because serde only derives
    /// arrays up to length 32.
    pub issuer_sig: [[u8; 32]; 2],
    /// Feature 5 (selective disclosure): the master blinding from which each
    /// disclosed field's per-field blinding is derived, so the on-chain
    /// commitment cannot be brute-forced from guessable fields. Shared with the
    /// auditor as part of the encrypted opening.
    pub disclosure_blinding: [u8; 32],
}

/// Length of the journal the guest commits.
///
/// Fixed 80-byte layout so the Soroban escrow can parse it with plain slicing
/// (no risc0 serde on-chain):
///   [0..8]   lc_id                 (little-endian u64)
///   [8..16]  release_amount        (little-endian u64)
///   [16..48] terms_digest          (sha256 of the canonical LcTerms)
///   [48..80] disclosure_commitment (per-field selective-disclosure root)
pub const JOURNAL_LEN: usize = 80;

// --- Selective-disclosure field set (feature 5, granular) ------------------
//
// Each disclosable field has a stable index. The on-chain commitment is
// H(c_0 || c_1 || ... || c_{N-1}) where c_i = H(i || value_i || blinding_i) and
// blinding_i = H("bz-field" || i || master). An auditor entitled to a subset S
// can be given the openings (value_i, blinding_i) for i in S and the raw c_j for
// j not in S, recompute the revealed c_i, and verify the same overall commitment
// WITHOUT learning the hidden fields. That is what makes the disclosure granular.

/// Number of disclosable fields.
pub const DISCLOSURE_FIELDS: usize = 11;

/// Field indices (also the domain-separation tag for each per-field commitment).
pub const F_AMOUNT: u8 = 0;
pub const F_QUANTITY: u8 = 1;
pub const F_UNIT_PRICE: u8 = 2;
pub const F_CURRENCY: u8 = 3;
pub const F_BUYER_ID: u8 = 4;
pub const F_SELLER_ID: u8 = 5;
pub const F_SHIP_DATE: u8 = 6;
pub const F_ORIGIN_ID: u8 = 7;
pub const F_BOL_NUMBER: u8 = 8;
pub const F_CARRIER_ID: u8 = 9;
pub const F_BUYER_BALANCE: u8 = 10;

/// Length of the disclosure opening blob the host encrypts to the auditor: every
/// field value in index order, then the 32-byte master blinding.
///   amount(8) quantity(8) unit_price(8) currency(4) buyer(32) seller(32)
///   ship_date(8) origin(32) bol_number(8) carrier(32) buyer_balance(8)
///   master_blinding(32)
pub const DISCLOSURE_OPENING_LEN: usize =
    8 + 8 + 8 + 4 + 32 + 32 + 8 + 32 + 8 + 32 + 8 + 32;

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
/// this against an allowlist root to prove membership.
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
/// every document field, so the signature pins the exact docs.
pub fn doc_digest(inv: &Invoice, bol: &BillOfLading) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(inv.amount.to_le_bytes());
    h.update(inv.quantity.to_le_bytes());
    h.update(inv.unit_price.to_le_bytes());
    h.update(inv.currency.to_le_bytes());
    h.update(inv.buyer_id);
    h.update(inv.seller_id);
    h.update(bol.ship_date.to_le_bytes());
    h.update(bol.buyer_id);
    h.update(bol.seller_id);
    h.update(bol.origin_id);
    h.update(bol.bol_number.to_le_bytes());
    h.update(bol.carrier_id);
    h.finalize().into()
}

/// Per-field blinding derived from the master blinding and the field index.
pub fn field_blinding(master: &[u8; 32], idx: u8) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"bz-field");
    h.update([idx]);
    h.update(master);
    h.finalize().into()
}

/// Per-field commitment c_i = H(idx || value || blinding_i).
pub fn commit_field(idx: u8, master: &[u8; 32], value: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update([idx]);
    h.update(value);
    h.update(field_blinding(master, idx));
    h.finalize().into()
}

/// Compute the per-field commitments for every disclosable field, in index order.
pub fn disclosure_field_commitments(docs: &DocumentSet) -> [[u8; 32]; DISCLOSURE_FIELDS] {
    let m = &docs.disclosure_blinding;
    let inv = &docs.invoice;
    let bol = &docs.bill_of_lading;
    [
        commit_field(F_AMOUNT, m, &inv.amount.to_le_bytes()),
        commit_field(F_QUANTITY, m, &inv.quantity.to_le_bytes()),
        commit_field(F_UNIT_PRICE, m, &inv.unit_price.to_le_bytes()),
        commit_field(F_CURRENCY, m, &inv.currency.to_le_bytes()),
        commit_field(F_BUYER_ID, m, &inv.buyer_id),
        commit_field(F_SELLER_ID, m, &inv.seller_id),
        commit_field(F_SHIP_DATE, m, &bol.ship_date.to_le_bytes()),
        commit_field(F_ORIGIN_ID, m, &bol.origin_id),
        commit_field(F_BOL_NUMBER, m, &bol.bol_number.to_le_bytes()),
        commit_field(F_CARRIER_ID, m, &bol.carrier_id),
        commit_field(F_BUYER_BALANCE, m, &docs.buyer_balance.to_le_bytes()),
    ]
}

/// Serialize the disclosure opening: every field value in index order, then the
/// 32-byte master blinding. This is the plaintext the host encrypts to the
/// auditor; `field_commitments_from_opening` reverses it. Offsets:
///   amount[0..8] quantity[8..16] unit_price[16..24] currency[24..28]
///   buyer[28..60] seller[60..92] ship_date[92..100] origin[100..132]
///   bol_number[132..140] carrier[140..172] buyer_balance[172..180]
///   master[180..212]
pub fn disclosure_opening(docs: &DocumentSet) -> [u8; DISCLOSURE_OPENING_LEN] {
    let mut out = [0u8; DISCLOSURE_OPENING_LEN];
    let inv = &docs.invoice;
    let bol = &docs.bill_of_lading;
    out[0..8].copy_from_slice(&inv.amount.to_le_bytes());
    out[8..16].copy_from_slice(&inv.quantity.to_le_bytes());
    out[16..24].copy_from_slice(&inv.unit_price.to_le_bytes());
    out[24..28].copy_from_slice(&inv.currency.to_le_bytes());
    out[28..60].copy_from_slice(&inv.buyer_id);
    out[60..92].copy_from_slice(&inv.seller_id);
    out[92..100].copy_from_slice(&bol.ship_date.to_le_bytes());
    out[100..132].copy_from_slice(&bol.origin_id);
    out[132..140].copy_from_slice(&bol.bol_number.to_le_bytes());
    out[140..172].copy_from_slice(&bol.carrier_id);
    out[172..180].copy_from_slice(&docs.buyer_balance.to_le_bytes());
    out[180..212].copy_from_slice(&docs.disclosure_blinding);
    out
}

/// Recompute the per-field commitments from a decrypted opening blob. Produces
/// the same array as `disclosure_field_commitments`, so the auditor can verify
/// the overall commitment from the opening alone.
pub fn field_commitments_from_opening(
    o: &[u8; DISCLOSURE_OPENING_LEN],
) -> [[u8; 32]; DISCLOSURE_FIELDS] {
    let mut master = [0u8; 32];
    master.copy_from_slice(&o[180..212]);
    [
        commit_field(F_AMOUNT, &master, &o[0..8]),
        commit_field(F_QUANTITY, &master, &o[8..16]),
        commit_field(F_UNIT_PRICE, &master, &o[16..24]),
        commit_field(F_CURRENCY, &master, &o[24..28]),
        commit_field(F_BUYER_ID, &master, &o[28..60]),
        commit_field(F_SELLER_ID, &master, &o[60..92]),
        commit_field(F_SHIP_DATE, &master, &o[92..100]),
        commit_field(F_ORIGIN_ID, &master, &o[100..132]),
        commit_field(F_BOL_NUMBER, &master, &o[132..140]),
        commit_field(F_CARRIER_ID, &master, &o[140..172]),
        commit_field(F_BUYER_BALANCE, &master, &o[172..180]),
    ]
}

/// Overall selective-disclosure commitment: H(c_0 || c_1 || ... || c_{N-1}).
/// Committed in the journal so the auditor's later (possibly partial) disclosure
/// is provably the same document set that settled, while leaking nothing on-chain.
pub fn disclosure_commitment(docs: &DocumentSet) -> [u8; 32] {
    commitment_from_fields(&disclosure_field_commitments(docs))
}

/// Combine per-field commitments into the overall commitment. The auditor calls
/// this with revealed c_i recomputed and hidden c_j supplied, to verify a match.
pub fn commitment_from_fields(fields: &[[u8; 32]; DISCLOSURE_FIELDS]) -> [u8; 32] {
    let mut h = Sha256::new();
    let mut i = 0;
    while i < DISCLOSURE_FIELDS {
        h.update(fields[i]);
        i += 1;
    }
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
    h.update(t.currency.to_le_bytes());
    h.update(t.buyer_id);
    h.update(t.seller_id);
    h.update(t.approved_root);
    h.update(t.origins_root);
    h.update(t.issuer_pubkey);
    h.update(t.auditor_pubkey);
    h.finalize().into()
}

/// Pack the public journal into its fixed 80-byte on-chain layout.
pub fn pack_journal(
    lc_id: u64,
    release_amount: u64,
    terms_digest: &[u8; 32],
    disclosure_commitment: &[u8; 32],
) -> [u8; JOURNAL_LEN] {
    let mut out = [0u8; JOURNAL_LEN];
    out[0..8].copy_from_slice(&lc_id.to_le_bytes());
    out[8..16].copy_from_slice(&release_amount.to_le_bytes());
    out[16..48].copy_from_slice(terms_digest);
    out[48..80].copy_from_slice(disclosure_commitment);
    out
}
