use soroban_sdk::{Address, Env, IntoVal, Map, Symbol};

pub struct NftHandler;

impl NftHandler {
    /// Mints an NFT via the NftReward contract and assigns it to the player.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `nft_contract` - Address of the NftReward contract
    /// * `hunt_id` - The hunt this NFT commemorates
    /// * `player` - The player receiving the NFT (initial owner)
    /// * `title` - NFT title
    /// * `description` - NFT description
    /// * `image_uri` - NFT image URI
    /// * `hunt_title` - Hunt title (for metadata context)
    /// * `rarity` - Rarity tier (0-5, 0 = default)
    /// * `tier` - Custom tier (0 = none)
    ///
    /// # Returns
    /// The unique NFT ID of the minted NFT
    pub fn distribute_nft(
        env: &Env,
        nft_contract: &Address,
        hunt_id: u64,
        player: &Address,
        title: soroban_sdk::String,
        description: soroban_sdk::String,
        image_uri: soroban_sdk::String,
        hunt_title: soroban_sdk::String,
        rarity: u32,
        tier: u32,
    ) -> u64 {
        let mut metadata: Map<soroban_sdk::Symbol, soroban_sdk::Val> = Map::new(env);
        metadata.set(soroban_sdk::Symbol::new(env, "title"), title.into_val(env));
        metadata.set(
            soroban_sdk::Symbol::new(env, "description"),
            description.into_val(env),
        );
        metadata.set(
            soroban_sdk::Symbol::new(env, "image_uri"),
            image_uri.into_val(env),
        );
        metadata.set(
            soroban_sdk::Symbol::new(env, "hunt_title"),
            hunt_title.into_val(env),
        );
        metadata.set(soroban_sdk::Symbol::new(env, "rarity"), rarity.into_val(env));
        metadata.set(soroban_sdk::Symbol::new(env, "tier"), tier.into_val(env));

        let mut args = soroban_sdk::Vec::new(env);
        args.push_back(hunt_id.into_val(env));
        args.push_back(player.clone().into_val(env));
        args.push_back(metadata.into_val(env));

        env.invoke_contract(
            nft_contract,
            &Symbol::new(env, "mint_reward_nft_from_map"),
            args,
        )
    }
}
