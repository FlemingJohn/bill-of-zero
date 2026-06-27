// Bill of Zero — Letter-of-Credit compliance guest.
//
// This program is what gets PROVEN in the RISC Zero zkVM. It reads the public
// LC terms and the PRIVATE document set, enforces the LC's conditions, and — only
// if every rule passes — commits a compact public journal. If any rule fails the
// program panics, so no proof can exist for a non-compliant presentation.
//
// Nothing about the documents (amounts beyond the released figure, goods, exact
// parties) is ever revealed: only the 48-byte journal leaves the zkVM.

use bz_core::{pack_journal, terms_digest, DocumentSet, LcTerms};
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

    // All checks passed. Bind the proof to THESE LC terms and reveal only the
    // LC id and the amount to release.
    let td = terms_digest(&terms);
    let journal = pack_journal(terms.lc_id, docs.invoice.amount, &td);
    env::commit_slice(&journal);
}
