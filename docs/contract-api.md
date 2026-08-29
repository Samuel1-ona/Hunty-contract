# Contract API Reference

This file is generated automatically from the Rust contract sources in `contracts/`.
Run `make build` to regenerate it whenever contract APIs change.


## `common` Contract

_No contract API functions found._

## `hunty-core` Contract

### `HuntyCore`

#### `initialize_admin`

Sets the contract admin once. Subsequent calls require current admin auth via set_admin.

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `pause_contract`

Pauses all player operations (registrations, answers, rewards) globally.

**Signature:**

```rust
pub fn pause_contract(env: Env, admin: Address) -> Result<(), HuntErrorCode>
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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `unpause_contract`

Resumes all player operations.

**Signature:**

```rust
pub fn unpause_contract(env: Env, admin: Address) -> Result<(), HuntErrorCode>
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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `is_contract_paused`

Returns whether the global contract pause is active.

**Signature:**

```rust
pub fn is_contract_paused(env: Env) -> bool
```

**Parameters:**

- `env: Env`

**Returns:** `bool`

---

#### `create_hunt`

Creates a new scavenger hunt with the provided metadata.

# Arguments
* `env` - The Soroban environment
* `creator` - The address of the hunt creator (typically use env.invoker() from the caller)
* `title` - The title of the hunt (max 200 characters)
* `description` - The description of the hunt (max 2000 characters)
* `start_time` - Optional start timestamp. When set, players cannot register
or submit answers until the ledger timestamp reaches this value. 0 means
no start time restriction (immediately playable once activated).
* `end_time` - Optional end timestamp (0 means no end time restriction)

# Returns
The unique hunt ID of the newly created hunt

# Errors
* `InvalidTitle` - If title is empty or exceeds maximum length
* `InvalidDescription` - If description exceeds maximum length
* `InvalidAddress` - If creator address is invalid

**Signature:**

```rust
pub fn create_hunt(env: Env, creator: Address, title: String, description: String, start_time: Option<u64>, end_time: Option<u64>, max_submissions_per_minute: u32, start_multiplier_bps: Option<u32>, default_points: Option<u32>) -> Result<u64, HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `creator: Address`
- `title: String`
- `description: String`
- `start_time: Option<u64>`
- `end_time: Option<u64>`
- `max_submissions_per_minute: u32`
- `start_multiplier_bps: Option<u32>`
- `default_points: Option<u32>`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `clone_hunt`

Creates a new draft hunt by copying clues from an existing completed hunt.

The template hunt must already be completed. The copied hunt starts as a fresh
draft with a new hunt ID, creator, title, and description, but reuses the
template's clue questions, hashes, points, and required flags.
Clones an existing hunt into a new draft.
The caller must be the original hunt creator.
All clues are duplicated with new clue IDs.
Returns the new hunt ID.

**Signature:**

```rust
pub fn clone_hunt(env: Env, template_hunt_id: u64, caller: Address) -> Result<u64, HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `template_hunt_id: u64`
- `caller: Address`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `set_time_bonus_config`

**Signature:**

```rust
pub fn set_time_bonus_config(env: Env, hunt_id: u64, caller: Address, time_bonus_config: Option<TimeBonusConfig>) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `caller: Address`
- `time_bonus_config: Option<TimeBonusConfig>`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `update_hunt`

Updates a draft hunt's title and description. Only the hunt creator can update it.

**Signature:**

```rust
pub fn update_hunt(env: Env, hunt_id: u64, caller: Address, max_attempts_per_clue: u32, attempt_cooldown_secs: u32) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `caller: Address`
- `max_attempts_per_clue: u32`
- `attempt_cooldown_secs: u32`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `update_hunt_description`

Updates a hunt's description. Only the hunt creator can update it, and it can be updated for any hunt status.

**Signature:**

```rust
pub fn update_hunt_description(env: Env, hunt_id: u64, caller: Address, description: String) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `caller: Address`
- `description: String`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `set_max_players`

Sets the maximum players for a hunt. Only the hunt creator can set it, and only in Draft status.

**Signature:**

```rust
pub fn set_max_players(env: Env, hunt_id: u64, caller: Address, max_players: u32) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `caller: Address`
- `max_players: u32`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `get_hunt_end_time`

Exposes the end time of a hunt.

**Signature:**

```rust
pub fn get_hunt_end_time(env: Env, hunt_id: u64) -> Result<u64, HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
pub fn add_clue(env: Env, hunt_id: u64, question: String, answer: String, points: u32, is_required: bool, difficulty: Option<u32>, weight: Option<u32>) -> Result<u32, HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `question: String`
- `answer: String`
- `points: u32`
- `is_required: bool`
- `difficulty: Option<u32>`
- `weight: Option<u32>`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `add_clues`

Adds multiple clues to a draft hunt in one invocation. Only the hunt creator can add clues.

The batch is validated against the per-hunt clue cap before writing any new clues,
so a request that would exceed the limit fails without partially adding clues.

**Signature:**

```rust
pub fn add_clues(env: Env, hunt_id: u64, clues: Vec<BatchClueInput>) -> Result<Vec<u32>, HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `clues: Vec<BatchClueInput>`

**Returns:** `Result<Vec<u32>, HuntErrorCode>`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `add_clue_aliases`

Adds alternative acceptable answers to an existing clue (synonyms).
Only the hunt creator can add aliases, and only while the hunt is in Draft status.

# Arguments
* `env` - The Soroban environment
* `hunt_id` - The hunt containing the clue
* `clue_id` - The existing clue to add aliases to
* `answers` - Alternative answers that should also be accepted

# Errors
* `HuntNotFound` - Hunt does not exist
* `InvalidHuntStatus` - Hunt is not in Draft
* `Unauthorized` - Caller is not the hunt creator
* `ClueNotFound` - Clue does not exist
* `InvalidAnswer` - Any answer is empty or exceeds max length

**Signature:**

```rust
pub fn add_clue_aliases(env: Env, hunt_id: u64, clue_id: u32, answers: Vec<String>) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `clue_id: u32`
- `answers: Vec<String>`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `list_clues`

Returns paginated clues for a hunt. Answer hashes are not exposed.

**Signature:**

```rust
pub fn list_clues(env: Env, hunt_id: u64, offset: u32, limit: u32) -> Vec<ClueInfo>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `offset: u32`
- `limit: u32`

**Returns:** `Vec<ClueInfo>`

---

#### `list_hunts`

Returns a list of all hunts (paginated).

**Signature:**

```rust
pub fn list_hunts(env: Env, offset: u32, limit: u32) -> Vec<Hunt>
```

**Parameters:**

- `env: Env`
- `offset: u32`
- `limit: u32`

**Returns:** `Vec<Hunt>`

---

#### `search_hunts`

Searches hunts by partial title match over a caller-bounded hunt-id window.

**Signature:**

```rust
pub fn search_hunts(env: Env, title_substring: String, offset: u32, limit: u32, scan_limit: u32) -> Vec<Hunt>
```

**Parameters:**

- `env: Env`
- `title_substring: String`
- `offset: u32`
- `limit: u32`
- `scan_limit: u32`

**Returns:** `Vec<Hunt>`

---

#### `set_hunt_categories`

Updates categories for a draft hunt. At most five categories are allowed.

**Signature:**

```rust
pub fn set_hunt_categories(env: Env, hunt_id: u64, caller: Address, categories: Vec<String>) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `caller: Address`
- `categories: Vec<String>`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `get_hunts_by_category`

Returns hunts whose categories include the exact category string.

**Signature:**

```rust
pub fn get_hunts_by_category(env: Env, category: String, offset: u32, limit: u32, scan_limit: u32) -> Vec<Hunt>
```

**Parameters:**

- `env: Env`
- `category: String`
- `offset: u32`
- `limit: u32`
- `scan_limit: u32`

**Returns:** `Vec<Hunt>`

---

#### `set_hunt_difficulty_override`

Sets or clears a manual hunt difficulty override. Without an override,
the rating is the average clue difficulty.

**Signature:**

```rust
pub fn set_hunt_difficulty_override(env: Env, hunt_id: u64, caller: Address, difficulty_override: Option<u32>) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `caller: Address`
- `difficulty_override: Option<u32>`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `set_clue_hint`

Sets or clears the optional hint for a draft clue.

**Signature:**

```rust
pub fn set_clue_hint(env: Env, hunt_id: u64, clue_id: u32, caller: Address, hint: Option<String>, hint_penalty_points: u32) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `clue_id: u32`
- `caller: Address`
- `hint: Option<String>`
- `hint_penalty_points: u32`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `request_hint`

Unlocks a clue hint for a registered player and deducts the clue's hint penalty.

**Signature:**

```rust
pub fn request_hint(env: Env, hunt_id: u64, clue_id: u32, player: Address) -> Result<String, HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `clue_id: u32`
- `player: Address`

