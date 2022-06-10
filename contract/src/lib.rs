use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{assert_one_yocto, env, log, ext_contract, near_bindgen, AccountId, PromiseResult, Gas, Balance, PromiseOrValue};
use near_sdk::serde::{Deserialize, Serialize};

/// Token contract for multisend. Cross-calls allows only for this contract
const TOKEN_CONTRACT:&str = "lnc.factory.tokenhub.testnet";
/// Token metadata decimals for human-readable convert balances
const TOKEN_DECIMALS:u8 = 18;
const TOKEN_TICKER:&str = "LNC";

/// Gas constants
pub const CALLBACK_GAS: Gas = Gas(5_000_000_000_000);
pub const GAS_FOR_FT_TRANSFER: Gas = Gas(10_000_000_000_000);
pub const NO_DEPOSIT: u128 = 0;
pub const STORAGE_PRICE_PER_BYTE: u128 = 10_000_000_000_000_000_000;

/// Define the methods we'll use on the other contract
#[ext_contract(ext_ft)]
pub trait FungibleToken {
    fn storage_deposit(&self, account_id: AccountId);
    fn ft_transfer(&mut self, receiver_id: String, amount: String);
    fn ft_transfer_call(&mut self, receiver_id: String, amount: String, msg: String);
}

/// Define methods we'll use as callbacks on our contract
#[ext_contract(ext_self)]
pub trait MyContract {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String,) -> PromiseOrValue<U128>;
    fn on_transfer_from_balance(&mut self, account_id: AccountId, amount_sent: U128, recipient: AccountId);
}

/*
You can use LookupMap from near_sdk::collections, 
but in that case you need to implement Default trait with key prefixes
 */
#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct MultisenderFt {
    deposits: HashMap<AccountId, u128>,
}

/// (account, amount) chunk from front-end input text field for multisend
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Operation {
    account_id: AccountId,
    amount: U128,
}

/// Internal section
impl MultisenderFt {
    //deposit to multisender
    pub fn deposit(&mut self, account_id: AccountId, deposit_amount: U128) -> U128 {
        let attached_tokens: u128 = deposit_amount.0; 
        let previous_balance: u128 = self.get_deposit(account_id.clone()).into();

        // update info about user deposit in MULTISENDER
        self.deposits.insert(account_id.clone(), previous_balance + attached_tokens);
        self.get_deposit(account_id)
    }

    //multisender transfer callback
    pub fn on_transfer_from_balance(&mut self, account_id: AccountId, amount_sent: U128, recipient: AccountId) {
        assert_self();
        let transfer_succeeded = is_promise_success();
        if !transfer_succeeded {
            log!(
                "Transaction to @{} failed. {} y{} (~{} {}) kept on the app deposit", 
                recipient, 
                amount_sent.0,
                TOKEN_TICKER.to_string(), 
                yocto_ft(amount_sent.0),
                TOKEN_TICKER.to_string(),
            );
            let previous_balance: u128 = self.get_deposit(account_id.clone()).into();
            self.deposits.insert(account_id, previous_balance + amount_sent.0);
        }
    }    
}

#[near_bindgen]
impl MultisenderFt {
    //register multiple accounts to TOKEN_CONTRACT. Because of gas limit it may be only less then 50 accounts
    #[payable]
    pub fn multi_storage_deposit(&mut self, accounts: Vec<AccountId>){

        let total_accounts = accounts.len();
        assert!(total_accounts <= 50, "ERR_TOO_MANY_ACCOUNTS!");

        //deposit requested for storage_deposit for 1 account into FT contract
        let storage_bond: u128 = 125 * STORAGE_PRICE_PER_BYTE;

        //deposit requested for storage_deposit for all accounts into FT contract
        let total_storage_bond: u128 = storage_bond * total_accounts as u128;

        assert!(
            env::attached_deposit() >= total_storage_bond,
            "ERR_SMALL_DEPOSIT: YOU NEED {} yN MORE FOR THIS FUNCTION_CALL", (total_storage_bond - env::attached_deposit())
        );

        for account in accounts {

            ext_ft::storage_deposit(
                account.clone(),
                account_from_str(TOKEN_CONTRACT),
                storage_bond,
                CALLBACK_GAS
            );

            log!("Register storage for account @{}", account);
        }
    }

    //withdraw all from multisender
    #[payable]
    pub fn withdraw_all(&mut self, account_id: AccountId) {

        assert_one_yocto();

        assert!(self.deposits.contains_key(&account_id), "ERR_UNKNOWN_USER");
        let deposit: u128 = self.get_deposit(account_id.clone()).into();
        assert!(
            deposit > NO_DEPOSIT,
            "ERR_NOTHING_TO_WITHDRAW"
        );

        ext_ft::ft_transfer(
            account_id.to_string(),
            deposit.to_string(),
            account_from_str(TOKEN_CONTRACT),
            1u128.into(),
            CALLBACK_GAS
        );
        
        self.deposits.insert(account_id, NO_DEPOSIT);
    }
    
