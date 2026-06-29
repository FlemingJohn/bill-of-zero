#!/usr/bin/env bash
export PATH="/usr/bin:/bin:$HOME/.cargo/bin"
cd "$HOME/bill-of-zero"
RPC=https://soroban-testnet.stellar.org
PP="Test SDF Network ; September 2015"
ESC=CDVLQX43SC3FVCLO42AZW34O5AK35CMQBGJRBEA7C6V6RPNTYSOXS3YE
SEAL=73c457ba29017c4e6c1f7e228f2a9249fdd25cfa33ec95927f408765050f4c93349886940040a74ea35891247628e53af27bf73aecf7d51cf5dfb1ea88555bc4f607aebc165544f323db7698d0e52e88bd5823d4b3caa22f497a07a5d8935e23708ad48c292f8f02044d5aa348acf6b37bbdb7dfabea19da5db85eaa8dbce52b000838b2252c7b75885d26b2a5abddef44e53386e74e08a883c9c581b0df7dc8e0c84cb307473f99bbce3c1c6d09800e150f1d4b487d3b591d29d95ab91303db3248b423108e544374f92c7c06b4b6c2e8e3e8056bab4ee8c4f5aaad36a8ef2d6b07feb117dc180420e799782bb6025b10ca91fb1cfa028ce3d82e78b1ca44bb5c1be7db
JOURNAL=e9030000000000001873010000000000ae47eeb66fa0be78bf4ad2651264d30fc61176cf7fa3f4ccce3c96e1600dacf43768b162c6052933852cba1d2b02caef50d3a5063f08e0f173d398f7e3de0a61

echo "=== fund 100000 ==="
stellar contract invoke --id "$ESC" --source-account deployer --rpc-url "$RPC" --network-passphrase "$PP" -- \
  fund --from GB7AMGGO45LKOIUNIUIPMYFDIOC5NJP5TWYTQBEUQW7DOYG36IPH4NTL --amount 100000 2>&1 | tail -1

echo "=== release (verifies proof + stamps Poseidon receipt) ==="
stellar contract invoke --id "$ESC" --source-account deployer --rpc-url "$RPC" --network-passphrase "$PP" -- \
  release --seal "$SEAL" --journal "$JOURNAL" 2>&1 | grep -E "transfer|poseidon_receipt|tx/|error" | tail -4

echo "=== is_released? ==="
stellar contract invoke --id "$ESC" --source-account deployer --rpc-url "$RPC" --network-passphrase "$PP" --send=no -- is_released 2>/dev/null

echo "=== Poseidon receipt() ==="
stellar contract invoke --id "$ESC" --source-account deployer --rpc-url "$RPC" --network-passphrase "$PP" --send=no -- receipt 2>/dev/null
