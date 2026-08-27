**Solution Overview**

Add a guard that checks the `max_winners` limit *before* the reward‑manager is called.  
If the limit is reached, return `Error::InsufficientRewardPool`.  
Move the `claimed_count += 1` into the success branch and use a checked add so that an
overflow can never silently happen.

Below are the minimal patches that implement this behaviour.

---

## 1.  Update the `Error` enum

```rust
// contracts/hunty-core/src/lib.rs

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    // … existing variants …
    InsufficientRewardPool,   // <‑‑ already documented
    Overflow,                 // <‑‑ new variant for checked add
}
```

---

## 2.  Patch `complete_hunt`

```rust
// contracts/hunty-core/src/lib.rs
pub fn complete_hunt(env: &Env, hunt_id: u64, winner: &Address) -> Result<(), Error> {
    // Load the hunt
    let mut hunt = Storage::load_hunt(&env, hunt_id)?;

    // 1️⃣  Ensure the hunt is still active
    if !hunt.is_active() {
        return Err(Error::HuntNotActive);
    }

    // 2️⃣  Enforce the `max_winners` cap
    if hunt.reward_config.max_winners != 0 {
        if hunt.reward_config.claimed_count >= hunt.reward_config.max_winners {
            // All reward slots are already taken
            return Err(Error::InsufficientRewardPool);
        }
    }

    // 3️⃣  Dispatch to the reward‑manager *before* we bump the counter
    let reward_result = RewardManager::distribute(&env, &hunt, winner);

    // 4️⃣  Increment `claimed_count` only on success
    match reward_result {
        Ok(_) => {
            // Use checked_add to guard against overflow
            hunt.reward_config.claimed_count = hunt
                .reward_config
                .claimed_count
                .checked_add(1)
                .ok_or(Error::Overflow)?;

            // Persist the updated hunt
            Storage::save_hunt(&env, &hunt);
            Ok(())
        }
        Err(e) => Err(e), // propagate the reward‑manager error
    }
}
``