use crate::*;
pub const STORAGE_PRICE_PER_BYTE: Balance = 10_000_000_000_000_000_000;
pub const STORAGE_DEPOSIT:Balance = 125 * STORAGE_PRICE_PER_BYTE;
//gas constants
pub const ONE_YOCTO: Balance = 1;
pub const NO_DEPOSIT: Balance = 0;
pub const CALLBACK_GAS: Gas = Gas(5 * Gas::ONE_TERA.0);
pub const GAS_FOR_FT_TRANSFER: Gas = Gas(30 * Gas::ONE_TERA.0);

#[allow(dead_code)]
//Multisender errors
pub const ERR_UNKNOWN_USER: &str = "User don't have any deposited tokens on Multisender balance";
pub const ERR_TOKEN_NOT_WHITELISTED: &str = "Cannot find this token in whitelisted. You must whitelist this one before deposit";
pub const ERR_TOKEN_ALREADY_WHITELISTED: &str = "Token is already whitelisted!";
pub const ERR_NOTHING_TO_WITHDRAW: &str = "No tokens on multisender balance to withdraw!. Check your balances";
pub const ERR_TOO_MANY_ACCOUNTS: &str = "Multisender functions have a limit of attached gas! This function call have amount of accounts limit!";
pub const ERR_SMALL_DEPOSIT: &str = "You need attach more tokens to this function call!";
//Callback errors
pub const ERR_FAILED_DEPOSIT_TRANSFER: &str = "Something wrong with transfer call from you to token contract. Check your balances";
pub const ERR_FAILED_PROMISE: &str = "Promise failed! Expected single result of callback!";

pub (crate) type TokenContract = AccountId;

pub fn assert_self() {
    assert_eq!(env::predecessor_account_id(), env::current_account_id());
}

pub fn is_promise_success() -> bool {
    assert_eq!(
        env::promise_results_count(),
        1,
        "{}", ERR_FAILED_PROMISE
    );
    match env::promise_result(0) {
        PromiseResult::Successful(_) => true,
        _ => false,
    }
}
#[allow(dead_code)]
pub fn yocto_ft(yocto_amount: Balance, decimals: u8) -> Balance {
    yocto_amount / 10u128.pow(decimals.into())
}
#[allow(dead_code)]
pub fn account_from_str(str: &str) -> AccountId {
    AccountId::try_from(str.to_string()).unwrap()
}