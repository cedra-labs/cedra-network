#!/bin/bash
set -x
set -e

scriptdir="$(cd "$(dirname "$0")" >/dev/null 2>&1 && pwd)"

echo "Executing from directory: $scriptdir"

repodir=$scriptdir/..

cd $repodir

(
  echo
  echo "Regenerating protobufs (in `pwd`)"
  echo "See https://github.com/cedra-labs/cedra-private/tree/main/protos/README.md if you're having troubles"
  cd protos/
  ./scripts/build_protos.sh
)


(
  echo
  echo "Regenerating serde-reflection to track type changes over time (in `pwd`)"
  cargo run -p generate-format -- --corpus api --record
  cargo run -p generate-format -- --corpus cedra --record
  cargo run -p generate-format -- --corpus consensus --record
  cargo run -p generate-format -- --corpus network --record
  cargo run -p generate-format -- --corpus move-abi --record
)

(
  echo
  echo "Regenerating Cedra Node APIs (in `pwd`)"
  # Cedra Node API
  cargo run -p cedra-openapi-spec-generator -- -f yaml -o api/doc/spec.yaml
  cargo run -p cedra-openapi-spec-generator -- -f json -o api/doc/spec.json
)

echo
echo "WARNING: If you are adding a new transaction authenticator..."
echo " 1. Check out https://github.com/cedra-labs/cedra-network/blob/main/testsuite/generate-format/README.md"
echo "    * In particular, be sure to edit the *.yaml files in testsuite/generate-format/tests/staged"
echo " 2. ecosystem/indexer-grpc/indexer-grpc-fullnode/src/convert.rs must be manually updated"
echo
