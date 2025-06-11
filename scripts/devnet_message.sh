#!/bin/bash -xe

TEMP="$(mktemp)"

curl https://devnet.cedra.dev/v1 > "$TEMP"

COMMIT="$(jq -r .git_hash "$TEMP")"
CHAIN_ID="$(jq -r .chain_id "$TEMP")"

DIGEST="$(crane digest cedralabs/validator:devnet_"$COMMIT")"
GENESIS_SHA="$(curl https://devnet.cedralabs.com/genesis.blob | shasum -a 256 | awk '{print $1}')"
WAYPOINT="$(curl https://devnet.cedralabs.com/waypoint.txt)"

cat <<EOF

Hey @everyone devnet finished release, please update your fullnodes now!

For upgrade, make sure you pulled the latest docker image, or build the rust binary from the latest devnet branch. To confirm:

- Devnet branch commit: $COMMIT
- Docker image tag: devnet_$COMMIT
- Docker image digest: $DIGEST
- genesis.blob sha256: $GENESIS_SHA
- waypoint: $WAYPOINT
- Chain ID: $CHAIN_ID
You can follow the instructions here for upgrade: https://cedra.dev/nodes/full-node/update-fullnode-with-new-devnet-releases

EOF
