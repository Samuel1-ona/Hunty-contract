use crate::errors::RewardErrorCode;
use soroban_sdk::{token, Address, Env};

pub struct TokenHandler;

impl TokenHandler {
    /// Validates that the given address is a SAC-compatible token contract.
    ///
    /// Verifies that the contract implements the standard token interface by
    /// attempting to query the balance method. This is a lightweight check
    /// that confirms basic SAC compatibility.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `token_address` - Address of the token contract to validate
    ///
    /// # Returns
    /// * `Ok(())` if the token contract is valid
    /// * `Err(RewardErrorCode::InvalidTokenContract)` if validation fails
    pub fn validate_token_contract(
        env: &Env,
        token_address: &Address,
    ) -> Result<(), RewardErrorCode> {
        // Try to create a token client and query balance
        // This verifies the contract implements the SAC token interface
        let client = token::Client::new(env, token_address);
        let contract_addr = env.current_contract_address();

        // Attempt to query balance - this will panic if not a valid token contract
        // We use a simple try-catch pattern by checking if we can instantiate the client
        // and the contract responds to the balance method
        match client.try_balance(&contract_addr) {
            Ok(_) => Ok(()),
            Err(_) => Err(RewardErrorCode::InvalidTokenContract),
        }
    }

    /// Transfers tokens from the contract to a recipient.
    ///
    /// Uses the Soroban token interface (SAC) to execute the transfer.
    /// The contract must have sufficient balance and must have authorized
    /// the transfer (handled automatically when called from within the contract).
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `token_address` - Address of the token contract
    /// * `contract_addr` - Address of this contract (sender)
    /// * `recipient` - Address of the recipient
    /// * `amount` - Amount to transfer
    #[allow(dead_code)]
    pub fn distribute_tokens(
        env: &Env,
        token_address: &Address,
        contract_addr: &Address,
        recipient: &Address,
        amount: i128,
    ) {
        let client = token::Client::new(env, token_address);
        client.transfer(contract_addr, recipient, &amount);
    }

    /// Checks if the contract holds enough tokens for the required amount.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `token_address` - Address of the token contract
    /// * `contract_addr` - Address of this contract
    /// * `required` - Required amount to validate
    ///
    /// # Returns
    /// * `true` if the contract has sufficient balance
    /// * `false` otherwise
    pub fn validate_pool(
        env: &Env,
        token_address: &Address,
        contract_addr: &Address,
        required: i128,
    ) -> bool {
        let balance = Self::get_balance(env, token_address, contract_addr);
        balance >= required
    }

    /// Returns the contract's current token balance.
    ///
    /// # Arguments
    /// * `env` - The contract environment
    /// * `token_address` - Address of the token contract
    /// * `contract_addr` - Address of this contract
    ///
    /// # Returns
    /// The current token balance
    pub fn get_balance(env: &Env, token_address: &Address, contract_addr: &Address) -> i128 {
        let client = token::Client::new(env, token_address);
        client.balance(contract_addr)
    }
}
