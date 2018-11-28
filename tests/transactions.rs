// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate exonum;
#[macro_use]
extern crate exonum_testkit;
extern crate exonum_rng;
#[macro_use]
extern crate pretty_assertions;
extern crate rand;

use rand::{Rng};
use exonum::{
    crypto::Hash,
    storage::Snapshot,
};

use exonum_testkit::{TestKitBuilder, TestNode};
use exonum_rng::{
    rng::{calculate_combined_seed, calculate_vdf},
    blockchain::{
        schema::{BigInt, RngSchema},
        transactions::{TxPublishVdfResult, TxPublishSeedCommitment},
        ToHash,
    },
    ExonumRngService,
};

fn assert_storage_values_eq<T: AsRef<Snapshot>>(
    snapshot: T,
    validators: &[TestNode],
    expected_seed: Option<Hash>,
    expected_last_randomness: Option<Hash>,
    expected_validators_seed_commitments: &[Option<BigInt>],
    expected_validators_vdf_results: &[Option<BigInt>],
) {
    let schema = RngSchema::new(snapshot);

    assert_eq!(schema.last_seed().get(), expected_seed);
    assert_eq!(schema.last_randomness().get(), expected_last_randomness);

    let validators_seed_commitments = schema.validators_seed_commitments();
    let validators_vdf_results = schema.validators_vdf_results();
    for (i, validator) in validators.iter().enumerate() {
        let public_key = &validator.public_keys().service_key;

        assert_eq!(
            validators_seed_commitments.get(public_key),
            expected_validators_seed_commitments[i]
        );

        assert_eq!(
            validators_vdf_results.get(public_key),
            expected_validators_vdf_results[i]
        );
    }
}

