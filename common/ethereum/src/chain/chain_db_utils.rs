use common::{utils::get_prefixed_db_key, DatabaseInterface};
use derive_getters::Getters;
use derive_more::Constructor;

/*
lazy_static! {
    static ref NAME_THIS_BETTER: [u8; 32] = crate::utils::get_prefixed_db_key($value);
}
*/

#[derive(Debug, Clone, PartialEq, Eq, Getters, Constructor)]
pub struct ChainDbUtils<'a, D: DatabaseInterface> {
    db: &'a D,
}
