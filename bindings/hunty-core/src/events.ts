import type {
  u32,
  u64,
  i128,
  Option,
} from "@stellar/stellar-sdk/contract";
import type { HuntStatus } from "./types.js";

/**
 * Emitted when a clue is added. Does not expose the answer hash.
 */
export interface ClueAddedEvent {
  clue_id: u32;
  creator: string;
  /**
 * Difficulty tier (1-5, 1 = easiest, 5 = hardest).
 */
difficulty: u32;
  hunt_id: u64;
  is_required: boolean;
  points: u32;
  question: string;
  /**
 * Weight multiplier (default 1).
 */
weight: u32;
}


export interface HuntCreatedEvent {
  creator: string;
  hunt_id: u64;
}


/**
 * Emitted when a hunt is cloned.
 */
export interface HuntClonedEvent {
  creator: string;
  new_hunt_id: u64;
  original_hunt_id: u64;
}


/**
 * Emitted when a creator force-closes a hunt early (marks it Completed) while
 * preserving player scores and any already-distributed rewards. `rewarded_players`
 * is the number of completed players who received a final reward as part of closing.
 */
export interface HuntClosedEvent {
  closed_at: u64;
  hunt_id: u64;
  rewarded_players: u32;
}


export interface HuntActivatedEvent {
  activated_at: u64;
  hunt_id: u64;
}


export interface HuntCancelledEvent {
  hunt_id: u64;
}


export interface HuntCompletedEvent {
  completion_rank: u32;
  completion_time: u64;
  hunt_id: u64;
  player: string;
  total_score: u32;
}


export interface HuntDeactivatedEvent {
  hunt_id: u64;
}


export interface HuntReactivatedEvent {
  activated_at: u64;
  hunt_id: u64;
}


export interface HuntArchivedEvent {
  hunt_id: u64;
}


export interface HuntStatusChangedEvent {
  changed_at: u64;
  hunt_id: u64;
  new_status: HuntStatus;
  old_status: HuntStatus;
}


export interface HuntDescriptionUpdatedEvent {
  creator: string;
  description: string;
  hunt_id: u64;
}


export interface ClueCompletedEvent {
  clue_id: u32;
  hunt_id: u64;
  player: string;
  points_earned: u32;
}


export interface ClueAliasesAddedEvent {
  aliases_count: u32;
  clue_id: u32;
  creator: string;
  hunt_id: u64;
}


export interface RewardClaimedEvent {
  hunt_id: u64;
  nft_awarded: boolean;
  player: string;
  xlm_amount: i128;
}


export interface RewardClaimFailedEvent {
  error_code: u32;
  hunt_id: u64;
  player: string;
}


/**
 * Emitted when a player registers for an active hunt.
 */
export interface PlayerRegisteredEvent {
  hunt_id: u64;
  player: string;
}


/**
 * Emitted when a player successfully registers using an invite code.
 */
export interface PlayerRegisteredWithInviteEvent {
  hunt_id: u64;
  player: string;
}


export interface PlayerBannedEvent {
  hunt_id: u64;
  player: string;
}


export interface PlayerUnbannedEvent {
  hunt_id: u64;
  player: string;
}


export interface AnswerIncorrectEvent {
  clue_id: u32;
  hunt_id: u64;
  player: string;
  timestamp: u64;
}


export interface PartialScoreClaimedEvent {
  clues_completed: u32;
  hunt_id: u64;
  partial_score: u32;
  player: string;
}


export interface TeamCreatedEvent {
  hunt_id: u64;
  leader: string;
  name: string;
  team_id: u32;
}


export interface TeamMemberJoinedEvent {
  hunt_id: u64;
  player: string;
  team_id: u32;
}


export interface RewardManagerSetEvent {
  new_address: string;
  old_address: Option<string>;
}


export interface RegistrationDeadlineSetEvent {
  hunt_id: u64;
  registration_deadline: u64;
}


/**
 * Emitted when a hunt creator generates or updates the invite code for a private hunt.
 * The invite code itself is never emitted or stored — only its hash.
 */
export interface InviteCodeGeneratedEvent {
  creator: string;
  hunt_id: u64;
}


/**
 * Emitted when a hunt creator clears the invite code, pausing new registrations.
 */
export interface InviteCodeRevokedEvent {
  creator: string;
  hunt_id: u64;
}


export interface CreatorBlacklistedEvent {
  admin: string;
  creator: string;
}


export interface CreatorRemovedFromBlacklistEvent {
  admin: string;
  creator: string;
}
