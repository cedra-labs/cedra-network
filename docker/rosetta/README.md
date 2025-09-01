# Rosetta API Dockerfile

This directory contains a Dockerfile meant to build a [Rosetta compliant Docker image](https://www.rosetta-api.org/docs/node_deployment.html) of Cedra.

## One-shot devnet deployment

Run the following commands for testing only. For production make sure you read the remainder of this README and adjust the steps as necessary.
This will build an image from the main branch of cedra-network at the time you run it.

```
./docker/rosetta/docker-build-rosetta.sh && \
mkdir -p data && \
cp config/src/config/test_data/public_full_node.yaml data/fullnode.yaml && \
curl -o data/genesis.blob https://devnet.cedralabs.com/genesis.blob && \
curl -o data/waypoint.txt https://devnet.cedralabs.com/waypoint.txt && \
docker run -p 8082:8082 --rm -v $(pwd)/data:/opt/cedra/data cedra-network:rosetta-latest online --config /opt/cedra/data/fullnode.yaml
```

## How to build the image

Use either option

Option 1:

```
GIT_REF=main docker/rosetta/docker-build-rosetta.sh
```

Option 2:

```
docker buildx build --file docker/rosetta/rosetta.Dockerfile --build-arg=GIT_REF=<GIT_REF_YOU_WANT_TO_BUILD> -t cedra-network:rosetta-<GIT_REF_YOU_WANT_TO_BUILD> -t cedra-network:rosetta-latest .
```

## How to run

The rosetta docker image contains a single binary `cedra-rosetta` which is meant to run a fullnode and rosetta API:

In order to run it, create a `data` directory and put a `fullnode.yaml`, `genesis.blob` and `waypoint.txt` into it.
Since cedra-rosetta is essentially just a special fullnode with a rosetta API, you can follow these instructions to fetch or create these config files: https://cedra.dev/nodes/full-node/fullnode-source-code-or-docker.

Once you've built the image and put all the config data in the `data` directory you can run cedra-rosetta via:

**online mode**

```
docker run -p 8082:8082 --rm -v $(pwd)/data:/opt/cedra cedra-network:rosetta-latest online --config /opt/cedra/fullnode.yaml
```

**offline mode**

```
docker run -p 8082:8082 --rm -v $(pwd)/data:/opt/cedra cedra-network:rosetta-latest offline
```

The Rosetta API is available under: http://localhost:8082
