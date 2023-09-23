use common::DatabaseInterface;
use derive_getters::Getters;
use derive_more::Constructor;

#[derive(Debug, Clone, PartialEq, Eq, Getters, Constructor)]
pub struct ChainDbUtils<'a, D: DatabaseInterface> {
    db: &'a D,
}
