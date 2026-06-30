# Contract API Reference

This file is generated automatically from the Rust contract sources in `contracts/`.
Run `make build` to regenerate it whenever contract APIs change.


## `common` Contract

_No contract API functions found._

## `hunty-core` Contract

### `HuntyCore`

#### `create_hunt`

Creates a new scavenger hunt with the provided metadata.

# Arguments
* `env` - The Soroban environment
* `creator` - The address of the hunt creator (typically use env.invoker() from the caller)
* `title` - The title of the hunt (max 200 characters)
* `description` - The description of the hunt (max 2000 characters)
* `start_time` - Optional start timestamp (0 means no start time restriction)
* `end_time` - Optional end timestamp (0 means no end time restriction)

# Returns
The unique hunt ID of the newly created hunt

# Errors
* `InvalidTitle` - If title is empty or exceeds maximum length
* `InvalidDescription` - If description exceeds maximum length
* `InvalidAddress` - If creator address is invalid

**Signature:**

```rust
pub fn create_hunt(env: Env, creator: Address, title: String, description: String, _start_time: Option<u64>, end_time: Option<u64>, max_submissions_per_minute: u32, start_multiplier_bps: Option<u32>) -> Result<u64, HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `creator: Address`
- `title: String`
- `description: String`
- `_start_time: Option<u64>`
- `end_time: Option<u64>`
- `max_submissions_per_minute: u32`
- `start_multiplier_bps: Option<u32>`

**Returns:** `Result<u64, HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `add_clue`

Adds a clue to a hunt. Only the hunt creator can add clues.
Answers are hashed with SHA256 before storage; the hash is never exposed.

# Arguments
* `env` - The Soroban environment
* `hunt_id` - The hunt to add the clue to
* `question` - The clue question text (max 2000 chars, non-empty)
* `answer` - Plain-text answer; normalized (trimmed, lowercased) then hashed
* `points` - Points awarded for solving this clue
* `is_required` - Whether this clue must be solved to complete the hunt

# Returns
The sequential clue ID assigned within the hunt

# Errors
* `HuntNotFound` - Hunt does not exist
* `InvalidHuntStatus` - Hunt is not in Draft
* `Unauthorized` - Caller is not the hunt creator
* `TooManyClues` - Hunt already has max clues
* `InvalidQuestion` - Question empty or too long
* `InvalidAnswer` - Answer empty or too long

**Signature:**

```rust
pub fn add_clue(env: Env, hunt_id: u64, question: String, answer: String, points: u32, is_required: bool, difficulty: Option<u32>) -> Result<u32, HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `question: String`
- `answer: String`
- `points: u32`
- `is_required: bool`
- `difficulty: Option<u32>`

**Returns:** `Result<u32, HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `get_clue`

Returns clue information for a hunt/clue. Does not expose the answer hash.

**Signature:**

```rust
pub fn get_clue(env: Env, hunt_id: u64, clue_id: u32) -> Result<ClueInfo, HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `clue_id: u32`

**Returns:** `Result<ClueInfo, HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `list_clues`

Returns all clues for a hunt (question, points, required). Answer hashes are not exposed.
This loads all clues; for large hunts use `list_clues_paginated` to limit gas cost.
Estimated gas: O(n) where n = total_clues, ~5_000 gas per clue.

**Signature:**

```rust
pub fn list_clues(env: Env, hunt_id: u64) -> Vec<ClueInfo>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`

**Returns:** `Vec<ClueInfo>`

---

#### `list_clues_paginated`

Returns a paginated slice of clues for a hunt. Useful for large hunts to bound gas.
Page is 0-indexed. Max page_size is capped at MAX_BATCH_SIZE (50).
Estimated gas: O(page_size) ~5_000 gas per clue + 10_000 overhead.

**Signature:**

```rust
pub fn list_clues_paginated(env: Env, hunt_id: u64, page: u32, page_size: u32) -> Vec<ClueInfo>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `page: u32`
- `page_size: u32`

**Returns:** `Vec<ClueInfo>`

---

#### `activate_hunt`

Normalizes answer (trim, lowercase) and returns SHA256 hash as BytesN<32>.
Uses hunt_id and clue_id as salt to prevent rainbow table precomputation.
Hashing scheme: SHA256(hunt_id || clue_id || normalized_answer)

**Signature:**

```rust
pub fn activate_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `caller: Address`

**Returns:** `Result<(), HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `deactivate_hunt`

**Signature:**

```rust
pub fn deactivate_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `caller: Address`

**Returns:** `Result<(), HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `cancel_hunt`

**Signature:**

```rust
pub fn cancel_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `caller: Address`

**Returns:** `Result<(), HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `get_hunt_info`

**Signature:**

```rust
pub fn get_hunt_info(env: Env, hunt_id: u64) -> Result<Hunt, HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`

**Returns:** `Result<Hunt, HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `set_reward_manager`

Sets the RewardManager contract address for cross-contract reward distribution.

**Signature:**

```rust
pub fn set_reward_manager(env: Env, reward_manager: Address) -> ()
```

**Parameters:**

- `env: Env`
- `reward_manager: Address`

**Returns:** `()`

---

#### `complete_hunt`

Completes a hunt for a player and distributes rewards.

This function verifies that the player has completed all required clues,
then distributes rewards via the RewardManager contract (if configured)
and updates the player's reward status.

# Arguments
* `env` - The Soroban environment
* `hunt_id` - The hunt ID
* `player` - The player claiming completion/rewards

# Returns
`Ok(())` on successful reward claim

# Errors
* `HuntNotFound` - Hunt does not exist
* `PlayerNotRegistered` - Player is not registered
* `HuntNotCompleted` - Player hasn't completed all required clues
* `RewardAlreadyClaimed` - Player already claimed their reward
* `NoRewardsConfigured` - No rewards set up for this hunt
* `InsufficientRewardPool` - All reward slots taken
* `RewardDistributionFailed` - Cross-contract call failed

**Signature:**

```rust
pub fn complete_hunt(env: Env, hunt_id: u64, player: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `player: Address`

**Returns:** `Result<(), HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `register_player`

Registers a player for an active hunt. The caller must pass their address and authorize;
only that identity can register themselves. Initializes player progress and prevents
duplicate registrations. Registration is only allowed while the hunt is active and
(if set) before end_time.

# Arguments
* `env` - The Soroban environment
* `hunt_id` - The hunt to register for
* `player` - The address of the player (must authorize the call via require_auth)

# Returns
`Ok(())` on success

# Errors
* `HuntNotFound` - Hunt does not exist
* `InvalidHuntStatus` - Hunt is not in Active status
* `HuntNotActive` - Hunt has ended (past end_time)
* `DuplicateRegistration` - Player is already registered for this hunt

**Signature:**

```rust
pub fn register_player(env: Env, hunt_id: u64, player: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `player: Address`

**Returns:** `Result<(), HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `submit_answer`

This function verifies the submitted answer by hashing it and comparing
with the stored answer hash. If correct, updates player progress and emits
success events. If incorrect, emits an analytics event and returns an error.

# Arguments
* `env` - The Soroban environment
* `hunt_id` - The hunt ID
* `clue_id` - The clue ID to answer
* `player` - The address of the player submitting the answer
* `answer` - The plain-text answer submission
* `submission_nonce` - Caller-chosen unique nonce for this submission envelope
* `submitted_at` - Client timestamp captured when the submission was signed

# Returns
`Ok(())` on successful answer verification and progress update

# Errors
* `HuntNotFound` - Hunt does not exist
* `HuntNotActive` - Hunt is not currently active or has ended
* `PlayerNotRegistered` - Player has not registered for this hunt
* `ClueNotFound` - Clue does not exist in this hunt
* `ClueAlreadyCompleted` - Player has already completed this clue
* `InvalidAnswer` - Submitted answer does not match the stored hash
* `DuplicateSubmission` - Submission nonce/timestamp envelope was already processed
* `SubmissionExpired` - Submission timestamp is too old or too far in the future

# Events
* `ClueCompleted` - Emitted when answer is correct
* `HuntCompleted` - Emitted when all required clues are completed
* `AnswerIncorrect` - Emitted when answer is wrong (for analytics)

**Signature:**

```rust
pub fn submit_answer(env: Env, hunt_id: u64, clue_id: u32, player: Address, answer: String, submission_nonce: u64, submitted_at: u64) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `clue_id: u32`
- `player: Address`
- `answer: String`
- `submission_nonce: u64`
- `submitted_at: u64`

**Returns:** `Result<(), HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `get_player_progress`

Checks if a player has completed all required clues for a hunt.

# Arguments
* `env` - The Soroban environment
* `hunt_id` - The hunt ID
* `progress` - The player's progress data

# Returns
`true` if all required clues are completed, `false` otherwise
Returns player progress for a hunt (read-only).
Includes completed clues, score, and completion status.
Returns error if player is not registered.

**Signature:**

```rust
pub fn get_player_progress(env: Env, hunt_id: u64, player: Address) -> Result<PlayerProgress, HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `player: Address`

**Returns:** `Result<PlayerProgress, HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `get_completed_clues`

Returns the list of clue IDs that the player has completed for a hunt (read-only).
Useful for UI to show progress. Returns empty vec if player is not registered.

**Signature:**

```rust
pub fn get_completed_clues(env: Env, hunt_id: u64, player: Address) -> Vec<u32>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `player: Address`

**Returns:** `Vec<u32>`

---

#### `get_hunt_leaderboard`

Returns the top N players by score for a hunt (read-only).
Sorted by score descending, then by completion time ascending (earlier = better).
Limit is capped at 20 to control gas. Returns error if hunt does not exist.

**Signature:**

```rust
pub fn get_hunt_leaderboard(env: Env, hunt_id: u64, limit: u32) -> Result<Vec<LeaderboardEntry>, HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `limit: u32`

