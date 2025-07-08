#Copyright © Cedra Foundation
# SPDX-License-Identifier: Apache-2.0

clean:
	- rm -rf go.mod go.sum cedrastdlib cedratypes

test:
	cargo run -p cedra-framework release
	cargo run -p cedra-sdk-builder -- \
		--language go --module-name cedrastdlib \
		--package-name testing \
		--with-cedra-types "../../../../testsuite/generate-format/tests/staged/cedra.yaml" \
		--target-source-dir . \
		"../../../framework/cedra-framework/build/CedraFramework"
	- go mod init testing
	go mod edit -replace testing=`pwd`
	go get github.com/cedra-labs/serde-reflection/serde-generate/runtime/golang/bcs
	go get github.com/cedra-labs/serde-reflection/serde-generate/runtime/golang/serde
	go run stdlib_demo.go

.PHONY: clean test
