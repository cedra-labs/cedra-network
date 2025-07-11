# Indexer GRPC

Indexer GRPC efficiently serves on-chain data for low-latency indexing


## Cloud Setup

In production, Indexer GRPC currently requires access to a GCS bucket for cold storage, and thus is best run in GCP.

### Requirements

* Redis with at least 75 GB of memory.
* If using the GCS File Operator, you also need Google Cloud Storage with one bucket and one service account JSON, which should be assigned as `Object Owner` and `Bucket Owner`.
  * `Object Owner` is to raed and write to each file.
  * `Bucket Owner` is to verify the bucket existence.

### General Startup

The implementation is up to the operator, but in general:
* Start the full node and cache worker (for more information, refer to `indexer-grpc-cache-worker`)
  * Note: : the cache worker will exit after 1 minute since the file store is not ready. Please restart it.
* Start the file store worker (for more information, refer to `indexer-grpc-file-store`).
* Start the data service (for more information, refer to `indexer-grpc-data-service`).

## Local Setup

These instructions set up the Indexer GRPC locally on your machine, using a local redis and file store.

### Requirements

* `grpcurl`: https://github.com/fullstorydev/grpcurl#installation
* `redis`: https://redis.io/docs/getting-started/installation/

### Running


#### 1) Running standalone indexer fullnode

Run a single cedra-node in test mode:
* Create a data directory for test, such as: `mkdir test_indexer_grpc`
* Run the fullnode, pointing at the data directory: `cargo run -p cedra-node -- --test --test-dir test_indexer_grpc`
* Stop the node temporarily: `crtl-c`
* Enable the indexer GRPC by adding the following configs to the autogenerated node config: `test_indexer_grpc/0/node.yaml`:
  * ```
    storage:
      enable_indexer: true
    
    indexer_grpc:
      enabled: true
      address: 0.0.0.0:50051
      processor_task_count: 10
      processor_batch_size: 100
      output_batch_size: 100```
* Run the fullnode again, with updated config: `cargo run -p cedra-node -- --test --test-dir test_indexer_grpc`

#### 2) Test with `grpcurl`

* Ensure `grpcurl` installed
* From the cedra-core (project base directory), try hitting a grpc endpoint directly on the fullnode: `grpcurl  -max-msg-sz 10000000 -d '{ "starting_version": 0 }' -import-path crates/cedra-protos/proto -proto cedra/internal/fullnode/v1/fullnode_data.proto  -plaintext 127.0.0.1:50051 cedra.internal.fullnode.v1.FullnodeData/GetTransactionsFromNode`

#### 3) Run redis

On MacOS, you can follow this setup in its entirety: https://redis.io/docs/getting-started/installation/install-redis-on-mac-os/
* For simplicity, run redis in the foreground: `redis-server`

#### 4) Create a local filestore

Instead of using a cloud storage bucket, you can use a local directory to store all the data in the file-store.
```
mkdir test_indexer_grpc_filestore
```

#### 5) Start each of the services

Each of these services requires its own config file, though there are some shared fields. Connection configs for redis and the fullnode should be the same.

Particularly, the `health_check_port` must be different for each config, to avoid clashing on ports on the same machine.

For the `file_store`, use the local type such as:

```
...
file_store:
  file_store_type: LocalFileStore
  local_file_store_path: test_indexer_grpc_filestore
```

At this point, redis and the fullnode should already be running. Then start the following in order:
* Start the cache-worker
* Start the file-store
* Start the data-service

#### Clean up

Clean up all the persistence layers:
* Remove redis data: `redis-cli -p 6379 flushall`
* Remove indexer fullnode data: `rm -rf test_indexer_grpc`
* Remove file store data: `rm -rf test_indexer_grpc_filestore`