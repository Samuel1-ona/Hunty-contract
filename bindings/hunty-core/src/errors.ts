export const HuntErrorCode = {
  1: {message:"HuntNotFound"},
  2: {message:"ClueNotFound"},
  3: {message:"InvalidHuntStatus"},
  4: {message:"PlayerNotRegistered"},
  5: {message:"ClueAlreadyCompleted"},
  6: {message:"InvalidAnswer"},
  7: {message:"HuntNotActive"},
  8: {message:"Unauthorized"},
  9: {message:"InsufficientRewardPool"},
  10: {message:"DuplicateRegistration"},
  11: {message:"InvalidTitle"},
  12: {message:"InvalidDescription"},
  13: {message:"InvalidAddress"},
  14: {message:"TooManyClues"},
  15: {message:"InvalidQuestion"},
  16: {message:"RefundFailed"},
  17: {message:"NoCluesAdded"},
  18: {message:"HuntNotCompleted"},
  19: {message:"RewardAlreadyClaimed"},
  20: {message:"RewardDistributionFailed"},
  21: {message:"NoRewardsConfigured"},
  22: {message:"DuplicateSubmission"},
  23: {message:"SubmissionExpired"},
  24: {message:"BannedPlayer"},
  25: {message:"NoRequiredClues"},
  26: {message:"RateLimitExceeded"},
  27: {message:"ScoreOverflow"},
  28: {message:"RegistrationsPaused"},
  29: {message:"AnswersPaused"},
  30: {message:"RewardsPaused"},
  31: {message:"HuntEndTimeInPast"},
  32: {message:"NoPendingAdmin"},
  33: {message:"PendingAdminMismatch"},
  34: {message:"InvalidRarity"},
  35: {message:"InvalidTimeBonusConfig"},
  36: {message:"AddressBlacklisted"},
  37: {message:"ContractPaused"},
  38: {message:"InvalidMaxAttempts"},
  39: {message:"InvalidWeight"},
  40: {message:"HintNotAvailable"},
  41: {message:"HintAlreadyUnlocked"},
  42: {message:"InsufficientScore"},
  43: {message:"TooManyCategories"},
  44: {message:"InvalidCategory"},
  45: {message:"InvalidDifficulty"},
  46: {message:"CorruptPlayerProgress"},
  47: {message:"HuntNotStarted"},
  48: {message:"AdminAlreadyProposed"},
  49: {message:"InvalidPoints"},
  50: {message:"HuntFull"}
}


export const UpgradeAuthError = {
  1: {message:"Unauthorized"},
  2: {message:"NoProposal"},
  3: {message:"TimelockPending"},
  4: {message:"VersionMismatch"},
  5: {message:"InvalidTimelock"}
}


export const RewardErrorCode = {
  1: {message:"NotInitialized"},
  2: {message:"InsufficientPool"},
  3: {message:"AlreadyDistributed"},
  4: {message:"TransferFailed"},
  5: {message:"InvalidAmount"},
  6: {message:"InvalidConfig"},
  7: {message:"NftMintFailed"},
  8: {message:"PoolAlreadyExists"},
  9: {message:"PoolNotFound"},
  10: {message:"Unauthorized"},
  11: {message:"BelowMinimumAmount"},
  12: {message:"AlreadyInitialized"},
  13: {message:"HuntNotFound"},
  /**
   * A recursive distribution attempt was detected during an external XLM or NFT call.
   */
  14: {message:"ReentrancyDetected"},
  /**
   * The tracked pool balance diverged from the actual XLM token balance.
   */
  15: {message:"PoolBalanceDivergence"},
  /**
   * Pool balance would overflow if this funding amount is added (pool balance limit exceeded).
   */
  16: {message:"PoolBalanceOverflow"},
  /**
   * Funding amount is below the minimum required (dust attack prevention).
   */
  17: {message:"BelowMinimumFunding"},
  /**
   * Funding amount exceeds the maximum single funding limit.
   */
  18: {message:"ExceedsMaximumFunding"},
  /**
   * Daily distribution cap for a specific pool has been exceeded.
   */
  19: {message:"DailyCapExceeded"},
  /**
   * Global daily distribution cap has been exceeded.
   */
  20: {message:"GlobalDailyCapExceeded"},
  /**
   * Contract is paused and cannot perform operations.
   */
  21: {message:"ContractPaused"},
  /**
   * Emergency withdrawal failed.
   */
  22: {message:"EmergencyWithdrawalFailed"}
}
