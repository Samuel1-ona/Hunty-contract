use soroban_sdk::{contracttype, Env};

// ==========================================
// --- 1. CONSTANTS & METRICS CONFIG ---
// ==========================================

// Approximate resource/gas cost bases for Stellar/Soroban network operations
const XLM_TRANSFER_GAS_BASE: u64 = 15_000;
const NFT_MINT_GAS_BASE: u64 = 45_000;
const CONSTANT_OVERHEAD_GAS: u64 = 5_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GasEstimationReport {
    pub xlm_transfer_estimated_gas: u64,
    pub nft_mint_estimated_gas: u64,
    pub total_estimated_gas: u64,
    pub safe_budget_total: u64, // Total + buffer to prevent out-of-gas errors
}

pub struct RewardGasEstimator;

impl RewardGasEstimator {
    /// Dry-run simulation function that estimates gas costs for distributing rewards.
    /// Helps creators budget for distribution costs before initiating batch execution.
    pub fn estimate_distribution_cost(
        _env: &Env,
        recipient_count: u32,
    ) -> GasEstimationReport {
        if recipient_count == 0 {
            return GasEstimationReport {
                xlm_transfer_estimated_gas: 0,
                nft_mint_estimated_gas: 0,
                total_estimated_gas: 0,
                safe_budget_total: 0,
            };
        }

        // Calculate linear scale costs based on recipient volume
        let xlm_transfer_estimated_gas = (recipient_count as u64) * XLM_TRANSFER_GAS_BASE;
        let nft_mint_estimated_gas = (recipient_count as u64) * NFT_MINT_GAS_BASE;
        
        let total_estimated_gas = xlm_transfer_estimated_gas + nft_mint_estimated_gas + CONSTANT_OVERHEAD_GAS;
        
        // Add a 10% safety buffer to shield creators against unexpected network congestion spikes
        let safe_budget_total = (total_estimated_gas * 110) / 100;

        GasEstimationReport {
            xlm_transfer_estimated_gas,
            nft_mint_estimated_gas,
            total_estimated_gas,
            safe_budget_total,
        }
    }
}

// ==========================================
// --- 2. AUTOMATED UNIT TESTS ---
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_estimate_distribution_cost_for_single_recipient() {
        let env = Env::default();
        let report = RewardGasEstimator::estimate_distribution_cost(&env, 1);

        assert_eq!(report.xlm_transfer_estimated_gas, 15_000);
        assert_eq!(report.nft_mint_estimated_gas, 45_000);
        assert_eq!(report.total_estimated_gas, 65_000); // 15k + 45k + 5k
        assert_eq!(report.safe_budget_total, 71_500); // 65k * 1.10
    }

    #[test]
    fn test_estimate_distribution_cost_for_batch_recipients() {
        let env = Env::default();
        let report = RewardGasEstimator::estimate_distribution_cost(&env, 10);

        assert_eq!(report.xlm_transfer_estimated_gas, 150_000);
        assert_eq!(report.nft_mint_estimated_gas, 450_000);
        assert_eq!(report.total_estimated_gas, 605_000); // 150k + 450k + 5k overhead
    }

    #[test]
    fn test_estimate_distribution_cost_zero_recipients() {
        let env = Env::default();
        let report = RewardGasEstimator::estimate_distribution_cost(&env, 0);

        assert_eq!(report.total_estimated_gas, 0);
        assert_eq!(report.safe_budget_total, 0);
    }
}