use near_sdk::require;

use crate::*;

#[near_bindgen]
impl MultisenderFt {
    pub (crate) fn assert_owner(&self) {
        require!(self.owner_id == env::predecessor_account_id(), "ERR_NOT_ALLOWED")
    }
    #[private]
    pub fn transfer_ownership(&mut self, new_owner: AccountId) -> bool {
        self.assert_owner();
        log!(
            "@{} transfers ownership to @{}",
            self.owner_id, new_owner
        );
        self.owner_id = new_owner;
        true
    }
    #[private]
    pub fn whitelist_token(&mut self, token_id: AccountId) {
        assert_self();
        if self.tokens.contains(&token_id) {
            panic!("{}", ERR_TOKEN_ALREADY_WHITELISTED)
        }

        self.tokens.insert(&token_id);
        log!(
            "{} whitelisted to Multisender App",
            &token_id
        );
        // register multisender into this token contract
        ext_ft::ext(token_id.clone())
            .with_attached_deposit(STORAGE_DEPOSIT)
            .with_static_gas(CALLBACK_GAS)
            .storage_deposit(env::predecessor_account_id()
        );
    }
}