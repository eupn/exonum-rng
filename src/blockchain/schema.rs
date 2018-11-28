use exonum::{
    crypto::{Hash, PublicKey},
    storage::{ProofMapIndex, Snapshot, Fork, Entry },
};

/// For simplicity, big integers are stored as strings
pub type BigInt = String;

#[derive(Debug)]
pub struct RngSchema<T> {
    view: T
}

impl<T> RngSchema<T> {
    pub fn new(snapshot: T) -> RngSchema<T> {
        RngSchema { view: snapshot }
    }

    pub fn into_view(self) -> T {
        self.view
    }
}

impl<T> RngSchema<T> where T: AsRef<Snapshot> {
    /// Maps validators to their commitments to a seed value.
    ///
    /// When majority of seed commitments is collected, validator concludes an agreement on the
    /// next VDF seed value.
    pub fn validators_seed_commitments(&self) -> ProofMapIndex<&dyn Snapshot, PublicKey, BigInt> {
        ProofMapIndex::new("exonum_rng.validators_commitments", self.view.as_ref())
    }

    /// Returns count of seed commitments posted by a validators in current round.
    pub fn num_seed_commitments(&self) -> usize {
        let seed_commitments = self.validators_seed_commitments();
        let count;
        {
            count = seed_commitments.values().count();
        }

        count
    }

    /// Returns count of valid VDF results posted by a validators in current round.
    pub fn num_vdf_results(&self) -> usize {
        let vdf_results = self.validators_vdf_results();
        let count;
        {
            count = vdf_results.values().count();
        }

        count
    }

    /// Maps validators to their VDF(seed) results.
    ///
    /// When  majority of the equal VDF results is collected, validator concludes an agreement on the
    /// next random number.
    pub fn validators_vdf_results(&self) -> ProofMapIndex<&dyn Snapshot, PublicKey, BigInt> {
        ProofMapIndex::new("exonum_rng.vdf_results", self.view.as_ref())
    }

    /// Returns last seed value that validators has agreed on.
    pub fn last_seed(&self) -> Entry<&dyn Snapshot, Hash> {
        Entry::new("exonum_rng.seed", self.view.as_ref())
    }

    /// Returns last random value that validators has agreed on.
    pub fn last_randomness(&self) -> Entry<&dyn Snapshot, Hash> {
        Entry::new("exonum_rng.randomness", self.view.as_ref())
    }

    pub fn state_hash(&self) -> Vec<Hash> {
        vec![self.validators_seed_commitments().merkle_root(), self.last_seed().hash(), self.last_randomness().hash()]
    }
}

impl<'a> RngSchema<&'a mut Fork> {
    /// Mutable reference to the `validators_seed_commitments` index.
    pub fn validators_seed_commitments_mut(&mut self) -> ProofMapIndex<&mut Fork, PublicKey, BigInt> {
        ProofMapIndex::new("exonum_rng.validators_commitments", self.view)
    }

    /// Mutable reference to the `validators_vdf_results` index.
    pub fn validators_vdf_results_mut(&mut self) -> ProofMapIndex<&mut Fork, PublicKey, BigInt> {
        ProofMapIndex::new("exonum_rng.vdf_results", self.view)
    }

    /// Mutable reference to the `last_randomness` index.
    pub fn last_randomness_mut(&mut self) -> Entry<&mut Fork, Hash> {
        Entry::new("exonum_rng.randomness", self.view)
    }

    /// Mutable reference to the `last_seed` index.
    pub fn last_seed_mut(&mut self) -> Entry<&mut Fork, Hash> {
        Entry::new("exonum_rng.seed", self.view)
    }
}