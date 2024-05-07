# Poor man's tracer 🕵️

This library provides a lightweight local tracer for Ethereum Virtual Machine (EVM) execution, leveraging the power of [revm](https://github.com/bluealloy/revm) and [reth-inspector](https://github.com/paradigmxyz/evm-inspectors).


### Use Cases

* **Local Tracing:** Trace transactions and smart contract executions directly on your machine, without relying on a full node. This is ideal for:
    * **Cost-effective development:** Experiment and debug smart contracts locally without incurring the cost of running a full node.
    * **Struct Traces:** Capture detailed struct information, which may not be available from some node providers.
* **Use public RPCs:** Free public RPC endpoints like the ones listed on [chainlist.org](https://chainlist.org) don't usually directly support tracing. They can be used however to trace locally, like with this library. Note that historical tracing still requires an archive node in most cases.

**Note:** Consider [Anvil](https://book.getfoundry.sh/anvil/) for a more comprehensive solution with local tracing capabilities if you don't have specific needs for a lightweight approach.

## Usage

### Run an example 
```bash
cargo run --example geth_struct_trace --features examples
```

### Support trace types
Currently the library only supports Geth's Struct, Call and Prestate traces. Feel free to add more!

### Dependencies 
For development purposes, the library currently uses forked versions of `revm` and `reth-inspector` to ensure dependency compatibility. 
