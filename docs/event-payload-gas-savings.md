# Event Payload Gas Savings

This change keeps event names and topic layouts stable for indexers, while removing
data fields that are either already addressable by event topics or queryable from
contract state.

Soroban event fees are driven by serialized event payload size. The figures below
measure the XDR `ScVal` bytes removed from each emitted event payload. Runtime gas
savings scale with the active network's per-byte event/ledger-entry fee settings.

## Removed Fields

| Contract | Event | Removed field | Replacement path for indexers | Bytes saved |
| --- | --- | --- | --- | ---: |
| `hunty-core` | `HuntCreated` | `title: String` | Use `hunt_id` from the existing topic/data and call `get_hunt_info(hunt_id)` | `16 + 8 + padded_len(title)` |
| `nft-reward` | `NftMinted` | `metadata: NftMetadata` | Use `nft_id` from the existing topic/data and call `get_nft_metadata(nft_id)` | `24 + metadata_xdr_bytes` |
| `reward-manager` | `POOL_FND` | `total_deposited: i128` | Use `hunt_id` from the existing topic/data and call `get_reward_pool(hunt_id)` | `44` |

## Representative Savings

Using current project validation limits and common payload examples:

| Event | Scenario | Payload bytes saved |
| --- | --- | ---: |
| `HuntCreated` | 24-byte title | `48` |
| `HuntCreated` | max 200-byte title | `224` |
| `NftMinted` | 20-byte title, 80-byte description, 64-byte image URI, 20-byte hunt title, no optional royalty/creator payload | `300+` |
| `NftMinted` | max configured NFT text lengths from `hunty-core` reward config | `900+` |
| `POOL_FND` | every funding event | `44` |

## Compatibility Notes

Existing indexers can continue matching the same event symbols/topics:

- `("HuntCreated", hunt_id)`
- `("NftMinted", nft_id)`
- `(POOL_FND, hunt_id)`

The removed values remain available through stable read APIs. Consumers that need
full objects should hydrate records by ID after receiving the event rather than
depending on full object snapshots in the event data.
