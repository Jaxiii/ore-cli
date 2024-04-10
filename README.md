# Without auto restart upon failure

```sh
cargo build --release
```

```sh
./target/release/ore --rpc <YOUR-RPC-CLIENT> --jito-client "https://mainnet.block-engine.jito.wtf/api/v1/transactions" --keypair ~/.config/solana/id.json --priority-fee 1001 --jito-enable --jito-fee 600000 
mine --threads 8
```

# Without auto restart upon failure - DEFAULT

```sh
./supervision.sh
```

# Without auto restart upon failure - Custom

```sh
./supervision.sh <YOUR-RPC-CLIENT> <KEY_PATH> <PRIORITY-FEE> <JITO-FEE> <THREADS>

```