fungible_token_multisender Smart Contract
==================

A [smart contract] written in [Rust] for an app initialized with [create-near-app]


Quick Start
===========

Before you compile this code, you will need to install Rust with [correct target]


Exploring The Code
==================

   1. The main smart contract code lives in `contract/src/lib.rs`. You can compile it with
   the `./compile` script.

   2. Configure token for multisender:
   ```rust
   /// Token contract for multisend. Cross-calls allows only for this contract
   const TOKEN_CONTRACT:&str = "lnc.factory.tokenhub.testnet";
   /// Token metadata decimals for human-readable convert balances
   const TOKEN_DECIMALS:u8 = 18;
   const TOKEN_TICKER:&str = "LNC";
   ```
   for example for REF token:
   ```rust
   /// Token contract for multisend. Cross-calls allows only for this contract
   const TOKEN_CONTRACT:&str = "ref.fakes.testnet";
   /// Token metadata decimals for human-readable convert balances
   const TOKEN_DECIMALS:u8 = 18;
   const TOKEN_TICKER:&str = "REF";
   ```

  [How to know TOKEN_DECIMALS?]
  ```shell
  export TOKEN_CONTRACT=lnc.factory.tokenhub.testnet
  near view $TOKEN_CONTRACT ft_metadata
  ```

  [smart contract]: https://docs.near.org/docs/develop/contracts/overview
  [Rust]: https://www.rust-lang.org/
  [create-near-app]: https://github.com/near/create-near-app
  [correct target]: https://github.com/near/near-sdk-rs#pre-requisites
  [cargo]: https://doc.rust-lang.org/book/ch01-03-hello-cargo.html


  RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
  near dev-deploy --wasmFile target/wasm32-unknown-unknown/release/ft_multisender.wasm

```shell
export OWNER_ID=rmlsnk.testnet
export USER_ID=participant_1.testnet
export CONTRACT_ID=dev-1655201876893-47068327273231

export REF=ref.fakes.testnet
export MISTAKE=kfbfhfjfkf.testnet
export LNC=lnc.factory.tokenhub.testnet
export CHEDDAR=token-v3.cheddar.testnet
export GAS=100000000000000
```

```shell
near call $CONTRACT_ID new '{"owner_id":"'$OWNER_ID'"}' --accountId=$CONTRACT_ID
near call $CONTRACT_ID whitelist_token '{"token_id":"'$REF'"}' --accountId=$CONTRACT_ID
near call $CONTRACT_ID whitelist_token '{"token_id":"'$LNC'"}' --accountId=$CONTRACT_ID
near call $CONTRACT_ID whitelist_token '{"token_id":"'$CHEDDAR'"}' --accountId=$CONTRACT_ID
near call $CONTRACT_ID is_token_whitelisted '{"token_id":"'$LNC'"}' --accountId=$CONTRACT_ID
near call $CONTRACT_ID get_whitelisted_tokens '' --accountId=$CONTRACT_ID
near call $LNC ft_transfer_call '{"receiver_id":"'$CONTRACT_ID'", "amount":"500000000000000000", "msg":""}' --accountId $OWNER_ID --depositYocto 1 --gas $GAS
near call $CONTRACT_ID get_user_deposit_by_token '{"account_id":"'$OWNER_ID'", "token_id": "'$LNC'"}' --accountId $CONTRACT_ID
near call $CONTRACT_ID get_token_metadata '{"token_id": "'$LNC'"}' --accountId $CONTRACT_ID 

#near call $CONTRACT_ID withdraw_all '{"account_id":"'$OWNER_ID'", "token_id": "'$LNC'"}' --accountId $CONTRACT_ID --depositYocto 1 --gas $GAS
```



```shell
near call $LNC ft_balance_of '{"account_id":"'$OWNER_ID'"}' --accountId=$OWNER_ID
```