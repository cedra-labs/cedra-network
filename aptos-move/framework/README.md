---
id: Cedra-framework
title: Cedra Framework
custom_edit_url: https://github.com/cedra-labs/cedra/edit/main/Cedra-move/Cedra-framework/README.md
---

## The Cedra Framework

The Cedra Framework defines the standard actions that can be performed on-chain
both by the Cedra VM---through the various prologue/epilogue functions---and by
users of the blockchain---through the allowed set of transactions. This
directory contains different directories that hold the source Move
modules and transaction scripts, along with a framework for generation of
documentation, ABIs, and error information from the Move source
files. See the [Layout](#layout) section for a more detailed overview of the structure.

## Documentation

Each of the main components of the Cedra Framework and contributing guidelines are documented separately. See them by version below:

* *Cedra tokens* - [main](https://github.com/cedra-labs/cedra/blob/main/cedra-move/framework/cedra-token/doc/overview.md), [testnet](https://github.com/cedra-labs/cedra/blob/testnet/cedra-move/framework/cedra-token/doc/overview.md), [devnet](https://github.com/cedra-labs/cedra/blob/devnet/cedra-move/framework/cedra-token/doc/overview.md)
* *Cedra framework* - [main](https://github.com/cedra-labs/cedra/blob/main/cedra-move/framework/cedra-framework/doc/overview.md), [testnet](https://github.com/cedra-labs/cedra/blob/testnet/cedra-move/framework/cedra-framework/doc/overview.md), [devnet](https://github.com/cedra-labs/cedra/blob/devnet/cedra-move/framework/cedra-framework/doc/overview.md)
* *Cedra stdlib* - [main](https://github.com/cedra-labs/cedra/blob/main/cedra-move/framework/cedra-stdlib/doc/overview.md), [testnet](https://github.com/cedra-labs/cedra/blob/testnet/cedra-move/framework/cedra-stdlib/doc/overview.md), [devnet](https://github.com/cedra-labs/cedra/blob/devnet/cedra-move/framework/cedra-stdlib/doc/overview.md)
* *Move stdlib* - [main](https://github.com/cedra-labs/cedra/blob/main/cedra-move/framework/move-stdlib/doc/overview.md), [testnet](https://github.com/cedra-labs/cedra/blob/testnet/cedra-move/framework/move-stdlib/doc/overview.md), [devnet](https://github.com/cedra-labs/cedra/blob/devnet/cedra-move/framework/move-stdlib/doc/overview.md)

Follow our [contributing guidelines](CONTRIBUTING.md) and basic coding standards for the Cedra Framework.

## Compilation and Generation

The documents above were created by the Move documentation generator for Cedra. It is available as part of the Cedra CLI. To see its options, run:
```shell
cedra move document --help
```

The documentation process is also integrated into the framework building process and will be automatically triggered like other derived artifacts, via `cached-packages` or explicit release building.

## Running Move tests

To test our Move code while developing the Cedra Framework, run `cargo test` inside this directory:

```
cargo test
```

(Alternatively, run `cargo test -p cedra-framework` from anywhere.)

To skip the Move prover tests, run:

```
cargo test -- --skip prover
```

To filter and run **all** the tests in specific packages (e.g., `cedra_stdlib`), run:

```
cargo test -- cedra_stdlib --skip prover
```

(See tests in `tests/move_unit_test.rs` to determine which filter to use; e.g., to run the tests in `cedra_framework` you must filter by `move_framework`.)

To **filter by test name or module name** in a specific package (e.g., run the `test_empty_range_proof` in `cedra_stdlib::ristretto255_bulletproofs`), run:

```
TEST_FILTER="test_range_proof" cargo test -- cedra_stdlib --skip prover
```

Or, e.g., run all the Bulletproof tests:
```
TEST_FILTER="bulletproofs" cargo test -- cedra_stdlib --skip prover
```

To show the amount of time and gas used in every test, set env var `REPORT_STATS=1`.
E.g.,
```
REPORT_STATS=1 TEST_FILTER="bulletproofs" cargo test -- cedra_stdlib --skip prover
```

Sometimes, Rust runs out of stack memory in dev build mode.  You can address this by either:
1. Adjusting the stack size

```
export RUST_MIN_STACK=4297152
```

2. Compiling in release mode

```
cargo test --release -- --skip prover
```

## Layout
The overall structure of the Cedra Framework is as follows:

```
├── cedra-framework                                 # Sources, testing and generated documentation for Cedra framework component
├── cedra-token                                 # Sources, testing and generated documentation for Cedra token component
├── cedra-stdlib                                 # Sources, testing and generated documentation for Cedra stdlib component
├── move-stdlib                                 # Sources, testing and generated documentation for Move stdlib component
├── cached-packages                                 # Tooling to generate SDK from move sources.
├── src                                     # Compilation and generation of information from Move source files in the Cedra Framework. Not designed to be used as a Rust library
├── releases                                    # Move release bundles
└── tests
```
