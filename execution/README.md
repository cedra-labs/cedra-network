---
id: executor
title: Executor
custom_edit_url: https://github.com/cedra-labs/cedra-network/edit/main/executor/README.md
---


## Overview

The Cedra Blockchain is a replicated state machine. Each validator is a replica
of the system. Starting from genesis state S<sub>0</sub>, each transaction
T<sub>i</sub> updates previous state S<sub>i-1</sub> to S<sub>i</sub>. Each
S<sub>i</sub> is a mapping from accounts (represented by 32-byte addresses) to
some data associated with each account.

The execution component takes the ordered transactions, computes the output
for each transaction via the Move virtual machine, applies the output on the
previous state, and generates the new state. The execution system cooperates
with the consensus algorithm to help it agree on a proposed set of transactions and their execution. Such a
group of transactions is a block. Unlike in other blockchain systems, blocks
have no significance other than being a batch of transactions — every
transaction is identified by its position within the ledger, which is also
referred to as its "version". Each consensus participant builds a tree of blocks
like the following:

```
                   ┌-- C
          ┌-- B <--┤
          |        └-- D
<--- A <--┤                            (A is the last committed block)
          |        ┌-- F <--- G
          └-- E <--┤
                   └-- H

          ↓  After committing block E

                 ┌-- F <--- G
<--- A <--- E <--┤                     (E is the last committed block)
                 └-- H
```

A block is a list of transactions that should be applied in the given order once
the block is committed. Each path from the last committed block to an
uncommitted block forms a valid chain. Regardless of the commit rule of the
consensus algorithm, there are two possible operations on this tree:

1. Adding a block to the tree using a given parent and extending a specific
   chain (for example, extending block `F` with the block `G`). When we extend a
   chain with a new block, the block should include the correct execution
   results of the transactions in the block as if all its ancestors have been
   committed in the same order. However, all the uncommitted blocks and their
   execution results are held in some temporary location and are not visible to
   external clients.
2. Committing a block. As consensus collects more and more votes on blocks, it
   decides to commit a block and all its ancestors according to some specific
   rules. Then we save all these blocks to permanent storage and also discard
   all the conflicting blocks at the same time.

Therefore, the executor component provides two primary APIs - `execute_block`
and `commit_block` - to support the above operations.

## Implementation Details

The state at each version is represented as a sparse Merkle tree in storage.
When a transaction modifies an account, the account and the siblings from the
root of the Merkle tree to the account are loaded into memory. For example, if
we execute a transaction T<sub>i</sub> on top of the committed state and the
transaction modified account `A`, we will end up having the following tree:

```
             S_i
            /   \
           o     y
          / \
         x   A
```

In the tree shown above, `A` has the new state of the account, and `y` and `x`
are the siblings on the path from the root of the tree to `A`. If the next
transaction T<sub>i+1</sub> modified another account `B` that lives in the
subtree at `y`, a new tree will be constructed, and the structure will look
like the following:

```
                S_i        S_{i+1}
               /   \      /       \
              /     y   /          \
             / _______/             \
            //                       \
           o                          y'
          / \                        / \
         x   A                      z   B
```

Using this structure, we are able to query the global state, taking into account
the output of uncommitted transactions. For example, if we want to execute
another transaction T<sub>i+1</sub><sup>'</sup>, we can use the tree
S<sub>i</sub>. If we look for account A, we can find its new value in the tree.
Otherwise, we know the account does not exist in the tree, and we can fall back on
storage. As another example, if we want to execute transaction T<sub>i+2</sub>,
we can use the tree S<sub>i+1</sub> that has updated values for both account `A`
and `B`.

## Configs

The executors cares about these config items. One needs to specify `genesis_file_location`
in the Cedra Node config file for it to work, but can leave other entries
default by not specifying them.

```yaml
execution:
    # see https://github.com/cedra-labs/cedra-network/blob/main/config/src/config/test_data/public_full_node.yaml
    # for explanation
    genesis_file_location: ""
    # Determines how many threads the Parallel Executor spawns for transaction execution.
    # This is a mixed CPU and IO workload, and a number greater than the number
    # of CPUs is ignored and the Parallel Executor will use the number of CPUs
    # instead.
    concurrency_level: 8
    # Determines how many threads the AsyncProofFetch spawns, which is used to
    # fetch state proof in parallel with transaction execution, this is IO bound
    # workload and we think the default value is good for most.
    num_proof_reading_threads: 32
```
