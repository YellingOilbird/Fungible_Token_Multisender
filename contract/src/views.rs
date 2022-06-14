use crate::*;
/*
#[derive(BorshSerialize, BorshDeserialize)]
pub struct MultisenderView {
    pub account_id: AccountId,
    pub total_sended: Vec<(TokenContract, Balance)>,
    pub app_balances: Vec<(TokenContract, Balance)>,
}

impl From<MultisenderFt> for MultisenderView {
    fn from(_: MultisenderFt) -> Self {

        Self { 
            account_id: (), 
            total_sended: (), 
            app_balances: () 
        }
    }
}
*/

#[near_bindgen]
impl MultisenderFt {
    pub fn get_owner(&self) -> AccountId {
        self.owner_id.clone()
    }
    //TODO - fix views
    pub fn get_whitelisted_tokens(&self) -> Vec<AccountId> {
        let mut deser_vec:Vec<AccountId> = vec![];
        for account in self.tokens.iter() {
            deser_vec.push(account);
        }
        deser_vec
    }
    pub fn is_token_whitelisted(&self, token_id: &AccountId) -> bool {
        self.tokens.contains(&token_id)
    }

    pub fn get_user_deposit_by_token(&self, account_id: &AccountId, token_id: &TokenContract) -> U128 {
        let deposits = self.get_user_deposits(account_id);
        match deposits.get(token_id) {
            Some(deposit) => {
                log!("deposit:{} ",deposit);
                U128::from(deposit)
            }
            None => {
                0.into()
            }
        }
    }

    pub fn get_token_metadata(&self, token_id: AccountId) -> PromiseOrValue<FungibleTokenMetadata> {
        let token_whitelisted = self.is_token_whitelisted(&token_id);
        if token_whitelisted {
            ext_ft::ext(token_id.clone())
                .with_attached_deposit(NO_DEPOSIT)
                .with_static_gas(CALLBACK_GAS)
                .ft_metadata(
                ).then(
                   ext_self::ext(env::current_account_id())
                       .with_attached_deposit(NO_DEPOSIT)
                       .with_static_gas(CALLBACK_GAS)
                       .on_ft_metadata()
                ).into()
        } else {
            panic!("{}",ERR_TOKEN_NOT_WHITELISTED)
        }
    }
}