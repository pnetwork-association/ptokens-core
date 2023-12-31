use common::{traits::DatabaseInterface, types::Result};

use crate::AlgoState;

pub fn check_parent_of_algo_block_in_state_exists<D: DatabaseInterface>(state: AlgoState<D>) -> Result<AlgoState<D>> {
    info!("✔ Checking if ALGO submission material's parent exists in database...");
    if cfg!(feature = "non-validating") {
        warn!("✘ Core is NOT validating ∴ skipping ALGO header-hash subsequency check!");
        Ok(state)
    } else {
        let sub_mat = state.get_algo_submission_material()?;
        let parent_hash = sub_mat.block.get_previous_block_hash()?;
        if state.algo_db_utils.get_submission_material(&parent_hash).is_ok() {
            info!("✔ ALGO submission material's parent exists in database!");
            Ok(state)
        } else {
            let n = sub_mat.block.round();
            let m = format!("✘ ALGO submission material rejected - no parent for block {n} exists in database!");
            Err(m.into())
        }
    }
}

#[cfg(all(test, not(feature = "non-validating")))]
mod tests {
    use common::{errors::AppError, test_utils::get_test_database};

    use super::*;
    use crate::test_utils::get_sample_contiguous_submission_material;

    #[test]
    fn should_check_parent_exists_correctly() {
        let db = get_test_database();
        let state = AlgoState::init(&db);
        let submission_materials = get_sample_contiguous_submission_material();
        submission_materials.iter().for_each(|material| {
            state
                .algo_db_utils
                .put_algo_submission_material_in_db(material)
                .unwrap()
        });
        let expected_error = "✘ ALGO submission material rejected - no parent for block 17962555 exists in database!";
        submission_materials.iter().enumerate().for_each(|(i, material)| {
            let state = AlgoState::init(&db).add_algo_submission_material(material).unwrap();
            if i == 0 {
                match check_parent_of_algo_block_in_state_exists(state) {
                    Ok(_) => panic!("Should not have succeeded!"),
                    Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
                    Err(_) => panic!("Wrong error received!"),
                }
            } else {
                assert!(check_parent_of_algo_block_in_state_exists(state).is_ok())
            }
        })
    }
}