    pub fn get_deposit(&self, account_id: AccountId) -> U128 {
        match self.deposits.get(&account_id) {
            Some(deposit) => {
                U128::from(*deposit)
            }
            None => {
                0.into()
            }
        }
    }

    //multisender transfer from deposit
    #[payable]
    pub fn multisend_from_balance(&mut self, accounts: Vec<Operation>) {
        assert_one_yocto();

        let account_id = env::predecessor_account_id();

        assert!(self.deposits.contains_key(&account_id), "Unknown user");

        let mut tokens: Balance = *self.deposits.get(&account_id).unwrap();
        let mut total: Balance = 0;

        for account in &accounts {
            let amount: Balance = account.amount.into();
            total += amount;
        }

        assert!(
            total <= tokens,
            "Not enough deposited tokens to run multisender (Supplied: {}. Demand: {})",
            tokens,
            total
        );

        for account in accounts {

            let amount_u128: u128 = account.amount.into();

            log!(
                "Sending {} y{} (~{} {}) to account @{}", 
                amount_u128,
                TOKEN_TICKER.to_string(), 
                yocto_ft(amount_u128),
                TOKEN_TICKER.to_string(), 
                account.account_id
            );

            tokens -= amount_u128;
            self.deposits.insert(account_id.clone(), tokens);

            //transfer
            ext_ft::ft_transfer(
                account.account_id.clone().to_string(),
                amount_u128.to_string(),
                account_from_str(TOKEN_CONTRACT),
                1u128.into(),
                CALLBACK_GAS
            )
            .then(
                ext_self::on_transfer_from_balance(
                    account.account_id.clone(),
                    account.amount,
                    account.account_id,
                    env::current_account_id(),
                    NO_DEPOSIT,
                    CALLBACK_GAS
                )
            );
            
        }
    }
    // Multisend from balance without callbacks - better gas efficient, but not usable for 2FA accs.
    // Allows 30 operations per transaction. But ChunkSize = 25 is reccomended (setting in App.js button event)
    #[payable]
    pub fn multisend_from_balance_unsafe(&mut self, accounts: Vec<Operation>) {
        assert_one_yocto();

        let account_id = env::predecessor_account_id();

        assert!(self.deposits.contains_key(&account_id), "Unknown user");

        let tokens: Balance = *self.deposits.get(&account_id).unwrap();
        let mut total: Balance = 0;

        for account in &accounts {
            let amount: Balance = account.amount.into();
            total += amount;
        }

        assert!(
            total <= tokens,
            "Not enough deposited tokens to run multisender (Supplied: {}. Demand: {})",
            tokens,
            total
        );

        for account in accounts {

            let amount_u128: u128 = account.amount.into();

            ext_ft::ft_transfer(
                account.account_id.clone().to_string(),
                amount_u128.to_string(),
                account_from_str(TOKEN_CONTRACT),
                1u128.into(),
                CALLBACK_GAS
            );
            log!(
                "Sending unsafe {} y{} to account @{}",
                amount_u128,
                TOKEN_TICKER.to_string(), 
                account.account_id
            );
        }

        self.deposits.insert(account_id, tokens - total);

        log!("Chunk Done!"); 
    }
    // Function which calls when someone transfer tokens to multisender account. Return transfer if msg is not empty
    pub fn ft_on_transfer(
        &mut self,
        //token_id: AccountId,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {

        //token contract which calls this function
        let token_id = env::predecessor_account_id();
        assert_eq!(
            token_id.clone(), 
            account_from_str(TOKEN_CONTRACT),
            "ERR_NOT_ALLOWED"
        );
        let sender: AccountId = sender_id.into();

        if msg.is_empty() {
            self.deposit(sender, amount);
            PromiseOrValue::Value(U128(0))
        } else {
            log!("ERR_WRONG_MSG");
            PromiseOrValue::Value(amount)
        }

    }

}

pub fn assert_self() {
    assert_eq!(env::predecessor_account_id(), env::current_account_id());
}

fn is_promise_success() -> bool {
    assert_eq!(
        env::promise_results_count(),
        1,
        "Contract expected a result on the callback"
    );
    match env::promise_result(0) {
        PromiseResult::Successful(_) => true,
        _ => false,
    }
}

pub fn yocto_ft(yocto_amount: Balance) -> Balance {
    yocto_amount / 10u128.pow(TOKEN_DECIMALS.into())
}

pub fn account_from_str(str: &str) -> AccountId {
    AccountId::try_from(str.to_string()).unwrap()
}