**Returns:** `Result<String, HuntErrorCode>`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
Resolves the XLM amount for the completing player.

If the hunt's rewardManager-configured pool has a non-empty
`time_based_tiers` list, this returns the tier's `xlm_amount`
whose `max_completion_secs >= (completion_at - registration_at)`.
If the elapsed time exceeds every configured tier, the last
(slowest) tier's amount is used as a fallback. If the pool has no
tiers configured (or is unreachable), this falls back to the
flat `hunt.reward_config.reward_per_winner()` amount.

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `close_hunt`

Force-closes (ends early) an in-progress hunt on behalf of its creator.

Unlike [`cancel_hunt`], closing preserves all player scores and any
rewards already collected: it marks the hunt `Completed` and triggers a
final reward distribution for every player who has completed the hunt but
not yet claimed. Players who have not completed the hunt keep their
progress and are simply not rewarded. Any unspent reward-pool balance is
left intact (a creator can refund it separately via [`cancel_hunt`] flows
only while a hunt is still cancellable — see project docs).

Only the creator may close a hunt, and only while it is `Active` or
`Paused`. Closing a `Draft`, `Completed`, `Cancelled`, `EmergencyStopped`,
or `Archived` hunt is rejected with `InvalidHuntStatus`.

# Arguments
* `env` - The Soroban environment
* `hunt_id` - The hunt to close
* `caller` - The creator (must authorize the call via require_auth)

# Returns
`Ok(())` on success

# Errors
* `HuntNotFound` - Hunt does not exist
* `Unauthorized` - Caller is not the hunt creator
* `InvalidHuntStatus` - Hunt is not in an early-closable status
* `RewardsPaused` - Reward distribution is globally paused
* `InvalidRarity` - The hunt's configured NFT rarity is out of range
* `RewardDistributionFailed` - A RewardManager cross-contract call failed

**Signature:**

```rust
pub fn close_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<(), HuntErrorCode>
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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `archive_hunt`

**Signature:**

```rust
pub fn archive_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<(), HuntErrorCode>
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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `gc_hunt`

