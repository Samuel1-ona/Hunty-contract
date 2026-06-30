const fs = require('fs');
const path = require('path');

const testFile = path.join(__dirname, '../contracts/hunty-core/src/test.rs');
let content = fs.readFileSync(testFile, 'utf8');

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

        let contract_id = env.register_contract(None, HuntyCore);
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

        let contract_id = env.register_contract(None, HuntyCore);
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

const startM = 'fn test_pause_contract_blocks_answer_submission_until_unpaused() {';
const endM = 'fn test_required_completed_counter_stays_isolated_per_player() {';
const startI = content.indexOf(startM);
const endI = content.indexOf(endM);

if (startI !== -1 && endI !== -1) {
    content = content.substring(0, startI - 17) + correctedClosedTests + '\n\n' + content.substring(endI - 5);
}

fs.writeFileSync(testFile, content, 'utf8');
console.log('Successfully fixed delimiters only in test.rs!');
