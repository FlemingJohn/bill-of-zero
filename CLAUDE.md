# Bill of Zero — agent & contributor guide

Zero-knowledge **Letter-of-Credit settlement on Stellar**. A seller proves (in a
RISC Zero zkVM) that a private set of trade documents satisfies an LC's terms;
the proof is a real Groth16 seal verified **on-chain** by the Nethermind RISC
Zero VerifierRouter, and only then does a Soroban escrow release funds to the
seller. Privacy with accountability: sensitive fields never go on-chain, but an
auditor with a view key can selectively open them.

The demo UI is a **terminal app (ratatui)** — there is no web frontend.

## Layout

| Path | Crate | What it is |
|---|---|---|
| `core/` | `bz-core` (`#![no_std]`) | Shared data model: `LcTerms`, `Invoice`, `BillOfLading`, `DocumentSet`; digests; Merkle; **granular selective-disclosure** commitments. Single source of truth for guest + host. |
| `methods/guest/` | guest | The program **proven** in the zkVM. Enforces the LC rules; panics on any violation (so no proof can exist for bad docs). |
| `host/` | `host` | Prover CLI (+ auditor). Reads JSON inputs, builds Merkle witnesses, signs as issuer, produces the Groth16 seal + 80-byte journal, encrypts the auditor disclosure. |
| `tui/` | `bz-tui` | The terminal UI. Orchestrates `host` (prove/audit) and the `stellar` CLI (fund/release/refund). Run this for the demo. |
| `contracts/escrow/` | escrow | Soroban escrow: `fund`, `release(seal, journal)` (calls the verifier router), `refund`, `disclosure`, `is_released`. |
| `contracts/poseidon-demo/` | demo | Standalone native-Poseidon receipt demo (separate SDK version; escrow untouched). |
| `deployment.json` | — | Live testnet addresses + `imageId`/`termsDigest`. The TUI reads this. |
| `sample_data/` | — | LC terms + allowlists (bank config) + sample docs. The TUI generates `.tui_docs.json` from typed input. |

## Prerequisites

- **Rust** (pinned by `rust-toolchain.toml`; tested rustc 1.96) + cargo.
- **RISC Zero** toolchain — `r0vm` (tested 3.0.5). Install via `rzup` from dev.risczero.com.
- **Docker** (tested 29.2) — required only for the **real** Groth16 (STARK→SNARK wrap). Not needed for `RISC0_DEV_MODE=1`.
- **Stellar CLI v27** (`stellar`) with two key identities: `deployer` and `seller`
  (`stellar keys generate <name> --network testnet --fund`).
- **Linux / WSL.** A real Groth16 proof peaks ~7 GB RAM; on WSL set
  `~/.wslconfig` → `[wsl2]\nmemory=24GB\nswap=16GB` and `wsl --shutdown`, else
  `r0vm` gets OOM-killed.

## Build

```bash
cargo build --release                       # core + guest + host + tui
(cd contracts/escrow && stellar contract build)   # escrow wasm (uses wasm32v1-none)
```
Note: build the escrow with `stellar contract build`, NOT raw `cargo build
--target wasm32-unknown-unknown` (that target is rejected by soroban-sdk 25+).

## Run the TUI (the demo)

```bash
cargo run -p bz-tui          # run from the repo root; reads deployment.json
```
- `Tab` switch role · `?` help · `Esc` quit.
- **Buyer:** `↑/↓` field, edit fund amount + private balance, `[f]` fund, `[x]` refund, `[s]` refresh.
- **Seller:** `↑/↓` field, type the documents, `Enter` = real proof (a few minutes), `Ctrl+R` = release.
- **Auditor:** `←/→` pick profile (tax / regulator / full), `[a]` decrypt + verify commitment match.
- Signing key = `BZ_SOURCE_KEY` env (default `deployer`).

## Host CLI (no UI)

```bash
# Fast logic check (fake seal, ~seconds) — proves the rules, NOT on-chain-verifiable:
RISC0_DEV_MODE=1 cargo run --release --bin host -- \
  sample_data/lc_terms.json sample_data/docs_valid.json sample_data/approved_sellers.json

# Real proof (Docker, minutes) — drop RISC0_DEV_MODE; seal starts with 73c457ba.

# Granular auditor view:
cargo run --release --bin host -- audit sample_data/disclosure.bin tax <escrow_disclosure_hex>
```
`docs_tampered.json` makes the guest panic on purpose → no proof (the security property).

## The LC rules (enforced in the guest)

amount ≤ credit limit · ship date ≤ deadline · invoice/BoL name the LC parties ·
docs internally consistent · buyer balance ≥ credit line (range proof) ·
`amount = quantity × unit_price` · currency = LC currency · seller ∈ approved-
exporter Merkle set · origin ∈ allowed-origin Merkle set · issuer ed25519
signature valid. Bill-of-lading number + carrier are **disclosed-only** (signed
and auditable, not gated).

## Gotchas (read before debugging)

- **`stellar` v27 flag is `--source-account`**, not `--source`. Read-only calls add `--send=no`.
- **Any guest change ⇒ new `image_id` ⇒ the escrow must be redeployed** (it pins `image_id` + `terms_digest` at `init`). Same for changing LC terms/allowlists.
- **LC terms are pinned on-chain** via `terms_digest`; the terms fed to the prover must byte-match what was deployed, or `release` rejects the proof. That's why `lc_terms.json` is fixed config, not free input.
- The disclosure commitment is **per-field** (`H(c_0..c_n)`), so an auditor can open a subset and still verify the whole commitment.

## Redeploy the escrow (after a guest/terms change)

```bash
# 1. new image_id + terms_digest (deterministic; dev mode is fine to read them):
RISC0_DEV_MODE=1 ./target/release/host sample_data/lc_terms.json sample_data/docs_valid.json sample_data/approved_sellers.json | grep -E "image_id|terms_digest"
# 2. deploy + init (reuse router/token from deployment.json):
stellar contract deploy --wasm contracts/escrow/target/wasm32v1-none/release/escrow.wasm \
  --source-account deployer --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"
stellar contract invoke --id <new_escrow> --source-account deployer --rpc-url <rpc> \
  --network-passphrase "Test SDF Network ; September 2015" -- init \
  --lc_id 1001 --terms_digest <hex> --image_id <hex> --router <C..> --token <C..> \
  --seller <G..> --buyer <G..> --expiry <unix>
# 3. update deployment.json (escrow, imageId, termsDigest).
```

## Current testnet deployment

In `deployment.json`. Escrow `CC5QLWW4LJBKF7YD56PORBXXAAGGY3QWBCW2ONFVBHJ4QBQJUEBGSAEF`,
verifier router `CDA5J4P2…`, Groth16 verifier (selector `73c457ba`) `CCKHZZY5…`,
token (SAC) `CDLZFC3S…` on Stellar testnet. View any tx at
`https://stellar.expert/explorer/testnet/tx/<hash>`.
