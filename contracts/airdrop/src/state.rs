use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub mirror_token: CanonicalAddr,
}

pub const CONFIG: Item<Config> = Item::new("\u{0}\u{6}config");
pub const LATEST_STAGE: Item<u8> = Item::new("\u{0}\u{c}latest_stage");

pub const MERKLE_ROOT: Map<&[u8], String> = Map::new("merkle_root");
pub const CLAIM_INDEX: Map<(&[u8], &[u8]), bool> = Map::new("claim_index");

#[cfg(test)]
mod test {
    use super::*;

    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{Api, StdResult, Storage};
    use cosmwasm_storage::{
        bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket,
    };
    const KEY_CONFIG: &[u8] = b"config";

    pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
        singleton(storage, KEY_CONFIG).save(config)
    }
    pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
        singleton_read(storage, KEY_CONFIG).load()
    }

    #[test]
    fn config_legacy_compatibility() {
        let mut deps = mock_dependencies(&[]);
        store_config(
            &mut deps.storage,
            &Config {
                owner: deps.api.addr_canonicalize("owner0000").unwrap(),
                mirror_token: deps.api.addr_canonicalize("mirror0000").unwrap(),
            },
        )
        .unwrap();

        assert_eq!(
            CONFIG.load(&deps.storage).unwrap(),
            read_config(&deps.storage).unwrap()
        );
    }

    const KEY_LATEST_STAGE: &[u8] = b"latest_stage";
    pub fn store_latest_stage(storage: &mut dyn Storage, stage: u8) -> StdResult<()> {
        singleton(storage, KEY_LATEST_STAGE).save(&stage)
    }
    pub fn read_latest_stage(storage: &dyn Storage) -> StdResult<u8> {
        singleton_read(storage, KEY_LATEST_STAGE).load()
    }

    #[test]
    fn latest_stage_legacy_compatibility() {
        let mut deps = mock_dependencies(&[]);
        store_latest_stage(&mut deps.storage, 5u8).unwrap();

        assert_eq!(
            LATEST_STAGE.load(&deps.storage).unwrap(),
            read_latest_stage(&deps.storage).unwrap()
        );
    }

    const PREFIX_MERKLE_ROOT: &[u8] = b"merkle_root";
    pub fn store_merkle_root(
        storage: &mut dyn Storage,
        stage: u8,
        merkle_root: String,
    ) -> StdResult<()> {
        let mut merkle_root_bucket: Bucket<String> = bucket(storage, PREFIX_MERKLE_ROOT);
        merkle_root_bucket.save(&[stage], &merkle_root)
    }
    pub fn read_merkle_root(storage: &dyn Storage, stage: u8) -> StdResult<String> {
        let merkle_root_bucket: ReadonlyBucket<String> = bucket_read(storage, PREFIX_MERKLE_ROOT);
        merkle_root_bucket.load(&[stage])
    }

    #[test]
    fn merkle_root_legacy_compatibility() {
        let mut deps = mock_dependencies(&[]);

        let merkle_root_1 = "123".to_string();
        let merkle_root_2 = "456".to_string();
        store_merkle_root(&mut deps.storage, 1, merkle_root_1).unwrap();
        store_merkle_root(&mut deps.storage, 2, merkle_root_2).unwrap();
        assert_eq!(
            MERKLE_ROOT.load(&deps.storage, &[1]).unwrap(),
            read_merkle_root(&deps.storage, 1).unwrap()
        );

        assert_eq!(
            MERKLE_ROOT.load(&deps.storage, &[2]).unwrap(),
            read_merkle_root(&deps.storage, 2).unwrap()
        );
    }

    const PREFIX_CLAIM_INDEX: &[u8] = b"claim_index";
    pub fn store_claim_index(
        storage: &mut dyn Storage,
        user: &CanonicalAddr,
        stage: u8,
    ) -> StdResult<()> {
        let mut claimed_index_bucket: Bucket<bool> =
            Bucket::multilevel(storage, &[PREFIX_CLAIM_INDEX, user.as_slice()]);
        claimed_index_bucket.save(&[stage], &true)
    }

    pub fn read_claim_index(
        storage: &dyn Storage,
        user: &CanonicalAddr,
        stage: u8,
    ) -> StdResult<bool> {
        let claim_index_bucket: ReadonlyBucket<bool> =
            ReadonlyBucket::multilevel(storage, &[PREFIX_CLAIM_INDEX, user.as_slice()]);
        claim_index_bucket.load(&[stage])
    }

    #[test]
    fn claim_index_legacy_compatibility() {
        let mut deps = mock_dependencies(&[]);

        let stage = 5u8;
        let addr = CanonicalAddr::from(vec![1u8, 2u8, 3u8]);

        store_claim_index(&mut deps.storage, &addr, stage).unwrap();
        assert_eq!(
            CLAIM_INDEX
                .load(&deps.storage, (addr.as_slice(), &[stage]))
                .unwrap(),
            read_claim_index(&deps.storage, &addr, stage).unwrap()
        );
    }
}
