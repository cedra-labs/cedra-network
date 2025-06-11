This is a small example of using the new `cedra` dependency. This shall be removed once we have
documentation/tests.

`pack2` contains a package which is used by `pack1` as follows:

```
[dependencies]
Pack2 = { cedra = "http://localhost:8080", address = "default" }
```

To see it working:

```shell
# Start a node with an account
cedra node run-local-testnet &
cedra account create --account default --use-faucet 
# Compile and publish pack2
cd pack2
cedra move compile --named-addresses project=default     
cedra move publish --named-addresses project=default
# Compile pack1 agains the published pack2
cd ../pack1
cedra move compile --named-addresses project=default     
```
