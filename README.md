# Ore CLI

A command line interface for the Ore program.

## Building

To build the Ore CLI, you will need to have the Rust programming language installed. You can install Rust by following the instructions on the [Rust website](https://www.rust-lang.org/tools/install).

Once you have Rust installed, you can build the Ore CLI by running the following command:

```sh
cargo build --release
```


```sh
./target/release/ore --rpc "" --jito-client "https://mainnet.block-engine.jito.wtf/api/v1/transactions" --keypair ./id.json --priority-fee 1001 --jito-enable --jito-fee 898765 
mine --threads 8
```