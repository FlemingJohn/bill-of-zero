#!/usr/bin/env bash
set -e
export PATH="/usr/bin:/bin:$HOME/.cargo/bin"
cd "$HOME/bill-of-zero"
RPC=https://soroban-testnet.stellar.org
PP="Test SDF Network ; September 2015"

OUT=$(stellar contract deploy \
  --wasm contracts/escrow/target/wasm32v1-none/release/escrow.wasm \
  --source-account deployer --rpc-url "$RPC" --network-passphrase "$PP" 2>&1)
ESC=$(echo "$OUT" | grep -oE 'C[A-Z0-9]{55}' | tail -1)
echo "NEW_ESCROW=$ESC"

stellar contract invoke --id "$ESC" --source-account deployer \
  --rpc-url "$RPC" --network-passphrase "$PP" -- init \
  --lc_id 1001 \
  --terms_digest ae47eeb66fa0be78bf4ad2651264d30fc61176cf7fa3f4ccce3c96e1600dacf4 \
  --image_id f36262818bcaf6b7a19e492ad465191632cb7616c9f5c6e7f7bc1910ffb87421 \
  --router CDA5J4P2VYDTBFZTDV6Y3YX3C3WKQGGCVQASOJW2FEN55DHRDGFN5BTU \
  --token CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC \
  --seller GDERZ3SXYJD74D54EDSCDDXZ7YC7TTNDXFRXN3MJFAG5P44DAUP2S6YX \
  --buyer GB7AMGGO45LKOIUNIUIPMYFDIOC5NJP5TWYTQBEUQW7DOYG36IPH4NTL \
  --expiry 1893456000 \
  --poseidon CDZHBSVGGFAESS56FXKIJ4MMCSLITN5SYPNL4VZTAGLLDJXDNBQBQ76Q 2>&1 | tail -2
echo "DONE_ESCROW=$ESC"
