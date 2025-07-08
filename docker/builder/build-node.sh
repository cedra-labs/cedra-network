#!/bin/bash
# Copyright (c) Cedra
# SPDX-License-Identifier: Apache-2.0
set -e

PROFILE=${PROFILE:-release}
FEATURES=${FEATURES:-""}

echo "Building cedra-node"
echo "PROFILE: $PROFILE"
echo "FEATURES: $FEATURES"
echo "CARGO_TARGET_DIR: $CARGO_TARGET_DIR"

PACKAGES=(
    cedra-node
    cedra-forge-cli
)

# We have to do these separately because we need to avoid feature unification
# between cedra-node and other binaries
for PACKAGE in "${PACKAGES[@]}"; do
    # Build and overwrite the cedra-node binary with features if specified
    if [ -n "$FEATURES" ] && [ "$PACKAGE" = "cedra-node" ]; then
        echo "Building cedra-node with features ${FEATURES}"
        cargo build --profile=$PROFILE --features=$FEATURES -p $PACKAGE "$@"
    else 
        # Build cedra-node separately
        cargo build --locked --profile=$PROFILE -p $PACKAGE "$@"
    fi
done

# After building, copy the binaries we need to `dist` since the `target` directory is used as docker cache mount and only available during the RUN step
BINS=(
    cedra-node
    forge
)

mkdir dist

for BIN in "${BINS[@]}"; do
    cp $CARGO_TARGET_DIR/$PROFILE/$BIN dist/$BIN
done
