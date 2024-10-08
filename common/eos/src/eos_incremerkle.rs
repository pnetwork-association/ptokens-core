use std::fmt;

use common::{
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    traits::DatabaseInterface,
    types::{NoneError, Result},
};
use derive_more::{Constructor, Deref, DerefMut};
use eos_chain::Checksum256;
use serde::{Deserialize, Serialize};

use crate::EosDbUtils;
#[cfg(not(feature = "spring_1-0"))]
use crate::EosState;

// NOTE: The light client for EOS doesn't keep blocks - they are too frequent and too numerous
// for efficient use in TEEs.
//
// Instead, we provide as the first trusted block one from the node which contains all the
// information to construct a merkle tree with the correct merkle root which is that block's ID.
//
// Going forward from that, we can submit a later, non-subsequent block, along with _all_ the block
// ID's between it and the previous one.
//
// Those IDs are then added to the incremerkle, whose root after those additions should now equal the
// currently-being-submitted block's ID. This is how we verify that that later block is indeed chained
// to the previous block, without having had to have seen every block in between.
//
// This does mean however that we can no no longer handle forks, because the incremerkle can never
// go backwards. And thus instead of keeping just the one incremerkle - that of the chain tip - we
// keep up to some X incremerkles around. This means we have a choice of incremerkle from which we can
// verifiy a new submission.

#[cfg(not(feature = "spring_1-0"))]
const MAX_NUM_INCREMERKLES: usize = 10;

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize, Constructor, Deref, DerefMut)]
pub struct Incremerkles(Vec<Incremerkle>);

impl Incremerkles {
    pub fn latest_block_id(&self) -> Result<Checksum256> {
        info!("getting latest block id from incremerkles...");
        self.get_incremerkle_at_index(0)
            .map(|incremerkle| incremerkle.get_root())
    }

    pub fn get_from_db<D: DatabaseInterface>(db_utils: &EosDbUtils<D>) -> Result<Self> {
        debug!("getting EOS incremerkles from db...");
        db_utils
            .get_db()
            .get(db_utils.get_eos_incremerkle_key(), MIN_DATA_SENSITIVITY_LEVEL)
            .and_then(|bytes| Ok(serde_json::from_slice(&bytes)?))
    }

