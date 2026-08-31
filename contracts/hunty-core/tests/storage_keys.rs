use soroban_sdk::{testutils::Address as _, Address, Env};

use hunty_core::rate_limit::{RateLimitData, RateLimiter, SECONDS_PER_DAY};

#[test]
fn test_single_rate_limit_storage_entry_across_days() {
    let env = Env::default();
    let creator = Address::generate(&env);

    let day1 = 0;
    let day2 = SECONDS_PER_DAY;
    let day3 = 2 * SECONDS_PER_DAY;

    for now in [day1, day2, day3] {
        assert!(RateLimiter::check_and_increment(&env, &creator, now).is_ok());
    }

    let entry: Option<RateLimitData> = env.storage().persistent().get(&creator);
    assert!(entry.is_some());
    let entry = entry.unwrap();

    let expected_day = day3 / SECONDS_PER_DAY;
    assert_eq!(entry.day, expected_day);
    assert_eq!(entry.count, 1);
}
