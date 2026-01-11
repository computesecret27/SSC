#![cfg(test)]
use crate::{StakingContract, StakingContractClient};
use soroban_sdk:: {log, symbol_short, testutils::Events, Env, String}; 
extern crate std;

#[test]
fn test() {
    let env = Env::default(); 
    let contract_id = env.register(StakingContract, ());
    let client = StakingContractClient::new(&env, &contract_id);
   
}