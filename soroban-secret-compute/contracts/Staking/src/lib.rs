#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, log, symbol_short, Address, Env, IntoVal, Map, String, Symbol, TryFromVal, Val, Vec};
use soroban_token_sdk::TokenUtils;

const MIN_STAKE_TOKENS: i128 = 1;
const ASSETER_STORAGE_KEY: Symbol = symbol_short!("asserter");
const TOKEN_SYMBOL: Symbol = symbol_short!("SOO");

#[contracttype]
pub enum DataKey {
    Balance(Address),
    Staked(Address),
    TokenAdmin
}

#[contract]
pub struct StakingContract;
#[contract]
pub struct TokenContract;
#[contract]
pub struct BalanceContract;

#[contractimpl]
impl StakingContract { 
    pub fn stake(env: Env, user_addr: Address, staking_contract: Address, amount: i128) {
        // Retrieve the user's balance
        let balance = BalanceContract::read_balance(&env, user_addr.clone());
    
        // Ensure the user has sufficient funds
        if balance < MIN_STAKE_TOKENS {
            panic!("Insufficient funds");
        }
    
        // Transfer tokens from the user to the staking contract
        TokenContract::transfer(env.clone(), user_addr.clone(), staking_contract, amount);
    
        // Update the staked amount in persistent storage
        let key = DataKey::Staked(user_addr.clone());
        let staked_amt: i128 = env.storage().persistent().get(&key).unwrap_or_default();
        env.storage().persistent().set(&key, &(staked_amt + amount));
    
        // Log the staking event
        log!(&env, "User {} has staked {} tokens", user_addr, amount);
    }
    
    pub fn unstake(env: Env, user_addr: Address, staking_contract: Address, amount: i128) {
        // Retrieve the current staked amount
        let key = DataKey::Staked(user_addr.clone());
        let staked_amt: i128 = env.storage().persistent().get(&key).unwrap_or_default();
    
        // Ensure the user has sufficient staked tokens
        if staked_amt < amount {
            panic!("Insufficient staked tokens");
        }
    
        // Deduct the specified amount from the staked tokens
        let new_staked_amt = staked_amt - amount;
        env.storage().persistent().set(&key, &new_staked_amt);
    
        // Transfer the tokens back to the user
        TokenContract::transfer(env.clone(), staking_contract, user_addr.clone(), amount);
    
        log!(&env, "User {} has unstaked {} tokens", user_addr, amount);
    }
}

#[contractimpl] 
impl TokenContract {  
    pub fn get_token_admin(e: &Env) -> Address {
        let key = DataKey::TokenAdmin;
        e.storage().instance().get(&key).unwrap() 
    } 

    pub fn set_token_admin(e: &Env, id: Address) {
        let key = DataKey::TokenAdmin; 
        e.storage().instance().set(&key, &id); 
    }

    pub fn mint(e: Env, to: Address, amount: i128) {
        // TODO: have authorized minters 
        let admin = Self::get_token_admin(&e);
        // require_auth  ensures that the user has allowed authorization for an operation and nobody else.
        admin.require_auth();
        BalanceContract::write_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().mint(admin, to, amount);
    }

    pub fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        BalanceContract::spend_balance(&e, from.clone(), amount); 
        BalanceContract::receive_balance(&e, to.clone(), amount);
        TokenUtils::new(&e).events().transfer(from, to, amount);
    }

}

#[contractimpl]
impl BalanceContract {
    pub fn read_balance(e: &Env, addr: Address) -> i128 {
        let key = DataKey::Balance(addr); 
        if let Some(balance) = e.storage().persistent().get(&key) {
            balance 
        } else {
            0
        }
    }

    pub fn write_balance(e: &Env, addr: Address, amount: i128) {
        let key = DataKey::Balance(addr); 
        e.storage().persistent().set(&key, &amount);
    }

    pub fn spend_balance(e: &Env, addr: Address, amount: i128) {
        let balance = Self::read_balance(e, addr.clone());
        if balance < amount {
            panic!("insufficient balance");
        }

        Self::write_balance(e, addr, balance-amount);
    }

    pub fn receive_balance(e: &Env, addr: Address, amount: i128) {
        let balance = Self::read_balance(e, addr.clone());
        Self::write_balance(e, addr, balance + amount);
    }
}

mod test;