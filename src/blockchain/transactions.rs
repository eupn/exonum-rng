use exonum::{
    storage::Fork,
    crypto::Hash,
    blockchain::{Transaction, ExecutionError, Schema as CoreSchema},
    messages::Message,
};

use blockchain::{
    schema::RngSchema,
    ToHash
};

use exonum::crypto::PublicKey;
use SERVICE_ID;

use rng::{calculate_combined_seed, validate_vdf};

transactions! {
    pub ExonumRngTransactions {
        const SERVICE_ID = SERVICE_ID;

        struct TxPublishSeedCommitment {
            /// Public key of the author.
            pub_key: &PublicKey,

            /// Value of the seed commitment.
            value: &str,
        }

        struct TxPublishVdfResult {
            /// Public key of the author.
            pub_key: &PublicKey,

            /// Seed from which VDF is calculated by validator.
            seed: &Hash,

            /// Value of the VDF function to be verified by others against the seed.
            /// Also serves as an unpredictable random number candidate.
            ///
            /// Majority of *valid* VDF results is used as current randomness value.
            value: &str,
        }
    }
}

impl Transaction for TxPublishSeedCommitment {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> Result<(), ExecutionError> {
        let num_nodes = {
            let core_schema = CoreSchema::new(&*fork);
            core_schema.actual_configuration().validator_keys.len()
        };

        let mut schema = RngSchema::new(fork);

        schema.validators_seed_commitments_mut().put(&self.pub_key(), self.value().to_owned());

        // Check that validator has collected enough seed commitments
        // NB: this rule probably could be relaxed
        let max_byzantine_nodes = (num_nodes - 1) / 3;
        if schema.num_seed_commitments() > 2 * max_byzantine_nodes {
            let commitments= {
                let commitments_idx = schema.validators_seed_commitments();
                let mut comms = commitments_idx
                    .values()
                    .collect::<Vec<_>>();

                // Sort commitments
                comms.sort_unstable();
                comms
            };

            let seed = calculate_combined_seed(&commitments);
            //println!("Calculated combined seed: {}", seed);

            schema.last_seed_mut().set(seed);
        } else {
            //println!("Not enough seed commitments: {} <= {}",
            //         schema.num_seed_commitments(), 2 * max_byzantine_nodes);
        }

        Ok(())
    }
}

impl Transaction for TxPublishVdfResult {
    fn verify(&self) -> bool {
        self.verify_signature(self.pub_key())
    }

    fn execute(&self, fork: &mut Fork) -> Result<(), ExecutionError> {
        let num_nodes = {
            let core_schema = CoreSchema::new(&*fork);
            core_schema.actual_configuration().validator_keys.len()
        };

        let mut schema = RngSchema::new(fork);

        // Ignore VDF result if there is no seed
        let current_seed = match schema.last_seed().get() {
            Some(seed) => seed,
            None => return Ok(())
        };

        // Ignore VDF result with seed different from agreed one
        if *self.seed() != current_seed {
            return Ok(())
        }

        if !validate_vdf(self.seed(), &self.value().to_owned()) {
            return Ok(())
        }

        schema.validators_vdf_results_mut().put(&self.pub_key(), self.value().to_owned());

        // Check that validator has collected enough VDF results
        // NB: this rule probably could be relaxed
        let max_byzantine_nodes = (num_nodes - 1) / 3;
        if schema.num_vdf_results() > 2 * max_byzantine_nodes {
            let vdf_result_candidate = self.value();
            schema.last_randomness_mut().set(vdf_result_candidate.to_hash());

            // Clear leftovers to not mess with next rounds
            schema.last_seed_mut().remove();
            schema.validators_vdf_results_mut().clear();
            schema.validators_seed_commitments_mut().clear();

            println!("[SUCCESS] Current randomness value: {}", vdf_result_candidate.to_hash().to_hex());
        }

        Ok(())
    }
}