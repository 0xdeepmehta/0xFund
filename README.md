# Completely decentralised fundraising and Incubation Platform.
I'm trying to make a decentralised fundraising platform with many features(for now they are vivid ideas)

## Todo (Small step at a time 😉)
[] simple fundraising contract
[] deploy on devnet
[] simple fundraising client app
[] deploy on vercel
[] ...


### Instructin to run it

Prerequisites
- Have rust and solana installed on your system.
- already have a local solana account with some sol in it

1. Clone > yarn install

2. Compile solana program uff smart-contract 😅
```cargo build-bpf --manifest-path=Cargo.toml --bpf-out-dir=dist/program```

3. Deploy the compiled program
```solana program deploy {BASE_DIR}/dist/program/program.so```

4. Replace program_id to newly deployed program id

5. start you client i.e frontend
``` yarn start ```