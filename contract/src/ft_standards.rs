pub(crate) use near_contract_standards::fungible_token::{
    metadata::FungibleTokenMetadata,
    receiver::FungibleTokenReceiver
};

use crate::*;

// define the methods we'll use on the token contracts
#[ext_contract(ext_ft)]
pub trait FungibleToken {
    fn ft_metadata(&self) -> Promise;
    fn storage_deposit(&self, account_id: AccountId);
    fn ft_transfer(&mut self, receiver_id: String, amount: String) -> Promise;
    fn ft_transfer_call(&mut self, receiver_id: String, amount: String, msg:String) -> Promise;
}

// define methods we'll use as callbacks on Multisender contract
#[ext_contract(ext_self)]
pub trait ExtMultisender {
    fn on_ft_metadata(&mut self) -> FungibleTokenMetadata;
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String,) -> PromiseOrValue<U128>;
    fn on_transfer_from_balance(&mut self, account_id: AccountId, amount: Balance, recipient: AccountId);
    fn on_withdraw(&mut self, account_id: AccountId, token_id:TokenContract, amount_sent: U128, recipient: AccountId);
}

#[near_bindgen]
impl FungibleTokenReceiver for MultisenderFt {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {

        // token contract which calls this function
        let token_id = env::predecessor_account_id();
        // Support only whitelisted tokens by owner
        let token_whitelisted = self.is_token_whitelisted(&token_id);

        if token_whitelisted {
            log!(
                "in {} tokens from @{} ft_on_transfer, msg = {}", 
                amount.0, sender_id.as_ref(), 
                msg
            );

            if msg.is_empty() {
                self.deposit(sender_id.clone(), token_id.clone(), amount);
                log!(
                    "Deposited {} {} from @{} to Multisender", 
                    amount.0, token_id, sender_id.as_ref()
                );
                PromiseOrValue::Value(0.into())
            } else {
                log!("{}", ERR_FAILED_DEPOSIT_TRANSFER);
                PromiseOrValue::Value(amount)
            }
        } else {
            panic!("{}", ERR_TOKEN_NOT_WHITELISTED)
        }
    }
}

#[near_bindgen]
impl MultisenderFt {
    pub fn on_ft_metadata(&mut self) -> FungibleTokenMetadata {
        assert_self();
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}", ERR_FAILED_PROMISE
        );
        match env::promise_result(0) {
            PromiseResult::Successful(result) => {
                let ft_metadata = near_sdk::serde_json::from_slice::<FungibleTokenMetadata>(&result).unwrap();
                ft_metadata
            }
            _ => panic!("{}", ERR_FAILED_PROMISE),
        }
    }
    
    pub fn on_withdraw(
        &mut self, 
        account_id: AccountId,
        token_id: TokenContract, 
        amount_sent: U128, 
        recipient: AccountId
    ) {
        assert_self();
        let transfer_succeeded = is_promise_success();
        if !transfer_succeeded {
            log!(
                "Withdraw transaction with token {} to @{} failed. {} kept on the app deposit",
                token_id, 
                recipient, 
                amount_sent.0,
            );
            let previous_balance: u128 = self.get_user_deposit_by_token(&account_id, &token_id).0;

            let mut deposits = self.get_user_deposits(&account_id);
            deposits.insert(&account_id, &(previous_balance + amount_sent.0));
    
            self.deposits.insert(&account_id, &deposits);

        } else {
            log!(
                "Success withdraw transaction to @{} with amount {} of token {} ", 
                recipient, 
                amount_sent.0,
                token_id
            );
            let mut deposits = self.get_user_deposits(&account_id);
            deposits.insert(&account_id, &NO_DEPOSIT);
            
            self.deposits.insert(&account_id, &deposits);
        }
    }
}
