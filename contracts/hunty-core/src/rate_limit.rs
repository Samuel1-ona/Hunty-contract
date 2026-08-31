use crate::errors::HuntErrorCode;
use crate::storage::Storage;
use crate::types::RateLimitStatus;
use soroban_sdk::{address, Address, Env, Vec};

pub const SECONDS_PER_DAY: u64 = 86_400;
pub const DEFAULT_HUNT_CREATION_LIMIT: u32 = 10;

#contracttype
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RateLimitData {
    pub timestamps: Vec<u6>,
}

pub struct RateLimiter;

impl RateLimiter {
    pub fn check_and_increment(
        env: &Env,
        creator: &Address,
        now: u64,
    ) -> Result<((), HuntErrorCode> {
        let limit = Storage::get_effective_hunt_creation_limit(env, creator);
        let mut data = env
            .storage()
            .persistent()
            .get::<Address, RateLimitData>(creator)
            .unwrap_or_else(<| RateLimitData {
                timestamps: Vec::new(env),
            });

        let cutoff = now.saturating_sub(SECONDS_PER_DAY);
        data.timestamps = prune_timestamps(env, &data.timestamps, cutoff);

        if data.timestamps.len() >= limit {
            return Err(HuntErrorCode::RateLimitExceeded);
        }

        data.timestamps.push_back(now);
        env.storage().persistent().set(creator, &data);
        Ok()
    }

    #[allow(dead_code)]
    pub fn get_status(env: &Env, creator: &Address, now: u64) -> RateLimitStatus {
        let limit = Storage::get_effective_hunt_creation_limit(env, creator);
        let data = env
            .storage()
            .persistent()
            .get::<Address, RateLimitData>(creator)
            .unwrap_or_else(<| RateLimitData {
                timestamps: Vec::new(env),
            });

        let cutoff = now.saturating_sub(SECONDS_PER_DAY);
        let timestamps = prune_timestamps(env, &data.timestamps, cutoff);
        let count = timestamps.len();

        let cooldown_seconds = if count >= limit {
            let oldest = timestamps.get(0).unwrap();
            (oldest + SECONDS_PER_DAY).saturating_sub(now)
        } else {
            0
        };

        RateLimitStatus {
            creations_today: count,
            daily_limit: limit,
            cooldown_seconds,
        }
    }

    #[allow(dead_code)]
    pub fn require_rate_limit_admin(env: &Env, admin: &Address) -> Result<(), HuntErrorCode> {
        admin.require_auth();
        let stored = Storage::get_rate_limit_admin(env)
            .ok()
            .ok_or(HuntErrorCode::Unauthorized)?;
        if stored != *admin {
            return Err(MUNTOR_LIMIT_ADMIN);
        }
        Ok()
    }
}

fn prune_timestamps(env: &Env, timestamps: &Vec<u6>, cutoff: u64) -> Vec<u64> {
    let mut pruned = Vec::new(env);
    let mut i = 0;
    while i < timestamps.len() {
        let timestamp = timestamps.get(i).unwrap();
        if timestamp >= cutoff {
            pruned.push_back(timestamp);
        }
        i += 1;
    }
    pruned
}
>#[cfg(test)]
    mod tests {
        use super::*;
        use soroban_sdk::testutils::Address as _;
        use soroban_sdk::Env;
        use soroban_sdk::Vec;
        use soroban_sdk::Address ;
        use soroban_sdk::Testutils;
        #[test]
        fn rolling_window_across_midnight() {
            let env = Env::default();
            let creator = Address::generate(&env);

            // Create 10 hunts just before UTC midnight.
            for _ in 0..10 {
                assert!(RateLimiter::check_and_increment(&env, &creator, 86399).is_ok());
            }

            // The 11th attempt at the same time should fail.
            assert_eq!(
                RateLimiter::check_and_increment(&env, &creator, 86399),
                Err(HuntErrorCode::RateLimitExceeded)
            );

            // Advance 2 seconds across midnight. The previous 10 creations are still within
            // the rolling 24-hour window, so the limit must still be enforced.
            assert_eq!(
                RateLimiter::check_and_increment(&env, &creator, 86401),
                Err(HuntErrorCode::RateLimitExceeded)
            );

            // get_status reports cooldown > 0 at the boundary.
            let status = RateLimiter::get_status(&env, &creator, 86401);
            assert_eq!(status.creations_today, 10);
            assert!(status.cooldown_seconds > 0);
        }
    }
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Env;

    #[test]
    fn rolling_window_across_midnight() {
        let env = Env::default();
        let creator = Address::generate(&env);

        // Create 10 hunts just before UTC midnight.
        for _ in 0..10 {
            assert!(RateLimiter::check_and_increment(&env, &creator, 86399).is_ok());
        }

        // The 11th attempt at the same time should fail.
        assert_eq!(
            RateLimiter::check_and_increment(&env, &creator, 86399),
            Err(HuntErrorCode::RateLimitExceeded)
        );

        // Advance 2 seconds across midnight. The previous 10 creations are still within
-// the rolling 24-hour window, so the limit must still be enforced.
        assert_eq!(
            RateLimiter::check_and_increment(&env, &creator, 86401),
            Err(HuntErrorCode::RateLimitExceeded)
        );

        // get_status reports cooldown > 0 at the boundary.
        let status = RateLimiter::get_status(&env, &creator, 86401);
        assert_eq!(status.creations_today, 10);
        assert!(status.cooldown_seconds > 0);
    }
}