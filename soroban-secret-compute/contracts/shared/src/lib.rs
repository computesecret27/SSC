#![no_std]

use soroban_sdk::{contracttype, String};

#[contracttype]
#[derive(Debug, Clone, PartialEq)]
pub enum AttestationStatus {
    PROPOSED,
    SUCCESS, 
    FAILED,
}

