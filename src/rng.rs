use exonum::crypto::{Hash, hash};

use blockchain::schema::BigInt;

use vdf::vdf_mimc::{verify, eval};
use rug::Integer;
use std::str::FromStr;

// TODO: make this configurable
const VDF_DIFFICULTY: u64 = 8096 * 16;

/// Calculates combined seed by hashing concatenated commitments
///
/// `seed = hash(c1 || c2 || ... || cn)`
pub fn calculate_combined_seed(commitments: &[BigInt]) -> Hash {
    // Concatenate all seed values together
    let combined_seed = commitments
        .iter()
        .fold("".to_owned(), |a, b| format!("{}{}", a, b));

    hash(combined_seed.as_bytes())
}

/// Validates VDF value against the provided seed.
///
/// In according to VDF properties, verification should be much faster than calculation of VDF,
/// so can be verified in transaction contract body.
pub fn validate_vdf(seed: &Hash, value: &BigInt) -> bool {
    let seed = Integer::from_str_radix(&seed.to_hex(), 16);
    let value = Integer::from_str(value);

    if let Ok(seed) = seed {
        if let Ok(value) = value {
            return verify(&seed, VDF_DIFFICULTY, &value)
        }
    }

    false
}

/// Calculates VDF against provided `seed`.
///
/// In according to VDF properties, calculation is deliberately made slow and sequential,
/// to provide verifiable delay.
///
/// Since VDFs are supposed to be slow, VDF shouldn't be executed in transaction contract body
/// and shouldn't be executed by validators that are fast-forwarding to the
/// current blockchain tip when synchronizing with others.
pub fn calculate_vdf(seed: &Hash) -> Option<BigInt> {
    let seed = Integer::from_str_radix(&seed.to_hex(), 16);

    if let Ok(seed) = seed {
        return Some(eval(&seed, VDF_DIFFICULTY).to_string())
    }

    None
}