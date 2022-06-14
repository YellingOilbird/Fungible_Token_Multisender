use crate::*;

#[derive(PartialEq)]
pub enum UpdateUserStatsAction {
    AddTokenDeposit,
    RemoveTokenDeposit,
    AddTotalDeposited,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct UserStats {
    account_id: AccountId,
    deposits: LookupMap<TokenContract, Balance>,
    total_deposited: LookupMap<TokenContract, Balance>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub enum UserVStats {
    Current(UserStats),
}

impl From<UserVStats> for UserStats {
    fn from(v_stats: UserVStats) -> Self {
        match v_stats {
            UserVStats::Current(user_stats) => user_stats,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(BorshSerialize, BorshDeserialize)]
pub struct UserStatsTokenOutput {
    account_id: AccountId,
    token_id: TokenContract,
    deposit: U128,
    total_deposited: U128,
}

impl UserStatsTokenOutput {
    fn from_by_token(user_stats: UserStats, token_id: TokenContract) -> UserStatsTokenOutput {
        UserStatsTokenOutput {
            account_id: user_stats.account_id,
            deposit: U128::from(user_stats.deposits.get(&token_id).unwrap_or(0)),
            total_deposited: U128::from(user_stats.total_deposited.get(&token_id).unwrap_or(0)),
            token_id, //if we add NEAR - .unwrap_or_else(|| "NEAR".into()),
        }
    }
}

impl UserStats {
    pub fn new(account_id: AccountId,) -> Self {
        UserStats {
            account_id,
            deposits: LookupMap::new(StorageKey::UserDeposits),
            total_deposited: LookupMap::new(StorageKey::UserTotalDeposits),
        }
    }
}

impl MultisenderFt {

    pub(crate) fn internal_get_stats(&self, account_id: AccountId) -> UserStats {
        if let Some(stats) = self.user_stats.get(&account_id) {
            stats.into()
        } else {
            UserStats::new(account_id)
        }
    }

    pub(crate) fn internal_update_user_stats(
        &mut self,
        account_id: AccountId,
        token_id: TokenContract,
        update_action: UpdateUserStatsAction,
        balance: Option<Balance>
    ) {
        let mut user_stats = self.internal_get_stats(account_id.clone());

        if update_action == UpdateUserStatsAction::AddTotalDeposited {

            if let Some(balance_unwrapped) = balance {
                let total_deposited = user_stats.total_deposited
                    .get(&token_id)
                    .unwrap_or(0);
                user_stats.total_deposited.insert(&token_id, &(total_deposited + balance_unwrapped));
            }

        } else if update_action == UpdateUserStatsAction::AddTokenDeposit {
            if let Some(balance_unwrapped) = balance {
                let deposit = user_stats.deposits
                    .get(&token_id)
                    .unwrap_or(0);
                user_stats.deposits.insert(&token_id, &(deposit + balance_unwrapped));
            }
        } else if update_action == UpdateUserStatsAction::RemoveTokenDeposit {
            user_stats.deposits.remove(&account_id);
        }

        self.user_stats.insert(&account_id, &UserVStats::Current(user_stats));
    }
}

#[near_bindgen]
impl MultisenderFt {
    pub fn get_stats(&self, account_id: AccountId, token_id: TokenContract) -> UserStatsTokenOutput {
        let user_stats = self.internal_get_stats(account_id);
        UserStatsTokenOutput::from_by_token(user_stats, token_id)
    }
}