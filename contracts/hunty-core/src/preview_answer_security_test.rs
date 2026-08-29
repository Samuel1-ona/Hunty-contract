/// Security tests for the preview_answer endpoint.
///
/// These verify that after the security hardening introduced in the fix for
/// the unauthenticated dictionary-attack vector:
///   1. preview_answer correctly returns Ok(true/false) for correct/wrong answers.
///   2. preview_answer is rate-limited by max_submissions_per_minute, just like
///      submit_answer, preventing offline dictionary attacks.
///   3. preview_answer enforces player.require_auth(), so it cannot be called on
///      behalf of another address without their authorisation.

#[cfg(test)]
mod preview_answer_security {
    use crate::errors::HuntErrorCode;
    use crate::HuntyCore;
    use soroban_sdk::testutils::{Address as _, Ledger as _};
    use soroban_sdk::{Address, Env, String};

    // ─── Helpers ──────────────────────────────────────────────────────────────

    fn execute_in_contract<T, F>(env: &Env, contract_id: &Address, f: F) -> T
    where
        F: FnOnce(&Env) -> T,
    {
        env.as_contract(contract_id, || f(env))
    }

    /// Set up a minimal active hunt with one clue and one registered player.
    /// Returns `(contract_id, hunt_id, player)`.
    fn setup_active_hunt(
        env: &Env,
        max_subs_per_minute: u32,
    ) -> (Address, u64, Address) {
        let creator = Address::generate(env);
        let player = Address::generate(env);
        let contract_id = env.register(HuntyCore, ());

        // Create hunt
        env.mock_all_auths();
        let hunt_id = execute_in_contract(env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Security Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
                max_subs_per_minute,
                None,
                None,
            )
            .unwrap()
        });

        // Add clue (answer = "correct") and activate
        env.mock_all_auths();
        execute_in_contract(env, &contract_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                String::from_str(env, "What is the answer?"),
                String::from_str(env, "correct"),
                10,
                true,
                None,
                None,
            )
            .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        // Register player
        env.mock_all_auths();
        execute_in_contract(env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });

        (contract_id, hunt_id, player)
    }

    // ─── Tests ────────────────────────────────────────────────────────────────

    #[test]
    fn test_preview_answer_correct_returns_true() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let (contract_id, hunt_id, player) = setup_active_hunt(&env, 0);

        env.mock_all_auths();
        let result = execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::preview_answer(
                env.clone(),
                hunt_id,
                1,
                player.clone(),
                String::from_str(env, "correct"),
            )
        });

        assert_eq!(result, Ok(true), "correct answer must return Ok(true)");
    }

    #[test]
    fn test_preview_answer_wrong_returns_false() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let (contract_id, hunt_id, player) = setup_active_hunt(&env, 0);

        env.mock_all_auths();
        let result = execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::preview_answer(
                env.clone(),
                hunt_id,
                1,
                player.clone(),
                String::from_str(env, "wrong_guess"),
            )
        });

        assert_eq!(result, Ok(false), "wrong answer must return Ok(false)");
    }

    /// Core security regression test: repeated preview_answer calls within the
    /// same minute must be throttled by max_submissions_per_minute, preventing
    /// offline dictionary attacks via unlimited free oracle calls.
    #[test]
    fn test_preview_answer_rate_limited() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        // Limit to 2 submissions per minute
        let (contract_id, hunt_id, player) = setup_active_hunt(&env, 2);

        // 1st preview — allowed
        env.mock_all_auths();
        let r1 = execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::preview_answer(
                env.clone(),
                hunt_id,
                1,
                player.clone(),
                String::from_str(env, "attempt_1"),
            )
        });
        assert!(r1.is_ok(), "first preview must succeed: {:?}", r1);

        // 2nd preview — still allowed
        env.mock_all_auths();
        let r2 = execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::preview_answer(
                env.clone(),
                hunt_id,
                1,
                player.clone(),
                String::from_str(env, "attempt_2"),
            )
        });
        assert!(r2.is_ok(), "second preview must succeed: {:?}", r2);

        // 3rd preview within the same 60-second window — must be rejected
        env.mock_all_auths();
        let r3 = execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::preview_answer(
                env.clone(),
                hunt_id,
                1,
                player.clone(),
                String::from_str(env, "attempt_3"),
            )
        });
        assert_eq!(
            r3,
            Err(HuntErrorCode::RateLimitExceeded),
            "third preview in the same minute must be rate-limited"
        );
    }

    /// Auth regression test: preview_answer must call player.require_auth();
    /// omitting the mock causes a Soroban auth panic, confirming the gate is
    /// present and cannot be bypassed by naming an arbitrary registered address.
    #[test]
    #[should_panic]
    fn test_preview_answer_requires_player_auth() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let (contract_id, hunt_id, player) = setup_active_hunt(&env, 0);

        // Clear mocked auths — preview_answer must panic when player.require_auth() is invoked
        env.set_auths(&[]);

        execute_in_contract(&env, &contract_id, |env| {
            let _ = HuntyCore::preview_answer(
                env.clone(),
                hunt_id,
                1,
                player.clone(),
                String::from_str(env, "ans"),
            );
        });
    }
}
