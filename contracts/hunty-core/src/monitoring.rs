use soroban_sdk::{contracttype, symbol_short, Env, Map, String, Vec};

const INVOCATIONS_KEY: soroban_sdk::Symbol = symbol_short!("INVCT");
const FAILURES_KEY: soroban_sdk::Symbol = symbol_short!("FAILCT");
const GAS_UNITS_KEY: soroban_sdk::Symbol = symbol_short!("GASUN");
const ALERTS_KEY: soroban_sdk::Symbol = symbol_short!("ALERT");
/// Per-kind alert counters: `Map<alert_type, HealthAlert>`.
const ALERT_MAP_KEY: soroban_sdk::Symbol = symbol_short!("ALERTMAP");

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
    pub alert_type: String,
    pub count: u32,
    pub last_ledger: u64,
}

pub struct Monitoring;

impl Monitoring {
    /// Telemetry-only invocation record. Emits an event instead of touching
    /// instance storage, so hot entrypoints (e.g. `create_hunt`) do not pay for
    /// a full instance read-modify-write — and do not grow the instance entry
    /// that the contract pays rent on for its whole life. Indexers aggregate
    /// these off-chain and get strictly more than three running totals would.
    pub fn record_invocation_event(env: &Env, gas_units: u64, succeeded: bool) {
        env.events().publish(
            (symbol_short!("invoke"), symbol_short!("telemetry")),
            (gas_units, succeeded),
        );

        if !succeeded {
            Self::raise_alert(env, "invocation_failure");
        }
    }

    /// On-chain counter variant. Prefer `record_invocation_event` on hot paths.
    #[allow(dead_code)]
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

    /// Records an alert *per kind* so an operator can tell seven failed
    /// invocations apart from seven suspicious withdrawals, and emits an event
    /// so off-chain monitoring can react without polling.
    pub fn raise_alert(env: &Env, kind: &str) {
        let alert_type = String::from_str(env, kind);
        let ledger = env.ledger().sequence() as u64;

        let mut alerts: Map<String, HealthAlert> = env
            .storage()
            .instance()
            .get(&ALERT_MAP_KEY)
            .unwrap_or_else(|| Map::new(env));

        let count = alerts
            .get(alert_type.clone())
            .map(|a: HealthAlert| a.count)
            .unwrap_or(0)
            .saturating_add(1);

        alerts.set(
            alert_type.clone(),
            HealthAlert {
                alert_type: alert_type.clone(),
                count,
                last_ledger: ledger,
            },
        );
        env.storage().instance().set(&ALERT_MAP_KEY, &alerts);

        // Aggregate counter kept for `ContractHealth::active_alerts`.
        let total: u32 = env.storage().instance().get(&ALERTS_KEY).unwrap_or(0);
        env.storage()
            .instance()
            .set(&ALERTS_KEY, &total.saturating_add(1));

        env.events().publish(
            (symbol_short!("alert"), symbol_short!("raised")),
            (alert_type, count, ledger),
        );
    }

    /// All alert kinds seen so far, with their individual counts.
    pub fn active_alerts(env: &Env) -> Vec<HealthAlert> {
        let alerts: Map<String, HealthAlert> = env
            .storage()
            .instance()
            .get(&ALERT_MAP_KEY)
            .unwrap_or_else(|| Map::new(env));

        let mut out: Vec<HealthAlert> = Vec::new(env);
        for (_, alert) in alerts.iter() {
            out.push_back(alert);
        }
        out
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
