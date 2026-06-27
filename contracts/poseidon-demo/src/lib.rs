#![no_std]
//! Bill of Zero — Poseidon settlement-receipt demo (feature 2).
//!
//! Standalone contract demonstrating Stellar's NATIVE Poseidon host function
//! (Protocol 25 "X-Ray", CAP-0075) running on-chain, via soroban-sdk 27's
//! `env.crypto_hazmat().poseidon_permutation(...)`. It computes a Poseidon
//! commitment over (lc_id, release_amount): a compact, ZK-friendly settlement
//! receipt that a downstream system could anchor or aggregate.
//!
//! Why this lives in its own contract: the Poseidon host function is only
//! exposed by soroban-sdk 27 (behind the `hazmat-crypto` feature), while the
//! escrow stays on soroban-sdk 25 to remain compatible with the Nethermind
//! RISC Zero verifier client. Keeping Poseidon separate means the working
//! on-chain proof verification is never put at risk.
//!
//! The MDS matrix below uses the values from the official soroban-sdk Poseidon
//! migration example; the round constants are a minimal valid set. This is a
//! demonstration instance of the permutation, not a standardized hash.

use soroban_sdk::{bytesn, contract, contractimpl, vec, Env, Symbol, Vec, U256};

const T: u32 = 2; // state width
const D: u32 = 5; // S-box degree (BN254)
const ROUNDS_F: u32 = 2; // full rounds (must be even)
const ROUNDS_P: u32 = 1; // partial rounds

/// t-by-t MDS matrix (values from the official soroban-sdk migration example).
fn mds(env: &Env) -> Vec<Vec<U256>> {
    vec![
        env,
        vec![
            env,
            U256::from_be_bytes(
                env,
                &bytesn!(env, 0x066f6f85d6f68a85ec10345351a23a3aaf07f38af8c952a7bceca70bd2af7ad5)
                    .into(),
            ),
            U256::from_be_bytes(
                env,
                &bytesn!(env, 0x2b9d4b4110c9ae997782e1509b1d0fdb20a7c02bbd8bea7305462b9f8125b1e8)
                    .into(),
            ),
        ],
        vec![
            env,
            U256::from_be_bytes(
                env,
                &bytesn!(env, 0x0cc57cdbb08507d62bf67a4493cc262fb6c09d557013fff1f573f431221f8ff9)
                    .into(),
            ),
            U256::from_be_bytes(
                env,
                &bytesn!(env, 0x1274e649a32ed355a31a6ed69724e1adade857e86eb5c3a121bcd147943203c8)
                    .into(),
            ),
        ],
    ]
}

/// (ROUNDS_F + ROUNDS_P)-by-t round constants.
fn round_constants(env: &Env) -> Vec<Vec<U256>> {
    vec![
        env,
        vec![env, U256::from_u32(env, 1), U256::from_u32(env, 2)],
        vec![env, U256::from_u32(env, 3), U256::from_u32(env, 4)],
        vec![env, U256::from_u32(env, 5), U256::from_u32(env, 6)],
    ]
}

#[contract]
pub struct PoseidonDemo;

#[contractimpl]
impl PoseidonDemo {
    /// Compute a native-Poseidon settlement receipt over (lc_id, release_amount).
    /// Returns the first element of the permuted state as the commitment.
    pub fn commit(env: Env, lc_id: u64, release_amount: u64) -> U256 {
        let input = vec![
            &env,
            U256::from_u128(&env, lc_id as u128),
            U256::from_u128(&env, release_amount as u128),
        ];
        let out = env.crypto_hazmat().poseidon_permutation(
            &input,
            Symbol::new(&env, "BN254"),
            T,
            D,
            ROUNDS_F,
            ROUNDS_P,
            &mds(&env),
            &round_constants(&env),
        );
        out.get(0).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn poseidon_commitment_is_deterministic_and_binding() {
        let env = Env::default();
        let id = env.register(PoseidonDemo, ());
        let client = PoseidonDemoClient::new(&env, &id);

        let a = client.commit(&1001u64, &95000u64);
        let b = client.commit(&1001u64, &95000u64);
        let c = client.commit(&1002u64, &95000u64);

        // Native Poseidon executed on-chain: same inputs -> same receipt,
        // different LC -> different receipt.
        assert_eq!(a, b, "same inputs must give the same Poseidon commitment");
        assert_ne!(a, c, "different lc_id must change the commitment");
    }
}
