use crate::{HorcrustSecret, HorcrustShare, HorcrustStoreKey};
use std::collections::HashMap;

pub struct SharesDatabase {
    shares: HashMap<HorcrustStoreKey, HorcrustShare>,
}

impl SharesDatabase {
    pub fn new() -> Self {
        Self {
            shares: HashMap::new(),
        }
    }
    pub fn insert<T: Into<HorcrustStoreKey>, S: Into<HorcrustShare>>(&mut self, key: T, share: S) {
        self.shares.insert(key.into(), share.into());
    }
    pub fn get<T: Into<HorcrustStoreKey>>(&self, key: T) -> Option<HorcrustShare> {
        // just to keep things easy, this get returns a copy of the value. Usually it should return a reference to it.
        self.shares.get(&key.into()).cloned()
    }
    pub fn modify<F, K: Into<HorcrustStoreKey>>(&mut self, key: K, f: F)
    where
        F: Fn(HorcrustShare) -> HorcrustShare,
    {
        let share = self.shares.get_mut(&key.into());
        if share.is_none() {
            return;
        }
        let share = share.unwrap();
        *share = f(*share);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AdditiveSecretSharing;

    #[test]
    fn test_database() {
        let secret_sharing = AdditiveSecretSharing::new();
        let mut db = SharesDatabase::new();
        let key = 0u32;
        let share = 1;
        let r = 2;
        db.insert(key, share);
        assert_eq!(db.get(key).unwrap(), share);

        db.modify(key, |share| share + r);
        assert_eq!(db.get(key).unwrap(), share + r);
    }
}