    pub(crate) fn put_in_db<D: DatabaseInterface>(&self, db_utils: &EosDbUtils<D>) -> Result<()> {
        debug!("putting EOS incremerkles in db...");
        db_utils.get_db().put(
            db_utils.get_eos_incremerkle_key(),
            serde_json::to_vec(&self)?,
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    #[cfg(not(feature = "spring_1-0"))]
    pub fn add_block_ids_and_return_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
        state
            .incremerkles
            .add_block_ids(
                &EosDbUtils::new(state.db),
                state.get_eos_block_num()? as usize,
                state.interim_block_ids.clone(),
            )
            .map(|i| state.add_incremerkles(i))
    }

    pub(crate) fn get_incremerkle_for_block_number(&self, block_num: u64) -> Result<Incremerkle> {
        info!("getting incremerkle for block num {block_num}...");
        self.get_incremerkle_index_for_block_num(block_num)
            .and_then(|idx| self.get_incremerkle_at_index(idx))
    }

    fn get_incremerkle_at_index(&self, idx: usize) -> Result<Incremerkle> {
        info!("getting incremerkle at index: {idx}...");
        self.get(idx)
            .cloned()
            .ok_or_else(|| format!("no incremerkle at index {idx}").into())
    }

    fn get_incremerkle_index_for_block_num(&self, block_num: u64) -> Result<usize> {
        info!("getting incremerkle index for block num {block_num}...");
        self.block_nums()
            .iter()
            .position(|x| *x == block_num)
            .ok_or_else(|| format!("no incremerkle found for block num {block_num}").into())
    }

    // TODO Make more efficient my taking &mut self, however that makes using the above
    // state-version of this more difficult to manage for the caller.
    #[cfg(not(feature = "spring_1-0"))]
    fn add_block_ids<D: DatabaseInterface>(
        &self,
        eos_db_utils: &EosDbUtils<D>,
        block_num: usize,
        ids: Vec<Checksum256>,
    ) -> Result<Self> {
        info!("adding block ids to incremerkle...");
        let mut mutable_self = self.clone();

        let num_ids = ids.len();
        let incremerkle_block_num = if block_num > num_ids {
            (block_num - num_ids) as u64
        } else {
            0
        };

        debug!("              num ids: {num_ids}");
        debug!("    sub mat block num: {block_num}");
        debug!("incremerkle block num: {incremerkle_block_num}");

        let idx = self.get_incremerkle_index_for_block_num(incremerkle_block_num)?;
        let mut incremerkle = mutable_self.get_incremerkle_at_index(idx)?;

        for id in ids.iter() {
            incremerkle.append(*id)?;
        }

        if idx == 0 {
            // NOTE: This adds the new incremerkle to the front of the incremerkles, and
            // removes the oldest one, then saves the structure back to the db.
            mutable_self.add(incremerkle);
            mutable_self.put_in_db(eos_db_utils)?;
        } else {
            // NOTE: This just replaces the incermerkle in question in the Incremerkles vec
            // without persisting the changes in the db.
            let _ = std::mem::replace(&mut mutable_self[idx], incremerkle);
        }

        Ok(mutable_self)
    }

    #[cfg(not(feature = "spring_1-0"))]
    fn add(&mut self, incremerkle: Incremerkle) {
        if incremerkle.block_num() > self.latest_block_num() || self.is_empty() {
            info!("adding new incremerkle to list");
            self.insert(0, incremerkle);
            self.truncate(MAX_NUM_INCREMERKLES);
        } else {
            warn!("not adding incremerkle to list because its block num is behind chain tip")
        }
    }

    fn block_nums(&self) -> Vec<u64> {
        self.iter().map(|i| i.block_num()).collect()
    }

    pub fn previous_block_nums(&self) -> Vec<u64> {
        // NOTE: We skip the first one since that's the latest block number.
        self.iter().skip(1).map(|i| i.block_num()).collect()
    }

    pub fn latest_block_num(&self) -> u64 {
        if self.is_empty() {
            warn!("no incremerkles to get latest block number from");
            0
        } else {
            self.0[0].block_num()
        }
    }

    #[cfg(not(feature = "spring_1-0"))]
    pub fn get_from_db_and_add_to_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
        info!("getting eos incremerkle from db...");
        Self::get_from_db(&state.eos_db_utils).map(|i| state.add_incremerkles(i))
    }

    #[cfg(not(feature = "spring_1-0"))]
    pub fn save_from_state_to_db<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
        info!("saving incremerkles from state to db...");
        state.incremerkles.put_in_db(&state.eos_db_utils).and(Ok(state))
    }
}

impl fmt::Display for Incremerkles {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let js = self.iter().map(IncremerkleJson::from).collect::<Vec<_>>();
        write!(f, "{}", serde_json::json!(js))
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Incremerkle {
    node_count: u64,
    active_nodes: Vec<Checksum256>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IncremerkleJson {
    node_count: u64,
    active_nodes: Vec<String>,
    active_node_sides: Vec<String>,
}

impl From<&Incremerkle> for IncremerkleJson {
    fn from(i: &Incremerkle) -> Self {
        Self {
            node_count: i.node_count,
            active_nodes: i
                .active_nodes
                .iter()
                .map(|c| hex::encode(c.as_bytes()))
                .collect::<Vec<String>>(),
            active_node_sides: i
                .active_nodes
                .iter()
                .map(|c| if Incremerkle::is_canonical_left(c) { "l" } else { "r" }.to_string())
                .collect::<Vec<String>>(),
        }
    }
}

impl fmt::Display for Incremerkle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::json!(IncremerkleJson::from(self)))
    }
}

// NOTE: A lot of this comes from: https://github.com/EOSIO/eos/blob/11d35f0f934402321853119d36caeb7022813743/libraries/chain/include/eosio/chain/incremental_merkle.hpp#L95
impl Incremerkle {
    fn block_num(&self) -> u64 {
        self.node_count()
    }

    fn node_count(&self) -> u64 {
        self.node_count
    }

    fn make_canonical_left(val: &Checksum256) -> Checksum256 {
        let mut r = *val;
        r.0[0] &= 0x7f;
        r
    }

    fn make_canonical_right(val: &Checksum256) -> Checksum256 {
        let mut r = *val;
        r.0[0] |= 0x80;
        r
    }

