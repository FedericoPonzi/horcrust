use crate::{HorcrustSecret, HorcrustShare};
use rand::Rng;

pub trait SecretSharing {
    fn split(&self, shares: usize, secret: HorcrustSecret) -> Vec<HorcrustShare>;
    fn combine(&self, shares: Vec<HorcrustShare>) -> HorcrustSecret;
    // Used by the server side to refresh the secret.
    fn refresh_share(&self, r: HorcrustShare, share: HorcrustShare) -> HorcrustShare;
    fn generate_refreshers(&self, shares: usize) -> Vec<HorcrustShare>;
    fn limit(&self) -> Option<u64>;
}
/// This simple implementation assumes that the original secret is always less than q.
const Q: u64 = 431;

pub struct AdditiveSecretSharing {
    q: u64,
}
impl Default for AdditiveSecretSharing {
    fn default() -> Self {
        Self { q: Q }
    }
}
impl SecretSharing for AdditiveSecretSharing {
    fn split(&self, shares: usize, secret: HorcrustSecret) -> Vec<HorcrustShare> {
        let mut rng = rand::thread_rng();
        let mut ret = vec![];

        // Generate random shares
        for _ in 0..shares - 1 {
            let share = rng.gen_range(0..self.q);
            ret.push(share as HorcrustShare);
        }

        // Calculate the last share to make the sum equal to the secret
        let last = secret as i64 - ret.iter().sum::<u64>() as i64;
        ret.push(last.rem_euclid(self.q as i64) as HorcrustShare);
        ret
    }

    fn combine(&self, shares: Vec<HorcrustShare>) -> HorcrustSecret {
        // returns sum(shares) % Q
        shares
            .iter()
            .fold(0, |acc, &share| (acc + share).rem_euclid(self.q))
    }

    fn refresh_share(&self, r: HorcrustShare, share: HorcrustShare) -> HorcrustShare {
        (r + share).rem_euclid(self.q)
    }
    fn generate_refreshers(&self, shares: usize) -> Vec<HorcrustShare> {
        assert!(shares > 1);
        let mut rng = rand::thread_rng();
        let mut ret = vec![];
        for _ in 0..shares - 1 {
            let share = rng.gen_range(0..self.q);
            ret.push(share as HorcrustShare);
        }

        let summed = -(ret.iter().sum::<u64>() as i64);
        ret.push(summed.rem_euclid(self.q as i64) as u64);
        ret
    }

    fn limit(&self) -> Option<u64> {
        Some(self.q)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    #[test]
    fn test_additive_secret_sharing() {
        let secret_sharing = AdditiveSecretSharing::default();
        let shares = secret_sharing.split(3, 10);
        let combined_secret = secret_sharing.combine(shares);
        assert_eq!(combined_secret, 10);
        let shares = secret_sharing.split(3, 0);
        let combined_secret = secret_sharing.combine(shares);
        assert_eq!(combined_secret, 0);
        let shares = secret_sharing.split(3, Q + 1);
        let combined_secret = secret_sharing.combine(shares);
        assert_eq!(combined_secret, 1);
        let shares = secret_sharing.split(3, Q - 1);
        let combined_secret = secret_sharing.combine(shares);
        assert_eq!(combined_secret, Q - 1);

        let shares_count = 3;
        let mut shares: Vec<u64> = secret_sharing.split(shares_count, 10);

        // generate random refreshers:
        let refreshes = secret_sharing.generate_refreshers(shares_count);
        // apply them to the shares
        let shares: Vec<HorcrustShare> = shares
            .into_iter()
            .zip(refreshes.clone())
            .map(|(share, r)| secret_sharing.refresh_share(r, share))
            .collect();
        // try to recombine
        let combined_secret = secret_sharing.combine(shares);
        assert_eq!(combined_secret, 10);
    }
}
