use crate::errors::HuntErrorCode;
use crate::storage::Storage;
use crate::types::RateLimitStatus;
use soroban_sdk::{Address, Env};

pub const SECONDS_PER_DAY: u64 = 86_400;
pub const DEFAULT_HUNT_CREATION_LIMIT: u32 = 10;

pub struct RateLimiter;

impl RateLimiter {
    pub fn check_and_increment(
        env: &Env,
        creator: &Address,
        now: u64,
    ) -> Result<(), HuntErrorCode> {
        let day = now / SECONDS_PER_DAY;
        let count = Storage::get_creator_daily_hunt_count(env, creator, day);
        let limit = Storage::get_effective_hunt_creation_limit(env, creator);
        if count >= limit {
            return Err(HuntErrorCode::RateLimitExceeded);
        }
        Storage::set_creator_daily_hunt_count(env, creator, day, count + 1);
        Ok(())
    }

    pub fn get_status(env: &Env, creator: &Address, now: u64) -> RateLimitStatus {
        let day = now / SECONDS_PER_DAY;
        let count = Storage::get_creator_daily_hunt_count(env, creator, day);
        let limit = Storage::get_effective_hunt_creation_limit(env, creator);
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
            cooldown_seconds,
        }
    }

    pub fn require_rate_limit_admin(env: &Env, admin: &Address) -> Result<(), HuntErrorCode> {
        admin.require_auth();
        let stored = Storage::get_rate_limit_admin(env).ok_or(HuntErrorCode::Unauthorized)?;
        if stored != *admin {
            return Err(HuntErrorCode::Unauthorized);
        }
        Ok(())
    }
}
