fungible_token_multisender
==================

This [React] app was initialized with [create-near-app]


Quick Start
===========

To run this project locally:

1. Prerequisites: Make sure you've installed [Node.js] ≥ 12
2. Install dependencies: `yarn install`
3. Export TOKEN_CONTRACT
   ```shell 
      export FT_CONTRACT_NAME="lnc.factory.tokenhub.testnet" 
   ```
   modify `src/.env` file with FT_CONTRACT_NAME="lnc.factory.tokenhub.testnet"

4. Go to `src/config.js` and comment all FT_CONTRACT .env declarations
   ```javascript
   //from this

   const FT_CONTRACT = process.env.FT_CONTRACT_NAME || 'lnc.factory.tokenhub.testnet',
   ftContractName: FT_CONTRACT,
   ...
   //to this

   //const FT_CONTRACT = process.env.FT_CONTRACT_NAME || 'lnc.factory.tokenhub.testnet',
   //ftContractName: FT_CONTRACT,
   ```
5. Run `yarn start`
6. Then uncomment lines in config.js and save it.
   Go to `src/config.js` and uncomment all FT_CONTRACT .env declarations
   ```javascript
   //from this

   //const FT_CONTRACT = process.env.FT_CONTRACT_NAME || 'lnc.factory.tokenhub.testnet',
   //ftContractName: FT_CONTRACT,
   ...
   //to this

   const FT_CONTRACT = process.env.FT_CONTRACT_NAME || 'lnc.factory.tokenhub.testnet',
   ftContractName: FT_CONTRACT,
   ```
This actions with comment/uncomment lines needs for easily run server on localhost with `yarn start`.
If we don't comment this lines it may be conflict between neardev-env and .env with FT_CONTRACT.
#### Always save config.js after uncomment, FT_CONTRACT methods are very important to multisender!


Troubleshooting
===============

On Windows, if you're seeing an error containing `EPERM` it may be related to spaces in your path. Please see [this issue](https://github.com/zkat/npx/issues/209) for more details.


  [React]: https://reactjs.org/
  [create-near-app]: https://github.com/near/create-near-app
  [Node.js]: https://nodejs.org/en/download/package-manager/
  [jest]: https://jestjs.io/
  [NEAR accounts]: https://docs.near.org/docs/concepts/account
  [NEAR Wallet]: https://wallet.testnet.near.org/
  [near-cli]: https://github.com/near/near-cli
  [gh-pages]: https://github.com/tschaub/gh-pages
