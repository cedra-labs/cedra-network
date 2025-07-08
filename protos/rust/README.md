# Cedra Protos

This repository contains the protobuf definitions for Cedra.

## Usage
Import generated structs like this:
```rust
use cedra_protos::transaction::v1::Transaction;
```

Then use them like this:
```rust
fn parse(transaction: Transaction) {
    // Parse the transaction.
}
```

## Contributing
See [CONTRIBUTING.md](CONTRIBUTING.md) for more information.
