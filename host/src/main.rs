// Bill of Zero — host / prover.
//
// Reads a (private) document set + the public LC terms, runs the LC-compliance
// guest in the RISC Zero zkVM, produces a Groth16 proof, and prints the exact
// values the on-chain escrow needs:
//   - image_id      : pins which guest ran (escrow stores this)
//   - terms_digest  : binds the proof to the real LC terms (escrow stores this)
//   - seal          : the Groth16 proof (passed to escrow.release)
//   - journal       : the 48-byte public output (passed to escrow.release)
//   - journal_digest : sha256(journal), what the verifier checks
//
// Usage: host <lc_terms.json> <docs.json>
// Set RISC0_DEV_MODE=1 to skip real proving while iterating on logic.

use std::path::PathBuf;

use anyhow::{Context, Result};
use bz_core::{id_from_name, terms_digest, BillOfLading, DocumentSet, Invoice, LcTerms};
use methods::{LC_CHECK_ELF, LC_CHECK_ID};
use risc0_ethereum_contracts::encode_seal;
use risc0_zkvm::sha::Digest;
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts};
use serde::Deserialize;
use sha2::{Digest as _, Sha256};

#[derive(Deserialize)]
struct LcTermsJson {
    lc_id: u64,
    credit_limit_usdc: u64,
    shipment_deadline_unix: u64,
    buyer: String,
    seller: String,
}

#[derive(Deserialize)]
struct InvoiceJson {
    amount_usdc: u64,
    buyer: String,
    seller: String,
}

#[derive(Deserialize)]
struct BolJson {
    ship_date_unix: u64,
    buyer: String,
    seller: String,
}

#[derive(Deserialize)]
struct DocsJson {
    invoice: InvoiceJson,
    bill_of_lading: BolJson,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let mut args = std::env::args().skip(1);
    let terms_path =
        PathBuf::from(args.next().unwrap_or_else(|| "sample_data/lc_terms.json".into()));
    let docs_path =
        PathBuf::from(args.next().unwrap_or_else(|| "sample_data/docs_valid.json".into()));

    // --- Load inputs -------------------------------------------------------
    let t: LcTermsJson = serde_json::from_str(
        &std::fs::read_to_string(&terms_path)
            .with_context(|| format!("reading {}", terms_path.display()))?,
    )?;
    let d: DocsJson = serde_json::from_str(
        &std::fs::read_to_string(&docs_path)
            .with_context(|| format!("reading {}", docs_path.display()))?,
    )?;

    let terms = LcTerms {
        lc_id: t.lc_id,
        credit_limit: t.credit_limit_usdc,
        deadline: t.shipment_deadline_unix,
        buyer_id: id_from_name(&t.buyer),
        seller_id: id_from_name(&t.seller),
    };
    let docs = DocumentSet {
        invoice: Invoice {
            amount: d.invoice.amount_usdc,
            buyer_id: id_from_name(&d.invoice.buyer),
            seller_id: id_from_name(&d.invoice.seller),
        },
        bill_of_lading: BillOfLading {
            ship_date: d.bill_of_lading.ship_date_unix,
            buyer_id: id_from_name(&d.bill_of_lading.buyer),
            seller_id: id_from_name(&d.bill_of_lading.seller),
        },
    };

    // --- Prove -------------------------------------------------------------
    eprintln!("Proving LC compliance for lc_id={} ...", terms.lc_id);
    let env = ExecutorEnv::builder()
        .write(&terms)?
        .write(&docs)?
        .build()?;

    let receipt = default_prover()
        .prove_with_opts(env, LC_CHECK_ELF, &ProverOpts::groth16())
        .context("proving failed — for compliant docs this means the prover/Docker setup; for tampered docs the guest panicked (expected)")?
        .receipt;

    // Sanity: the receipt verifies against our guest image id.
    receipt.verify(LC_CHECK_ID)?;

    // --- Extract on-chain values ------------------------------------------
    let seal = encode_seal(&receipt)?;
    let journal = receipt.journal.bytes.clone();
    let journal_digest = Sha256::digest(&journal);
    let image_id = Digest::from(LC_CHECK_ID);
    let td = terms_digest(&terms);

    println!("\n=== Bill of Zero — proof generated ===");
    println!("lc_id          : {}", terms.lc_id);
    println!("image_id       : {}", hex::encode(image_id.as_bytes()));
    println!("terms_digest   : {}", hex::encode(td));
    println!("journal        : {}", hex::encode(&journal));
    println!("journal_digest : {}", hex::encode(journal_digest));
    println!("seal           : {}", hex::encode(&seal));
    Ok(())
}
