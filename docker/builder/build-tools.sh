#!/bin/bash
# Copyright (c) Cedra
# SPDX-License-Identifier: Apache-2.0
set -e

PROFILE=cli

echo "Building tools and services docker images"
echo "PROFILE: $PROFILE"
echo "CARGO_TARGET_DIR: $CARGO_TARGET_DIR"

# Build all the rust binaries
cargo build --locked --profile=$PROFILE \
    -p cedra \
    -p cedra-backup-cli \
    -p cedra-faucet-service \
    -p cedra-fn-check-client \
    -p cedra-node-checker \
    -p cedra-openapi-spec-generator \
    -p cedra-telemetry-service \
    -p cedra-keyless-pepper-service \
    -p cedra-debugger \
    -p cedra-transaction-emitter \
    -p cedra-api-tester \
    "$@"

# After building, copy the binaries we need to `dist` since the `target` directory is used as docker cache mount and only available during the RUN step
BINS=(
    cedra
    cedra-faucet-service
    cedra-node-checker
    cedra-openapi-spec-generator
    cedra-telemetry-service
    cedra-keyless-pepper-service
    cedra-fn-check-client
    cedra-debugger
    cedra-transaction-emitter
    cedra-api-tester
)

mkdir dist

for BIN in "${BINS[@]}"; do
    cp $CARGO_TARGET_DIR/$PROFILE/$BIN dist/$BIN
done

# Build the Cedra Move framework and place it in dist. It can be found afterwards in the current directory.
echo "Building the Cedra Move framework..."
(cd dist && cargo run --locked --profile=$PROFILE --package cedra-framework -- release)
