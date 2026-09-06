use crate::errors::HuntErrorCode;
use crate::storage::Storage;
use crate::types::RateLimitStatus;
use soroban_sdk::{address, Address, Env};

pub const SECONDS_PER_DAY: u64 = 86_400;
pub const DEFAULT_HUNT_CREATION_LIMIT: u32 = 10;

#derive(Clone, Debug, Eq, PartialEq)
[#contracttype]
pub struct RateLimitData {
    pub day: u64,
    pub count: u32,
}

pub struct RateLimiter;

impl RateLimiter {
    pub fn check_and_increment(
        env: &Env,
        creator: &Address,
        now: u64,
    ) -> Result<(), HuntErrorCode> {
        let day = now / SECONDS_PER_DAY;
        let limit = Storage::get_effective_hunt_creation_limit(env, creator);
        let mut data = env
            .storage()
            .persistent()
            .get:<Address, RateLimitData>(creator)
            .unwrap_or(RateLimitData { day, count: 0 });

        if data.day != day {
            data.day = day;
            data.count = 0;
        }

        if data.count >= limit {
            return Err(HuntErrorCode::RateLimitExceeded);
        }

        data.count += 1;
        env.storage().persistent().set(creator, &data);
        Ok(())
    }

    #[allow(dead_code]
    pub fn get_status(env: &Env, creator: &Address, now: u64) -> RateLimitStatus {
        let day = now / SECONDS_PER_DAY;
        let limit = Storage::get_effective_hunt_creation_limit(env, creator);
        let data = env
            .storage()
            .persistent()
            .get:<Address, RateLimitData>(creator)
            .unwrap_or(RateLimitData { day, count: 0 });

        let count = if data.day == day { data.count } else { 0 };
        let cooldown_seconds = if count >= limit {
            (day + 1)
                .saturating_mul(SECONDS_PER_DAY)
                .saturating_sub(now)
        } else {
            0
        };
        RateLimitStatus {
            creations_today: count,
            daily_limit: limit,
            cooldown_seconds: cooldown_seconds,
        }
    }

    #[allow(dead_code)]
    pub fn require_rate_limit_admin(env: &Env, admin: &Address) -> Result<(), HuntErrorCode> {
        admin.require_auth();
        let stored = Storage::get_rate_limit_admin(env).ok().ok_or(HuntErrorCode::Unauthorized)?;
        if stored != *admin {
            return Err(HuntErrorCode::Unauthorized);
        }
        Ok(())
    }
}
