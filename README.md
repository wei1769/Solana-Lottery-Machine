# Solana-Lottery-Machine

## How to use

- fill in the Private key and make a Wsol ATA on devnet first
- or change the mint to your token
- edit /rust-cli/src/main.rs file to try

```bash
cd rust-cli
cargo build --release
./target/release/lottery --help
```

### Wrap SOL (remember to do this on Devnet)

```bash
./target/release/lottery -w <amount>
```

### JS Script

```bash
cd scripts
yarn start
```
