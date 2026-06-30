# NFT Metadata Schema Versioning

## Current Version

- **`METADATA_SCHEMA_VERSION`: 1** (since Hunty-contract #390)

## Schema Fields (v1)

The metadata schema version is tracked **per NFT** via a dedicated storage key
`(NVER, nft_id) -> u32` rather than as a field inside the `NftMetadata` / `NftData`
struct. This avoids breaking the deserialisation of existing stored records,
because the Soroban host enforces an exact field-count match when unpacking
`#[contracttype]` structs from persistent storage.

| Field | Type | Description |
|-------|------|-------------|
| `title` | `String` | Display title of the NFT |
| `description` | `String` | Human-readable description |
| `image_uri` | `String` | Off-chain URI (IPFS / HTTPS) |
| `hunt_title` | `String` | Hunt title at mint time |
| `rarity` | `u32` | 0=default, 1=common, 2=uncommon, 3=rare, 4=epic, 5=legendary |
| `tier` | `u32` | Custom category (0 = none) |
| `creator` | `Option<Address>` | Original creator for provenance / royalties |
| `royalty_bps` | `Option<u32>` | Royalty in basis points (e.g. 250 = 2.5%) |

This metadata schema is exposed in the `get_nft_metadata` response via
`schema_version: u32`.

## Evolution Rules

1. **Increment the version** whenever a field is added, removed, or changed
   in the `NftMetadata` struct (or when the semantic meaning of an existing
   field changes).

2. **Old versions must remain readable.** Every code path that reads an NFT's
   metadata must handle the case where the stored `schema_version` is lower
   than the current version. This is ensured by:

   - `Storage::get_nft_version()` defaults to `1` when no version key exists
     (legacy pre-#390 data).
   - The `run_migration()` framework steps through each intermediate version
     sequentially, transforming data incrementally.

3. **Do not change the on-disk struct layout** (`NftMetadata`, `NftData`) when
   adding metadata-level fields.  The Soroban host rejects stored `#[contracttype]`
   structs whose ScVal map entry-count differs from the current struct definition.
   Instead, use auxiliary storage keys (like `(NVER, nft_id)`) and compose the
   result at read time.

4. **Each metadata version bump must have a corresponding migration step**
   in `contracts/nft-reward/src/migration.rs` (see "How to Add a Migration Step"
   below).

## How to Add a Migration Step

Suppose the current metadata schema version is `N` and you need to add version
`N + 1`:

1. Bump `METADATA_SCHEMA_VERSION` in `contracts/nft-reward/src/lib.rs` to `N + 1`.

2. Add a new migration function in `contracts/nft-reward/src/migration.rs`:

   ```rust
   fn migrate_vN_to_vNplus1(env: &Env) {
       let total = Storage::get_nft_counter(env);
       for nft_id in 1..=total {
           // Only process NFTs at the old schema version.
           if Storage::get_nft_version(env, nft_id) != N {
               continue;
           }
           // --- transform metadata here ---
           // e.g. add a new auxiliary key, transform existing data, etc.
           Storage::set_nft_version(env, nft_id, N + 1);
       }
   }
   ```

3. Wire the new function into the `run_migration` loop:

   ```rust
   N => {
       if !dry_run {
           Self::migrate_vN_to_vNplus1(env);
       }
       current = N + 1;
   }
   ```

4. Add a test that simulates an NFT at version `N`, runs the migration to
   `N + 1`, and asserts the expected output.

5. Update this document's "Current Version" header and the version table.

## Version History

| Version | Description | Migration Step |
|---------|-------------|----------------|
| 1       | Initial versioned schema. All pre-#390 NFTs are treated as v1 on read. | `migrate_v0_to_v1` assigns the `(NVER, nft_id)` key. |
| …       | Future versions | Add steps above. |