**Returns:** `Result<Vec<LeaderboardEntry>, HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `get_hunt_statistics`

Picks the index of the best entry not in `selected`. Order: score desc, then completed_at asc (0 = last).
Returns aggregate statistics for a hunt (read-only): total players, completion rate, average score.
Returns error if hunt does not exist.

**Signature:**

```rust
pub fn get_hunt_statistics(env: Env, hunt_id: u64) -> Result<HuntStatistics, HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`

**Returns:** `Result<HuntStatistics, HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `add_view_only_access`

**Signature:**

```rust
pub fn add_view_only_access(env: Env, hunt_id: u64, creator: Address, viewer: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `creator: Address`
- `viewer: Address`

**Returns:** `Result<(), HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `remove_view_only_access`

**Signature:**

```rust
pub fn remove_view_only_access(env: Env, hunt_id: u64, creator: Address, viewer: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `creator: Address`
- `viewer: Address`

**Returns:** `Result<(), HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `is_view_only`

**Signature:**

```rust
pub fn is_view_only(env: Env, hunt_id: u64, address: Address) -> bool
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `address: Address`

**Returns:** `bool`

---

#### `get_view_only_list`

**Signature:**

```rust
pub fn get_view_only_list(env: Env, hunt_id: u64) -> Vec<Address>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`

**Returns:** `Vec<Address>`

---

#### `initialize_admin`

**Signature:**

```rust
pub fn initialize_admin(env: Env, admin: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`

**Returns:** `Result<(), HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `add_global_view_only`

**Signature:**

```rust
pub fn add_global_view_only(env: Env, admin: Address, viewer: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `viewer: Address`

**Returns:** `Result<(), HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `remove_global_view_only`

**Signature:**

```rust
pub fn remove_global_view_only(env: Env, admin: Address, viewer: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `viewer: Address`

**Returns:** `Result<(), HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `is_global_view_only`

**Signature:**

```rust
pub fn is_global_view_only(env: Env, address: Address) -> bool
```

**Parameters:**

- `env: Env`
- `address: Address`

**Returns:** `bool`

---

#### `get_global_view_only_list`

**Signature:**

```rust
pub fn get_global_view_only_list(env: Env) -> Vec<Address>
```

**Parameters:**

- `env: Env`

**Returns:** `Vec<Address>`

---

#### `pause_registrations`

**Signature:**

```rust
pub fn pause_registrations(env: Env, admin: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`

**Returns:** `Result<(), HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `unpause_registrations`

**Signature:**

```rust
pub fn unpause_registrations(env: Env, admin: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`

**Returns:** `Result<(), HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `pause_answers`

**Signature:**

```rust
pub fn pause_answers(env: Env, admin: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`

**Returns:** `Result<(), HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `unpause_answers`

**Signature:**

```rust
pub fn unpause_answers(env: Env, admin: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`

**Returns:** `Result<(), HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `pause_rewards`

**Signature:**

```rust
pub fn pause_rewards(env: Env, admin: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`

**Returns:** `Result<(), HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `unpause_rewards`

**Signature:**

```rust
pub fn unpause_rewards(env: Env, admin: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`

**Returns:** `Result<(), HuntErrorCode>`

**Error type:** `HuntErrorCode`

**Error codes:**

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

---

#### `get_pause_state`

**Signature:**

```rust
pub fn get_pause_state(env: Env) -> (bool, bool, bool)
```

**Parameters:**

- `env: Env`

**Returns:** `(bool, bool, bool)`

---

#### `get_schema_version`

**Signature:**

```rust
pub fn get_schema_version(env: Env) -> u32
```

**Parameters:**

- `env: Env`

**Returns:** `u32`

---

#### `initialize_schema`

**Signature:**

```rust
pub fn initialize_schema(env: Env, admin: Address) -> ()
```

**Parameters:**

- `env: Env`
- `admin: Address`

**Returns:** `()`

---

#### `run_migration`

**Signature:**

```rust
pub fn run_migration(env: Env, admin: Address, target_version: u32, dry_run: bool) -> migration::MigrationReport
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `target_version: u32`
- `dry_run: bool`

**Returns:** `migration::MigrationReport`

---

#### `rollback_migration`

**Signature:**

```rust
pub fn rollback_migration(env: Env, admin: Address) -> Option<migration::MigrationReport>
```

**Parameters:**

- `env: Env`
- `admin: Address`

**Returns:** `Option<migration::MigrationReport>`

---

#### `get_health_dashboard`

**Signature:**

```rust
pub fn get_health_dashboard(env: Env) -> monitoring::ContractHealth
```

**Parameters:**

- `env: Env`

**Returns:** `monitoring::ContractHealth`

---

## `migration` Contract

_No contract API functions found._

## `nft-reward` Contract

### `NftReward`

#### `initialize`

Initializes the NFT reward contract with an admin address and optional max supply cap.
Call this once to set the admin who can manage the contract.

**Signature:**

```rust
pub fn initialize(env: Env, admin: Address, max_supply: Option<u64>) -> Result<(), crate::errors::NftErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `max_supply: Option<u64>`

**Returns:** `Result<(), crate::errors::NftErrorCode>`

**Error type:** `NftErrorCode`

**Error codes:**

- `NftNotFound` = 1
- `Unauthorized` = 2
- `NotOwner` = 3
- `InvalidRecipient` = 4
- `SoulboundNft` = 5
- `InvalidRarity` = 6
- `AlreadyInitialized` = 7
- `MaxSupplyReached` = 8
- `NotInitialized` = 9
- `NotOperator` = 10
- `NftNotTransferable` = 11
- `NftLocked` = 12

---

#### `mint_reward_nft`

Mints a unique NFT as a reward for hunt completion.

`minter` must be an authorized minter (and must sign the transaction) when the
contract has been initialized.  Before initialization the check is skipped so
that existing deployments remain functional.

# Arguments
* `minter` - Address performing the mint (must be whitelisted after init)
* `hunt_id` - The hunt this NFT commemorates
* `player_address` - The address of the player completing the hunt (initial owner)
* `metadata` - NFT metadata (title, description, image URI, hunt_title, rarity, tier)

# Returns
The unique NFT ID of the minted NFT

**Signature:**

```rust
pub fn mint_reward_nft(env: Env, _minter: Address, hunt_id: u64, player_address: Address, metadata: NftMetadata) -> u64
```

**Parameters:**

- `env: Env`
- `_minter: Address`
- `hunt_id: u64`
- `player_address: Address`
- `metadata: NftMetadata`

**Returns:** `u64`

---

#### `mint_reward_nft_from_map`

Mints a reward NFT from a generic metadata map. This is the entrypoint
used by cross-contract callers (e.g. RewardManager) that cannot depend
on this crate's `NftMetadata` type directly.

`minter` is the calling contract's address and must be whitelisted when the
contract has been initialized.

Expected keys in `metadata` (all optional, with sensible defaults):
- "title": String
- "description": String
- "image_uri": String
- "hunt_title": String (defaults to title when omitted/empty)
- "rarity": u32
- "tier": u32
- "creator": Address (defaults to player_address if omitted)
- "royalty_bps": u32 (optional, basis points for royalty percentage)
- "transferable": bool

**Signature:**

```rust
pub fn mint_reward_nft_from_map(env: Env, _minter: Address, hunt_id: u64, player_address: Address, metadata: Map<Symbol, Val>) -> u64
```

**Parameters:**

- `env: Env`
- `_minter: Address`
- `hunt_id: u64`
- `player_address: Address`
- `metadata: Map<Symbol, Val>`

**Returns:** `u64`

---

#### `get_nft`

Retrieves NFT data by ID.

**Signature:**

```rust
pub fn get_nft(env: Env, nft_id: u64) -> Option<NftData>
```

**Parameters:**

- `env: Env`
- `nft_id: u64`

**Returns:** `Option<NftData>`

---

#### `get_nft_metadata`

Returns complete metadata for an NFT, including hunt info and completion details.

**Signature:**

```rust
pub fn get_nft_metadata(env: Env, nft_id: u64) -> Option<NftMetadataResponse>
```

**Parameters:**

- `env: Env`
- `nft_id: u64`

**Returns:** `Option<NftMetadataResponse>`

---

#### `get_admin`

Returns the configured admin address, if set.

**Signature:**

```rust
pub fn get_admin(env: Env) -> Option<Address>
```

**Parameters:**

- `env: Env`

**Returns:** `Option<Address>`

---

#### `set_reward_manager`

Sets the RewardManager contract address. Only the admin can call this.

**Signature:**

```rust
pub fn set_reward_manager(env: Env, admin: Address, reward_manager: Address) -> Result<(), crate::errors::NftErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `reward_manager: Address`

**Returns:** `Result<(), crate::errors::NftErrorCode>`

**Error type:** `NftErrorCode`

**Error codes:**

- `NftNotFound` = 1
- `Unauthorized` = 2
- `NotOwner` = 3
- `InvalidRecipient` = 4
- `SoulboundNft` = 5
- `InvalidRarity` = 6
- `AlreadyInitialized` = 7
- `MaxSupplyReached` = 8
- `NotInitialized` = 9
- `NotOperator` = 10
- `NftNotTransferable` = 11
- `NftLocked` = 12

---

#### `admin_update_image_uris`

Batch-updates image URIs for all NFTs whose `image_uri` starts with `old_prefix`,
replacing it with `new_prefix`. Useful for migrating between IPFS gateways or CDNs.

# Authorization
Only the configured admin can call this function.

# Arguments
* `admin` - The admin address (must match the stored admin)
* `old_prefix` - The prefix to match (e.g. "ipfs://oldgateway/")
* `new_prefix` - The replacement prefix (e.g. "ipfs://newgateway/")

# Returns
The number of NFTs whose image URIs were updated.

**Signature:**

```rust
pub fn admin_update_image_uris(env: Env, admin: Address, old_prefix: String, new_prefix: String) -> Result<u32, crate::errors::NftErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `old_prefix: String`
- `new_prefix: String`

**Returns:** `Result<u32, crate::errors::NftErrorCode>`

**Error type:** `NftErrorCode`

**Error codes:**

- `NftNotFound` = 1
- `Unauthorized` = 2
- `NotOwner` = 3
- `InvalidRecipient` = 4
- `SoulboundNft` = 5
- `InvalidRarity` = 6
- `AlreadyInitialized` = 7
- `MaxSupplyReached` = 8
- `NotInitialized` = 9
- `NotOperator` = 10
- `NftNotTransferable` = 11
- `NftLocked` = 12

---

#### `update_nft_metadata`

Updates mutable metadata fields (description, image_uri). Owner only.
Title, hunt info, and attributes remain immutable for collectibility.

**Signature:**

```rust
pub fn update_nft_metadata(env: Env, nft_id: u64, updater: Address, new_description: String, new_image_uri: String) -> Result<(), crate::errors::NftErrorCode>
```

**Parameters:**

- `env: Env`
- `nft_id: u64`
- `updater: Address`
- `new_description: String`
- `new_image_uri: String`

**Returns:** `Result<(), crate::errors::NftErrorCode>`

**Error type:** `NftErrorCode`

**Error codes:**

- `NftNotFound` = 1
- `Unauthorized` = 2
- `NotOwner` = 3
- `InvalidRecipient` = 4
- `SoulboundNft` = 5
- `InvalidRarity` = 6
- `AlreadyInitialized` = 7
- `MaxSupplyReached` = 8
- `NotInitialized` = 9
- `NotOperator` = 10
- `NftNotTransferable` = 11
- `NftLocked` = 12

---

#### `total_supply`

Returns the total number of NFTs minted so far.

**Signature:**

```rust
pub fn total_supply(env: Env) -> u64
```

**Parameters:**

- `env: Env`

**Returns:** `u64`

---

#### `owner_of`

Returns the owner of an NFT.

**Signature:**

```rust
pub fn owner_of(env: Env, nft_id: u64) -> Option<Address>
```

**Parameters:**

- `env: Env`
- `nft_id: u64`

**Returns:** `Option<Address>`

---

#### `get_nft_owner`

Alias for owner_of. Returns the owner of an NFT.

**Signature:**

```rust
pub fn get_nft_owner(env: Env, nft_id: u64) -> Option<Address>
```

**Parameters:**

- `env: Env`
- `nft_id: u64`

**Returns:** `Option<Address>`

---

#### `verify_ownership`

Verifies whether `address` is the current owner of `nft_id`.
Returns `true` when the NFT exists and the stored owner equals `address`.

**Signature:**

```rust
pub fn verify_ownership(env: Env, address: Address, nft_id: u64) -> bool
```

**Parameters:**

- `env: Env`
- `address: Address`
- `nft_id: u64`

**Returns:** `bool`

---

#### `has_hunt_nft`

Returns `true` if `address` owns any NFT minted for `hunt_id`.
Scans the owner's indexed NFT IDs and checks each NFT's `hunt_id`.

**Signature:**

```rust
pub fn has_hunt_nft(env: Env, address: Address, hunt_id: u64) -> bool
```

**Parameters:**

- `env: Env`
- `address: Address`
- `hunt_id: u64`

**Returns:** `bool`

---

#### `get_player_nfts`

Returns paginated NFT IDs owned by an address.

**Signature:**

```rust
pub fn get_player_nfts(env: Env, owner: Address, offset: u32, limit: u32) -> Vec<u64>
```

**Parameters:**

- `env: Env`
- `owner: Address`
- `offset: u32`
- `limit: u32`

**Returns:** `Vec<u64>`

---

#### `get_nfts_by_hunt`

Returns paginated NFT IDs minted for a hunt.

**Signature:**

```rust
pub fn get_nfts_by_hunt(env: Env, hunt_id: u64, offset: u32, limit: u32) -> Vec<u64>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `offset: u32`
- `limit: u32`

**Returns:** `Vec<u64>`

---

#### `get_hunt_nft_count`

Returns the total number of NFTs minted for a hunt.

**Signature:**

```rust
pub fn get_hunt_nft_count(env: Env, hunt_id: u64) -> u32
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`

**Returns:** `u32`

---

#### `burn`

Burns (permanently destroys) an NFT, removing it from storage and the owner's list.

# Authorization
The `owner` must authorize this call. The caller must also be the current owner.

**Signature:**

```rust
pub fn burn(env: Env, nft_id: u64, owner: Address) -> Result<(), crate::errors::NftErrorCode>
```

**Parameters:**

- `env: Env`
- `nft_id: u64`
- `owner: Address`

**Returns:** `Result<(), crate::errors::NftErrorCode>`

**Error type:** `NftErrorCode`

**Error codes:**

- `NftNotFound` = 1
- `Unauthorized` = 2
- `NotOwner` = 3
- `InvalidRecipient` = 4
- `SoulboundNft` = 5
- `InvalidRarity` = 6
- `AlreadyInitialized` = 7
- `MaxSupplyReached` = 8
- `NotInitialized` = 9
- `NotOperator` = 10
- `NftNotTransferable` = 11
- `NftLocked` = 12

---

#### `search_by_title`

Searches NFTs by title (case-insensitive partial match).
Returns a vector of NFT IDs whose titles contain the search query.

**Signature:**

```rust
pub fn search_by_title(env: Env, query: String) -> Vec<u64>
```

**Parameters:**

- `env: Env`
- `query: String`

**Returns:** `Vec<u64>`

---

#### `search_by_hunt_title`

Searches NFTs by hunt title (case-insensitive partial match).
Returns a vector of NFT IDs whose hunt titles contain the search query.

**Signature:**

```rust
pub fn search_by_hunt_title(env: Env, query: String) -> Vec<u64>
```

**Parameters:**

- `env: Env`
- `query: String`

**Returns:** `Vec<u64>`

---

#### `search_by_rarity`

Filters NFTs by rarity tier.
Returns a vector of NFT IDs with the specified rarity.
Rarity tiers: 0 = default, 1 = common, 2 = uncommon, 3 = rare, 4 = epic, 5 = legendary.

**Signature:**

```rust
pub fn search_by_rarity(env: Env, rarity: u32) -> Vec<u64>
```

**Parameters:**

- `env: Env`
- `rarity: u32`

**Returns:** `Vec<u64>`

---

#### `search_by_tier`

Filters NFTs by custom tier.
Returns a vector of NFT IDs with the specified tier.
Tier: 0 = none, other values for custom categories.

**Signature:**

```rust
pub fn search_by_tier(env: Env, tier: u32) -> Vec<u64>
```

**Parameters:**

- `env: Env`
- `tier: u32`

**Returns:** `Vec<u64>`

---

#### `search_nfts`

General search function with multiple metadata filters.
All parameters are optional - NFTs must match all provided filters.

# Arguments
* `title_query` - Optional partial match for NFT title (case-insensitive)
* `hunt_title_query` - Optional partial match for hunt title (case-insensitive)
* `rarity` - Optional rarity filter (exact match)
* `tier` - Optional tier filter (exact match)

# Returns
Vector of NFT IDs matching all provided filters

**Signature:**

```rust
pub fn search_nfts(env: Env, title_query: Option<String>, hunt_title_query: Option<String>, rarity: Option<u32>, tier: Option<u32>) -> Vec<u64>
```

**Parameters:**

- `env: Env`
- `title_query: Option<String>`
- `hunt_title_query: Option<String>`
- `rarity: Option<u32>`
- `tier: Option<u32>`

**Returns:** `Vec<u64>`

---

#### `transfer_nft`

Transfers an NFT from one address to another.

# Arguments
* `nft_id` - The NFT to transfer
* `from_address` - Current owner of the NFT
* `to_address` - New owner
* `caller` - Address authorizing the transfer (must be owner or approved operator)

# Authorization
`caller` must authorize this call. `caller` must be either the current owner
or an operator approved by the owner via `set_operator`.

**Signature:**

```rust
pub fn transfer_nft(env: Env, nft_id: u64, from_address: Address, to_address: Address, caller: Address) -> Result<(), crate::errors::NftErrorCode>
```

**Parameters:**

- `env: Env`
- `nft_id: u64`
- `from_address: Address`
- `to_address: Address`
- `caller: Address`

**Returns:** `Result<(), crate::errors::NftErrorCode>`

**Error type:** `NftErrorCode`

**Error codes:**

- `NftNotFound` = 1
- `Unauthorized` = 2
- `NotOwner` = 3
- `InvalidRecipient` = 4
- `SoulboundNft` = 5
- `InvalidRarity` = 6
- `AlreadyInitialized` = 7
- `MaxSupplyReached` = 8
- `NotInitialized` = 9
- `NotOperator` = 10
- `NftNotTransferable` = 11
- `NftLocked` = 12

---

#### `contract_version`

Returns the on-chain version stored during initialize, or the compiled constant.

**Signature:**

```rust
pub fn contract_version(env: Env) -> u32
```

**Parameters:**

- `env: Env`

**Returns:** `u32`

---

#### `set_operator`

Grants `operator` the ability to manage all NFTs owned by `owner`.

# Authorization
`owner` must authorize this call.

**Signature:**

```rust
pub fn set_operator(env: Env, owner: Address, operator: Address) -> ()
```

**Parameters:**

- `env: Env`
- `owner: Address`
- `operator: Address`

**Returns:** `()`

---

#### `remove_operator`

Revokes operator approval for `operator` over `owner`'s NFTs.

# Authorization
`owner` must authorize this call.

**Signature:**

```rust
pub fn remove_operator(env: Env, owner: Address, operator: Address) -> ()
```

**Parameters:**

- `env: Env`
- `owner: Address`
- `operator: Address`

**Returns:** `()`

---

#### `is_operator`

Returns true if `operator` is approved to manage all NFTs of `owner`.

**Signature:**

```rust
pub fn is_operator(env: Env, owner: Address, operator: Address) -> bool
```

**Parameters:**

- `env: Env`
- `owner: Address`
- `operator: Address`

**Returns:** `bool`

---

#### `get_schema_version`

**Signature:**

```rust
pub fn get_schema_version(env: Env) -> u32
```

**Parameters:**

- `env: Env`

**Returns:** `u32`

---

#### `initialize_schema`

**Signature:**

```rust
pub fn initialize_schema(env: Env, admin: Address) -> ()
```

**Parameters:**

- `env: Env`
- `admin: Address`

**Returns:** `()`

---

#### `propose_upgrade`

**Signature:**

```rust
pub fn propose_upgrade(env: Env, admin: Address, target_version: u32) -> Result<hunty_migration::UpgradeProposal, hunty_migration::UpgradeAuthError>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `target_version: u32`

**Returns:** `Result<hunty_migration::UpgradeProposal, hunty_migration::UpgradeAuthError>`

**Error type:** `UpgradeAuthError`

**Error codes:**

- `Unauthorized` = 1
- `NoProposal` = 2
- `TimelockPending` = 3
- `VersionMismatch` = 4
- `InvalidTimelock` = 5

---

#### `set_upgrade_timelock`

**Signature:**

```rust
pub fn set_upgrade_timelock(env: Env, admin: Address, delay_seconds: u64) -> Result<(), hunty_migration::UpgradeAuthError>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `delay_seconds: u64`

**Returns:** `Result<(), hunty_migration::UpgradeAuthError>`

**Error type:** `UpgradeAuthError`

**Error codes:**

- `Unauthorized` = 1
- `NoProposal` = 2
- `TimelockPending` = 3
- `VersionMismatch` = 4
- `InvalidTimelock` = 5

---

#### `get_upgrade_proposal`

**Signature:**

```rust
pub fn get_upgrade_proposal(env: Env) -> Option<hunty_migration::UpgradeProposal>
```

**Parameters:**

- `env: Env`

**Returns:** `Option<hunty_migration::UpgradeProposal>`

---

#### `get_upgrade_timelock`

**Signature:**

```rust
pub fn get_upgrade_timelock(env: Env) -> u64
```

**Parameters:**

- `env: Env`

**Returns:** `u64`

---

#### `get_upgrade_history`

**Signature:**

```rust
pub fn get_upgrade_history(env: Env, offset: u32, limit: u32) -> soroban_sdk::Vec<hunty_migration::UpgradeHistoryEntry>
```

**Parameters:**

- `env: Env`
- `offset: u32`
- `limit: u32`

**Returns:** `soroban_sdk::Vec<hunty_migration::UpgradeHistoryEntry>`

---

#### `run_migration`

**Signature:**

```rust
pub fn run_migration(env: Env, admin: Address, target_version: u32, dry_run: bool) -> Result<migration::MigrationReport, hunty_migration::UpgradeAuthError>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `target_version: u32`
- `dry_run: bool`

**Returns:** `Result<migration::MigrationReport, hunty_migration::UpgradeAuthError>`

**Error type:** `UpgradeAuthError`

**Error codes:**

- `Unauthorized` = 1
- `NoProposal` = 2
- `TimelockPending` = 3
- `VersionMismatch` = 4
- `InvalidTimelock` = 5

---

#### `rollback_migration`

**Signature:**

```rust
pub fn rollback_migration(env: Env, admin: Address) -> Result<migration::MigrationReport, hunty_migration::UpgradeAuthError>
```

**Parameters:**

- `env: Env`
- `admin: Address`

**Returns:** `Result<migration::MigrationReport, hunty_migration::UpgradeAuthError>`

**Error type:** `UpgradeAuthError`

**Error codes:**

- `Unauthorized` = 1
- `NoProposal` = 2
- `TimelockPending` = 3
- `VersionMismatch` = 4
- `InvalidTimelock` = 5

---

#### `search_by_hunt_id`

Searches NFTs by hunt_id using the hunt collection index.

**Signature:**

```rust
pub fn search_by_hunt_id(env: Env, hunt_id: u64) -> Vec<u64>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`

**Returns:** `Vec<u64>`

---

#### `search_by_rarity_range`

Searches NFTs by rarity range (inclusive).

**Signature:**

```rust
pub fn search_by_rarity_range(env: Env, min_rarity: u32, max_rarity: u32) -> Vec<u64>
```

**Parameters:**

- `env: Env`
- `min_rarity: u32`
- `max_rarity: u32`

**Returns:** `Vec<u64>`

---

#### `lock_nft`

Locks an NFT to prevent transfers. Only authorized contracts can lock NFTs.

# Arguments
* `nft_id` - The NFT to lock
* `locker` - The authorized contract locking the NFT (must be whitelisted)

# Authorization
The `locker` must be an authorized locker contract and must authorize this call.

**Signature:**

```rust
pub fn lock_nft(env: Env, nft_id: u64, locker: Address) -> Result<(), crate::errors::NftErrorCode>
```

**Parameters:**

- `env: Env`
- `nft_id: u64`
- `locker: Address`

**Returns:** `Result<(), crate::errors::NftErrorCode>`

**Error type:** `NftErrorCode`

**Error codes:**

- `NftNotFound` = 1
- `Unauthorized` = 2
- `NotOwner` = 3
- `InvalidRecipient` = 4
- `SoulboundNft` = 5
- `InvalidRarity` = 6
- `AlreadyInitialized` = 7
- `MaxSupplyReached` = 8
- `NotInitialized` = 9
- `NotOperator` = 10
- `NftNotTransferable` = 11
- `NftLocked` = 12

---

#### `unlock_nft`

Unlocks an NFT to allow transfers. Only authorized contracts can unlock NFTs.

# Arguments
* `nft_id` - The NFT to unlock
* `locker` - The authorized contract unlocking the NFT (must be whitelisted)

# Authorization
The `locker` must be an authorized locker contract and must authorize this call.

**Signature:**

```rust
pub fn unlock_nft(env: Env, nft_id: u64, locker: Address) -> Result<(), crate::errors::NftErrorCode>
```

**Parameters:**

- `env: Env`
- `nft_id: u64`
- `locker: Address`

**Returns:** `Result<(), crate::errors::NftErrorCode>`

**Error type:** `NftErrorCode`

**Error codes:**

- `NftNotFound` = 1
- `Unauthorized` = 2
- `NotOwner` = 3
- `InvalidRecipient` = 4
- `SoulboundNft` = 5
- `InvalidRarity` = 6
- `AlreadyInitialized` = 7
- `MaxSupplyReached` = 8
- `NotInitialized` = 9
- `NotOperator` = 10
- `NftNotTransferable` = 11
- `NftLocked` = 12

---

#### `add_locker`

Adds an authorized locker contract. Admin only.

# Arguments
* `admin` - The admin address (must authorize)
* `locker` - The contract address to authorize for locking/unlocking NFTs

**Signature:**

```rust
pub fn add_locker(env: Env, admin: Address, locker: Address) -> Result<(), crate::errors::NftErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `locker: Address`

**Returns:** `Result<(), crate::errors::NftErrorCode>`

**Error type:** `NftErrorCode`

**Error codes:**

- `NftNotFound` = 1
- `Unauthorized` = 2
- `NotOwner` = 3
- `InvalidRecipient` = 4
- `SoulboundNft` = 5
- `InvalidRarity` = 6
- `AlreadyInitialized` = 7
- `MaxSupplyReached` = 8
- `NotInitialized` = 9
- `NotOperator` = 10
- `NftNotTransferable` = 11
- `NftLocked` = 12

---

#### `remove_locker`

Removes an authorized locker contract. Admin only.

# Arguments
* `admin` - The admin address (must authorize)
* `locker` - The contract address to remove authorization from

**Signature:**

```rust
pub fn remove_locker(env: Env, admin: Address, locker: Address) -> Result<(), crate::errors::NftErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `locker: Address`

**Returns:** `Result<(), crate::errors::NftErrorCode>`

**Error type:** `NftErrorCode`

**Error codes:**

- `NftNotFound` = 1
- `Unauthorized` = 2
- `NotOwner` = 3
- `InvalidRecipient` = 4
- `SoulboundNft` = 5
- `InvalidRarity` = 6
- `AlreadyInitialized` = 7
- `MaxSupplyReached` = 8
- `NotInitialized` = 9
- `NotOperator` = 10
- `NftNotTransferable` = 11
- `NftLocked` = 12

---

## `reward-interface` Contract

_No contract API functions found._

## `reward-manager` Contract

### `RewardManager`

#### `initialize`

Current semantic version of this contract.
Minimum NftReward version this contract requires.
Initializes the RewardManager with the XLM token contract address (SAC).
Must be called once before any reward distribution.

**Signature:**

```rust
pub fn initialize(env: Env, admin: Address, xlm_token: Address) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `xlm_token: Address`

**Returns:** `Result<(), RewardErrorCode>`

**Error type:** `RewardErrorCode`

**Error codes:**

- `NotInitialized` = 1
- `InsufficientPool` = 2
- `AlreadyDistributed` = 3
- `TransferFailed` = 4
- `InvalidAmount` = 5
- `InvalidConfig` = 6
- `NftMintFailed` = 7
- `PoolAlreadyExists` = 8
- `PoolNotFound` = 9
- `Unauthorized` = 10
- `BelowMinimumAmount` = 11
- `AlreadyInitialized` = 12
- `HuntNotFound` = 13
- `ReentrancyDetected` = 14 - A recursive distribution attempt was detected during an external XLM or NFT call.
- `PoolBalanceDivergence` = 15 - The tracked pool balance diverged from the actual XLM token balance.
- `PoolBalanceOverflow` = 16 - Pool balance would overflow if this funding amount is added (pool balance limit exceeded).
- `BelowMinimumFunding` = 17 - Funding amount is below the minimum required (dust attack prevention).
- `ExceedsMaximumFunding` = 18 - Funding amount exceeds the maximum single funding limit.
- `DailyCapExceeded` = 19 - Daily distribution cap for a specific pool has been exceeded.
- `GlobalDailyCapExceeded` = 20 - Global daily distribution cap has been exceeded.
- `ContractPaused` = 21 - Contract is paused and cannot perform operations.
- `EmergencyWithdrawalFailed` = 22 - Emergency withdrawal failed.

---

#### `set_nft_reward_contract`

Sets the default NftReward contract address used for NFT distributions
when a per-call NFT contract is not provided.
Emits an NftContractSetEvent with the old and new contract addresses.

**Signature:**

```rust
pub fn set_nft_reward_contract(env: Env, admin: Address, nft_contract: Address) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `nft_contract: Address`

**Returns:** `Result<(), RewardErrorCode>`

**Error type:** `RewardErrorCode`

**Error codes:**

- `NotInitialized` = 1
- `InsufficientPool` = 2
- `AlreadyDistributed` = 3
- `TransferFailed` = 4
- `InvalidAmount` = 5
- `InvalidConfig` = 6
- `NftMintFailed` = 7
- `PoolAlreadyExists` = 8
- `PoolNotFound` = 9
- `Unauthorized` = 10
- `BelowMinimumAmount` = 11
- `AlreadyInitialized` = 12
- `HuntNotFound` = 13
- `ReentrancyDetected` = 14 - A recursive distribution attempt was detected during an external XLM or NFT call.
- `PoolBalanceDivergence` = 15 - The tracked pool balance diverged from the actual XLM token balance.
- `PoolBalanceOverflow` = 16 - Pool balance would overflow if this funding amount is added (pool balance limit exceeded).
- `BelowMinimumFunding` = 17 - Funding amount is below the minimum required (dust attack prevention).
- `ExceedsMaximumFunding` = 18 - Funding amount exceeds the maximum single funding limit.
- `DailyCapExceeded` = 19 - Daily distribution cap for a specific pool has been exceeded.
- `GlobalDailyCapExceeded` = 20 - Global daily distribution cap has been exceeded.
- `ContractPaused` = 21 - Contract is paused and cannot perform operations.
- `EmergencyWithdrawalFailed` = 22 - Emergency withdrawal failed.

---

#### `set_hunty_core`

Sets the optional HuntyCore contract address used to validate hunt_id existence
in `create_reward_pool`. When set, pool creation will be rejected for unknown
hunt IDs. If not set, hunt_id is assumed caller-trusted.

**Signature:**

```rust
pub fn set_hunty_core(env: Env, admin: Address, hunty_core: Address) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `hunty_core: Address`

**Returns:** `Result<(), RewardErrorCode>`

**Error type:** `RewardErrorCode`

**Error codes:**

- `NotInitialized` = 1
- `InsufficientPool` = 2
- `AlreadyDistributed` = 3
- `TransferFailed` = 4
- `InvalidAmount` = 5
- `InvalidConfig` = 6
- `NftMintFailed` = 7
- `PoolAlreadyExists` = 8
- `PoolNotFound` = 9
- `Unauthorized` = 10
- `BelowMinimumAmount` = 11
- `AlreadyInitialized` = 12
- `HuntNotFound` = 13
- `ReentrancyDetected` = 14 - A recursive distribution attempt was detected during an external XLM or NFT call.
- `PoolBalanceDivergence` = 15 - The tracked pool balance diverged from the actual XLM token balance.
- `PoolBalanceOverflow` = 16 - Pool balance would overflow if this funding amount is added (pool balance limit exceeded).
- `BelowMinimumFunding` = 17 - Funding amount is below the minimum required (dust attack prevention).
- `ExceedsMaximumFunding` = 18 - Funding amount exceeds the maximum single funding limit.
- `DailyCapExceeded` = 19 - Daily distribution cap for a specific pool has been exceeded.
- `GlobalDailyCapExceeded` = 20 - Global daily distribution cap has been exceeded.
- `ContractPaused` = 21 - Contract is paused and cannot perform operations.
- `EmergencyWithdrawalFailed` = 22 - Emergency withdrawal failed.

---

#### `create_reward_pool`

Creates a reward pool for a specific hunt.

Must be called before `fund_reward_pool`. Only the creator is authorized
to fund the pool after creation.

# Arguments
* `creator` - The hunt creator who will own and fund the pool
* `hunt_id` - The hunt this pool is for
* `min_distribution_amount` - Minimum XLM per distribution (0 = no minimum)

# Errors
* `PoolAlreadyExists` - A pool already exists for this hunt_id
* `InvalidAmount` - min_distribution_amount is negative
* `HuntNotFound` - hunt_id does not exist in HuntyCore (only when `set_hunty_core` has been called)

**Signature:**

```rust
pub fn create_reward_pool(env: Env, creator: Address, hunt_id: u64, min_distribution_amount: i128) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `creator: Address`
- `hunt_id: u64`
- `min_distribution_amount: i128`

**Returns:** `Result<(), RewardErrorCode>`

**Error type:** `RewardErrorCode`

**Error codes:**

- `NotInitialized` = 1
- `InsufficientPool` = 2
- `AlreadyDistributed` = 3
- `TransferFailed` = 4
- `InvalidAmount` = 5
- `InvalidConfig` = 6
- `NftMintFailed` = 7
- `PoolAlreadyExists` = 8
- `PoolNotFound` = 9
- `Unauthorized` = 10
- `BelowMinimumAmount` = 11
- `AlreadyInitialized` = 12
- `HuntNotFound` = 13
- `ReentrancyDetected` = 14 - A recursive distribution attempt was detected during an external XLM or NFT call.
- `PoolBalanceDivergence` = 15 - The tracked pool balance diverged from the actual XLM token balance.
- `PoolBalanceOverflow` = 16 - Pool balance would overflow if this funding amount is added (pool balance limit exceeded).
- `BelowMinimumFunding` = 17 - Funding amount is below the minimum required (dust attack prevention).
- `ExceedsMaximumFunding` = 18 - Funding amount exceeds the maximum single funding limit.
- `DailyCapExceeded` = 19 - Daily distribution cap for a specific pool has been exceeded.
- `GlobalDailyCapExceeded` = 20 - Global daily distribution cap has been exceeded.
- `ContractPaused` = 21 - Contract is paused and cannot perform operations.
- `EmergencyWithdrawalFailed` = 22 - Emergency withdrawal failed.

---

#### `update_pool_config`

Updates the `min_distribution_amount` for an existing reward pool.

Only the pool creator is authorized to call this. Useful when a creator
has underfunded the pool and needs to lower the minimum so distributions
can proceed.

# Arguments
* `creator` - The pool creator (must match the stored creator)
* `hunt_id` - The hunt whose pool config to update
* `min_distribution_amount` - New minimum XLM per distribution (0 = no minimum)

# Errors
* `PoolNotFound` - No pool exists for this hunt_id
* `Unauthorized` - Caller is not the pool creator
* `InvalidAmount` - min_distribution_amount is negative

**Signature:**

```rust
pub fn update_pool_config(env: Env, creator: Address, hunt_id: u64, min_distribution_amount: i128) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `creator: Address`
- `hunt_id: u64`
- `min_distribution_amount: i128`

**Returns:** `Result<(), RewardErrorCode>`

**Error type:** `RewardErrorCode`

**Error codes:**

- `NotInitialized` = 1
- `InsufficientPool` = 2
- `AlreadyDistributed` = 3
- `TransferFailed` = 4
- `InvalidAmount` = 5
- `InvalidConfig` = 6
- `NftMintFailed` = 7
- `PoolAlreadyExists` = 8
- `PoolNotFound` = 9
- `Unauthorized` = 10
- `BelowMinimumAmount` = 11
- `AlreadyInitialized` = 12
- `HuntNotFound` = 13
- `ReentrancyDetected` = 14 - A recursive distribution attempt was detected during an external XLM or NFT call.
- `PoolBalanceDivergence` = 15 - The tracked pool balance diverged from the actual XLM token balance.
- `PoolBalanceOverflow` = 16 - Pool balance would overflow if this funding amount is added (pool balance limit exceeded).
- `BelowMinimumFunding` = 17 - Funding amount is below the minimum required (dust attack prevention).
- `ExceedsMaximumFunding` = 18 - Funding amount exceeds the maximum single funding limit.
- `DailyCapExceeded` = 19 - Daily distribution cap for a specific pool has been exceeded.
- `GlobalDailyCapExceeded` = 20 - Global daily distribution cap has been exceeded.
- `ContractPaused` = 21 - Contract is paused and cannot perform operations.
- `EmergencyWithdrawalFailed` = 22 - Emergency withdrawal failed.

---

#### `fund_reward_pool`

Funds the reward pool for a specific hunt.

The pool must have been created via `create_reward_pool` first.
Only the original pool creator is authorized to fund it.
Transfers XLM from the funder to this contract and records the balance.

# Validation
- Minimum funding: 1 XLM (10,000,000 stroops) to prevent dust attacks
- Maximum single funding: 1 billion XLM to prevent overflow
- Pool balance limit: 1 billion XLM total to prevent overflow
- Rejects zero or negative amounts

# Arguments
* `funder` - The address funding the pool (must be the pool creator)
* `hunt_id` - The hunt to fund
* `amount` - XLM amount to add to the pool (must be > 0)

# Errors
* `PoolNotFound` - Pool has not been created yet
* `Unauthorized` - Funder is not the pool creator
* `InvalidAmount` - Amount is <= 0
* `BelowMinimumFunding` - Amount is less than 1 XLM (dust attack prevention)
* `ExceedsMaximumFunding` - Amount exceeds 1 billion XLM
* `PoolBalanceOverflow` - Adding this amount would exceed pool balance limit
* `NotInitialized` - XLM token address not set

**Signature:**

```rust
pub fn fund_reward_pool(env: Env, funder: Address, hunt_id: u64, amount: i128) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `funder: Address`
- `hunt_id: u64`
- `amount: i128`

**Returns:** `Result<(), RewardErrorCode>`

**Error type:** `RewardErrorCode`

**Error codes:**

- `NotInitialized` = 1
- `InsufficientPool` = 2
- `AlreadyDistributed` = 3
- `TransferFailed` = 4
- `InvalidAmount` = 5
- `InvalidConfig` = 6
- `NftMintFailed` = 7
- `PoolAlreadyExists` = 8
- `PoolNotFound` = 9
- `Unauthorized` = 10
- `BelowMinimumAmount` = 11
- `AlreadyInitialized` = 12
- `HuntNotFound` = 13
- `ReentrancyDetected` = 14 - A recursive distribution attempt was detected during an external XLM or NFT call.
- `PoolBalanceDivergence` = 15 - The tracked pool balance diverged from the actual XLM token balance.
- `PoolBalanceOverflow` = 16 - Pool balance would overflow if this funding amount is added (pool balance limit exceeded).
- `BelowMinimumFunding` = 17 - Funding amount is below the minimum required (dust attack prevention).
- `ExceedsMaximumFunding` = 18 - Funding amount exceeds the maximum single funding limit.
- `DailyCapExceeded` = 19 - Daily distribution cap for a specific pool has been exceeded.
- `GlobalDailyCapExceeded` = 20 - Global daily distribution cap has been exceeded.
- `ContractPaused` = 21 - Contract is paused and cannot perform operations.
- `EmergencyWithdrawalFailed` = 22 - Emergency withdrawal failed.

---

#### `refund_pool`

Refunds the entire remaining pool balance for a hunt back to the pool creator.
Can only be called by the same creator that owns the pool.

**Signature:**

```rust
pub fn refund_pool(env: Env, creator: Address, hunt_id: u64) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `creator: Address`
- `hunt_id: u64`

**Returns:** `Result<(), RewardErrorCode>`

**Error type:** `RewardErrorCode`

**Error codes:**

- `NotInitialized` = 1
- `InsufficientPool` = 2
- `AlreadyDistributed` = 3
- `TransferFailed` = 4
- `InvalidAmount` = 5
- `InvalidConfig` = 6
- `NftMintFailed` = 7
- `PoolAlreadyExists` = 8
- `PoolNotFound` = 9
- `Unauthorized` = 10
- `BelowMinimumAmount` = 11
- `AlreadyInitialized` = 12
- `HuntNotFound` = 13
- `ReentrancyDetected` = 14 - A recursive distribution attempt was detected during an external XLM or NFT call.
- `PoolBalanceDivergence` = 15 - The tracked pool balance diverged from the actual XLM token balance.
- `PoolBalanceOverflow` = 16 - Pool balance would overflow if this funding amount is added (pool balance limit exceeded).
- `BelowMinimumFunding` = 17 - Funding amount is below the minimum required (dust attack prevention).
- `ExceedsMaximumFunding` = 18 - Funding amount exceeds the maximum single funding limit.
- `DailyCapExceeded` = 19 - Daily distribution cap for a specific pool has been exceeded.
- `GlobalDailyCapExceeded` = 20 - Global daily distribution cap has been exceeded.
- `ContractPaused` = 21 - Contract is paused and cannot perform operations.
- `EmergencyWithdrawalFailed` = 22 - Emergency withdrawal failed.

---

#### `get_reward_pool`

Returns the full status of a reward pool, including balance, totals, and configuration.
Returns None if no pool has been created for the given hunt_id.

**Signature:**

```rust
pub fn get_reward_pool(env: Env, hunt_id: u64) -> Option<RewardPoolStatus>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`

**Returns:** `Option<RewardPoolStatus>`

---

#### `validate_pool`

Validates whether a pool can cover a given distribution amount.

Checks that:
- The pool exists (was created via create_reward_pool)
- The required_amount is positive
- The pool balance >= required_amount
- The required_amount meets the pool's minimum distribution threshold (if set)

Returns a `ValidationResult` with balance details regardless of validity,
so callers can diagnose shortfalls without a separate query.

**Signature:**

```rust
pub fn validate_pool(env: Env, hunt_id: u64, required_amount: i128) -> ValidationResult
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `required_amount: i128`

**Returns:** `ValidationResult`

---

#### `set_daily_pool_cap`

**Signature:**

```rust
pub fn set_daily_pool_cap(env: Env, admin: Address, hunt_id: u64, cap: i128) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `hunt_id: u64`
- `cap: i128`

**Returns:** `Result<(), RewardErrorCode>`

**Error type:** `RewardErrorCode`

**Error codes:**

- `NotInitialized` = 1
- `InsufficientPool` = 2
- `AlreadyDistributed` = 3
- `TransferFailed` = 4
- `InvalidAmount` = 5
- `InvalidConfig` = 6
- `NftMintFailed` = 7
- `PoolAlreadyExists` = 8
- `PoolNotFound` = 9
- `Unauthorized` = 10
- `BelowMinimumAmount` = 11
- `AlreadyInitialized` = 12
- `HuntNotFound` = 13
- `ReentrancyDetected` = 14 - A recursive distribution attempt was detected during an external XLM or NFT call.
- `PoolBalanceDivergence` = 15 - The tracked pool balance diverged from the actual XLM token balance.
- `PoolBalanceOverflow` = 16 - Pool balance would overflow if this funding amount is added (pool balance limit exceeded).
- `BelowMinimumFunding` = 17 - Funding amount is below the minimum required (dust attack prevention).
- `ExceedsMaximumFunding` = 18 - Funding amount exceeds the maximum single funding limit.
- `DailyCapExceeded` = 19 - Daily distribution cap for a specific pool has been exceeded.
- `GlobalDailyCapExceeded` = 20 - Global daily distribution cap has been exceeded.
- `ContractPaused` = 21 - Contract is paused and cannot perform operations.
- `EmergencyWithdrawalFailed` = 22 - Emergency withdrawal failed.

---

#### `set_daily_global_cap`

**Signature:**

```rust
pub fn set_daily_global_cap(env: Env, admin: Address, cap: i128) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `cap: i128`

**Returns:** `Result<(), RewardErrorCode>`

**Error type:** `RewardErrorCode`

**Error codes:**

- `NotInitialized` = 1
- `InsufficientPool` = 2
- `AlreadyDistributed` = 3
- `TransferFailed` = 4
- `InvalidAmount` = 5
- `InvalidConfig` = 6
- `NftMintFailed` = 7
- `PoolAlreadyExists` = 8
- `PoolNotFound` = 9
- `Unauthorized` = 10
- `BelowMinimumAmount` = 11
- `AlreadyInitialized` = 12
- `HuntNotFound` = 13
- `ReentrancyDetected` = 14 - A recursive distribution attempt was detected during an external XLM or NFT call.
- `PoolBalanceDivergence` = 15 - The tracked pool balance diverged from the actual XLM token balance.
- `PoolBalanceOverflow` = 16 - Pool balance would overflow if this funding amount is added (pool balance limit exceeded).
- `BelowMinimumFunding` = 17 - Funding amount is below the minimum required (dust attack prevention).
- `ExceedsMaximumFunding` = 18 - Funding amount exceeds the maximum single funding limit.
- `DailyCapExceeded` = 19 - Daily distribution cap for a specific pool has been exceeded.
- `GlobalDailyCapExceeded` = 20 - Global daily distribution cap has been exceeded.
- `ContractPaused` = 21 - Contract is paused and cannot perform operations.
- `EmergencyWithdrawalFailed` = 22 - Emergency withdrawal failed.

---

#### `distribute_rewards`

**Signature:**

```rust
pub fn distribute_rewards(env: Env, hunt_id: u64, player_address: Address, reward_config: RewardConfig) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `player_address: Address`
- `reward_config: RewardConfig`

**Returns:** `Result<(), RewardErrorCode>`

**Error type:** `RewardErrorCode`

**Error codes:**

- `NotInitialized` = 1
- `InsufficientPool` = 2
- `AlreadyDistributed` = 3
- `TransferFailed` = 4
- `InvalidAmount` = 5
- `InvalidConfig` = 6
- `NftMintFailed` = 7
- `PoolAlreadyExists` = 8
- `PoolNotFound` = 9
- `Unauthorized` = 10
- `BelowMinimumAmount` = 11
- `AlreadyInitialized` = 12
- `HuntNotFound` = 13
- `ReentrancyDetected` = 14 - A recursive distribution attempt was detected during an external XLM or NFT call.
- `PoolBalanceDivergence` = 15 - The tracked pool balance diverged from the actual XLM token balance.
- `PoolBalanceOverflow` = 16 - Pool balance would overflow if this funding amount is added (pool balance limit exceeded).
- `BelowMinimumFunding` = 17 - Funding amount is below the minimum required (dust attack prevention).
- `ExceedsMaximumFunding` = 18 - Funding amount exceeds the maximum single funding limit.
- `DailyCapExceeded` = 19 - Daily distribution cap for a specific pool has been exceeded.
- `GlobalDailyCapExceeded` = 20 - Global daily distribution cap has been exceeded.
- `ContractPaused` = 21 - Contract is paused and cannot perform operations.
- `EmergencyWithdrawalFailed` = 22 - Emergency withdrawal failed.

---

#### `get_total_xlm_distributed`

Returns the total XLM distributed across all hunts (protocol-level metric).

**Signature:**

```rust
pub fn get_total_xlm_distributed(env: Env) -> i128
```

**Parameters:**

- `env: Env`

**Returns:** `i128`

---

#### `distribute_rewards_legacy`

Legacy entry point for XLM-only distribution.
Kept for backward compatibility with HuntyCore. For NFT or full config support use distribute_rewards.

Note: `nft_enabled` is ignored — NFT distribution requires metadata and a contract address
that are not available on this path. Use `distribute_rewards` with `RewardConfig` instead.

**Signature:**

```rust
pub fn distribute_rewards_legacy(env: Env, player: Address, hunt_id: u64, xlm_amount: i128, _nft_enabled: bool, // ignored: NFT not supported on legacy path) -> bool
```

**Parameters:**

- `env: Env`
- `player: Address`
- `hunt_id: u64`
- `xlm_amount: i128`
- `_nft_enabled: bool`
- `// ignored: NFT not supported on legacy path`

**Returns:** `bool`

---

#### `get_distribution_status`

Returns the distribution status for a hunt/player pair.

**Signature:**

```rust
pub fn get_distribution_status(env: Env, hunt_id: u64, player: Address) -> DistributionStatus
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `player: Address`

**Returns:** `DistributionStatus`

---

#### `get_pool_balance`

Returns the current reward pool balance for a hunt.

**Signature:**

```rust
pub fn get_pool_balance(env: Env, hunt_id: u64) -> i128
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`

**Returns:** `i128`

---

#### `is_reward_distributed`

Returns whether a reward has been distributed to a player for a hunt.

**Signature:**

```rust
pub fn is_reward_distributed(env: Env, hunt_id: u64, player: Address) -> bool
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `player: Address`

**Returns:** `bool`

---

#### `admin_withdraw_unclaimed`

Allows the admin to withdraw any unclaimed (surplus) XLM remaining in a reward pool.

This is needed when a hunt concludes with fewer winners than anticipated,
leaving unspent XLM locked in the pool. Only the contract admin may call this.

# Arguments
* `admin` - The contract admin address (must match the stored admin)
* `hunt_id` - The hunt whose remaining pool balance to withdraw
* `recipient` - The address that will receive the withdrawn XLM

# Errors
* `NotInitialized` - Contract has not been initialized (no admin set)
* `Unauthorized` - Caller is not the contract admin
* `PoolNotFound` - No pool exists for this hunt_id
* `InvalidAmount` - Pool balance is zero (nothing to withdraw)

**Signature:**

```rust
pub fn admin_withdraw_unclaimed(env: Env, admin: Address, hunt_id: u64, recipient: Address, amount: i128) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `hunt_id: u64`
- `recipient: Address`
- `amount: i128`

**Returns:** `Result<(), RewardErrorCode>`

**Error type:** `RewardErrorCode`

**Error codes:**

- `NotInitialized` = 1
- `InsufficientPool` = 2
- `AlreadyDistributed` = 3
- `TransferFailed` = 4
- `InvalidAmount` = 5
- `InvalidConfig` = 6
- `NftMintFailed` = 7
- `PoolAlreadyExists` = 8
- `PoolNotFound` = 9
- `Unauthorized` = 10
- `BelowMinimumAmount` = 11
- `AlreadyInitialized` = 12
- `HuntNotFound` = 13
- `ReentrancyDetected` = 14 - A recursive distribution attempt was detected during an external XLM or NFT call.
- `PoolBalanceDivergence` = 15 - The tracked pool balance diverged from the actual XLM token balance.
- `PoolBalanceOverflow` = 16 - Pool balance would overflow if this funding amount is added (pool balance limit exceeded).
- `BelowMinimumFunding` = 17 - Funding amount is below the minimum required (dust attack prevention).
- `ExceedsMaximumFunding` = 18 - Funding amount exceeds the maximum single funding limit.
- `DailyCapExceeded` = 19 - Daily distribution cap for a specific pool has been exceeded.
- `GlobalDailyCapExceeded` = 20 - Global daily distribution cap has been exceeded.
- `ContractPaused` = 21 - Contract is paused and cannot perform operations.
- `EmergencyWithdrawalFailed` = 22 - Emergency withdrawal failed.

---

#### `pause`

Pauses the contract, preventing reward distributions and withdrawals.
Only the contract admin can call this. Emits an emergency event.

**Signature:**

```rust
pub fn pause(env: Env, admin: Address, reason: soroban_sdk::String) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `reason: soroban_sdk::String`

**Returns:** `Result<(), RewardErrorCode>`

**Error type:** `RewardErrorCode`

**Error codes:**

- `NotInitialized` = 1
- `InsufficientPool` = 2
- `AlreadyDistributed` = 3
- `TransferFailed` = 4
- `InvalidAmount` = 5
- `InvalidConfig` = 6
- `NftMintFailed` = 7
- `PoolAlreadyExists` = 8
- `PoolNotFound` = 9
- `Unauthorized` = 10
- `BelowMinimumAmount` = 11
- `AlreadyInitialized` = 12
- `HuntNotFound` = 13
- `ReentrancyDetected` = 14 - A recursive distribution attempt was detected during an external XLM or NFT call.
- `PoolBalanceDivergence` = 15 - The tracked pool balance diverged from the actual XLM token balance.
- `PoolBalanceOverflow` = 16 - Pool balance would overflow if this funding amount is added (pool balance limit exceeded).
- `BelowMinimumFunding` = 17 - Funding amount is below the minimum required (dust attack prevention).
- `ExceedsMaximumFunding` = 18 - Funding amount exceeds the maximum single funding limit.
- `DailyCapExceeded` = 19 - Daily distribution cap for a specific pool has been exceeded.
- `GlobalDailyCapExceeded` = 20 - Global daily distribution cap has been exceeded.
- `ContractPaused` = 21 - Contract is paused and cannot perform operations.
- `EmergencyWithdrawalFailed` = 22 - Emergency withdrawal failed.

---

#### `unpause`

Unpauses the contract, resuming normal operations.
Only the contract admin can call this.

**Signature:**

```rust
pub fn unpause(env: Env, admin: Address) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`

**Returns:** `Result<(), RewardErrorCode>`

**Error type:** `RewardErrorCode`

**Error codes:**

- `NotInitialized` = 1
- `InsufficientPool` = 2
- `AlreadyDistributed` = 3
- `TransferFailed` = 4
- `InvalidAmount` = 5
- `InvalidConfig` = 6
- `NftMintFailed` = 7
- `PoolAlreadyExists` = 8
- `PoolNotFound` = 9
- `Unauthorized` = 10
- `BelowMinimumAmount` = 11
- `AlreadyInitialized` = 12
- `HuntNotFound` = 13
- `ReentrancyDetected` = 14 - A recursive distribution attempt was detected during an external XLM or NFT call.
- `PoolBalanceDivergence` = 15 - The tracked pool balance diverged from the actual XLM token balance.
- `PoolBalanceOverflow` = 16 - Pool balance would overflow if this funding amount is added (pool balance limit exceeded).
- `BelowMinimumFunding` = 17 - Funding amount is below the minimum required (dust attack prevention).
- `ExceedsMaximumFunding` = 18 - Funding amount exceeds the maximum single funding limit.
- `DailyCapExceeded` = 19 - Daily distribution cap for a specific pool has been exceeded.
- `GlobalDailyCapExceeded` = 20 - Global daily distribution cap has been exceeded.
- `ContractPaused` = 21 - Contract is paused and cannot perform operations.
- `EmergencyWithdrawalFailed` = 22 - Emergency withdrawal failed.

---

#### `is_paused`

Returns whether the contract is currently paused.

**Signature:**

```rust
pub fn is_paused(env: Env) -> bool
```

**Parameters:**

- `env: Env`

**Returns:** `bool`

---

#### `emergency_withdraw`

Emergency withdrawal: allows the admin to withdraw all funds from one or all
reward pools when the contract is paused (e.g. due to a critical vulnerability).
When `hunt_id` is 0, all pools with non-zero balances are drained.
When `all_pools` is true, iterates all hunts up to `max_hunt_id` and withdraws.

# Arguments
* `admin` - The contract admin address
* `hunt_id` - Specific hunt pool to drain (0 = all pools up to max_hunt_id)
* `recipient` - Address to receive the withdrawn funds
* `reason` - Reason for the emergency withdrawal (emitted in events)
* `max_hunt_id` - When hunt_id is 0, drains all pools from 1..=max_hunt_id

# Errors
* `NotInitialized` - Contract not initialized
* `Unauthorized` - Caller is not admin
* `ContractPaused` - Contract must be paused to call this

**Signature:**

```rust
pub fn emergency_withdraw(env: Env, admin: Address, hunt_id: u64, recipient: Address, reason: soroban_sdk::String, max_hunt_id: u64) -> Result<i128, RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `hunt_id: u64`
- `recipient: Address`
- `reason: soroban_sdk::String`
- `max_hunt_id: u64`

**Returns:** `Result<i128, RewardErrorCode>`

**Error type:** `RewardErrorCode`

**Error codes:**

- `NotInitialized` = 1
- `InsufficientPool` = 2
- `AlreadyDistributed` = 3
- `TransferFailed` = 4
- `InvalidAmount` = 5
- `InvalidConfig` = 6
- `NftMintFailed` = 7
- `PoolAlreadyExists` = 8
- `PoolNotFound` = 9
- `Unauthorized` = 10
- `BelowMinimumAmount` = 11
- `AlreadyInitialized` = 12
- `HuntNotFound` = 13
- `ReentrancyDetected` = 14 - A recursive distribution attempt was detected during an external XLM or NFT call.
- `PoolBalanceDivergence` = 15 - The tracked pool balance diverged from the actual XLM token balance.
- `PoolBalanceOverflow` = 16 - Pool balance would overflow if this funding amount is added (pool balance limit exceeded).
- `BelowMinimumFunding` = 17 - Funding amount is below the minimum required (dust attack prevention).
- `ExceedsMaximumFunding` = 18 - Funding amount exceeds the maximum single funding limit.
- `DailyCapExceeded` = 19 - Daily distribution cap for a specific pool has been exceeded.
- `GlobalDailyCapExceeded` = 20 - Global daily distribution cap has been exceeded.
- `ContractPaused` = 21 - Contract is paused and cannot perform operations.
- `EmergencyWithdrawalFailed` = 22 - Emergency withdrawal failed.

---

#### `get_emergency_logs`

Returns the emergency withdrawal log entries.

**Signature:**

```rust
pub fn get_emergency_logs(env: Env) -> soroban_sdk::Vec<EmergencyWithdrawalLogEntry>
```

**Parameters:**

- `env: Env`

**Returns:** `soroban_sdk::Vec<EmergencyWithdrawalLogEntry>`

---

#### `contract_version`

Returns the on-chain version stored during initialize, or the compiled constant.

**Signature:**

```rust
pub fn contract_version(env: Env) -> u32
```

**Parameters:**

- `env: Env`

**Returns:** `u32`

---

#### `check_nft_reward_compatibility`

Returns true if the given NftReward contract meets the minimum required version.

**Signature:**

```rust
pub fn check_nft_reward_compatibility(env: Env, nft_reward_address: Address) -> bool
```

**Parameters:**

- `env: Env`
- `nft_reward_address: Address`

**Returns:** `bool`

---

#### `get_schema_version`

**Signature:**

```rust
pub fn get_schema_version(env: Env) -> u32
```

**Parameters:**

- `env: Env`

**Returns:** `u32`

---

#### `initialize_schema`

**Signature:**

```rust
pub fn initialize_schema(env: Env, admin: Address) -> ()
```

**Parameters:**

- `env: Env`
- `admin: Address`

**Returns:** `()`

---

#### `propose_upgrade`

**Signature:**

```rust
pub fn propose_upgrade(env: Env, admin: Address, target_version: u32) -> Result<hunty_migration::UpgradeProposal, hunty_migration::UpgradeAuthError>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `target_version: u32`

**Returns:** `Result<hunty_migration::UpgradeProposal, hunty_migration::UpgradeAuthError>`

**Error type:** `UpgradeAuthError`

**Error codes:**

- `Unauthorized` = 1
- `NoProposal` = 2
- `TimelockPending` = 3
- `VersionMismatch` = 4
- `InvalidTimelock` = 5

---

#### `set_upgrade_timelock`

**Signature:**

```rust
pub fn set_upgrade_timelock(env: Env, admin: Address, delay_seconds: u64) -> Result<(), hunty_migration::UpgradeAuthError>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `delay_seconds: u64`

**Returns:** `Result<(), hunty_migration::UpgradeAuthError>`

**Error type:** `UpgradeAuthError`

**Error codes:**

- `Unauthorized` = 1
- `NoProposal` = 2
- `TimelockPending` = 3
- `VersionMismatch` = 4
- `InvalidTimelock` = 5

---

#### `get_upgrade_proposal`

**Signature:**

```rust
pub fn get_upgrade_proposal(env: Env) -> Option<hunty_migration::UpgradeProposal>
```

**Parameters:**

- `env: Env`

**Returns:** `Option<hunty_migration::UpgradeProposal>`

---

#### `get_upgrade_timelock`

**Signature:**

```rust
pub fn get_upgrade_timelock(env: Env) -> u64
```

**Parameters:**

- `env: Env`

**Returns:** `u64`

---

#### `get_upgrade_history`

**Signature:**

```rust
pub fn get_upgrade_history(env: Env, offset: u32, limit: u32) -> soroban_sdk::Vec<hunty_migration::UpgradeHistoryEntry>
```

**Parameters:**

- `env: Env`
- `offset: u32`
- `limit: u32`

**Returns:** `soroban_sdk::Vec<hunty_migration::UpgradeHistoryEntry>`

---

#### `run_migration`

**Signature:**

```rust
pub fn run_migration(env: Env, admin: Address, target_version: u32, dry_run: bool) -> Result<migration::MigrationReport, hunty_migration::UpgradeAuthError>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `target_version: u32`
- `dry_run: bool`

**Returns:** `Result<migration::MigrationReport, hunty_migration::UpgradeAuthError>`

**Error type:** `UpgradeAuthError`

**Error codes:**

- `Unauthorized` = 1
- `NoProposal` = 2
- `TimelockPending` = 3
- `VersionMismatch` = 4
- `InvalidTimelock` = 5

---

#### `rollback_migration`

**Signature:**

```rust
pub fn rollback_migration(env: Env, admin: Address) -> Result<migration::MigrationReport, hunty_migration::UpgradeAuthError>
```

**Parameters:**

- `env: Env`
- `admin: Address`

**Returns:** `Result<migration::MigrationReport, hunty_migration::UpgradeAuthError>`

**Error type:** `UpgradeAuthError`

**Error codes:**

- `Unauthorized` = 1
- `NoProposal` = 2
- `TimelockPending` = 3
- `VersionMismatch` = 4
- `InvalidTimelock` = 5

---

#### `get_health_dashboard`

**Signature:**

```rust
pub fn get_health_dashboard(env: Env) -> monitoring::ContractHealth
```

**Parameters:**

- `env: Env`

**Returns:** `monitoring::ContractHealth`

---

### `MaliciousToken`

#### `configure`

**Signature:**

```rust
pub fn configure(env: Env, reward_manager: Address, hunt_id: u64, player: Address, amount: i128) -> ()
```

**Parameters:**

- `env: Env`
- `reward_manager: Address`
- `hunt_id: u64`
- `player: Address`
- `amount: i128`

**Returns:** `()`

---

#### `mint`

**Signature:**

```rust
pub fn mint(env: Env, to: Address, amount: i128) -> ()
```

**Parameters:**

- `env: Env`
- `to: Address`
- `amount: i128`

**Returns:** `()`

---

#### `balance`

**Signature:**

```rust
pub fn balance(env: Env, who: Address) -> i128
```

**Parameters:**

- `env: Env`
- `who: Address`

**Returns:** `i128`

---

#### `transfer`

**Signature:**

```rust
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> ()
```

**Parameters:**

- `env: Env`
- `from: Address`
- `to: Address`
- `amount: i128`

**Returns:** `()`

---

#### `nested_result`

**Signature:**

```rust
pub fn nested_result(env: Env) -> u32
```

**Parameters:**

- `env: Env`

**Returns:** `u32`

---

# Error Code Reference

## `HuntErrorCode`

- `HuntNotFound` = 1
- `ClueNotFound` = 2
- `InvalidHuntStatus` = 3
- `PlayerNotRegistered` = 4
- `ClueAlreadyCompleted` = 5
- `InvalidAnswer` = 6
- `HuntNotActive` = 7
- `Unauthorized` = 8
- `InsufficientRewardPool` = 9
- `DuplicateRegistration` = 10
- `InvalidTitle` = 11
- `InvalidDescription` = 12
- `InvalidAddress` = 13
- `TooManyClues` = 14
- `InvalidQuestion` = 15
- `RefundFailed` = 16
- `NoCluesAdded` = 17
- `HuntNotCompleted` = 18
- `RewardAlreadyClaimed` = 19
- `RewardDistributionFailed` = 20
- `NoRewardsConfigured` = 21
- `DuplicateSubmission` = 22
- `SubmissionExpired` = 23
- `BannedPlayer` = 24
- `NoRequiredClues` = 25
- `RateLimitExceeded` = 26
- `ScoreOverflow` = 27
- `RegistrationsPaused` = 28
- `AnswersPaused` = 29
- `RewardsPaused` = 30
- `HuntEndTimeInPast` = 31

## `NftErrorCode`

- `NftNotFound` = 1
- `Unauthorized` = 2
- `NotOwner` = 3
- `InvalidRecipient` = 4
- `SoulboundNft` = 5
- `InvalidRarity` = 6
- `AlreadyInitialized` = 7
- `MaxSupplyReached` = 8
- `NotInitialized` = 9
- `NotOperator` = 10
- `NftNotTransferable` = 11
- `NftLocked` = 12

## `RewardErrorCode`

- `NotInitialized` = 1
- `InsufficientPool` = 2
- `AlreadyDistributed` = 3
- `TransferFailed` = 4
- `InvalidAmount` = 5
- `InvalidConfig` = 6
- `NftMintFailed` = 7
- `PoolAlreadyExists` = 8
- `PoolNotFound` = 9
- `Unauthorized` = 10
- `BelowMinimumAmount` = 11
- `AlreadyInitialized` = 12
- `HuntNotFound` = 13
- `ReentrancyDetected` = 14 - A recursive distribution attempt was detected during an external XLM or NFT call.
- `PoolBalanceDivergence` = 15 - The tracked pool balance diverged from the actual XLM token balance.
- `PoolBalanceOverflow` = 16 - Pool balance would overflow if this funding amount is added (pool balance limit exceeded).
- `BelowMinimumFunding` = 17 - Funding amount is below the minimum required (dust attack prevention).
- `ExceedsMaximumFunding` = 18 - Funding amount exceeds the maximum single funding limit.
- `DailyCapExceeded` = 19 - Daily distribution cap for a specific pool has been exceeded.
- `GlobalDailyCapExceeded` = 20 - Global daily distribution cap has been exceeded.
- `ContractPaused` = 21 - Contract is paused and cannot perform operations.
- `EmergencyWithdrawalFailed` = 22 - Emergency withdrawal failed.

## `UpgradeAuthError`

- `Unauthorized` = 1
- `NoProposal` = 2
- `TimelockPending` = 3
- `VersionMismatch` = 4
- `InvalidTimelock` = 5