Reclaims the storage of a cancelled or archived hunt (issue #446).

A cancelled hunt keeps every clue, player-progress, team, leaderboard
and bookkeeping entry it ever wrote. Nothing referenced those entries
any more, but nothing removed them either, so they sat in persistent
storage paying rent until their TTL lapsed.

Only `Cancelled` and `Archived` hunts may be collected — those are the
two terminal states. Anything else is rejected with `InvalidHuntStatus`,
because collecting a live hunt would destroy player progress.

The sweep is **idempotent**: running it twice reports zero the second
time rather than failing, so an interrupted call is safe to retry.

# Authorization
The hunt creator or the contract admin.

# Returns
A [`GcReport`] describing what was reclaimed.

**Signature:**

```rust
pub fn gc_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<GcReport, HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `caller: Address`

**Returns:** `Result<GcReport, HuntErrorCode>`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `get_hunt_storage_footprint`

Reports how much storage a hunt currently occupies, without removing
anything. Read-only, so it needs no authorization — hunt existence and
size are already public via `get_hunt_info`.

**Signature:**

```rust
pub fn get_hunt_storage_footprint(env: Env, hunt_id: u64) -> GcReport
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`

**Returns:** `GcReport`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `set_reward_manager`

Sets the RewardManager contract address for cross-contract reward distribution.

**Signature:**

```rust
pub fn set_reward_manager(env: Env, admin: Address, reward_manager: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `reward_manager: Address`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `blacklist_creator`

Blacklists a creator address, preventing them from creating new hunts.
Caller must be the admin.

**Signature:**

```rust
pub fn blacklist_creator(env: Env, admin: Address, creator: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `creator: Address`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `remove_from_blacklist`

Removes a creator from the blacklist, restoring their ability to create hunts.
Caller must be the admin.

**Signature:**

```rust
pub fn remove_from_blacklist(env: Env, admin: Address, creator: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `creator: Address`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `is_blacklisted`

Returns true if the given address is blacklisted.

**Signature:**

```rust
pub fn is_blacklisted(env: Env, creator: Address) -> bool
```

**Parameters:**

- `env: Env`
- `creator: Address`

**Returns:** `bool`

---

#### `complete_hunt`

Completes a hunt for a player and distributes rewards.

This function verifies that the player has completed all required clues,
then distributes rewards via the RewardManager contract (if configured)
and updates the player's reward status.

Reward amounts can be either flat (`xlm_pool / max_winners`) or
time-based (configured via `RewardManager::set_pool_tiers`), in which
case the amount depends on `completion_at - started_at` for the
completing player.

# Arguments
* `env` - The Soroban environment
* `hunt_id` - The hunt ID
* `player` - The player claiming completion/rewards

# Returns
`Ok(())` on successful reward claim

# Errors
* `HuntNotFound` - Hunt does not exist
* `InvalidHuntStatus` - Hunt is not Active (e.g. already Completed or Cancelled)
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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `register_player`

Distributes the reward for a single completed, unclaimed player.

Resolves the player's XLM amount (flat or tier-based), invokes the
RewardManager (if configured and there is at least one reward type),
marks the player's progress as claimed, increments the hunt's
`claimed_count` (in memory — the caller is responsible for persisting
the hunt), and emits a `RewardClaimed` event.

The caller must ensure `progress.is_completed == true` and
`progress.reward_claimed == false` before invoking this.

# Errors
* `InvalidRarity` - The hunt's configured NFT rarity is out of range
* `RewardDistributionFailed` - The RewardManager cross-contract call failed
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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `generate_invite_code`

Generates or updates the invite code for a private hunt.

The invite code is hashed with SHA256 (using hunt_id as salt) and only the hash
is stored on-chain. The plain-text code is never persisted or emitted in events.
Calling this function overwrites any previously set invite code.

# Arguments
* `env` - The Soroban environment
* `hunt_id` - The hunt to generate an invite code for
* `creator` - The hunt creator (must authorize the call)
* `invite_code` - The plain-text invite code to hash and store

# Returns
`Ok(())` on success

# Errors
* `HuntNotFound` - Hunt does not exist
* `Unauthorized` - Caller is not the hunt creator
* `InvalidHuntStatus` - Hunt is not in Draft status

**Signature:**

```rust
pub fn generate_invite_code(env: Env, hunt_id: u64, creator: Address, invite_code: String) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `creator: Address`
- `invite_code: String`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `set_hunt_privacy`

Sets whether a hunt is private (invite-only).

Only the hunt creator can call this, and only while the hunt is in Draft status.
When making a hunt private, an invite code must already be configured via
`generate_invite_code` before the hunt can be activated.

# Arguments
* `env` - The Soroban environment
* `hunt_id` - The hunt to update privacy for
* `creator` - The hunt creator (must authorize the call)
* `is_private` - Whether the hunt should be invite-only

# Returns
`Ok(())` on success

# Errors
* `HuntNotFound` - Hunt does not exist
* `Unauthorized` - Caller is not the hunt creator
* `InvalidHuntStatus` - Hunt is not in Draft status

**Signature:**

```rust
pub fn set_hunt_privacy(env: Env, hunt_id: u64, creator: Address, is_private: bool) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `creator: Address`
- `is_private: bool`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `revoke_invite_code`

Clears the invite code for a private hunt, effectively pausing new registrations.
The hunt creator can generate a new code later via `generate_invite_code`.

# Arguments
* `env` - The Soroban environment
* `hunt_id` - The hunt to revoke the invite code for
* `creator` - The hunt creator (must authorize the call)

# Returns
`Ok(())` on success

# Errors
* `HuntNotFound` - Hunt does not exist
* `Unauthorized` - Caller is not the hunt creator
* `InvalidHuntStatus` - Hunt is not in Draft status

**Signature:**

```rust
pub fn revoke_invite_code(env: Env, hunt_id: u64, creator: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `creator: Address`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `register_with_invite`

Registers a player for a private hunt using a valid invite code.

The provided invite code is hashed (with hunt_id as salt) and compared against
the stored `invite_code_hash`. If they match, the player is registered.

# Arguments
* `env` - The Soroban environment
* `hunt_id` - The private hunt to register for
* `player` - The address of the player (must authorize the call via require_auth)
* `invite_code` - The plain-text invite code to validate

# Returns
`Ok(())` on success

# Errors
* `HuntNotFound` - Hunt does not exist
* `InvalidHuntStatus` - Hunt is not in Active status, is not private (use
`register_player` instead), or has no invite code configured
* `InvalidAnswer` - The provided invite code is empty or does not match
* `DuplicateRegistration` - Player is already registered for this hunt

**Signature:**

```rust
pub fn register_with_invite(env: Env, hunt_id: u64, player: Address, invite_code: String) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `player: Address`
- `invite_code: String`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `preview_answer`

Verifies a candidate answer for a registered player with authorization and rate limiting.

Unlike `submit_answer`, `preview_answer` does not mark the clue as completed, award points,
or emit clue completion events, but requires player authorization and enforces the same
per-minute rate limits and attempt cooldowns to prevent brute-force dictionary attacks.

**Signature:**

```rust
pub fn preview_answer(env: Env, hunt_id: u64, clue_id: u32, player: Address, answer: String) -> Result<bool, HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `clue_id: u32`
- `player: Address`
- `answer: String`

**Returns:** `Result<bool, HuntErrorCode>`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
In team mode, returns true if any teammate has already completed this clue.
In team mode, records a clue completion against the player's team so
teammates see it as already solved and share the earned score.

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `submit_answer_with_hash`

Variant of `submit_answer` which accepts a precomputed SHA256 answer hash.
This avoids on-chain normalization and hashing when the client supplies
the correctly computed `answer_hash = SHA256(hunt_id || clue_id || normalized_answer)`.
Use this from off-chain callers that can perform normalization+hashing cheaply.

**Signature:**

```rust
pub fn submit_answer_with_hash(env: Env, hunt_id: u64, clue_id: u32, player: Address, answer_hash: BytesN<32>, submission_nonce: u64, submitted_at: u64) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `clue_id: u32`
- `player: Address`
- `answer_hash: BytesN<32>`
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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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

#### `get_hunt_count`

Returns the total number of hunts created (read-only).

**Signature:**

```rust
pub fn get_hunt_count(env: Env) -> u64
```

**Parameters:**

- `env: Env`

**Returns:** `u64`

---

#### `get_hunt_leaderboard`

Returns ranked players for a hunt with pagination support (read-only).
Sorted by score descending, then by completion time ascending (earlier = better).
Limit is capped at 20 to control gas. Returns error if hunt does not exist.

**Signature:**

```rust
pub fn get_hunt_leaderboard(env: Env, hunt_id: u64, limit: u32) -> Result<LeaderboardResult, HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `limit: u32`

**Returns:** `Result<LeaderboardResult, HuntErrorCode>`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `get_hunt_leaderboard_window`

Scans a bounded window of registered players for a hunt and returns
their compact rows. This method enables clients to page through all
registered players in multiple calls (bounded by `MAX_LEADERBOARD_SCAN_SIZE`)
and merge results off-chain to build a full leaderboard without a single
large on-chain scan.

**Signature:**

```rust
pub fn get_hunt_leaderboard_window(env: Env, hunt_id: u64, start_index: u32, window_size: u32) -> Result<crate::types::LeaderboardWindow, HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `start_index: u32`
- `window_size: u32`

**Returns:** `Result<crate::types::LeaderboardWindow, HuntErrorCode>`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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

#### `add_co_creator`

**Signature:**

```rust
pub fn add_co_creator(env: Env, hunt_id: u64, creator: Address, new_co_creator: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `creator: Address`
- `new_co_creator: Address`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `remove_co_creator`

**Signature:**

```rust
pub fn remove_co_creator(env: Env, hunt_id: u64, creator: Address, co_creator_to_remove: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `creator: Address`
- `co_creator_to_remove: Address`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `get_co_creators`

**Signature:**

```rust
pub fn get_co_creators(env: Env, hunt_id: u64) -> Vec<Address>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`

**Returns:** `Vec<Address>`

---

#### `propose_new_admin`

Step one of a two-step admin key rotation.

The current admin proposes a new admin. The change is NOT applied until the
proposed address calls `accept_admin`, which prevents accidental lockout: a
typo in `propose_new_admin` can simply be overwritten or ignored, and the
current admin never loses access until the new admin actively accepts.

**Signature:**

```rust
pub fn propose_new_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `new_admin: Address`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

---

#### `accept_admin`

Step two of a two-step admin key rotation.

The proposed new admin accepts the role, completing the rotation. Only the
address stored by `propose_new_admin` may accept, so a wrong proposal cannot
silently take over the contract.

**Signature:**

```rust
pub fn accept_admin(env: Env, new_admin: Address) -> Result<(), HuntErrorCode>
```

**Parameters:**

- `env: Env`
- `new_admin: Address`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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

## `migration` Contract

_No contract API functions found._

## `nft-reward` Contract

### `NftReward`

#### `initialize`

Initializes the NFT reward contract with an admin, minter, and optional max supply cap.
Passing `Some(0)` as `max_supply` is rejected with `InvalidMaxSupply`; use `None` for unlimited minting.

**Signature:**

```rust
pub fn initialize(env: Env, admin: Address, minter: Address, max_supply: Option<u64>, collection_metadata: CollectionMetadata) -> Result<(), crate::errors::NftErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `minter: Address`
- `max_supply: Option<u64>`
- `collection_metadata: CollectionMetadata`

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
- `InvalidMetadata` = 13
- `MetadataFrozen` = 14
- `TooManyExtensions` = 15
- `InvalidExtensionKey` = 16
- `InvalidExtensionValue` = 17
- `ExtensionNotFound` = 18
- `InvalidMaxSupply` = 19

---

#### `mint_reward_nft`

Mints a unique NFT as a reward for hunt completion.

`minter` must be an authorized minter (and must sign the transaction) when the
contract has been initialized. Before initialization the check is skipped so
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
pub fn mint_reward_nft(env: Env, minter: Address, hunt_id: u64, player_address: Address, metadata: NftMetadata) -> u64
```

**Parameters:**

- `env: Env`
- `minter: Address`
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
- "extensions": Map<String, String> (optional, arbitrary key-value metadata)

**Signature:**

```rust
pub fn mint_reward_nft_from_map(env: Env, minter: Address, hunt_id: u64, player_address: Address, metadata: Map<Symbol, Val>) -> u64
```

**Parameters:**

- `env: Env`
- `minter: Address`
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

#### `get_collection_metadata`

Returns the collection-level metadata configured at initialization.

**Signature:**

```rust
pub fn get_collection_metadata(env: Env) -> Option<CollectionMetadata>
```

**Parameters:**

- `env: Env`

**Returns:** `Option<CollectionMetadata>`

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

#### `set_nft_extension`

Sets an extension field on an NFT. Only the NFT owner can call this.
Max 10 extension fields per NFT. If the key already exists, it is updated.
If the maximum is reached and the key is new, it returns an error.

# Arguments
* `nft_id` - The NFT to extend
* `owner` - The current owner (must authorize)
* `key` - The extension key (max 64 bytes)
* `value` - The extension value (max 512 bytes)

**Signature:**

```rust
pub fn set_nft_extension(env: Env, nft_id: u64, owner: Address, key: String, value: String) -> Result<(), crate::errors::NftErrorCode>
```

**Parameters:**

- `env: Env`
- `nft_id: u64`
- `owner: Address`
- `key: String`
- `value: String`

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
- `InvalidMetadata` = 13
- `MetadataFrozen` = 14
- `TooManyExtensions` = 15
- `InvalidExtensionKey` = 16
- `InvalidExtensionValue` = 17
- `ExtensionNotFound` = 18
- `InvalidMaxSupply` = 19

---

#### `get_nft_extension`

Gets the value of a specific extension field for an NFT.

# Arguments
* `nft_id` - The NFT to query
* `key` - The extension key to look up

# Returns
The extension value if found, None otherwise.

**Signature:**

```rust
pub fn get_nft_extension(env: Env, nft_id: u64, key: String) -> Option<String>
```

**Parameters:**

- `env: Env`
- `nft_id: u64`
- `key: String`

**Returns:** `Option<String>`

---

#### `get_nft_extensions`

Gets all extension fields for an NFT.

# Arguments
* `nft_id` - The NFT to query

# Returns
Map of all extension key-value pairs.

**Signature:**

```rust
pub fn get_nft_extensions(env: Env, nft_id: u64) -> Option<Map<String, String>>
```

**Parameters:**

- `env: Env`
- `nft_id: u64`

**Returns:** `Option<Map<String, String>>`

---

#### `remove_nft_extension`

Removes an extension field from an NFT. Only the NFT owner can call this.

# Arguments
* `nft_id` - The NFT to modify
* `owner` - The current owner (must authorize)
* `key` - The extension key to remove

**Signature:**

```rust
pub fn remove_nft_extension(env: Env, nft_id: u64, owner: Address, key: String) -> Result<(), crate::errors::NftErrorCode>
```

**Parameters:**

- `env: Env`
- `nft_id: u64`
- `owner: Address`
- `key: String`

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
- `InvalidMetadata` = 13
- `MetadataFrozen` = 14
- `TooManyExtensions` = 15
- `InvalidExtensionKey` = 16
- `InvalidExtensionValue` = 17
- `ExtensionNotFound` = 18
- `InvalidMaxSupply` = 19

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
- `InvalidMetadata` = 13
- `MetadataFrozen` = 14
- `TooManyExtensions` = 15
- `InvalidExtensionKey` = 16
- `InvalidExtensionValue` = 17
- `ExtensionNotFound` = 18
- `InvalidMaxSupply` = 19

---

#### `add_authorized_contract`

Adds a contract to the authorized callers list. Only the admin can call this.

**Signature:**

```rust
pub fn add_authorized_contract(env: Env, admin: Address, contract: Address) -> Result<(), crate::errors::NftErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `contract: Address`

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
- `InvalidMetadata` = 13
- `MetadataFrozen` = 14
- `TooManyExtensions` = 15
- `InvalidExtensionKey` = 16
- `InvalidExtensionValue` = 17
- `ExtensionNotFound` = 18
- `InvalidMaxSupply` = 19

---

#### `remove_authorized_contract`

Removes a contract from the authorized callers list. Only the admin can call this.

**Signature:**

```rust
pub fn remove_authorized_contract(env: Env, admin: Address, contract: Address) -> Result<(), crate::errors::NftErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `contract: Address`

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
- `InvalidMetadata` = 13
- `MetadataFrozen` = 14
- `TooManyExtensions` = 15
- `InvalidExtensionKey` = 16
- `InvalidExtensionValue` = 17
- `ExtensionNotFound` = 18
- `InvalidMaxSupply` = 19

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
- `InvalidMetadata` = 13
- `MetadataFrozen` = 14
- `TooManyExtensions` = 15
- `InvalidExtensionKey` = 16
- `InvalidExtensionValue` = 17
- `ExtensionNotFound` = 18
- `InvalidMaxSupply` = 19

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
- `InvalidMetadata` = 13
- `MetadataFrozen` = 14
- `TooManyExtensions` = 15
- `InvalidExtensionKey` = 16
- `InvalidExtensionValue` = 17
- `ExtensionNotFound` = 18
- `InvalidMaxSupply` = 19

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

#### `get_max_supply`

Returns the configured maximum total supply of NFTs.

- `None`  → no cap was set (unlimited minting)
- `Some(0)` → unlimited (explicit zero treated as unlimited)
- `Some(n)` → at most `n` NFTs may ever be minted

**Signature:**

```rust
pub fn get_max_supply(env: Env) -> Option<u64>
```

**Parameters:**

- `env: Env`

**Returns:** `Option<u64>`

---

#### `set_max_supply`

Updates the maximum total supply cap. Admin only.

- Pass `None` to remove the cap (unlimited).
- Pass `Some(n)` where `n > 0` and `n >= current total_supply` to set or raise the cap.
  Attempting to set a cap of `0` or lower than the already-minted count is
  rejected with `InvalidMaxSupply`.

# Errors
* `NotInitialized` - Contract has not been initialized yet
* `Unauthorized`   - Caller is not the admin
* `InvalidMaxSupply` - `new_max` is `Some(0)` or less than the current minted supply

**Signature:**

```rust
pub fn set_max_supply(env: Env, admin: Address, new_max: Option<u64>) -> Result<(), crate::errors::NftErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `new_max: Option<u64>`

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
- `InvalidMetadata` = 13
- `MetadataFrozen` = 14
- `TooManyExtensions` = 15
- `InvalidExtensionKey` = 16
- `InvalidExtensionValue` = 17
- `ExtensionNotFound` = 18
- `InvalidMaxSupply` = 19

---

#### `get_remaining_supply`

Returns the number of NFTs that can still be minted.

- `None`  → unlimited (no cap configured)
- `Some(n)` → exactly `n` more NFTs may be minted before the cap is hit

Once the cap is reached this returns `Some(0)`, and any subsequent mint
will panic with `MaxSupplyReached`.

**Signature:**

```rust
pub fn get_remaining_supply(env: Env) -> Option<u64>
```

**Parameters:**

- `env: Env`

**Returns:** `Option<u64>`

---

#### `list_all_nfts`

Lists all NFTs minted by the contract with pagination support.

Returns a vector of NftData structs, paginated by offset and limit.
The limit is bounded to MAX_SCAN_LIMIT (1000) to prevent excessive gas consumption.

# Arguments
* `env` - The Soroban environment
* `offset` - The starting index for pagination (0-based)
* `limit` - The maximum number of NFTs to return (capped at MAX_SCAN_LIMIT)

# Returns
Vec<NftData> - A vector of NFT data structures, bounded by limit or remaining NFTs

**Signature:**

```rust
pub fn list_all_nfts(env: Env, offset: u32, limit: u32) -> Vec<NftData>
```

**Parameters:**

- `env: Env`
- `offset: u32`
- `limit: u32`

**Returns:** `Vec<NftData>`

---

#### `search_nfts_by_metadata`

Searches NFTs by metadata fields with pagination support.

Allows filtering NFTs by various metadata fields. All filter parameters are optional -
only provided filters are applied. Returns matching NFTs with pagination.

# Arguments
* `env` - The Soroban environment
* `offset` - The starting index for pagination (0-based)
* `limit` - The maximum number of NFTs to return (capped at MAX_SCAN_LIMIT)
* `title_filter` - Optional filter for NFT title (exact match)
* `hunt_title_filter` - Optional filter for hunt title (exact match)
* `rarity_filter` - Optional filter for rarity tier (0-5)
* `tier_filter` - Optional filter for custom tier
* `creator_filter` - Optional filter for creator address
* `hunt_id_filter` - Optional filter for hunt ID
* `extension_key` - Optional extension key to search for
* `extension_value` - Optional extension value to match (requires extension_key)

# Returns
Vec<NftData> - A vector of matching NFT data structures, paginated by offset and limit

**Signature:**

```rust
pub fn search_nfts_by_metadata(env: Env, offset: u32, limit: u32, title_filter: Option<String>, hunt_title_filter: Option<String>, rarity_filter: Option<u32>, tier_filter: Option<u32>, creator_filter: Option<Address>, hunt_id_filter: Option<u64>, extension_key: Option<String>, extension_value: Option<String>) -> Vec<NftData>
```

**Parameters:**

- `env: Env`
- `offset: u32`
- `limit: u32`
- `title_filter: Option<String>`
- `hunt_title_filter: Option<String>`
- `rarity_filter: Option<u32>`
- `tier_filter: Option<u32>`
- `creator_filter: Option<Address>`
- `hunt_id_filter: Option<u64>`
- `extension_key: Option<String>`
- `extension_value: Option<String>`

**Returns:** `Vec<NftData>`

---

#### `transfer_nft`

Transfers an NFT to a new owner when the NFT is transferable.
Non-transferable (soulbound) NFTs remain bound to the minting recipient.

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
- `InvalidMetadata` = 13
- `MetadataFrozen` = 14
- `TooManyExtensions` = 15
- `InvalidExtensionKey` = 16
- `InvalidExtensionValue` = 17
- `ExtensionNotFound` = 18
- `InvalidMaxSupply` = 19

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
Performs an O(1) indexed lookup via the stored (owner, hunt_id) count mapping.

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
The limit is bounded to MAX_SCAN_LIMIT (1000) to prevent excessive gas consumption.

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
The limit is bounded to MAX_SCAN_LIMIT (1000) to prevent excessive gas consumption.

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

#### `burn_nft`

Burns (permanently destroys) an NFT, removing it from storage and the owner's list.

# Authorization
The `owner` must authorize this call and be the current owner of the NFT.

# Errors
Returns `NftNotFound` if the NFT does not exist.
Returns `NotOwner` if the caller is not the current owner.
Returns `NftLocked` if the NFT is locked (e.g., staked elsewhere).

**Signature:**

```rust
pub fn burn_nft(env: Env, nft_id: u64, owner: Address) -> Result<(), crate::errors::NftErrorCode>
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
- `InvalidMetadata` = 13
- `MetadataFrozen` = 14
- `TooManyExtensions` = 15
- `InvalidExtensionKey` = 16
- `InvalidExtensionValue` = 17
- `ExtensionNotFound` = 18
- `InvalidMaxSupply` = 19

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
pub fn initialize(env: Env, admin: Address, xlm_token: Address, hunty_core: Address) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `xlm_token: Address`
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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `propose_new_admin`

Step one of a two-step admin key rotation.

**Signature:**

```rust
pub fn propose_new_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `new_admin: Address`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `accept_admin`

Step two of a two-step admin key rotation.

**Signature:**

```rust
pub fn accept_admin(env: Env, new_admin: Address) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `new_admin: Address`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `add_authorized_contract`

Adds a contract to the authorized callers list for `distribute_rewards`.
Only the contract admin can call this.

The distribution gate is fail-closed: `distribute_rewards` reads the immediate invoker via `env.invoker()`, requires that address to authenticate, and then checks whether the address is present in the allowlist. If no contracts are allowlisted, or the invoker is not in the allowlist, the call returns `Unauthorized`.

**Signature:**

```rust
pub fn add_authorized_contract(env: Env, admin: Address, contract: Address) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `contract: Address`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `remove_authorized_contract`

Removes a contract from the authorized callers list.
Only the contract admin can call this.

**Signature:**

```rust
pub fn remove_authorized_contract(env: Env, admin: Address, contract: Address) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `contract: Address`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `create_reward_pool_with_nft`

Creates a reward pool for a specific hunt with a specified token.

Must be called before `fund_reward_pool`. Any address may fund the pool
after creation (see `fund_reward_pool`); the token contract must be
SAC-compatible.

For NFT-only pools (pools that distribute only NFTs without any token component),
set `min_distribution_amount` to 0 and provide an `nft_contract` address.

# Arguments
* `creator` - The hunt creator who will own and fund the pool
* `hunt_id` - The hunt this pool is for
* `token_address` - Address of the SAC-compatible token contract (e.g., XLM, USDC)
* `min_distribution_amount` - Minimum token amount per distribution (0 for NFT-only pools)
* `nft_contract` - Optional NFT contract address for NFT rewards
* `nft_royalty_bps` - Creator royalty basis points (0-10000) for secondary market sales
* `nft_transferable` - Whether reward NFTs from this pool are transferable

# Errors
* `PoolAlreadyExists` - A pool already exists for this hunt_id
* `InvalidAmount` - min_distribution_amount is negative
* `InvalidTokenContract` - token_address is not a valid SAC-compatible token
* `InvalidConfig` - min_distribution_amount is 0 but no NFT contract provided
* `NotInitialized` - hunty_core has not been configured (set during initialize)
* `HuntNotFound` - hunt_id does not exist in HuntyCore

**Signature:**

```rust
pub fn create_reward_pool_with_nft(env: Env, creator: Address, hunt_id: u64, token_address: Address, min_distribution_amount: i128, nft_contract: Option<Address>, nft_royalty_bps: u32, nft_transferable: bool) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `creator: Address`
- `hunt_id: u64`
- `token_address: Address`
- `min_distribution_amount: i128`
- `nft_contract: Option<Address>`
- `nft_royalty_bps: u32`
- `nft_transferable: bool`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `create_reward_pool`

Creates a reward pool for a specific hunt with a specified token.

Must be called before `fund_reward_pool`. Any address may fund the pool
after creation (see `fund_reward_pool`); the token contract must be
SAC-compatible.

# Arguments
* `creator` - The hunt creator who will own and fund the pool
* `hunt_id` - The hunt this pool is for
* `token_address` - Address of the SAC-compatible token contract (e.g., XLM, USDC)
* `min_distribution_amount` - Minimum token amount per distribution (0 = no minimum)
* `nft_royalty_bps` - Creator royalty basis points (0-10000) for secondary market sales
* `nft_transferable` - Whether reward NFTs from this pool are transferable

# Errors
* `PoolAlreadyExists` - A pool already exists for this hunt_id
* `InvalidAmount` - min_distribution_amount is negative
* `InvalidTokenContract` - token_address is not a valid SAC-compatible token
* `NotInitialized` - hunty_core has not been configured (set during initialize)
* `HuntNotFound` - hunt_id does not exist in HuntyCore

**Signature:**

```rust
pub fn create_reward_pool(env: Env, creator: Address, hunt_id: u64, token_address: Address, min_distribution_amount: i128, nft_royalty_bps: u32, nft_transferable: bool) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `creator: Address`
- `hunt_id: u64`
- `token_address: Address`
- `min_distribution_amount: i128`
- `nft_royalty_bps: u32`
- `nft_transferable: bool`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `set_pool_target_amount`

Sets the funding target used for top-up progress notifications.
`target_amount` of 0 disables percentage tracking (events report 0%).

**Signature:**

```rust
pub fn set_pool_target_amount(env: Env, creator: Address, hunt_id: u64, target_amount: i128) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `creator: Address`
- `hunt_id: u64`
- `target_amount: i128`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `set_min_distribution_interval`

Sets the minimum seconds between distributions for a pool (0 disables).

**Signature:**

```rust
pub fn set_min_distribution_interval(env: Env, creator: Address, hunt_id: u64, min_distribution_interval_secs: u64) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `creator: Address`
- `hunt_id: u64`
- `min_distribution_interval_secs: u64`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `set_distribution_mode`

Sets the distribution mode (Fixed or Proportional) for a pool.

**Signature:**

```rust
pub fn set_distribution_mode(env: Env, creator: Address, hunt_id: u64, mode: DistributionMode) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `creator: Address`
- `hunt_id: u64`
- `mode: DistributionMode`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `set_pool_tiers`

Updates (or installs) the time-based reward tier schedule on an existing
reward pool, enabling conditional reward amounts based on player completion
time (acceptance criteria: "Define time-based reward tiers in pool config").

Tiers must be supplied in strictly ascending order of `max_completion_secs`
(i.e. faster tiers first), and every `xlm_amount` must be strictly positive.
Passing an empty `Vec` disables tier-based rewards so the pool reverts
to the flat `xlm_pool / max_winners` amount.

Only the pool creator is authorized to call this. The new tiers are
persisted immediately and become effective for any subsequent distribution
call. Already-distributed rewards are not affected.

# Arguments
* `creator` - The pool creator (must match the stored creator)
* `hunt_id` - The hunt whose pool config to update
* `time_based_tiers` - New tier list (strictly ascending by time, all amounts > 0;
an empty list disables tier-based rewards)

# Errors
* `PoolNotFound` - No pool exists for this hunt_id
* `Unauthorized` - Caller is not the pool creator
* `InvalidConfig` - Tier list (when non-empty) contains a zero/negative
amount or is not strictly ascending

**Signature:**

```rust
pub fn set_pool_tiers(env: Env, creator: Address, hunt_id: u64, time_based_tiers: Vec<TimeBasedRewardTier>) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `creator: Address`
- `hunt_id: u64`
- `time_based_tiers: Vec<TimeBasedRewardTier>`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `set_pool_nft_contract`

Sets or updates the NFT contract address for an existing reward pool.
This allows pools to distribute NFTs alongside or instead of tokens.

# Arguments
* `creator` - The pool creator (must match the stored creator)
* `hunt_id` - The hunt whose pool config to update
* `nft_contract` - NFT contract address (or None to disable NFT rewards)

# Errors
* `PoolNotFound` - No pool exists for this hunt_id
* `Unauthorized` - Caller is not the pool creator

**Signature:**

```rust
pub fn set_pool_nft_contract(env: Env, creator: Address, hunt_id: u64, nft_contract: Option<Address>) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `creator: Address`
- `hunt_id: u64`
- `nft_contract: Option<Address>`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `add_delegate`

Adds a delegate allowed to distribute rewards for a pool.
Only the pool creator can manage delegates.

**Signature:**

```rust
pub fn add_delegate(env: Env, creator: Address, hunt_id: u64, delegate: Address) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `creator: Address`
- `hunt_id: u64`
- `delegate: Address`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `remove_delegate`

Removes a delegate from a pool.
Only the pool creator can manage delegates.

**Signature:**

```rust
pub fn remove_delegate(env: Env, creator: Address, hunt_id: u64, delegate: Address) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `creator: Address`
- `hunt_id: u64`
- `delegate: Address`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `get_pool_config`

Returns the full configuration of a reward pool, including its tier list.
`None` when no pool has been created for the given `hunt_id`.

This is the primary read path used by HuntyCore at completion time to
resolve which tier (if any) applies to a player's completion time.

**Signature:**

```rust
pub fn get_pool_config(env: Env, hunt_id: u64) -> Option<RewardPoolConfig>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`

**Returns:** `Option<RewardPoolConfig>`

---

#### `fund_reward_pool`

Records `amount` as a contribution from `funder` toward `hunt_id`'s
pool sponsorship ledger, adding them to the pool's funder list the
first time they contribute. Shared by `fund_reward_pool` and
`migrate_pool` (which attributes the migrated lump sum to the shared
creator) so `refund_pool` can always pay the current balance back out
in proportion to who funded it.
Wipes a pool's sponsorship ledger — every tracked funder's recorded
contribution and the funder list itself. Used once a pool's balance
has been fully paid out (`refund_pool`) or moved elsewhere
(`migrate_pool`'s source pool), so stale contribution records can
never be double-counted against a pool's balance again.
Funds the reward pool for a specific hunt.

The pool must have been created via `create_reward_pool` first.
**Anyone may fund a pool** — this supports sponsorship (a brand funding
a community hunt, a DAO topping up a pool, several people pooling a
prize), not just the creator. Each funder must authorize the call
themselves; their contribution is tracked individually so that
`refund_pool` can later pay the remaining balance back out in
proportion to what each funder put in, and never hand one funder's
contribution to another party. See `docs/adr/006-reward-pool-sponsorship.md`.

Transfers tokens from the funder to this contract and records the balance.
Uses the token address specified when the pool was created.

# Validation
- Minimum funding: 1 XLM equivalent (10,000,000 base units) to prevent dust attacks
- Maximum single funding: 1 billion tokens to prevent overflow
- Pool balance limit: 1 billion tokens total to prevent overflow
- Rejects zero or negative amounts
- At most `MAX_FUNDERS_PER_POOL` distinct funders are tracked per pool

# Arguments
* `funder` - The address funding the pool (must authorize this call)
* `hunt_id` - The hunt to fund
* `amount` - Token amount to add to the pool (must be > 0)

# Errors
* `PoolNotFound` - Pool has not been created yet
* `InvalidAmount` - Amount is <= 0
* `BelowMinimumFunding` - Amount is less than minimum (dust attack prevention)
* `ExceedsMaximumFunding` - Amount exceeds maximum limit
* `PoolBalanceOverflow` - Adding this amount would exceed pool balance limit
* `TooManyFunders` - This would be a new funder and the pool already
tracks the maximum number of distinct funders

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `refund_pool`

Refunds the remaining pool balance for a hunt, paid out **pro rata**
across every address that funded it (see `fund_reward_pool`) in
proportion to each funder's share of total contributions — never
paying one funder's contribution to another party. A pool funded by a
single address (the common case) simply gets its whole balance back.

Can only be triggered by the pool creator, who must authorize the
call; the payout destinations are the tracked funders, not the caller.
Uses the token address specified when the pool was created. The hunt
must be in a terminal state (cancelled or ended) when HuntyCore is
configured — refunding an active hunt's pool out from under its
players is rejected.

**Important:** This is a destructive operation. Ensure all distributions are complete
before calling this function, as any remaining unclaimed rewards cannot be distributed
after the pool is refunded.

# Accounting
This function updates:
- Pool balance: Set to 0
- Total refunded: Incremented by the refund amount
- Audit log: Entry recorded with PoolOperation::Refund

After a refund, the accounting identity is:
`total_deposited == balance + total_distributed + total_refunded`

# Events
Emits one `PoolRefundedEvent` per funder paid out (a single event for
the common single-funder case).

# Arguments
* `creator` - The pool creator (must authorize this call)
* `hunt_id` - The hunt whose pool is being refunded

# Errors
* `PoolNotFound` - Pool has not been created yet
* `InvalidHuntStatus` - The hunt is not cancelled or ended (only when
`set_hunty_core` has been called)
* `Unauthorized` - Caller is not the pool creator

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `migrate_pool`

Migrates the unused balance of an expired or cancelled hunt's pool into
an existing destination pool owned by the same creator.

This lets a creator recycle funds locked in a finished hunt into a fresh
hunt without withdrawing and re-depositing. The XLM never leaves this
contract; only the internal per-hunt balance accounting is re-keyed.

# Eligibility (acceptance criteria)
* The source pool's hunt must be **expired or cancelled** — verified via
a cross-contract call to the configured HuntyCore contract
(`is_hunt_expired_or_cancelled`). If HuntyCore is not configured, the
source cannot be shown eligible and migration is rejected.
* The **destination pool must already exist** (created via
`create_reward_pool`).
* **Both pools must have the same creator**, who must authorize the call.

# Arguments
* `creator` - The shared creator of both pools (must authorize the call)
* `source_hunt_id` - The expired/cancelled hunt to drain
* `dest_hunt_id` - The destination hunt to credit

# Returns
The amount of XLM migrated from the source pool to the destination pool.

# Errors
* `InvalidMigration` - source and destination are the same hunt, or the
source pool has no balance to migrate
* `PoolNotFound` - the source pool does not exist
* `DestinationPoolNotFound` - the destination pool does not exist
* `Unauthorized` - the caller does not own both pools
* `SourcePoolNotEligible` - the source hunt is neither expired nor cancelled
* `PoolBalanceOverflow` - crediting the destination would overflow the pool cap
* `TooManyFunders` - the destination already tracks the maximum number of
distinct funders and the creator is not already one of them

**Signature:**

```rust
pub fn migrate_pool(env: Env, creator: Address, source_hunt_id: u64, dest_hunt_id: u64) -> Result<i128, RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `creator: Address`
- `source_hunt_id: u64`
- `dest_hunt_id: u64`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `get_reward_pool`

Returns true when the source hunt is expired or cancelled, as reported by
the configured HuntyCore contract. When HuntyCore is not configured, or
the cross-contract call fails, the source is treated as not eligible.
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

#### `get_pool_funders`

Returns the distinct addresses currently tracked as funders of a pool
(i.e. that have contributed and not yet been refunded), in the order
they first contributed. Empty if the pool has never been funded, has
been fully refunded, or has no sponsorship ledger (see `refund_pool`).

**Signature:**

```rust
pub fn get_pool_funders(env: Env, hunt_id: u64) -> Vec<Address>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`

**Returns:** `Vec<Address>`

---

#### `get_pool_funder_contribution`

Returns how much `funder` has contributed to a pool that has not yet
been refunded. 0 if they have never funded it or were already refunded.

**Signature:**

```rust
pub fn get_pool_funder_contribution(env: Env, hunt_id: u64, funder: Address) -> i128
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `funder: Address`

**Returns:** `i128`

---

#### `get_pool_statistics`

Returns comprehensive statistics for a reward pool.
Returns None if no pool has been created for the given hunt_id.

**Signature:**

```rust
pub fn get_pool_statistics(env: Env, hunt_id: u64) -> Option<RewardPoolStatistics>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`

**Returns:** `Option<RewardPoolStatistics>`

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

#### `freeze_pool`

Freezes a reward pool, preventing any further distributions.

Can be called by either the pool creator or the contract admin.
Emits a `PoolFrozenEvent`.

# Arguments
* `caller` - The address calling freeze (must be pool creator or admin)
* `hunt_id` - The hunt whose pool to freeze

# Errors
* `PoolNotFound` - No pool exists for this hunt_id
* `Unauthorized` - Caller is neither the pool creator nor the contract admin

**Signature:**

```rust
pub fn freeze_pool(env: Env, caller: Address, hunt_id: u64) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `caller: Address`
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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `unfreeze_pool`

Unfreezes a reward pool, re-enabling distributions.

Can be called by either the pool creator or the contract admin.
Emits a `PoolUnfrozenEvent`.

# Arguments
* `caller` - The address calling unfreeze (must be pool creator or admin)
* `hunt_id` - The hunt whose pool to unfreeze

# Errors
* `PoolNotFound` - No pool exists for this hunt_id
* `Unauthorized` - Caller is neither the pool creator nor the contract admin

**Signature:**

```rust
pub fn unfreeze_pool(env: Env, caller: Address, hunt_id: u64) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `caller: Address`
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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `is_pool_frozen`

Returns whether a reward pool is currently frozen.
Returns `false` if no pool exists for the given `hunt_id`.

**Signature:**

```rust
pub fn is_pool_frozen(env: Env, hunt_id: u64) -> bool
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`

**Returns:** `bool`

---

#### `set_daily_pool_cap`

Sets the daily distribution cap for a specific pool.

This limit controls the maximum amount of rewards that can be distributed from
a pool in a single day (24-hour rolling window). This is a live operational control
and should be validated to prevent silent misconfiguration.

# Arguments
* `admin` - The contract admin address (must match the stored admin)
* `hunt_id` - The hunt whose pool cap to set
* `cap` - The maximum amount to distribute per day. Must be positive (> 0).
A cap of 0 means no distributions are allowed (use to disable).

# Errors
* `NotInitialized` - Contract has not been initialized (no admin set)
* `Unauthorized` - Caller is not the contract admin
* `PoolNotFound` - No pool exists for this hunt_id
* `InvalidAmount` - Cap is negative (negative caps silently block distributions)

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `distribute_batch`

Distributes rewards to multiple players in a single atomic transaction.

Every entry in the batch is validated first (no state changes). If all
entries pass validation, all transfers are executed. If any single entry
fails validation, the entire batch is rejected with no state changes.

# Atomicity guarantee

The two-phase design (validate-all, execute-all) means callers get a
simple all-or-nothing contract:
- If the function returns `Ok(())`, every entry was processed.
- If it returns `Err(_)`, no tokens were moved and no distribution
records were created.

# Gas limit consideration

The batch size is capped at [`MAX_BATCH_SIZE`] (10 entries) to keep the
transaction within Soroban's per-transaction instruction budget even
when every entry performs both XLM and NFT operations.

# Arguments
* `distributions` - A `Vec` of `BatchDistributionEntry`, each containing
a `hunt_id`, `player_address`, and `reward_config`.

# Errors
* `InvalidConfig` - Batch is empty or an entry has an invalid config.
* `BatchTooLarge` - Batch exceeds `MAX_BATCH_SIZE`.
* `AlreadyDistributed` - A player has already received a reward for this hunt.
* `ReplayDetected` - Distribution nonce inconsistency for an entry.
* `InsufficientPool` - A pool cannot cover the combined XLM amount for its hunt.
* `BelowMinimumAmount` - An entry's XLM amount is below the pool's minimum.
* `PoolNotFound` - No pool exists for an entry's hunt_id.
* `NotInitialized` - XLM token address not set.
* `Unauthorized` - Caller is not an authorized contract.

**Signature:**

```rust
pub fn distribute_batch(env: Env, distributions: Vec<BatchDistributionEntry>) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `distributions: Vec<BatchDistributionEntry>`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `retry_failed_nft_mint`

Retries a failed NFT mint for a previously distributed reward.

When NFT minting fails during `distribute_rewards`, the failure is logged
and the pending mint data is stored. This function allows the admin to
retry the failed NFT mint and update the distribution record.

# Arguments
* `admin` - The contract admin address
* `hunt_id` - The hunt associated with the failed NFT mint
* `player` - The player who should receive the NFT

# Returns
The NFT ID of the successfully minted NFT

# Errors
* `NotInitialized` - Contract not initialized
* `Unauthorized` - Caller is not the contract admin
* `NftMintPendingNotFound` - No pending failed NFT mint for this hunt/player
* `NftMintFailed` - NFT mint attempt failed again

**Signature:**

```rust
pub fn retry_failed_nft_mint(env: Env, admin: Address, hunt_id: u64, player: Address) -> Result<u64, RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `hunt_id: u64`
- `player: Address`

**Returns:** `Result<u64, RewardErrorCode>`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

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
**DEPRECATED: Do not use for new integrations.**

This legacy distribution path is maintained only for backward compatibility.
All new integrations must use `distribute_rewards` instead.

This function wraps `distribute_rewards` and therefore inherits all the same
security constraints:
- Replays are rejected via the same nonce-based mechanism
- The ReentrancyGuard is acquired identically
- `min_distribution_amount` and daily caps are enforced
- Authorization checks are identical (fail-open by caller)

**Removal timeline:** This function is scheduled for removal in a future major release.
The exact deprecation timeline will be announced in contract release notes.

**Security note:** Any attacker analyzing this contract should understand that
`distribute_rewards_legacy` and `distribute_rewards` use identical security checks.
The legacy path is not a bypass vector.

# Arguments
* `player` - The address receiving the distribution
* `hunt_id` - The hunt pool to distribute from
* `xlm_amount` - Token amount to distribute (0 = no token transfer)
* `_nft_enabled` - Ignored; NFTs are not supported on this path

# Returns
- `true` if the distribution succeeded
- `false` if the distribution failed (check the transaction result for the error code)

# Differences from `distribute_rewards`
- Returns `bool` instead of `Result<(), RewardErrorCode>` (loses error detail)
- Discards `_nft_enabled` parameter (NFTs cannot be distributed)
- No structured logging of the error

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

#### `get_dist_cooldown`

Remaining seconds until the next distribution is allowed for this pool.
Returns 0 if no interval is configured or the cooldown has elapsed.

**Signature:**

```rust
pub fn get_dist_cooldown(env: Env, hunt_id: u64) -> u64
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`

**Returns:** `u64`

---

#### `get_distribution_proof`

Returns the on-chain distribution receipt/proof for a hunt/player pair.

**Signature:**

```rust
pub fn get_distribution_proof(env: Env, hunt_id: u64, player: Address) -> Option<DistributionProof>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `player: Address`

**Returns:** `Option<DistributionProof>`

---

#### `verify_distribution`

Verifies a distribution proof against the on-chain receipt.

Recomputes SHA-256(pool_id || player || amount || timestamp) and checks
it matches both the provided `hash` and the stored receipt (when present).

**Signature:**

```rust
pub fn verify_distribution(env: Env, pool_id: u64, player: Address, amount: i128, timestamp: u64, hash: BytesN<32>) -> bool
```

**Parameters:**

- `env: Env`
- `pool_id: u64`
- `player: Address`
- `amount: i128`
- `timestamp: u64`
- `hash: BytesN<32>`

**Returns:** `bool`

---

#### `distribute_proportional`

Distribute a proportional share of the pool based on player score.

Amount = floor((player_score / total_scores) * pool_balance).
Remainder stays in the pool. Enforces min_distribution_amount when set.
Requires the pool's distribution_mode to be Proportional (or will still
compute proportionally when called via this entry point).

Returns the XLM amount distributed.

**Signature:**

```rust
pub fn distribute_proportional(env: Env, hunt_id: u64, player: Address, player_score: u64, total_scores: u64) -> Result<i128, RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `player: Address`
- `player_score: u64`
- `total_scores: u64`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

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

#### `get_min_distribution_amount`

Returns the minimum distribution amount configured for a hunt's reward pool.
Returns 0 if no pool has been created for the hunt.

**Signature:**

```rust
pub fn get_min_distribution_amount(env: Env, hunt_id: u64) -> i128
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

#### `set_vesting_period_secs`

Sets the vesting period (in seconds) on an existing reward pool.

When `vesting_period_secs > 0`, subsequent `distribute_rewards` calls
will **not** transfer XLM immediately. Instead a `VestingRecord` is
stored and the player must call `claim_vested` to receive tokens
proportionally as time elapses after distribution.

Setting this to `0` disables vesting and reverts to instant payouts for
future distributions (already-pending vesting records are unaffected).

# Arguments
* `creator` - Pool owner (must match stored creator)
* `hunt_id` - The hunt whose pool to configure
* `vesting_period_secs` - Vesting duration in seconds (0 = disabled)

# Errors
* `PoolNotFound` - Pool does not exist
* `Unauthorized` - Caller is not the pool creator

**Signature:**

```rust
pub fn set_vesting_period_secs(env: Env, creator: Address, hunt_id: u64, vesting_period_secs: u64) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `creator: Address`
- `hunt_id: u64`
- `vesting_period_secs: u64`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `claim_vested`

Claims the proportionally vested XLM reward for the caller.

The claimable amount is: `total_amount * min(elapsed / vesting_period_secs, 1) - claimed_amount`.

The player can call this any number of times over the vesting period.
Each call transfers whatever has newly vested since the last claim.
Once `claimed_amount == total_amount` the schedule is fully exhausted.

# Arguments
* `player` - The player claiming their vested reward
* `hunt_id` - The hunt whose vesting record to claim from

# Returns
The XLM amount (in stroops) transferred to the player.

# Errors
* `VestingNotStarted` - No vesting record exists for this (hunt_id, player)
* `VestingAlreadyClaimed` - Full vesting amount has already been claimed
* `NothingToVest` - Nothing has vested yet at the current timestamp
* `InsufficientPool` - Contract token balance is too low (should not normally occur)

**Signature:**

```rust
pub fn claim_vested(env: Env, player: Address, hunt_id: u64) -> Result<i128, RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `player: Address`
- `hunt_id: u64`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `get_vesting_status`

Returns the current vesting status for a (hunt_id, player) pair.

Returns `None` when no vesting record exists (i.e. the pool either had
no vesting configured or the player has not completed that hunt yet).

**Signature:**

```rust
pub fn get_vesting_status(env: Env, hunt_id: u64, player: Address) -> Option<VestingStatus>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `player: Address`

**Returns:** `Option<VestingStatus>`

---

#### `admin_resolve_distribution`

Manually resolves a distribution that failed mid-execution.

Allows the contract admin to mark a distribution as either `Completed`
or `Refunded` when the automatic distribution process could not finish
(e.g., XLM was sent but NFT mint failed). This is a bookkeeping-only
operation and does not move funds.

# Arguments
* `admin` - The contract admin address (must match the stored admin)
* `hunt_id` - The hunt whose distribution to resolve
* `player` - The player whose distribution to resolve
* `resolution` - Outcome: `ResolutionStatus::Completed` or `ResolutionStatus::Refunded`

# Errors
* `NotInitialized` - Contract has not been initialized (no admin set)
* `Unauthorized` - Caller is not the contract admin
* `DistributionNotFound` - No distribution record exists for this hunt/player

**Signature:**

```rust
pub fn admin_resolve_distribution(env: Env, admin: Address, hunt_id: u64, player: Address, resolution: ResolutionStatus) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `hunt_id: u64`
- `player: Address`
- `resolution: ResolutionStatus`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `get_pool_distributions`

Returns a paginated list of distributions made from a specific reward pool.

# Arguments
* `hunt_id` - The hunt whose pool distributions to query
* `offset` - Starting index for pagination (0-based)
* `limit` - Maximum number of entries to return

# Returns
A Vec of PoolDistribution entries containing player addresses and distribution details.
Returns an empty Vec if the pool has no distributions or offset is beyond the list.

**Signature:**

```rust
pub fn get_pool_distributions(env: Env, hunt_id: u64, offset: u32, limit: u32) -> Vec<PoolDistribution>
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `offset: u32`
- `limit: u32`

**Returns:** `Vec<PoolDistribution>`

---

#### `get_pool_distribution_count`

Returns the total count of distributions made from a specific reward pool.

# Arguments
* `hunt_id` - The hunt whose pool distribution count to query

# Returns
The total number of distributions for the pool.

**Signature:**

```rust
pub fn get_pool_distribution_count(env: Env, hunt_id: u64) -> u64
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`

**Returns:** `u64`

---

#### `get_distribution_analytics`

Returns distribution analytics (average, median, min, max) across a reward pool.

Supports optional time-range filtering via `start_time` and `end_time`
(ledger timestamps). Only distributions within `[start_time, end_time)`
are included when both bounds are provided; `None` means unbounded.

The computation is gas-bounded: at most [`MAX_ANALYTICS_ENTRIES`] (500)
distributions are processed. If the pool has more entries than this limit,
only the most recent entries (up to the limit) are analysed.

# Arguments
* `hunt_id` - The hunt whose pool analytics to query
* `start_time` - Optional lower bound (inclusive) ledger timestamp filter
* `end_time` - Optional upper bound (exclusive) ledger timestamp filter

# Returns
A `DistributionAnalytics` struct with count, total, average, median, min, max.
All fields are zero when the pool has no distributions or no entries match
the time filter.

**Signature:**

```rust
pub fn get_distribution_analytics(env: Env, hunt_id: u64, start_time: Option<u64>, end_time: Option<u64>) -> DistributionAnalytics
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `start_time: Option<u64>`
- `end_time: Option<u64>`

**Returns:** `DistributionAnalytics`

---

#### `admin_withdraw_unclaimed`

Allows the admin to withdraw unclaimed (surplus) XLM remaining in a reward pool
after the hunt has ended and all winners have been determined.

This is needed when a hunt concludes with fewer winners than anticipated,
leaving unspent XLM locked in the pool. Only the contract admin may call this.

Withdrawal is only permitted after the hunt has ended (end_time passed) or been
cancelled. This prevents draining pools while a hunt is active and players may
still be mid-game. When HuntyCore is configured, the hunt status is verified.

# Arguments
* `admin` - The contract admin address (must match the stored admin)
* `hunt_id` - The hunt whose remaining pool balance to withdraw
* `recipient` - The address that will receive the withdrawn XLM
* `amount` - The amount to withdraw. Must be positive (> 0).

# Errors
* `NotInitialized` - Contract has not been initialized (no admin set)
* `Unauthorized` - Caller is not the contract admin
* `PoolNotFound` - No pool exists for this hunt_id
* `InvalidAmount` - Amount is <= 0, or exceeds the available pool balance
* `SourcePoolNotEligible` - Hunt is still active (not ended or cancelled)

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `admin_withdraw_all`

Explicitly withdraws the entire remaining balance from a reward pool.

This function provides an explicit, intentional way to drain a pool completely.
Unlike `admin_withdraw_unclaimed`, which handles partial withdrawals of unclaimed
amounts, this function is semantically clear: it empties the pool by name.

Withdrawal is only permitted after the hunt has ended (end_time passed) or been
cancelled. This prevents draining pools while a hunt is active and players may
still be mid-game. When HuntyCore is configured, the hunt status is verified.

# Arguments
* `admin` - The contract admin address (must match the stored admin)
* `hunt_id` - The hunt whose pool to drain completely
* `recipient` - The address that will receive the full pool balance

# Errors
* `NotInitialized` - Contract has not been initialized (no admin set)
* `Unauthorized` - Caller is not the contract admin
* `PoolNotFound` - No pool exists for this hunt_id
* `InvalidAmount` - Pool balance is zero (nothing to withdraw)
* `SourcePoolNotEligible` - Hunt is still active (not ended or cancelled)

**Signature:**

```rust
pub fn admin_withdraw_all(env: Env, admin: Address, hunt_id: u64, recipient: Address) -> Result<(), RewardErrorCode>
```

**Parameters:**

- `env: Env`
- `admin: Address`
- `hunt_id: u64`
- `recipient: Address`

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

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

#### `pause_funding`

Blocks pool funding. Distribution is unaffected unless separately paused.

**Signature:**

```rust
pub fn pause_funding(env: Env, admin: Address) -> Result<(), RewardErrorCode>
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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `unpause_funding`

Resumes pool funding. Has no effect while the global pause is engaged.

**Signature:**

```rust
pub fn unpause_funding(env: Env, admin: Address) -> Result<(), RewardErrorCode>
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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `pause_distribution`

Blocks reward distribution. Funding is unaffected unless separately paused.

**Signature:**

```rust
pub fn pause_distribution(env: Env, admin: Address) -> Result<(), RewardErrorCode>
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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `unpause_distribution`

Resumes reward distribution. Has no effect while the global pause is engaged.

**Signature:**

```rust
pub fn unpause_distribution(env: Env, admin: Address) -> Result<(), RewardErrorCode>
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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

---

#### `get_pause_state`

Effective pause state as `(global, funding, distribution)`.

The two granular values are the *effective* ones, so they read `true`
whenever the global stop is engaged. Mirrors `HuntyCore::get_pause_state`.

**Signature:**

```rust
pub fn get_pause_state(env: Env) -> (bool, bool, bool)
```

**Parameters:**

- `env: Env`

**Returns:** `(bool, bool, bool)`

---

#### `get_raw_pause_flags`

The granular flags as stored, ignoring the global stop — lets an
operator see what will still be paused after `unpause()`.

**Signature:**

```rust
pub fn get_raw_pause_flags(env: Env) -> (bool, bool)
```

**Parameters:**

- `env: Env`

**Returns:** `(bool, bool)`

---

#### `emergency_withdraw`

Rejects the call when funding is paused.
Rejects the call when distribution is paused.
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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

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

#### `get_pool_audit_log`

Exposes a paginated read query for the audit log of a given pool.

**Signature:**

```rust
pub fn get_pool_audit_log(env: Env, hunt_id: u64, start_after: Option<u64>, limit: Option<u32>) -> PoolAuditLogResponse
```

**Parameters:**

- `env: Env`
- `hunt_id: u64`
- `start_after: Option<u64>`
- `limit: Option<u32>`

**Returns:** `PoolAuditLogResponse`

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
- `NoPendingAdmin` = 32
- `PendingAdminMismatch` = 33
- `InvalidRarity` = 34
- `InvalidTimeBonusConfig` = 35
- `AddressBlacklisted` = 36
- `ContractPaused` = 37
- `InvalidMaxAttempts` = 38
- `InvalidWeight` = 39
- `HintNotAvailable` = 40
- `HintAlreadyUnlocked` = 41
- `InsufficientScore` = 42
- `TooManyCategories` = 43
- `InvalidCategory` = 44
- `InvalidDifficulty` = 45
- `CorruptPlayerProgress` = 46
- `HuntNotStarted` = 47
- `AdminAlreadyProposed` = 48
- `InvalidPoints` = 49
- `HuntFull` = 50

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
- `InvalidMetadata` = 13
- `MetadataFrozen` = 14
- `TooManyExtensions` = 15
- `InvalidExtensionKey` = 16
- `InvalidExtensionValue` = 17
- `ExtensionNotFound` = 18
- `InvalidMaxSupply` = 19

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
- `FundingPaused` = 23 - Pool funding is paused (issue #628). Distribution may still be running.
- `DistributionPaused` = 24 - Reward distribution is paused (issue #628). Funding may still be open.

## `UpgradeAuthError`

- `Unauthorized` = 1
- `NoProposal` = 2
- `TimelockPending` = 3
- `VersionMismatch` = 4
- `InvalidTimelock` = 5
