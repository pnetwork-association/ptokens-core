#![allow(unused)] // FIXME rm!

use common::{get_prefixed_db_key, Byte, Bytes, DatabaseInterface, MIN_DATA_SENSITIVITY_LEVEL};

use crate::{
    relevant_logs::{HostRelevantLogs, NativeRelevantLogs},
    SentinelError,
};

type DbKey = [Byte; 32];

macro_rules! create_db_keys {
    ($($name:ident),* $(,)?) => {
        lazy_static! {
            $(
                static ref $name: DbKey = get_prefixed_db_key(stringify!($name));
            )*
        }

        impl<'a, D: DatabaseInterface> SentinelDbUtils<'a, D> {
            fn db(&self) -> &D {
                self.0
            }

            paste! {
                $(
                    fn [< get_ $name:lower _key >]() -> Bytes {
                        $name.to_vec()
                    }

                    fn [< get_ $name:lower >](&self) -> Result<[< $name:camel >], SentinelError> {
                        let result = self.db().get(
                            Self::[< get_ $name:lower _key >](),
                            MIN_DATA_SENSITIVITY_LEVEL
                        );

                        match result {
                            Ok(bytes) => [< $name:camel >]::try_from(bytes),
                            Err(e) => {
                                warn!("Error getting {} from db, defaulting to empty set!", stringify!([< $name:camel >]));
                                Ok([< $name:camel >]::default())
                            },
                        }

                    }

                    fn [< put_ $name:lower >](&self, thing: [< $name:camel >]) -> Result<(), SentinelError> {
                        self.db().put(
                            Self::[< get_ $name:lower _key >](),
                            thing.try_into()?,
                            MIN_DATA_SENSITIVITY_LEVEL
                        )?;
                        Ok(())
                    }
                )*
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;
            use std::time::Duration;
            use crate::{
                RelevantLogs,
                RelevantLogsFromBlock,
                get_utc_timestamp,
            };
            use common::get_test_database;

            paste! {
                $(
                    #[test]
                    fn [< should_get_empty_ $name:lower >]() {
                        let db = get_test_database();
                        let db_utils = SentinelDbUtils::new(&db);
                        let result = db_utils.[< get_ $name:lower >]().unwrap();
                        let expected_result = [< $name:camel >]::default();
                        assert_eq!(result, expected_result);
                    }

                    #[test]
                    fn [< should_get_and_put $name:lower in_db>]() {
                        let db = get_test_database();
                        let db_utils = SentinelDbUtils::new(&db);
                        let mut x = RelevantLogsFromBlock::default();
                        x.set_timestamp(Duration::new(1337, 0));
                        let mut expected_result = [< $name:camel >]::new(RelevantLogs::new(vec![x]));
                        db_utils.[< put_ $name:lower >](expected_result.clone()).unwrap();
                        let result = db_utils.[< get_ $name:lower >]().unwrap();
                        assert_eq!(result, expected_result);
                    }
                )*
            }
        }
    }
}

create_db_keys!(HOST_RELEVANT_LOGS, NATIVE_RELEVANT_LOGS,);

pub struct SentinelDbUtils<'a, D: DatabaseInterface>(&'a D);

impl<'a, D: DatabaseInterface> SentinelDbUtils<'a, D> {
    pub fn new(db: &'a D) -> Self {
        Self(db)
    }
}
