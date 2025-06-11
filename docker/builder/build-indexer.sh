#!/bin/bash
# Copyright (c) Cedra
# SPDX-License-Identifier: Apache-2.0
set -e

PROFILE=${PROFILE:-release}

echo "Building indexer and related binaries"
echo "PROFILE: $PROFILE"

echo "CARGO_TARGET_DIR: $CARGO_TARGET_DIR"

# Build all the rust binaries
cargo build --locked --profile=$PROFILE \
    -p cedra-indexer-grpc-cache-worker \
    -p cedra-indexer-grpc-file-store \
    -p cedra-indexer-grpc-data-service \
    -p cedra-nft-metadata-crawler \
    -p cedra-indexer-grpc-file-checker \
    -p cedra-indexer-grpc-data-service-v2 \
    -p cedra-indexer-grpc-manager \
    "$@"

# After building, copy the binaries we need to `dist` since the `target` directory is used as docker cache mount and only available during the RUN step
BINS=(
    cedra-indexer-grpc-cache-worker
    cedra-indexer-grpc-file-store
    cedra-indexer-grpc-data-service
    cedra-nft-metadata-crawler
    cedra-indexer-grpc-file-checker
    cedra-indexer-grpc-data-service-v2
    cedra-indexer-grpc-manager
)

mkdir dist

for BIN in "${BINS[@]}"; do
    cp $CARGO_TARGET_DIR/$PROFILE/$BIN dist/$BIN
done
