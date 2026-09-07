//! Tests for the view-only access bounds fix (#837).
//!
//! Verifies that:
//!   1. Both the per-hunt and global view-only lists are capped at
//!      `MAX_VIEW_ONLY_ENTRIES` and reject additions past the cap.
//!   2. Both `get_view_only_list` and `get_global_view_only_list` are
//!      paginated by `offset`/`limit`.
//!   3. `is_view_only` / `is_global_view_only` are backed by O(1)
//!      membership keys instead of scanning a list.

#[cfg(test)]
mod view_only_bounds {
    use crate::errors::HuntErrorCode;
    use crate::storage::MAX_VIEW_ONLY_ENTRIES;
    use crate::HuntyCore;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env, String, Vec};

    /// Batch size used by the pagination tests (must mirror `MAX_BATCH_SIZE`
    /// in lib.rs, which caps each getter's `limit`).
    const PAGE_SIZE: u32 = 50;

    // ─── Helpers ──────────────────────────────────────────────────────────────

    fn execute_in_contract<T, F>(env: &Env, contract_id: &Address, f: F) -> T
    where
        F: FnOnce(&Env) -> T,
    {
        env.as_contract(contract_id, || f(env))
    }

    /// Collect the full per-hunt view-only list by paging through the contract
    /// getter (whose `limit` is capped at `MAX_BATCH_SIZE`).
    fn collect_view_only(
        env: &Env,
        contract_id: &Address,
        hunt_id: u64,
    ) -> Vec<Address> {
        let mut all: Vec<Address> = Vec::new(env);
        let mut offset = 0u32;
        loop {
            let page = execute_in_contract(env, contract_id, |env| {
                HuntyCore::get_view_only_list(env.clone(), hunt_id, offset, PAGE_SIZE)
            });
            let n = page.len();
            for i in 0..n {
                all.push_back(page.get(i).unwrap());
            }
            if n < PAGE_SIZE {
                break;
            }
            offset += PAGE_SIZE;
        }
        all
    }

    /// Collect the full global view-only list by paging through the contract
    /// getter (whose `limit` is capped at `MAX_BATCH_SIZE`).
    fn collect_global_view_only(env: &Env, contract_id: &Address) -> Vec<Address> {
        let mut all: Vec<Address> = Vec::new(env);
        let mut offset = 0u32;
        loop {
            let page = execute_in_contract(env, contract_id, |env| {
                HuntyCore::get_global_view_only_list(env.clone(), offset, PAGE_SIZE)
            });
            let n = page.len();
            for i in 0..n {
                all.push_back(page.get(i).unwrap());
            }
            if n < PAGE_SIZE {
                break;
            }
            offset += PAGE_SIZE;
        }
        all
    }

    /// Register a contract and create a hunt owned by `creator`.
    fn setup_hunt(env: &Env) -> (Address, u64, Address) {
        let creator = Address::generate(env);
        let contract_id = env.register(HuntyCore, ());
        env.mock_all_auths();
        let hunt_id = execute_in_contract(env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "List Bounds Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
                10,
                None,
                None,
            )
            .unwrap()
        });
        (contract_id, hunt_id, creator)
    }

    /// Register the contract and set `admin` as contract admin.
    fn setup_admin(env: &Env) -> (Address, Address) {
        let admin = Address::generate(&env);
        let contract_id = env.register(HuntyCore, ());
        env.mock_all_auths();
        execute_in_contract(env, &contract_id, |env| {
            HuntyCore::initialize_admin(env.clone(), admin.clone()).unwrap();
        });
        (contract_id, admin)
    }

    fn grant_view_only(
        env: &Env,
        contract_id: &Address,
        hunt_id: u64,
        creator: &Address,
        viewer: &Address,
    ) {
        execute_in_contract(env, contract_id, |env| {
            HuntyCore::add_view_only_access(
                env.clone(),
                hunt_id,
                creator.clone(),
                viewer.clone(),
            )
            .unwrap();
        });
    }

    // ─── Per-hunt list ────────────────────────────────────────────────────────

    #[test]
    fn add_view_only_is_limited_and_idempotent() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, hunt_id, creator) = setup_hunt(&env);

        // Adding is idempotent for an existing member.
        let viewer = Address::generate(&env);
        grant_view_only(&env, &contract_id, hunt_id, &creator, &viewer);
        grant_view_only(&env, &contract_id, hunt_id, &creator, &viewer);

        let list = collect_view_only(&env, &contract_id, hunt_id);
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn per_hunt_list_reaches_cap_then_rejects() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, hunt_id, creator) = setup_hunt(&env);

        // Fill the list up to the cap with unique addresses.
        for _ in 0..MAX_VIEW_ONLY_ENTRIES {
            let viewer = Address::generate(&env);
            grant_view_only(&env, &contract_id, hunt_id, &creator, &viewer);
        }

        // One more unique address must be rejected.
        let extra = Address::generate(&env);
        let result = execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::add_view_only_access(env.clone(), hunt_id, creator.clone(), extra.clone())
        });
        assert_eq!(result, Err(HuntErrorCode::HuntFull));

        // The list still holds exactly MAX_VIEW_ONLY_ENTRIES members.
        let list = collect_view_only(&env, &contract_id, hunt_id);
        assert_eq!(list.len(), MAX_VIEW_ONLY_ENTRIES as u32);
    }

    #[test]
    fn is_view_only_reflects_membership() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, hunt_id, creator) = setup_hunt(&env);

        let member = Address::generate(&env);
        let non_member = Address::generate(&env);
        grant_view_only(&env, &contract_id, hunt_id, &creator, &member);

        assert!(execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::is_view_only(env.clone(), hunt_id, member.clone())
        }));
        assert!(!execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::is_view_only(env.clone(), hunt_id, non_member.clone())
        }));

        // After removal the member is no longer view-only.
        execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::remove_view_only_access(
                env.clone(),
                hunt_id,
                creator.clone(),
                member.clone(),
            )
            .unwrap();
        });
        assert!(!execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::is_view_only(env.clone(), hunt_id, member.clone())
        }));
    }

    #[test]
    fn paginated_view_only_list_pages_correctly() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, hunt_id, creator) = setup_hunt(&env);

        let mut expected: Vec<Address> = Vec::new(&env);
        for _ in 0..7 {
            let viewer = Address::generate(&env);
            grant_view_only(&env, &contract_id, hunt_id, &creator, &viewer);
            expected.push_back(viewer);
        }

        // Page 0 (3 entries).
        let page0 = execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::get_view_only_list(env.clone(), hunt_id, 0, 3)
        });
        assert_eq!(page0.len(), 3);
        assert_eq!(page0.get(0).unwrap(), expected.get(0).unwrap());
        assert_eq!(page0.get(2).unwrap(), expected.get(2).unwrap());

        // Page 1 (next 3).
        let page1 = execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::get_view_only_list(env.clone(), hunt_id, 3, 3)
        });
        assert_eq!(page1.len(), 3);
        assert_eq!(page1.get(0).unwrap(), expected.get(3).unwrap());

        // Final partial page + past-the-end page.
        let page2 = execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::get_view_only_list(env.clone(), hunt_id, 6, 3)
        });
        assert_eq!(page2.len(), 1);
        assert_eq!(page2.get(0).unwrap(), expected.get(6).unwrap());

        let past_end = execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::get_view_only_list(env.clone(), hunt_id, 50, 10)
        });
        assert_eq!(past_end.len(), 0);
    }

    // ─── Global list ──────────────────────────────────────────────────────────

    #[test]
    fn global_list_reaches_cap_then_rejects() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, admin) = setup_admin(&env);

        for _ in 0..MAX_VIEW_ONLY_ENTRIES {
            let viewer = Address::generate(&env);
            execute_in_contract(&env, &contract_id, |env| {
                HuntyCore::add_global_view_only(env.clone(), admin.clone(), viewer.clone())
                    .unwrap();
            });
        }

        let extra = Address::generate(&env);
        let result = execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::add_global_view_only(env.clone(), admin.clone(), extra.clone())
        });
        assert_eq!(result, Err(HuntErrorCode::HuntFull));

        let list = collect_global_view_only(&env, &contract_id);
        assert_eq!(list.len(), MAX_VIEW_ONLY_ENTRIES as u32);
    }

    #[test]
    fn is_global_view_only_reflects_membership() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, admin) = setup_admin(&env);

        let member = Address::generate(&env);
        let non_member = Address::generate(&env);
        execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::add_global_view_only(env.clone(), admin.clone(), member.clone()).unwrap();
        });

        assert!(execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::is_global_view_only(env.clone(), member.clone())
        }));
        assert!(!execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::is_global_view_only(env.clone(), non_member.clone())
        }));

        execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::remove_global_view_only(env.clone(), admin.clone(), member.clone()).unwrap();
        });
        assert!(!execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::is_global_view_only(env.clone(), member.clone())
        }));
    }

    #[test]
    fn paginated_global_list_pages_correctly() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, admin) = setup_admin(&env);

        let mut expected: Vec<Address> = Vec::new(&env);
        for _ in 0..4 {
            let viewer = Address::generate(&env);
            execute_in_contract(&env, &contract_id, |env| {
                HuntyCore::add_global_view_only(env.clone(), admin.clone(), viewer.clone())
                    .unwrap();
            });
            expected.push_back(viewer);
        }

        let page0 = execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::get_global_view_only_list(env.clone(), 0, 2)
        });
        assert_eq!(page0.len(), 2);
        assert_eq!(page0.get(0).unwrap(), expected.get(0).unwrap());

        let page1 = execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::get_global_view_only_list(env.clone(), 2, 2)
        });
        assert_eq!(page1.len(), 2);
        assert_eq!(page1.get(0).unwrap(), expected.get(2).unwrap());

        let past_end = execute_in_contract(&env, &contract_id, |env| {
            HuntyCore::get_global_view_only_list(env.clone(), 10, 2)
        });
        assert_eq!(past_end.len(), 0);
    }
}