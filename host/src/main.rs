// Bill of Zero — host / prover.
//
// Reads a (private) document set + the public LC terms, builds the auxiliary
// witnesses (approved-exporter Merkle proof), runs the LC-compliance guest in
// the RISC Zero zkVM, produces a Groth16 proof, and prints the exact values the
// on-chain escrow needs:
//   - image_id      : pins which guest ran (escrow stores this)
//   - terms_digest  : binds the proof to the real LC terms (escrow stores this)
//   - seal          : the Groth16 proof (passed to escrow.release)
//   - journal       : the 48-byte public output (passed to escrow.release)
//   - journal_digest : sha256(journal), what the verifier checks
//
// Usage: host <lc_terms.json> <docs.json> [approved_sellers.json]
// Set RISC0_DEV_MODE=1 to skip real proving while iterating on logic.

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use bz_core::{
    doc_digest, hash_pair, id_from_name, merkle_root, terms_digest, BillOfLading, DocumentSet,
    Invoice, LcTerms, MerkleProof, TREE_DEPTH,
};
use ed25519_dalek::{Signer, SigningKey};
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
    /// Feature 3 (range proof): buyer's private escrow balance.
    buyer_balance_usdc: u64,
}

// --- Merkle helpers (host side) -------------------------------------------

/// Build all tree levels (level 0 = padded leaves) for a fixed-depth tree.
fn build_tree(mut leaves: Vec<[u8; 32]>) -> Vec<Vec<[u8; 32]>> {
    let width = 1usize << TREE_DEPTH;
    assert!(leaves.len() <= width, "too many approved sellers for tree depth");
    leaves.resize(width, [0u8; 32]); // pad with zero leaves
    let mut levels = vec![leaves];
    while levels.last().unwrap().len() > 1 {
        let cur = levels.last().unwrap();
        let mut next = Vec::with_capacity(cur.len() / 2);
        for pair in cur.chunks(2) {
            next.push(hash_pair(&pair[0], &pair[1]));
        }
        levels.push(next);
    }
    levels
}

/// Produce the inclusion proof (siblings + index bits) for leaf `idx`.
fn proof_for(levels: &[Vec<[u8; 32]>], idx: usize) -> MerkleProof {
    let mut siblings = [[0u8; 32]; TREE_DEPTH];
    let mut i = idx;
    for level in 0..TREE_DEPTH {
        siblings[level] = levels[level][i ^ 1];
        i /= 2;
    }
    MerkleProof {
        siblings,
        index_bits: idx as u32,
    }
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
    let approved_path = PathBuf::from(
        args.next()
            .unwrap_or_else(|| "sample_data/approved_sellers.json".into()),
    );

    // --- Load inputs -------------------------------------------------------
    let t: LcTermsJson = serde_json::from_str(
        &std::fs::read_to_string(&terms_path)
            .with_context(|| format!("reading {}", terms_path.display()))?,
    )?;
    let d: DocsJson = serde_json::from_str(
        &std::fs::read_to_string(&docs_path)
            .with_context(|| format!("reading {}", docs_path.display()))?,
    )?;
    let approved_names: Vec<String> = serde_json::from_str(
        &std::fs::read_to_string(&approved_path)
            .with_context(|| format!("reading {}", approved_path.display()))?,
    )?;

    // --- Feature 1: build the approved-exporter tree + the seller's proof --
    let leaves: Vec<[u8; 32]> = approved_names.iter().map(|n| id_from_name(n)).collect();
    let levels = build_tree(leaves.clone());
    let approved_root = levels.last().unwrap()[0];

    let seller_id = id_from_name(&t.seller);
    let seller_idx = leaves
        .iter()
        .position(|l| *l == seller_id)
        .with_context(|| format!("seller '{}' is not in the approved-exporter list", t.seller))?;
    let seller_merkle = proof_for(&levels, seller_idx);

    // Sanity: the proof we built must reconstruct the root.
    if merkle_root(&seller_id, &seller_merkle) != approved_root {
        bail!("internal error: constructed Merkle proof does not match root");
    }

    let invoice = Invoice {
        amount: d.invoice.amount_usdc,
        buyer_id: id_from_name(&d.invoice.buyer),
        seller_id: id_from_name(&d.invoice.seller),
    };
    let bill_of_lading = BillOfLading {
        ship_date: d.bill_of_lading.ship_date_unix,
        buyer_id: id_from_name(&d.bill_of_lading.buyer),
        seller_id: id_from_name(&d.bill_of_lading.seller),
    };

    // --- Feature 4: the issuer signs the documents ------------------------
    // TEST ONLY: a deterministic issuer key derived from a fixed seed so the
    // demo is reproducible. In production this key belongs to the carrier /
    // issuing bank and the signature would arrive with the real documents.
    let issuer = SigningKey::from_bytes(&[7u8; 32]);
    let issuer_pubkey = issuer.verifying_key().to_bytes();
    let sig = issuer.sign(&doc_digest(&invoice, &bill_of_lading)).to_bytes();
    let mut issuer_sig = [[0u8; 32]; 2];
    issuer_sig[0].copy_from_slice(&sig[..32]);
    issuer_sig[1].copy_from_slice(&sig[32..]);

    let terms = LcTerms {
        lc_id: t.lc_id,
        credit_limit: t.credit_limit_usdc,
        deadline: t.shipment_deadline_unix,
        buyer_id: id_from_name(&t.buyer),
        seller_id,
        approved_root,
        issuer_pubkey,
    };
    let docs = DocumentSet {
        invoice,
        bill_of_lading,
        buyer_balance: d.buyer_balance_usdc,
        seller_merkle,
        issuer_sig,
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
    println!("approved_root  : {}", hex::encode(approved_root));
    println!("image_id       : {}", hex::encode(image_id.as_bytes()));
    println!("terms_digest   : {}", hex::encode(td));
    println!("journal        : {}", hex::encode(&journal));
    println!("journal_digest : {}", hex::encode(journal_digest));
    println!("seal           : {}", hex::encode(&seal));
    Ok(())
}
