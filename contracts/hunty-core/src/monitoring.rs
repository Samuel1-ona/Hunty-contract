use soroban_sdk::{contracttype, symbol_short, Env};

const INVOCATIONS_KEY: soroban_sdk::Symbol = symbol_short!("INVCT");
const FAILURES_KEY: soroban_sdk::Symbol = symbol_short!("FAILCT");
const GAS_UNITS_KEY: soroban_sdk::Symbol = symbol_short!("GASUN");
const ALERTS_KEY: soroban_sdk::Symbol = symbol_short!("ALERT");

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContractHealth {
    pub total_invocations: u64,
    pub failed_invocations: u64,
    pub failure_rate_bps: u32,
    pub avg_gas_units: u64,
    pub active_alerts: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HealthAlert {
    pub alert_type: soroban_sdk::String,
    pub count: u32,
    pub last_ledger: u64,
}

pub struct Monitoring;

impl Monitoring {
    pub fn record_invocation(env: &Env, gas_units: u64, succeeded: bool) {
        let total: u64 = env.storage().instance().get(&INVOCATIONS_KEY).unwrap_or(0);
        env.storage().instance().set(&INVOCATIONS_KEY, &(total + 1));

        let gas_total: u64 = env.storage().instance().get(&GAS_UNITS_KEY).unwrap_or(0);
        env.storage()
            .instance()
            .set(&GAS_UNITS_KEY, &(gas_total + gas_units));

        if !succeeded {
            let failures: u64 = env.storage().instance().get(&FAILURES_KEY).unwrap_or(0);
            env.storage().instance().set(&FAILURES_KEY, &(failures + 1));
            Self::raise_alert(env, "invocation_failure");
        }
    }

    #[allow(dead_code)]
    pub fn record_large_withdrawal(env: &Env, amount: i128) {
        if amount > 1_000_000_000 {
            Self::raise_alert(env, "large_withdrawal");
        }
    }

    fn raise_alert(env: &Env, _kind: &str) {
        let alerts: u32 = env.storage().instance().get(&ALERTS_KEY).unwrap_or(0);
        env.storage().instance().set(&ALERTS_KEY, &(alerts + 1));
    }

    pub fn health_dashboard(env: &Env) -> ContractHealth {
        let total: u64 = env.storage().instance().get(&INVOCATIONS_KEY).unwrap_or(0);
        let failures: u64 = env.storage().instance().get(&FAILURES_KEY).unwrap_or(0);
        let gas_total: u64 = env.storage().instance().get(&GAS_UNITS_KEY).unwrap_or(0);
        let alerts: u32 = env.storage().instance().get(&ALERTS_KEY).unwrap_or(0);

        let failure_rate_bps = if let Some(rate) = failures
            .checked_mul(10_000)
            .and_then(|n| n.checked_div(total))
        {
            rate as u32
        } else {
            0
        };
        let avg_gas_units = gas_total.checked_div(total).unwrap_or(0);

        ContractHealth {
            total_invocations: total,
            failed_invocations: failures,
            failure_rate_bps,
            avg_gas_units,
            active_alerts: alerts,
        }
    }
}