    pub(crate) fn make_canonical_pair(l: &Checksum256, r: &Checksum256) -> (Checksum256, Checksum256) {
        (Self::make_canonical_left(l), Self::make_canonical_right(r))
    }

    // NOTE: Some logic in here courtesy of: https://github.com/bifrost-codes/rust-eos/
    // NOTE: Given a power-of-2 (assumed to be correct) return the number of leading zeros.
    //
    // This is a classic count-leading-zeros in parallel without the necessary
    // math to make it safe for anything that is not already a power-of-2
    //
    // Note that `count_leading_zeroes_of_a_power_of_2(0)` returns 64 because the values are
    // 64 bit integers, and so in the case of 0, there are indeed 64 leading zeroes, despite `0`
    // not actually being a power of 2.
    //
    // @param value - a power-of-2 (unchecked)
    // @return the number of leading zeros
    fn count_leading_zeroes_of_a_power_of_2(value: u64) -> Result<usize> {
        if value > 9223372036854775808 {
            return Err(format!("Cannot count leading zeres of {} without overflowing!", value).into());
        };
        let mut leading_zeroes: usize = 64;
        if value != 0 {
            leading_zeroes -= 1;
        }
        if (value & 0x0000_0000_FFFF_FFFF_u64) != 0 {
            leading_zeroes -= 32;
        }
        if (value & 0x0000_FFFF_0000_FFFF_u64) != 0 {
            leading_zeroes -= 16;
        }
        if (value & 0x00FF_00FF_00FF_00FF_u64) != 0 {
            leading_zeroes -= 8;
        }
        if (value & 0x0F0F_0F0F_0F0F_0F0F_u64) != 0 {
            leading_zeroes -= 4;
        }
        if (value & 0x3333_3333_3333_3333_u64) != 0 {
            leading_zeroes -= 2;
        }
        if (value & 0x5555_5555_5555_5555_u64) != 0 {
            leading_zeroes -= 1;
        }
        Ok(leading_zeroes)
    }

    // NOTE: Given a number of nodes return the depth required to store them
    // in a fully balanced binary tree.
    //
    // @param node_count - the number of nodes in the implied tree
    // @return the max depth of the minimal tree that stores them
    fn calculate_max_depth(node_count: u64) -> Result<usize> {
        if node_count == 0 {
            return Ok(0);
        }
        let implied_count =
            u64::checked_next_power_of_two(node_count).ok_or(NoneError("Next power of two has overflowed!"))?;
        Ok(Self::count_leading_zeroes_of_a_power_of_2(implied_count)? + 1)
    }

    pub(crate) fn new(node_count: u64, active_nodes: Vec<Checksum256>) -> Self {
        Incremerkle {
            node_count,
            active_nodes,
        }
    }

    pub fn append(&mut self, digest: Checksum256) -> Result<Checksum256> {
        let mut partial = false;
        let max_depth = Self::calculate_max_depth(self.node_count + 1)?;
        let mut current_depth = max_depth - 1;
        let mut index = self.node_count;
        let mut top = digest;
        let mut active_iter = self.active_nodes.iter();
        let mut updated_active_nodes: Vec<Checksum256> = Vec::with_capacity(max_depth);

        while current_depth > 0 {
            if index % 2 == 0 {
                // we are collapsing from a "left" value and an implied "right" creating a partial node

                // we only need to append this node if it is fully-realized and by definition
                // if we have encountered a partial node during collapse this cannot be
                // fully-realized
                if !partial {
                    updated_active_nodes.push(top);
                }

                // calculate the partially realized node value by implying the "right" value is identical
                // to the "left" value
                top = Checksum256::hash(Self::make_canonical_pair(&top, &top))?;
                partial = true;
            } else {
                // we are collapsing from a "right" value and an fully-realized "left"

                // pull a "left" value from the previous active nodes
                let left_value = active_iter.next().unwrap();

                // if the "right" value is a partial node we will need to copy the "left" as future appends still need
                // it otherwise, it can be dropped from the set of active nodes as we are collapsing a
                // fully-realized node
                if partial {
                    updated_active_nodes.push(*left_value);
                }

                // calculate the node
                top = Checksum256::hash(Self::make_canonical_pair(left_value, &top))?;
            }

            // move up a level in the tree
            current_depth -= 1;
            //index = index >> 1;
            index /= 2;
        }

        // append the top of the collapsed tree (aka the root of the merkle)
        updated_active_nodes.push(top);

        // store the new active_nodes
        self.active_nodes = updated_active_nodes;

        // update the node count
        self.node_count += 1;
        Ok(self.active_nodes[self.active_nodes.len() - 1])
    }

