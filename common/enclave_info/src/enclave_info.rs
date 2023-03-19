use common::{
    constants::{DB_KEY_PREFIX, MAX_FEE_BASIS_POINTS},
    traits::DatabaseInterface,
    types::Result,
    utils::get_core_version,
    CoreType,
};
use common_debug_signers::DebugSignatories;
use serde::Serialize;
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize)]
pub struct EnclaveInfo {
    core_version: String,
    db_key_prefix: String,
    max_fee_basis_points: u64,
    core_type: Option<CoreType>,
    debug_signatories: JsonValue,
}

impl EnclaveInfo {
    fn init<D: DatabaseInterface>(db: &D, core_type: Option<CoreType>) -> Result<Self> {
        Ok(Self {
            core_type,
            core_version: get_core_version(),
            db_key_prefix: DB_KEY_PREFIX.to_string(),
            max_fee_basis_points: MAX_FEE_BASIS_POINTS,
            debug_signatories: DebugSignatories::get_from_db(db)?.to_enclave_state_json(),
        })
    }

    pub fn new<D: DatabaseInterface>(db: &D) -> Result<Self> {
        Self::init(db, None)
    }

    pub fn new_with_core_type<D: DatabaseInterface>(db: &D, core_type: CoreType) -> Result<Self> {
        Self::init(db, Some(core_type))
    }
}
