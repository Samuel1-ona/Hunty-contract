#[cfg(test)]
mod stress_tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, String, Vec};

    #[test]
    fn test_max_clues_stress_and_gas() {
        let env = Env::default();
        let contract_id = env.register_contract(None, HuntyContract);
        let client = HuntyContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let hunt_name = String::from_str(&env, "Ultimate Stress Hunt");
        
        // Create 101 clues
        let mut clues = Vec::new(&env);
        for i in 1..=101 {
            clues.push_back(String::from_str(&env, &format!("Evidence clue #{}", i)));
        }

        // 1. Create hunt
        client.create_hunt(&admin, &hunt_name);

        // 2. Add exactly 100 clues (should succeed)
        for i in 0..100 {
            client.add_clue(&admin, &hunt_name, &clues.get(i).unwrap());
        }

        // 3. Try to add 101st clue (must fail with TooManyClues)
        let result = client.try_add_clue(&admin, &hunt_name, &clues.get(100).unwrap());
        assert!(
            result.is_err(),
            "Adding 101st clue should fail, but it succeeded"
        );
        
        // Match specific error if your contract defines it
        if let Err(err) = result {
            let err_str = err.to_string();
            assert!(
                err_str.contains("TooManyClues") || err_str.contains("max clues"),
                "Wrong error type: {}",
                err_str
            );
        }

        // 4. Benchmark list_clues gas at capacity (100 clues)
        env.budget().reset_default();
        
        let start_instructions = env.budget().cpu_instructions().0;
        let clue_list = client.list_clues(&hunt_name);
        let end_instructions = env.budget().cpu_instructions().0;
        
        let gas_used = end_instructions - start_instructions;
        
        // Verify all 100 clues are returned correctly
        assert_eq!(
            clue_list.len(),
            100,
            "Should return exactly 100 clues, got {}",
            clue_list.len()
        );
        
        for i in 0..100 {
            assert_eq!(clue_list.get(i).unwrap(), clues.get(i).unwrap());
        }
        
        // Log gas (visible with --nocapture)
        env.print(&format!(
            "✅ Stress test passed. Gas (CPU instructions) to list 100 clues: {}",
            gas_used
        ));
        
        // Optional: Assert reasonable gas limit (adjust based on your contract)
        assert!(
            gas_used < 50_000_000,
            "Gas too high: {} instructions",
            gas_used
        );
    }
}