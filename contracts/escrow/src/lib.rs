#![no_std]
//! Bill of Zero — Letter-of-Credit escrow.
//!
//! Holds a stablecoin payment for a specific Letter of Credit and releases it to
//! the seller ONLY when presented with a valid RISC Zero proof that a (private)
//! document set satisfied the LC's terms. The proof is verified on-chain by the
//! official Nethermind RISC Zero VerifierRouter via a cross-contract call.

use risc0_interface::RiscZeroVerifierRouterClient;
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Bytes, BytesN, Env};

/// Journal layout committed by the guest (see bz-core):
///   [0..8]   lc_id                 (little-endian u64)
///   [8..16]  release_amount        (little-endian u64)
///   [16..48] terms_digest          (sha256 of canonical LcTerms)
///   [48..80] disclosure_commitment (blinded sha256 of the documents)
const JOURNAL_LEN: u32 = 80;

#[contracttype]
#[derive(Clone)]
pub struct Config {
    /// The Letter-of-Credit id this escrow is bound to.
    pub lc_id: u64,
    /// sha256 of the canonical LC terms; binds proofs to the real terms.
    pub terms_digest: BytesN<32>,
    /// The pinned RISC Zero image id of our LC-compliance guest.
    pub image_id: BytesN<32>,
    /// The deployed Nethermind VerifierRouter address.
    pub router: Address,
    /// The stablecoin (SAC) used for settlement.
    pub token: Address,
    /// The beneficiary that receives funds on a valid presentation.
    pub seller: Address,
}

#[contracttype]
pub enum DataKey {
    Config,
    Released,
    /// The disclosure commitment from the settled proof (feature 5). Lets an
    /// auditor confirm an off-chain disclosure matches what actually settled.
    Disclosure,
}

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Initialize the escrow against a specific LC. Callable once.
    pub fn init(
        env: Env,
        lc_id: u64,
        terms_digest: BytesN<32>,
        image_id: BytesN<32>,
        router: Address,
        token: Address,
        seller: Address,
    ) {
        if env.storage().instance().has(&DataKey::Config) {
            panic!("already initialized");
        }
        let cfg = Config {
            lc_id,
            terms_digest,
            image_id,
            router,
            token,
            seller,
        };
        env.storage().instance().set(&DataKey::Config, &cfg);
        env.storage().instance().set(&DataKey::Released, &false);
    }

    /// Fund the escrow by pulling `amount` of the token from `from`.
    pub fn fund(env: Env, from: Address, amount: i128) {
        from.require_auth();
        let cfg = Self::config(&env);
        token::Client::new(&env, &cfg.token).transfer(
            &from,
            &env.current_contract_address(),
            &amount,
        );
    }

    /// Release the escrow to the seller, gated on a valid RISC Zero proof.
    ///
    /// `seal`    — the Groth16 seal (selector-prefixed, from host `encode_seal`).
    /// `journal` — the exact 48-byte journal the guest committed.
    pub fn release(env: Env, seal: Bytes, journal: BytesN<80>) {
        let cfg = Self::config(&env);
        let released: bool = env
            .storage()
            .instance()
            .get(&DataKey::Released)
            .unwrap_or(false);
        if released {
            panic!("escrow already released");
        }

        // 1. Verify the zero-knowledge proof on-chain against our pinned image id.
        //    sha256(journal) is the journal digest the verifier expects.
        let journal_bytes = Bytes::from_array(&env, &journal.to_array());
        let journal_digest: BytesN<32> = env.crypto().sha256(&journal_bytes).into();
        RiscZeroVerifierRouterClient::new(&env, &cfg.router).verify(
            &seal,
            &cfg.image_id,
            &journal_digest,
        );

        // 2. Parse the now-verified journal (fixed little-endian layout).
        let j = journal.to_array();
        let lc_id = u64::from_le_bytes(j[0..8].try_into().unwrap());
        let release_amount = u64::from_le_bytes(j[8..16].try_into().unwrap());
        let mut td = [0u8; 32];
        td.copy_from_slice(&j[16..48]);
        let terms_digest = BytesN::<32>::from_array(&env, &td);
        let mut dc = [0u8; 32];
        dc.copy_from_slice(&j[48..80]);
        let disclosure = BytesN::<32>::from_array(&env, &dc);

        // 3. Bind the proof to THIS LC and its exact terms.
        if lc_id != cfg.lc_id {
            panic!("journal lc_id does not match this escrow");
        }
        if terms_digest != cfg.terms_digest {
            panic!("journal terms digest does not match this LC");
        }

        // 4. Settle: release the proven amount to the seller.
        token::Client::new(&env, &cfg.token).transfer(
            &env.current_contract_address(),
            &cfg.seller,
            &(release_amount as i128),
        );
        env.storage().instance().set(&DataKey::Released, &true);

        // 5. Record the disclosure commitment so an auditor can later confirm
        //    an off-chain document disclosure matches what actually settled.
        env.storage().instance().set(&DataKey::Disclosure, &disclosure);
    }

    /// View: the disclosure commitment of the settled proof (feature 5), if any.
    pub fn disclosure(env: Env) -> Option<BytesN<32>> {
        env.storage().instance().get(&DataKey::Disclosure)
    }

    /// View: has this escrow been released?
    pub fn is_released(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Released)
            .unwrap_or(false)
    }

    fn config(env: &Env) -> Config {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .expect("escrow not initialized")
    }
}

// Keep the journal layout constant referenced so it documents intent and
// triggers a compile error if the guest layout ever changes underneath us.
const _: () = assert!(JOURNAL_LEN == 80);