#[test]
fn test_exonum_rng_service_with_4_validators() {
    let mut testkit = TestKitBuilder::validator()
        .with_validators(4)
        .with_service(ExonumRngService::new())
        .create();

    // Instance of pseudo-random number generator to use in tests
    let mut rng = rand::thread_rng();

    let validators = testkit.network().validators().to_vec();

    let mut prev_randomness = None;
    for round_num in 0..3 {
        // Validators seed commitments, that are saved in storage, look like this:
        // number       | 0    | 1    | 2    | 3    |
        // commitment   | None | None | None | None |
        //
        // max_byzantine_nodes = (4 - 1) / 3 = 1.
        //
        // Consolidated seed is None (not enough commitments)
        // Consolidated randomness is None (no seed yet)

        assert_storage_values_eq(
            testkit.snapshot(),
            &validators,
            None,
            prev_randomness.clone(),
            &[None, None, None, None],
            &[None, None, None, None],
        );

        // Add first transaction `tx0` from first validator with seed commitment `sc0`.
        // After that validators time look like this:
        // number       | 0       | 1    | 2    | 3    |
        // commitment   | `sc0`   | None | None | None |
        //
        // Consolidated seed is: None (not enough commitments)
        // Consolidated randomness is None (no seed yet)

        let sc0 = rng.gen::<u64>().to_string();
        let tx0 = {
            let (pub_key, sec_key) = validators[0].service_keypair();
            TxPublishSeedCommitment::new(pub_key, &sc0, sec_key)
        };
        testkit.create_block_with_transactions(txvec![tx0]);

        assert_storage_values_eq(
            testkit.snapshot(),
            &validators,
            None,
            prev_randomness.clone(),
            &[Some(sc0.clone()), None, None, None],
            &[None, None, None, None],
        );

        // Add second transaction `tx1` from second validator with another seed commitment.
        // After that validators time look like this:
        // number       | 0       | 1       | 2    | 3    |
        // commitment   | `sc0`   | `sc1`   | None | None |
        //
        // Consolidated seed is: None (not enough commitments)
        // Consolidated randomness is None (no seed yet)

        let sc1 = rng.gen::<u64>().to_string();
        let tx1 = {
            let (pub_key, sec_key) = validators[1].service_keypair();
            TxPublishSeedCommitment::new(pub_key, &sc1, sec_key)
        };
        testkit.create_block_with_transactions(txvec![tx1]);

        assert_storage_values_eq(
            testkit.snapshot(),
            &validators,
            None,
            prev_randomness.clone(),
            &[Some(sc0.clone()), Some(sc1.clone()), None, None],
            &[None, None, None, None],
        );

        // Add third transaction `tx2` from third validator
        // After that validators seed commitments look like this:
        // number       | 0       | 1       | 2       | 3    |
        // commitment   | `sc0`   | `sc1`   | `sc2`   | None |
        //
        // Consolidated seed is: `hash(sc0 || sc1 || sc2)`
        // Consolidated randomness is None (VDF is not calculated yet)
        let sc2 = rng.gen::<u64>().to_string();
        let tx2 = {
            let (pub_key, sec_key) = validators[2].service_keypair();
            TxPublishSeedCommitment::new(pub_key, &sc2, sec_key)
        };
        testkit.create_block_with_transactions(txvec![tx2]);

        // As soon as +2/3 of seed commitments is collected, nodes are proceeding
        // to calculate their combined seed value and VDF(seed) values
        // to create their unpredictable random value

        // Combine and sort commitments
        let mut commitments = [sc0.clone(), sc1.clone(), sc2.clone()].to_vec();
        commitments.sort_unstable();

        let combined_seed = calculate_combined_seed(&commitments);

        assert_storage_values_eq(
            testkit.snapshot(),
            &validators,
            Some(combined_seed),
            prev_randomness.clone(),
            &[Some(sc0.clone()), Some(sc1.clone()), Some(sc2.clone()), None],
            &[None, None, None, None],
        );

        // Publish VDF result from validator 0
        // After that validators seed commitments look like this:
        // number       | 0       | 1       | 2       | 3    |
        // commitment   | `sc0`   | `sc1`   | `sc2`   | None |
        // vdf result   | `vdf0`  | None    | None    | None |
        //
        // Consolidated seed is: `hash(sc0 || sc1 || sc2)`
        // Consolidated randomness is None (not enough VDF results)
        let vdf_res0 = calculate_vdf(&combined_seed).unwrap();
        let vdf_tx0 = {
            let (pub_key, sec_key) = validators[0].service_keypair();
            TxPublishVdfResult::new(pub_key, &combined_seed, &vdf_res0, sec_key)
        };
        testkit.create_block_with_transactions(txvec![vdf_tx0]);

        assert_storage_values_eq(
            testkit.snapshot(),
            &validators,
            Some(combined_seed),
            prev_randomness.clone(),
            &[Some(sc0.clone()), Some(sc1.clone()), Some(sc2.clone()), None],
            &[Some(vdf_res0.clone()), None, None, None],
        );

        // Publish VDF result from validator 1
        // After that validators seed commitments look like this:
        // number       | 0       | 1       | 2       | 3    |
        // commitment   | `sc0`   | `sc1`   | `sc2`   | None |
        // vdf result   | `vdf0`  | `vdf1`  | None    | None |
        //
        // Consolidated seed is: `hash(sc0 || sc1 || sc2)`
        // Consolidated randomness is None (not enough VDF results)
        let vdf_res1 = calculate_vdf(&combined_seed).unwrap();
        let vdf_tx1 = {
            let (pub_key, sec_key) = validators[1].service_keypair();
            TxPublishVdfResult::new(pub_key, &combined_seed, &vdf_res1, sec_key)
        };
        testkit.create_block_with_transactions(txvec![vdf_tx1]);

        assert_storage_values_eq(
            testkit.snapshot(),
            &validators,
            Some(combined_seed),
            prev_randomness.clone(),
            &[Some(sc0.clone()), Some(sc1.clone()), Some(sc2.clone()), None],
            &[Some(vdf_res0.clone()), Some(vdf_res1.clone()), None, None],
        );

        // Publish VDF result from validator 2
        // After that validators seed commitments look like this:
        // number       | 0       | 1       | 2       | 3    |
        // commitment   | `sc0`   | `sc1`   | `sc2`   | None |
        // vdf result   | `vdf0`  | `vdf1`  | `vdf2`  | None |
        //
        // Consolidated seed is: `hash(sc0 || sc1 || sc2)`
        // Consolidated randomness is `vdf0`
        println!("vdf tx 2");
        let vdf_res2 = calculate_vdf(&combined_seed).unwrap();
        let vdf_tx2 = {
            let (pub_key, sec_key) = validators[2].service_keypair();
            TxPublishVdfResult::new(pub_key, &combined_seed, &vdf_res2, sec_key)
        };
        testkit.create_block_with_transactions(txvec![vdf_tx2]);

        // At this moment there's available randomness equal to VDF results
        // And seed, seed commitments and vdf results are reset
        assert_storage_values_eq(
            testkit.snapshot(),
            &validators,
            None,
            Some(vdf_res0.to_hash()),
            &[None, None, None, None],
            &[None, None, None, None],
        );

        println!("Resulting randomness generated in round {}: {}", round_num, vdf_res0.to_hash().to_hex());
        prev_randomness = Some(vdf_res0.to_hash());
    }
}