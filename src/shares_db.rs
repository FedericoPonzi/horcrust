use crate::{HorcrustShare, HorcrustStoreKey};
use std::collections::HashMap;
use std::time::{Duration, Instant};

const REFRESH_THRESHOLD: Duration = Duration::from_millis(5000);

#[derive(Default)]
pub struct SharesDatabase {
    shares: HashMap<HorcrustStoreKey, HorcrustShare>,
    shares_refresh: HashMap<HorcrustStoreKey, Instant>,
}

impl SharesDatabase {
    pub fn new() -> Self {
        Self {
            shares: HashMap::new(),
            shares_refresh: HashMap::new(),
        }
    }
    pub fn stale_keys(&self) -> Vec<HorcrustStoreKey> {
        self.shares_refresh
            .iter()
            .filter(|(_, t)| t.elapsed() > REFRESH_THRESHOLD)
            .map(|(k, _)| *k)
            .collect()
    }
    pub fn insert<T: Into<HorcrustStoreKey> + Copy, S: Into<HorcrustShare>>(
        &mut self,
        key: T,
        share: S,
    ) {
        self.shares.insert(key.into(), share.into());
        self.shares_refresh.insert(key.into(), Instant::now());
    }
    pub fn get<T: Into<HorcrustStoreKey>>(&self, key: T) -> Option<HorcrustShare> {
        // just to keep things easy, this get returns a copy of the value. Usually it should return a reference to it.
        self.shares.get(&key.into()).cloned()
    }
    pub fn modify<F, K: Into<HorcrustStoreKey> + Copy>(&mut self, key: K, f: F)
    where
        F: Fn(HorcrustShare) -> HorcrustShare,
    {
        let share = self.shares.get_mut(&key.into());
        if share.is_none() {
            return;
        }
        let share = share.unwrap();
        *share = f(*share);
        *self.shares_refresh.get_mut(&key.into()).unwrap() = Instant::now();
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