    pub(crate) fn get_root(&self) -> Checksum256 {
        if self.node_count > 0 {
            self.active_nodes[self.active_nodes.len() - 1]
        } else {
            Default::default()
        }
    }

    pub fn is_canonical_left(val: &Checksum256) -> bool {
        (val.hash0() & 0x0000000000000080u64) == 0
    }

    pub fn is_canonical_right(val: &Checksum256) -> bool {
        (val.hash0() & 0x0000000000000080u64) != 0
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::needless_range_loop)]
    use std::str::FromStr;

    #[cfg(not(feature = "spring_1-0"))]
    use common::test_utils::get_test_database;
    use common::types::{Byte, Bytes};
    use eos_chain::{AccountName, Action, ActionName, PermissionLevel, PermissionName, SerializeData};

    use super::*;
    use crate::{
        bitcoin_crate_alias::hashes::{sha256, Hash},
        core_initialization::EosInitJson,
        eos_action_receipt::{AuthSequence, EosActionReceipt},
        eos_test_utils::{get_sample_action_digests, get_sample_eos_submission_material_n},
        eos_utils::convert_hex_to_checksum256,
        EosSubmissionMaterial,
        MerkleProof,
    };

    fn get_expected_digest_hex_1() -> &'static str {
        "9b9babebfbdff48ce4002b5f3c7f999c0ee74707b6d121c47ef5db68c6be7262"
    }

    fn get_expected_digest_hex_2() -> &'static str {
        "122cd09d66ca7df007a35bd9c9be5484833f1a69ad0c8527c3e2a56b6955e761"
    }

    fn get_expected_digest_1() -> Checksum256 {
        convert_hex_to_checksum256(get_expected_digest_hex_1()).unwrap()
    }

    fn get_expected_digest_2() -> Checksum256 {
        convert_hex_to_checksum256(get_expected_digest_hex_2()).unwrap()
    }

    fn get_expected_first_byte_1() -> Byte {
        0b0001_1011
    }

    fn get_expected_first_byte_2() -> Byte {
        0b1001_0010
    }

    pub(crate) fn get_merkle_digest(mut leaves: Vec<Bytes>) -> Bytes {
        if leaves.is_empty() {
            return vec![0x00];
        }
        while leaves.len() > 1 {
            if leaves.len() % 2 != 0 {
                let last = leaves[leaves.len() - 1].clone();
                leaves.push(last);
            }
            for i in 0..(leaves.len() / 2) {
                leaves[i] = MerkleProof::hash_canonical_pair(Incremerkle::make_canonical_pair(
                    &convert_hex_to_checksum256(hex::encode(&leaves[2 * i])).unwrap(),
                    &convert_hex_to_checksum256(hex::encode(&leaves[(2 * i) + 1])).unwrap(),
                ))
                .unwrap()
                .as_bytes()
                .to_vec();
            }
            leaves.resize(leaves.len() / 2, vec![0x00]);
        }
        leaves[0].clone()
    }

    #[test]
    fn should_make_hash_canonical_right() {
        let hash = get_expected_digest_2();
        let result = Incremerkle::make_canonical_right(&hash);
        for i in 0..hash.0.len() {
            if i == 0 {
                assert_eq!(result.0[i], get_expected_first_byte_2());
            } else {
                assert_eq!(result.0[i], hash.0[i]);
            }
        }
    }

    #[test]
    fn should_make_hash_canonical_left() {
        let hash = get_expected_digest_1();
        let result = Incremerkle::make_canonical_left(&hash);
        for i in 0..hash.0.len() {
            if i == 0 {
                assert_eq!(result.0[i], get_expected_first_byte_1());
            } else {
                assert_eq!(result.0[i], hash.0[i]);
            }
        }
    }

    #[test]
    fn canonical_left_hash_should_be_canonical_left() {
        let hash = get_expected_digest_1();
        let canonical_left_hash = Incremerkle::make_canonical_left(&hash);
        let is_left = Incremerkle::is_canonical_left(&canonical_left_hash);
        let is_right = Incremerkle::is_canonical_right(&canonical_left_hash);
        assert!(is_left);
        assert!(!is_right);
    }

    #[test]
    fn canonical_right_hash_should_be_canonical_right() {
        let hash = get_expected_digest_2();
        let canonical_right_hash = Incremerkle::make_canonical_right(&hash);
        let is_left = Incremerkle::is_canonical_left(&canonical_right_hash);
        let is_right = Incremerkle::is_canonical_right(&canonical_right_hash);
        assert!(!is_left);
        assert!(is_right);
    }

    #[test]
    fn should_get_correct_action_digest() {
        let account_name = AccountName::from_str("provabletokn").unwrap();
        let action_name = ActionName::from_str("event").unwrap();
        let actor = AccountName::from_str("provabletokn").unwrap();
        let permission = PermissionName::from_str("active").unwrap();
        let permission_level = PermissionLevel { actor, permission };
        let authorization = vec![permission_level];
        let data = hex::decode(
            "e0d2b86b1a3962343021cd2a1eb3e9ad672b00000000000004454f53000000002a3078303236644336413433353631444138413641373735353338623139324133653933366330463239422301000000000000"
            ).unwrap();
        let action = Action {
            account: account_name,
            name: action_name,
            authorization,
            data,
        };
        let serialized_action = action.to_serialize_data().unwrap();
        let result = sha256::Hash::hash(&serialized_action).to_vec();
        assert_eq!(result, get_expected_digest_1().0.to_vec());
    }

    #[test]
    fn should_serialize_a_simple_action_receipt_correctly() {
        let expected_result = "6cd473b189a292bd520cac3430cc7934273da81cc3417376194a5d757b4abdc8".to_string();
        let result = EosActionReceipt::new(
            "eosio",
            "a6a370c6569034a4cc41935dd88f83d1c64e0414580872f29d87f69fe7a5d769",
            60725518,
            12,
            498637597,
            10,
            vec![AuthSequence::new("eosio", 59191700).unwrap()],
        )
        .unwrap()
        .to_digest()
        .unwrap();
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_get_merkle_root_for_an_even_number_of_action_receipts() {
        // NOTE: Test vector = https://jungle.bloks.io/block/58316764
        let expected_result = "2f013d3ed57c89f1824772d18a4a74c043574bad47e9c6f088136e7595511810";
        let action_digest_1 = EosActionReceipt::new(
            "eosio",
            "8e3e721a497dbae5e5fde0bb43e9086628809efaf102b763a3e9820adce9ce8f",
            62815613,
            12,
            503056735,
            10,
            vec![AuthSequence::new("eosio", 61275209).unwrap()],
        )
        .unwrap()
        .to_digest()
        .unwrap();
        let action_digest_2 = EosActionReceipt::new(
            "provabletokn",
            "4b991cebb3e6667b242aca3fb011623cd8ce2be2e8c24958da551c7b3ba68903",
            2884,
            48,
            503056736,
            80,
            vec![AuthSequence::new("provabletokn", 3090).unwrap()],
        )
        .unwrap()
        .to_digest()
        .unwrap();
        let result = get_merkle_digest(vec![action_digest_1, action_digest_2]);
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_get_merkle_root_for_an_odd_number_of_action_receipts_gt_one() {
        // NOTE: Test vector = https://jungle.bloks.io/block/58319528
        let expected_result = "593f54cbc0b877b30cec5e510838b2b16ca00aca43e21d204d21eb8e8f947aa0";
        let action_digest_1 = EosActionReceipt::new(
            "eosio",
            "23ab74b930cceea6061e1c4580ec988bf483a77e225cfca254d832928b4d1b36",
            62818486,
            12,
            503062766,
            10,
            vec![AuthSequence::new("eosio", 61277973).unwrap()],
        )
        .unwrap()
        .to_digest()
        .unwrap();
        let action_digest_2 = EosActionReceipt::new(
            "eosebetbullj",
            "b9243d8513e25705e89d7ccd0491f4a57d07b9866fd89d3446887af852cfed15",
            1440226,
            215,
            503062767,
            215,
            vec![AuthSequence::new("eosebetbullj", 1440687).unwrap()],
        )
        .unwrap()
        .to_digest()
        .unwrap();
        let action_digest_3 = EosActionReceipt::new(
            "dvmh1tbb1him",
            "4bd1d3e987cd13e3d108a9a0cd185bf022cb1a826f69f163fcd109db54ba799f",
            804629,
            2,
            503062768,
            1,
            vec![AuthSequence::new("dvmh1tbb1him", 804649).unwrap()],
        )
        .unwrap()
        .to_digest()
        .unwrap();
        let result = get_merkle_digest(vec![action_digest_1, action_digest_2, action_digest_3]);
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_get_action_mroot_when_action_has_gt_one_auth_sequence() {
        // NOTE: Test vector = https://jungle.bloks.io/block/58345436
        let expected_result = "f93a91688d12170c24807d4bd507cf52dcde962ae4a41a86fe55231dee4df348";
        let action_receipt_1 = EosActionReceipt::new(
            "eosio",
            "2d5371b958af052629f3fb62ede1bfcd94703675bc734535bf87fb615284dba3",
            62844592,
            12,
            503124645,
            10,
            vec![AuthSequence::new("eosio", 61303891).unwrap()],
        )
        .unwrap()
        .to_digest()
        .unwrap();
        let action_receipt_2 = EosActionReceipt::new(
            "authsequence",
            "ae341469a7b3936c70e9684a42ef8fc1975f1bb2fe1f3b0b1105eda7d3a6276a",
            10,
            1,
            503124646,
            1,
            vec![
                AuthSequence::new("othrsequence", 14).unwrap(),
                AuthSequence::new("rick11111111", 268).unwrap(),
            ],
        )
        .unwrap()
        .to_digest()
        .unwrap();
        let result = get_merkle_digest(vec![action_receipt_1, action_receipt_2]);
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_get_action_mroot_for_four_actions_correctly() {
        let digests = get_sample_action_digests().unwrap();
        let expected_result = "8b4e5e5d3e7587065896d0076d65c72e03c11a9159d414eb3a2363b59108116a";
        let result = get_merkle_digest(digests);
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_get_incremerkle_root_from_interim_block_idss() {
        let expected_incremerkle_root = "1894edef851c070852f55a4dc8fc50ea8f2eafc67d8daad767e4f985dfe54071";
        let submission_material = get_sample_eos_submission_material_n(5);
        let active_nodes = submission_material.interim_block_ids.clone();
        let node_count: u64 = submission_material.block_header.block_num().into();
        let incremerkle = Incremerkle::new(node_count, active_nodes);
        let incremerkle_root = hex::encode(incremerkle.get_root().to_bytes());
        assert_eq!(incremerkle_root, expected_incremerkle_root);
    }

    #[test]
    fn should_count_leading_zeroes_of_powers_of_2() {
        let num_digits_in_u64: u32 = 64;
        let mut results: Vec<u32> = vec![];
        let mut expected_results: Vec<u32> = vec![];
        for i in 0..num_digits_in_u64 {
            let two_to_the_power_of_i = 2_u64.pow(i);
            let result = Incremerkle::count_leading_zeroes_of_a_power_of_2(two_to_the_power_of_i).unwrap();
            results.push(result.try_into().unwrap());
            expected_results.push(i)
        }
        results
            .iter()
            .zip(expected_results.iter())
            .for_each(|(result, expected_result)| assert_eq!(result, expected_result))
    }

    #[cfg(not(feature = "spring_1-0"))]
    #[test]
    fn should_only_allow_max_num_incremerkles() {
        let mut incremerkles = Incremerkles::default();
        for i in 0..MAX_NUM_INCREMERKLES + 10 {
            incremerkles.add(Incremerkle::new(i as u64, vec![]));
            assert_eq!(incremerkles.latest_block_num(), i as u64);
            if i < MAX_NUM_INCREMERKLES {
                assert_eq!(incremerkles.len(), i + 1)
            } else {
                assert_eq!(incremerkles.len(), MAX_NUM_INCREMERKLES)
            }
        }
    }

    #[cfg(not(feature = "spring_1-0"))]
    #[test]
    fn should_put_and_get_incremerkles_in_db() {
        let mut incremerkles = Incremerkles::default();
        for i in 0..MAX_NUM_INCREMERKLES {
            incremerkles.add(Incremerkle::new(i as u64, vec![]));
        }
        let db = get_test_database();
        let db_utils = EosDbUtils::new(&db);
        incremerkles.put_in_db(&db_utils).unwrap();
        let incremerkles_from_db = Incremerkles::get_from_db(&db_utils).unwrap();
        assert_eq!(incremerkles, incremerkles_from_db);
        assert_eq!(incremerkles.block_nums(), incremerkles_from_db.block_nums());
    }

    #[cfg(not(feature = "spring_1-0"))]
    #[test]
    fn should_only_add_subsequent_incremerkles() {
        let mut incremerkles = Incremerkles::default();
        assert_eq!(incremerkles.len(), 0);
        let i1 = Incremerkle::new(1, vec![]);
        let i2 = Incremerkle::new(2, vec![]);
        let i3 = Incremerkle::new(3, vec![]);
        incremerkles.add(i2);
        assert_eq!(incremerkles.len(), 1);
        assert_eq!(incremerkles.latest_block_num(), 2);
        incremerkles.add(i3);
        assert_eq!(incremerkles.len(), 2);
        assert_eq!(incremerkles.latest_block_num(), 3);
        incremerkles.add(i1);
        assert_eq!(incremerkles.len(), 2);
        assert_eq!(incremerkles.latest_block_num(), 3);
    }

    #[test]
    #[ignore] // TODO This failing test shows the bug in the incremerkle impl.
    fn should_calculate_correct_merkle_root() {
        // NOTE: An "init" block from the eos node includes the full blockroot merkle for that
        // block, which - as per the spec - includes the root as the last element in the
        // incremerkle's active node list
        let init_block_1 = EosInitJson::from_str(&crate::eos_test_utils::get_init_block_352283689()).unwrap();
        let init_block_1_num = init_block_1.block.block_num;
        // NOTE: So now, when we validate this block, we do so using the above root, and thus if
        // this passes without error, we know the correct signing key was recovered and thus the
        // merkle root is correct at this point.
        init_block_1.validate();
        assert_eq!(init_block_1_num, 352283689);

        let init_block_2 = EosInitJson::from_str(&crate::eos_test_utils::get_init_block_352283690()).unwrap();
        let init_block_2_num = init_block_2.block.block_num;
        // NOTE: See above. Now we have two subsequent blocks whose merkle roots we know and are
        // verifiably correct.
        init_block_2.validate();
        assert_eq!(init_block_2_num, init_block_1_num + 1);

        let mut incremerkle_1 = Incremerkle::new(
            init_block_1.block.block_num,
            init_block_1
                .blockroot_merkle
                .iter()
                .map(|x| convert_hex_to_checksum256(x).unwrap())
                .collect::<Vec<_>>()
                .clone(),
        );
        let mroot_1 = incremerkle_1.get_root();
        let expected_mroot_1 =
            Checksum256::from_str("e0797a3e5bc13e5c1ed92093ae686b88d8d800678cca1a80fa5466a285792143").unwrap();
        assert_eq!(mroot_1, expected_mroot_1);

        let incremerkle_2 = Incremerkle::new(
            init_block_2.block.block_num,
            init_block_2
                .blockroot_merkle
                .iter()
                .map(|x| convert_hex_to_checksum256(x).unwrap())
                .collect::<Vec<_>>()
                .clone(),
        );
        let mroot_2 = incremerkle_2.get_root();
        let expected_mroot_2 =
            Checksum256::from_str("3e529c17df12ed8c7ccd90bca419a5cbb0609b7fecd15afd8f4a3d1491130775").unwrap();
        assert_eq!(mroot_2, expected_mroot_2);

        // NOTE: So now we _know_ the above two merkle roots are correct. We also know that due to
        // the way the incremerkle works, if we take the block ID of the second block and append it
        // to the first incremerkle, the root should be updated to the second root above.
        let second_block_id = EosSubmissionMaterial::parse_eos_block_header_from_json(&init_block_2.block)
            .unwrap()
            .id()
            .unwrap();
        let expected_second_block_id =
            Checksum256::from_str("14ff6c2a6731de810dd516112096aead0c3da4dd13854843b4b4d0f788239bcd").unwrap();
        assert_eq!(second_block_id, expected_second_block_id);

        incremerkle_1.append(second_block_id).unwrap();
        let new_root = incremerkle_1.get_root();
        assert_eq!(new_root, expected_mroot_2);
    }
}
