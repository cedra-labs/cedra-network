# README

To play with these examples:
1. Clone this repo by running `git clone https://github.com/cedra-labs/cedra-network.git`
2. Open a new terminal and navigate to this folder by running `cd cedra-move/move-examples`
3. Navigate into the specific tutorial you are interested (ex. `cd hello_blockchain`)
4. You can use the Cedra CLI to compile, test, publish, and run these contracts by using the commands outlined here: https://cedra.dev/move/move-on-cedra/cli/
     - If you need to install the Cedra CLI, you can follow these instructions: https://cedra.dev/tools/cedra-cli/install-cli/

**WARNING:** These Move examples have NOT been audited. If you are using them in a production system, proceed at your own risk.
Particular care should be taken with Move examples that contain complex cryptographic code (e.g., `drand`).

# Additional Resources

-  [Cedra Learn](https://learn.cedralabs.com/code-examples/) provides more step-by-step guides on some high-quality examples. 
- We also have another repo [move-by-examples](https://github.com/cedra-labs/move-by-examples). It has more newer examples and is actively maintained.

# Contributing

## Writing a Move example

When creating a Move example, make the directory name be the same as the source file name and as the package name.

For example, for the `drand` randomness beacon example, create a `drand` directory with a `sources/drand.move` file in it that has a `module drand::some_module_name { /* ... */ }` in it.
This is because the testing harness will only assign an address to `drand`, based on the directory name, not based on what the named address is in `drand.move`.

## Running tests

To run the tests for **all** examples:

```
cargo test -- --nocapture
```

To run tests for a specific example (e.g., `hello_blockchain`):

```
cargo test -- hello_blockchain --nocapture
```
