// Bill of Zero — Letter-of-Credit compliance guest.
//
// This program is what gets PROVEN in the RISC Zero zkVM. It reads the public
// LC terms and the PRIVATE document set, enforces the LC's conditions, and — only
// if every rule passes — commits a compact public journal. If any rule fails the
// program panics, so no proof can exist for a non-compliant presentation.
//
// Nothing about the documents (amounts beyond the released figure, goods, exact
// parties, the buyer's balance, or which approved exporter the seller is) is ever
// revealed: only the 48-byte journal leaves the zkVM.

use bz_core::{
    disclosure_commitment, doc_digest, merkle_root, pack_journal, terms_digest, DocumentSet,
    LcTerms,
};
use ed25519_dalek::{Signature, VerifyingKey};
use risc0_zkvm::guest::env;

fn main() {
    // Inputs are written by the host in this exact order.
    let terms: LcTerms = env::read();
    let docs: DocumentSet = env::read();

    // Rule 1: the invoiced amount must not exceed the LC credit limit.
    assert!(
        docs.invoice.amount <= terms.credit_limit,
        "invoice amount exceeds LC credit limit"
    );

    // Rule 2: shipment must be on or before the LC deadline.
    assert!(
        docs.bill_of_lading.ship_date <= terms.deadline,
        "shipment date is after the LC deadline"
    );

    // Rules 3 & 4: both documents must name the LC's buyer and seller.
    assert!(docs.invoice.buyer_id == terms.buyer_id, "invoice buyer != LC buyer");
    assert!(docs.invoice.seller_id == terms.seller_id, "invoice seller != LC seller");
    assert!(
        docs.bill_of_lading.buyer_id == terms.buyer_id,
        "bill-of-lading buyer != LC buyer"
    );
    assert!(
        docs.bill_of_lading.seller_id == terms.seller_id,
        "bill-of-lading seller != LC seller"
    );

    // Rule 5: the two documents must be internally consistent with each other.
    assert!(
        docs.invoice.buyer_id == docs.bill_of_lading.buyer_id,
        "buyer differs between invoice and bill of lading"
    );
    assert!(
        docs.invoice.seller_id == docs.bill_of_lading.seller_id,
        "seller differs between invoice and bill of lading"
    );

    // Rule 6 (range proof): the buyer's escrow balance must cover the full LC
    // credit line. The exact balance stays private — we only prove the bound.
    assert!(
        docs.buyer_balance >= terms.credit_limit,
        "buyer balance does not cover the LC credit line"
    );

    // Rule 6b (line-item integrity): the invoiced total must equal
    // quantity * unit_price. Checked multiply so an overflow can't sneak a
    // bogus total past the rule.
    let computed = docs
        .invoice
        .quantity
        .checked_mul(docs.invoice.unit_price)
        .expect("quantity * unit_price overflows");
    assert!(
        docs.invoice.amount == computed,
        "invoice amount does not equal quantity * unit_price"
    );

    // Rule 6c (currency): the invoice must be denominated in the LC's currency.
    assert!(
        docs.invoice.currency == terms.currency,
        "invoice currency does not match the LC currency"
    );

    // Rule 7 (Merkle membership): the seller must be on the bank's approved
    // exporter allowlist. We prove the seller_id hashes up to the LC's
    // approved_root without revealing which leaf it is.
    assert!(
        merkle_root(&docs.invoice.seller_id, &docs.seller_merkle) == terms.approved_root,
        "seller is not in the approved-exporter allowlist"
    );

    // Rule 7b (origin membership): the goods' country of origin must be on the
    // LC's allowed-origin allowlist, proved without revealing which country.
    assert!(
        merkle_root(&docs.bill_of_lading.origin_id, &docs.origin_merkle) == terms.origins_root,
        "country of origin is not in the LC's allowed-origin list"
    );

    // Rule 8 (issuer signature): the documents must be signed by the LC's
    // trusted issuer. We verify the ed25519 signature over doc_digest INSIDE the
    // zkVM, so the proof attests the documents are authentic, not just well-formed.
    let vk = VerifyingKey::from_bytes(&terms.issuer_pubkey).expect("malformed issuer public key");
    let mut sig_bytes = [0u8; 64];
    sig_bytes[..32].copy_from_slice(&docs.issuer_sig[0]);
    sig_bytes[32..].copy_from_slice(&docs.issuer_sig[1]);
    let sig = Signature::from_bytes(&sig_bytes);
    let msg = doc_digest(&docs.invoice, &docs.bill_of_lading);
    vk.verify_strict(&msg, &sig)
        .expect("issuer signature on documents is invalid");

    // All checks passed. Bind the proof to THESE LC terms, commit a blinded
    // disclosure commitment for the auditor (feature 5), and reveal only the
    // LC id and the amount to release.
    let td = terms_digest(&terms);
    let dc = disclosure_commitment(&docs);
    let journal = pack_journal(terms.lc_id, docs.invoice.amount, &td, &dc);
    env::commit_slice(&journal);
}
