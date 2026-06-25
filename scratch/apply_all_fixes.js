const fs = require('fs');
const path = require('path');

const testFile = path.join(__dirname, '../contracts/hunty-core/src/test.rs');
let content = fs.readFileSync(testFile, 'utf8');

// 1. Replace register_contract(None, HuntyCore) with register_contract(None, super::HuntyCore)
content = content.replace(/register_contract\(None,\s*HuntyCore\)/g, 'register_contract(None, super::HuntyCore)');

// 2. Replace env.register(HuntyCore with env.register(super::HuntyCore
content = content.replace(/env\.register\(HuntyCore/g, 'env.register(super::HuntyCore');

// 3. Define the correct, robust client wrapper struct
const wrapperCode = `    use crate::HuntyCoreClient;
    use crate::types::{PlayerProgress, LeaderboardEntry, HuntStatistics, Hunt, HuntRewardConfig};

    struct HuntyCore;
    impl HuntyCore {
        pub fn initialize_admin(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_initialize_admin(&admin) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn pause_contract(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_pause_contract(&admin) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn unpause_contract(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_unpause_contract(&admin) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn is_contract_paused(env: Env) -> bool {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            client.is_contract_paused()
        }

        pub fn create_hunt(
            env: Env,
            creator: Address,
            title: String,
            description: String,
            start_time: Option<u64>,
            end_time: Option<u64>,
        ) -> Result<u64, HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_create_hunt(&creator, &title, &description, &start_time, &end_time) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn create_hunt_from_template(
            env: Env,
            template_hunt_id: u64,
            creator: Address,
            title: String,
            description: String,
            start_time: Option<u64>,
            end_time: Option<u64>,
        ) -> Result<u64, HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_create_hunt_from_template(&template_hunt_id, &creator, &title, &description, &start_time, &end_time) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn set_time_bonus_config(
            env: Env,
            hunt_id: u64,
            caller: Address,
            config: Option<TimeBonusConfig>,
        ) -> Result<(), HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_set_time_bonus_config(&hunt_id, &caller, &config) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn update_hunt(
            env: Env,
            hunt_id: u64,
            caller: Address,
            max_attempts_per_clue: u32,
        ) -> Result<(), HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_update_hunt(&hunt_id, &caller, &max_attempts_per_clue) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn add_clue(
            env: Env,
            hunt_id: u64,
            question: String,
            answer: String,
            points: u32,
            is_required: bool,
            difficulty: u32,
        ) -> Result<u32, HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_add_clue(&hunt_id, &question, &answer, &points, &is_required, &difficulty) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn add_clue_aliases(
            env: Env,
            hunt_id: u64,
            clue_id: u32,
            answers: Vec<String>,
        ) -> Result<(), HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_add_clue_aliases(&hunt_id, &clue_id, &answers) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn get_clue(env: Env, hunt_id: u64, clue_id: u32) -> Result<ClueInfo, HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_get_clue(&hunt_id, &clue_id) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn list_clues(env: Env, hunt_id: u64, offset: u32, limit: u32) -> Vec<ClueInfo> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            client.list_clues(&hunt_id, &offset, &limit)
        }

        pub fn activate_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<(), HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_activate_hunt(&hunt_id, &caller) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn deactivate_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<(), HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_deactivate_hunt(&hunt_id, &caller) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn register_player(env: Env, hunt_id: u64, player: Address) -> Result<(), HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_register_player(&hunt_id, &player) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn submit_answer(
            env: Env,
            hunt_id: u64,
            clue_id: u32,
            player: Address,
            answer: String,
        ) -> Result<(), HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_submit_answer(&hunt_id, &clue_id, &player, &answer) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn complete_hunt(env: Env, hunt_id: u64, player: Address) -> Result<(), HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_complete_hunt(&hunt_id, &player) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn batch_complete_hunt(
            env: Env,
            hunt_id: u64,
            creator: Address,
            players: Vec<Address>,
        ) -> Result<(), HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_batch_complete_hunt(&hunt_id, &creator, &players) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn cancel_hunt(env: Env, hunt_id: u64, creator: Address) -> Result<(), HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_cancel_hunt(&hunt_id, &creator) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn get_player_progress(
            env: Env,
            hunt_id: u64,
            player: Address,
        ) -> Result<PlayerProgress, HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_get_player_progress(&hunt_id, &player) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn get_completed_clues(
            env: Env,
            hunt_id: u64,
            player: Address,
        ) -> Vec<u32> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            client.get_completed_clues(&hunt_id, &player)
        }

        pub fn get_hunt_count(env: Env) -> u64 {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            client.get_hunt_count()
        }

        pub fn get_hunt_leaderboard(
            env: Env,
            hunt_id: u64,
            window_size: u32,
            start_index: u32,
        ) -> Result<Vec<LeaderboardEntry>, HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_get_hunt_leaderboard(&hunt_id, &window_size, &start_index) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn get_hunt_statistics(
            env: Env,
            hunt_id: u64,
        ) -> Result<HuntStatistics, HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_get_hunt_statistics(&hunt_id) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn get_hunt_info(env: Env, hunt_id: u64) -> Result<Hunt, HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_get_hunt_info(&hunt_id) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn set_reward_manager(
            env: Env,
            caller: Address,
            reward_manager: Address,
        ) -> Result<(), HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_set_reward_manager(&caller, &reward_manager) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }

        pub fn get_reward_manager(env: Env) -> Option<Address> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            client.get_reward_manager()
        }

        pub fn cleanup_hunt(env: Env, admin: Address, hunt_id: u64) -> Result<u32, HuntErrorCode> {
            let cid = env.current_contract_address();
            let client = HuntyCoreClient::new(&env, &cid);
            match client.try_cleanup_hunt(&admin, &hunt_id) {
                Ok(Ok(res)) => Ok(res),
                Ok(Err(err)) => Err(err),
                Err(e) => panic!("Invocation error: {:?}", e),
            }
        }
    }`;

content = content.replace('    use crate::HuntyCore;', wrapperCode);

// 4. Fix specific compilation issues in test.rs
// Fix 1: add_clue in test_deactivate_draft_hunt_fails
content = content.replace(
    'HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true).unwrap();',
    'HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();'
);

// Fix 2: add_clue in test_leaderboard_pagination_and_sorting
content = content.replace(
    `            HuntyCore::add_clue(
                env.clone(),
                hid,
                String::from_str(env, "Q"),
                String::from_str(env, "a"),
                10,
                true,
            )
            .unwrap();`,
    `            HuntyCore::add_clue(
                env.clone(),
                hid,
                String::from_str(env, "Q"),
                String::from_str(env, "a"),
                10,
                true,
                1,
            )
            .unwrap();`
);

// Fix 3: RewardConfig -> HuntRewardConfig in test_leaderboard_pagination_and_sorting
content = content.replace(
    'hunt.reward_config = crate::types::RewardConfig::new(1000, false, None, 10, 0, 0);',
    'hunt.reward_config = crate::types::HuntRewardConfig::new(env, 1000, false, None, 10, 0, 0);'
);

// Fix 4: Restructure test_register_player_allowed_after_reactivation
const originalReactivationTest = `    #[test]
    fn test_register_player_allowed_after_reactivation() {
        // A player who registered in a previous activation cycle must be able to
        // re-register after the hunt is deactivated and reactivated.
        let env = Env::default();
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let (hunt_id, core_id) = with_core_contract(&env, |env, cid| {
            env.ledger().set_timestamp(1_000);
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();

            // First activation — player registers
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
            let first_progress =
                HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap();

            // Creator deactivates then reactivates (new cycle)
            HuntyCore::deactivate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            env.ledger().set_timestamp(2_000);
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            let hunt = Storage::get_hunt(&env, hunt_id).unwrap();
            assert!(first_progress.started_at < hunt.activated_at);

            // Player should be able to register again — old progress is stale
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
            let latest_progress =
                HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap();
            assert!(latest_progress.started_at >= hunt.activated_at);
            assert_eq!(latest_progress.completed_clues.len(), 0);

            // But a second call in the same cycle must still be rejected
            let err = HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap_err();
            assert_eq!(err, HuntErrorCode::DuplicateRegistration);
        });
    }`;

const newReactivationTest = `    #[test]
    fn test_register_player_allowed_after_reactivation() {
        // A player who registered in a previous activation cycle must be able to
        // re-register after the hunt is deactivated and reactivated.
        let env = Env::default();
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let (hunt_id, core_id) = with_core_contract(&env, |env, cid| {
            env.ledger().set_timestamp(1_000);
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();
            (hunt_id, cid.clone())
        });

        // First activation
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        // Player registers
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });

        let first_progress = as_core_contract(&env, &core_id, |env| {
            HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap()
        });

        // Creator deactivates
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::deactivate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        env.ledger().set_timestamp(2_000);

        // Reactivate
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        let hunt = as_core_contract(&env, &core_id, |env| {
            Storage::get_hunt(env, hunt_id).unwrap()
        });
        assert!(first_progress.started_at < hunt.activated_at);

        // Player should be able to register again — old progress is stale
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });

        let latest_progress = as_core_contract(&env, &core_id, |env| {
            HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap()
        });
        assert!(latest_progress.started_at >= hunt.activated_at);
        assert_eq!(latest_progress.completed_clues.len(), 0);

        // But a second call in the same cycle must still be rejected
        let err = as_core_contract(&env, &core_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap_err()
        });
        assert_eq!(err, HuntErrorCode::DuplicateRegistration);
    }`;

// Normalize line endings for replacement mapping
const normalize = str => str.replace(/\r\n/g, '\n').trim();

// Fallback replacement if direct regex match fails
const startMarker = 'fn test_register_player_allowed_after_reactivation() {';
const endMarker = '    #[test]\n    fn test_register_player_blocked_when_paused() {';
const startIdx = content.indexOf(startMarker);
// Let's find the matching end index
if (startIdx !== -1) {
    const nextTestIdx = content.indexOf('    #[test]', startIdx + startMarker.length);
    if (nextTestIdx !== -1) {
        console.log("Replacing test_register_player_allowed_after_reactivation...");
        content = content.substring(0, startIdx - 17) + newReactivationTest + '\n\n' + content.substring(nextTestIdx);
    }
}

// Fix 5: Mismatched unclosed delimiters in the clean file
const correctedClosedTests = `    #[test]
    fn test_pause_contract_blocks_answer_submission_until_unpaused() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let contract_id = env.register_contract(None, super::HuntyCore);
        env.mock_all_auths();
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::initialize_admin(env.clone(), admin.clone()).unwrap();
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer.clone(), 10, true, 1)
                .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
            HuntyCore::pause_contract(env.clone(), admin.clone()).unwrap();
            hunt_id
        });

        env.mock_all_auths();
        let err = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(
                env.clone(),
                hunt_id,
                1,
                player.clone(),
                answer.clone(),
            )
            .unwrap_err()
        });
        assert_eq!(err, HuntErrorCode::ContractPaused);

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::unpause_contract(env.clone(), admin.clone()).unwrap();
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player.clone(), answer)
                .unwrap();
        });
    }

    #[test]
    fn test_required_completed_counter_is_not_double_incremented_on_resubmit() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let contract_id = env.register_contract(None, super::HuntyCore);
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer.clone(), 10, true, 1).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            hunt_id
        });

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player.clone(), answer.clone()).unwrap();
        });
        env.mock_all_auths();
        let resubmit = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player.clone(), answer)
        });

        assert_eq!(resubmit, Err(HuntErrorCode::ClueAlreadyCompleted));

        let progress = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap()
        });
        assert_eq!(progress.required_completed_count, 1);
    }`;

// Fallback: replace using exact startIdx and endIdx
const startM = 'fn test_pause_contract_blocks_answer_submission_until_unpaused() {';
const endM = 'fn test_required_completed_counter_stays_isolated_per_player() {';
const startI = content.indexOf(startM);
const endI = content.indexOf(endM);

console.log("startI of mismatched tests:", startI);
console.log("endI of mismatched tests:", endI);

if (startI !== -1 && endI !== -1) {
    console.log("Replacing mismatched tests...");
    content = content.substring(0, startI - 17) + correctedClosedTests + '\n\n' + content.substring(endI - 5);
}

fs.writeFileSync(testFile, content, 'utf8');
console.log('Successfully applied all fixes, closing delimiters, and the corrected wrapper to test.rs!');
