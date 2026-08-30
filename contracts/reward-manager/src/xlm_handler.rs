use soroban_sdk::{token, Address, Env};

pub struct XlmHandler;

impl XlmHandler {
    /// Transfers XLM from the contract to a player.
    ///
    /// Uses the Soroban token interface (SAC) to execute the transfer.
    /// The contract must have sufficient balance and must have authorized
    /// the transfer (handled automatically when called from within the contract).
    pub fn distribute_xlm(
        env: &Env,
        xlm_token: &Address,
        contract_addr: &Address,
        player: &Address,
        amount: i128,
    ) {
        let client = token::Client::new(env, xlm_token);
        client.transfer(contract_addr, player, &amount);
    }

    /// Checks if the contract holds enough XLM for the required amount.
    pub fn validate_pool(
        env: &Env,
        xlm_token: &Address,
        contract_addr: &Address,
        required: i128,
    ) -> bool {
        let balance = Self::get_balance(env, xlm_token, contract_addr);
        balance >= required
    }

    /// Returns the contract's current XLM balance.
    pub fn get_balance(env: &Env, xlm_token: &Address, contract_addr: &Address) -> i128 {
        let client = token::Client::new(env, xlm_token);
        client.balance(contract_addr)
    }
}
