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
