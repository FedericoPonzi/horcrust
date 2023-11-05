use rand::Rng;

pub trait SecretSharing {
    fn split(&self, shares: usize, secret: u64) -> Vec<u64>;
    fn combine(&self, shares: Vec<u64>) -> u64;
    // Used by the server side to refresh the secret.
    fn refresh_shares(&self, r: u64, secret: u64) -> u64;
}

/// This simple implementation assumes that the original secret is always less than q.
const Q: u64 = 431;
pub struct AdditiveSecretSharing {
    q: u64,
}
impl AdditiveSecretSharing {
    fn new() -> AdditiveSecretSharing {
        AdditiveSecretSharing { q: Q }
    }
}

impl SecretSharing for AdditiveSecretSharing {
    fn split(&self, shares: usize, secret: u64) -> Vec<u64> {
        let mut rng = rand::thread_rng();
        let mut ret = vec![];

        // Generate random shares
        for _ in 0..shares - 1 {
            let share = rng.gen_range(0..self.q);
            ret.push(share);
        }

        // Calculate the last share to make the sum equal to the secret
        let first = secret as i64 - ret.iter().sum::<u64>() as i64;
        ret.push(first.rem_euclid(self.q as i64) as u64);
        ret
    }
    fn combine(&self, shares: Vec<u64>) -> u64 {
        // returns sum(shares) % Q
        shares
            .iter()
            .fold(0, |acc, &share| (acc + share).rem_euclid(self.q))
    }

    fn refresh_shares(&self, r: u64, secret: u64) -> u64 {
        (r + secret).rem_euclid(self.q)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    #[test]
    fn test_additive_secret_sharing() {
        let secret_sharing = AdditiveSecretSharing::new();
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

        let mut shares: Vec<u64> = secret_sharing
            .split(3, 10)
            .into_iter()
            .map(|secret| secret_sharing.refresh_shares(2, secret))
            .collect();
        if let Some(v) = shares.last_mut() {
            *v = (*v - 6).rem_euclid(Q);
        }
        let combined_secret = secret_sharing.combine(shares);
        assert_eq!(combined_secret, 10);
    }
}
