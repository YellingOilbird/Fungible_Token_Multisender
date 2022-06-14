use std::convert::TryFrom;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::U128;
use near_sdk::{assert_one_yocto, env, log, ext_contract, near_bindgen, PanicOnDefault, require};
use near_sdk::{AccountId, BorshStorageKey, PromiseResult, Promise, Gas, Balance, PromiseOrValue};
use near_sdk::serde::{Deserialize, Serialize};

use user::UserVStats;

use crate::utils::*;
use crate::ft_standards::*;

mod owner;
mod utils;
mod views;
mod ft_standards;
mod user;

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct MultisenderFt {
    owner_id: AccountId,
    deposits: LookupMap<AccountId, LookupMap<TokenContract, Balance>>,
    tokens: UnorderedSet<AccountId>,
    user_stats: UnorderedMap<AccountId, UserVStats>
}

//StorageKey implementation for Default prefixes in Multisender contract
#[derive(BorshSerialize, BorshStorageKey)]
pub (crate) enum StorageKey {
    WhitelistedTokens,
    UserDeposits,
    UserTotalDeposits,
    UserStats
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
    pub fn get_user_deposits(&self, account_id: &AccountId) -> LookupMap<TokenContract, Balance> {
        match self.deposits.get(account_id) {
            Some(deposits) => {
                deposits
            },
            _ => LookupMap::new(b"d".to_vec()),
        }    
    }
    //deposit to multisender
    pub fn deposit(
        &mut self, 
        account_id: AccountId, 
        token_id: TokenContract,
        deposit_amount: U128
    ) {
        let token_whitelisted = self.is_token_whitelisted(&token_id);
        if token_whitelisted {
            let attached_tokens: u128 = deposit_amount.0;
            let previous_balance: u128 = self.get_user_deposit_by_token(&account_id, &token_id).0;
            match self.deposits.get(&account_id) {
                Some(mut token_deposits) => {
                    token_deposits.insert(&token_id, &(attached_tokens + previous_balance));
                    self.deposits.insert(&account_id, &token_deposits);
                },
                None => {
                    let mut token_deposits = LookupMap::new(StorageKey::UserDeposits);
                    token_deposits.insert(&token_id, &attached_tokens);
                    self.deposits.insert(&account_id, &token_deposits);
                }
            }
            log!(
                "success deposited {:?} of {} from @{}",
                deposit_amount,
                token_id,
                account_id
            );
        } else {
            panic!("{}",ERR_TOKEN_NOT_WHITELISTED)
        }
    }

    /*
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
            self.deposits.insert(&account_id,&(previous_balance + amount_sent.0));
        }
    }
    */    
}

#[near_bindgen]
impl MultisenderFt {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        require!(!env::state_exists(), "Already initialized");
        MultisenderFt { 
            owner_id, 
            deposits: LookupMap::new(StorageKey::UserDeposits),
            tokens: UnorderedSet::new(StorageKey::WhitelistedTokens),
            user_stats: UnorderedMap::new(StorageKey::UserStats)
        }
    }
    //register multiple accounts to TOKEN_CONTRACT. Because of gas limit it may be only less then 50 accounts
    #[payable]
    pub fn multi_storage_deposit(&mut self, token_id: TokenContract, accounts: Vec<AccountId>){

        let total_accounts = accounts.len();
        //deposit requested for storage_deposit for all accounts into FT contract
        let total_storage_bond: u128 = STORAGE_DEPOSIT * total_accounts as u128;

        assert!(total_accounts <= 50, "{}", ERR_TOO_MANY_ACCOUNTS);
        assert!(
            env::attached_deposit() >= total_storage_bond,
            "{}: YOU NEED {} yN MORE FOR THIS FUNCTION_CALL", ERR_SMALL_DEPOSIT, (total_storage_bond - env::attached_deposit())
        );

        let token_whitelisted = self.is_token_whitelisted(&token_id);
        if token_whitelisted {

            for account in accounts {

                ext_ft::ext(token_id.clone())
                    .with_attached_deposit(STORAGE_DEPOSIT)
                    .with_static_gas(CALLBACK_GAS)
                    .storage_deposit(account.clone());
    
                log!("Register storage for account @{}", account);
            }
        } else {
            panic!("{}", ERR_TOKEN_NOT_WHITELISTED)
        }
    }

    //withdraw all from multisender
    #[payable]
    pub fn withdraw_all(&mut self, account_id: AccountId, token_id: TokenContract) {

        assert_one_yocto();

        assert!(self.deposits.contains_key(&account_id), "{}", ERR_UNKNOWN_USER);
        let mut deposit: u128 = self.get_user_deposit_by_token(&account_id, &token_id).0;
        assert!(
            deposit > NO_DEPOSIT,
            "{}", ERR_NOTHING_TO_WITHDRAW
        );
        //TODO fix this with least 1 token assert
        deposit-=100000000000000000;
        // 100000000000000000

        ext_ft::ext(token_id.clone())
            .with_attached_deposit(ONE_YOCTO)
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .ft_transfer(
                account_id.to_string(), 
                deposit.to_string()
            ).then(
                ext_self::ext(env::current_account_id())
                    .with_attached_deposit(NO_DEPOSIT)
                    .with_static_gas(CALLBACK_GAS)
                    .on_withdraw(
                        account_id.clone(), 
                        token_id, 
                        deposit.into(), 
                        account_id
                    )
            );
    }

    /*
    #[payable]
    pub fn multisend_from_balance(&mut self, accounts: Vec<Operation>) {
        assert_one_yocto();

        let account_id = env::predecessor_account_id();

        assert!(self.deposits.contains_key(&account_id), "Unknown user");

        let mut tokens: Balance = self.deposits.get(&account_id).unwrap();
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
            self.deposits.insert(&account_id, &tokens);

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

        let tokens: Balance = self.deposits.get(&account_id).unwrap();
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

        self.deposits.insert(&account_id,&(tokens - total));

        log!("Chunk Done!"); 
    }
    */
}
