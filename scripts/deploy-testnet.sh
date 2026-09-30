#!/usr/bin/env bash
#
# Deploys the schoolfees contract to Stellar testnet and prints the contract id.
#
# Run this yourself; never let an agent run it. Two gates apply before it is
# useful:
#   1. The v0 fee lifecycle must be implemented (see ROADMAP.md).
#   2. A real school or tutorial centre must have agreed to try the flow, so
#      there is somebody to pilot it with.
#
# The signing identity is read from STELLAR_ACCOUNT. This script never reads,
# prints, or stores a secret key itself.
#
# Usage:
#   STELLAR_ACCOUNT=dev ./scripts/deploy-testnet.sh
#
set -euo pipefail

: "${STELLAR_ACCOUNT:?set STELLAR_ACCOUNT to a stellar keys identity name, for example: STELLAR_ACCOUNT=dev}"

WASM="target/wasm32v1-none/release/schoolfees.wasm"

if [ ! -f "$WASM" ]; then
  echo "no wasm at $WASM yet, building it first" >&2
  stellar contract build
fi

echo "deploying $WASM to testnet as '$STELLAR_ACCOUNT'" >&2

stellar contract deploy \
  --wasm "$WASM" \
  --source-account "$STELLAR_ACCOUNT" \
  --network testnet \
  --alias schoolfees
