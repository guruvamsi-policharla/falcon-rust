//! Test utilities for generating valid and invalid signature data.
//!
//! This module provides helpers for creating test datasets that include
//! both valid signatures and invalid signatures (signed with a different key).

use rand::{seq::SliceRandom, Rng};

use crate::{falcon1024, falcon512};

/// A test data item containing a message, signature, and validity flag for Falcon512.
#[derive(Clone)]
pub struct TestDataItem512 {
    /// Whether this signature is valid for the associated public key.
    pub is_valid: bool,
    /// The message that was signed.
    pub message: [u8; 32],
    /// The signature.
    pub signature: falcon512::Signature,
}

/// A collection of test data with mixed valid and invalid signatures for Falcon512.
pub struct TestData512 {
    /// The public key used for verification.
    pub public_key: falcon512::PublicKey,
    /// The test data items (shuffled mix of valid and invalid).
    pub items: Vec<TestDataItem512>,
}

impl TestData512 {
    /// Generate expanded signatures for all items.
    pub fn expanded_signatures(&self) -> Vec<falcon512::ExpandedSignature> {
        self.items
            .iter()
            .map(|item| {
                falcon512::ExpandedSignature::from_signature(
                    &item.message,
                    &item.signature,
                    &self.public_key,
                )
            })
            .collect()
    }
}

/// A test data item containing a message, signature, and validity flag for Falcon1024.
#[derive(Clone)]
pub struct TestDataItem1024 {
    /// Whether this signature is valid for the associated public key.
    pub is_valid: bool,
    /// The message that was signed.
    pub message: [u8; 32],
    /// The signature.
    pub signature: falcon1024::Signature,
}

/// A collection of test data with mixed valid and invalid signatures for Falcon1024.
pub struct TestData1024 {
    /// The public key used for verification.
    pub public_key: falcon1024::PublicKey,
    /// The test data items (shuffled mix of valid and invalid).
    pub items: Vec<TestDataItem1024>,
}

impl TestData1024 {
    /// Generate expanded signatures for all items.
    pub fn expanded_signatures(&self) -> Vec<falcon1024::ExpandedSignature> {
        self.items
            .iter()
            .map(|item| {
                falcon1024::ExpandedSignature::from_signature(
                    &item.message,
                    &item.signature,
                    &self.public_key,
                )
            })
            .collect()
    }
}

/// Generate test data with a mix of valid and invalid signatures for Falcon512.
///
/// # Arguments
/// * `rng` - Random number generator
/// * `num_valid` - Number of valid signatures to generate
/// * `num_invalid` - Number of invalid signatures to generate  
/// * `shuffle` - Whether to shuffle the resulting items
///
/// # Returns
/// A `TestData512` struct containing the public key and items.
pub fn generate_test_data_512<R: Rng>(
    rng: &mut R,
    num_valid: usize,
    num_invalid: usize,
    shuffle: bool,
) -> TestData512 {
    let (sk, pk) = falcon512::keygen(rng.gen());
    let (sk_invalid, _) = falcon512::keygen(rng.gen());

    let mut items = Vec::with_capacity(num_valid + num_invalid);

    // Generate valid signatures
    for _ in 0..num_valid {
        let message: [u8; 32] = rng.gen();
        let signature = falcon512::sign(&message, &sk);
        items.push(TestDataItem512 {
            is_valid: true,
            message,
            signature,
        });
    }

    // Generate invalid signatures (signed with wrong key)
    for _ in 0..num_invalid {
        let message: [u8; 32] = rng.gen();
        let signature = falcon512::sign(&message, &sk_invalid);
        items.push(TestDataItem512 {
            is_valid: false,
            message,
            signature,
        });
    }

    if shuffle {
        items.shuffle(rng);
    }

    TestData512 {
        public_key: pk,
        items,
    }
}

/// Generate test data with a mix of valid and invalid signatures for Falcon1024.
///
/// # Arguments
/// * `rng` - Random number generator
/// * `num_valid` - Number of valid signatures to generate
/// * `num_invalid` - Number of invalid signatures to generate  
/// * `shuffle` - Whether to shuffle the resulting items
///
/// # Returns
/// A `TestData1024` struct containing the public key and items.
pub fn generate_test_data_1024<R: Rng>(
    rng: &mut R,
    num_valid: usize,
    num_invalid: usize,
    shuffle: bool,
) -> TestData1024 {
    let (sk, pk) = falcon1024::keygen(rng.gen());
    let (sk_invalid, _) = falcon1024::keygen(rng.gen());

    let mut items = Vec::with_capacity(num_valid + num_invalid);

    // Generate valid signatures
    for _ in 0..num_valid {
        let message: [u8; 32] = rng.gen();
        let signature = falcon1024::sign(&message, &sk);
        items.push(TestDataItem1024 {
            is_valid: true,
            message,
            signature,
        });
    }

    // Generate invalid signatures (signed with wrong key)
    for _ in 0..num_invalid {
        let message: [u8; 32] = rng.gen();
        let signature = falcon1024::sign(&message, &sk_invalid);
        items.push(TestDataItem1024 {
            is_valid: false,
            message,
            signature,
        });
    }

    if shuffle {
        items.shuffle(rng);
    }

    TestData1024 {
        public_key: pk,
        items,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_generate_test_data_512() {
        let mut rng = thread_rng();
        let data = generate_test_data_512(&mut rng, 5, 3, true);

        assert_eq!(data.items.len(), 8);

        let valid_count = data.items.iter().filter(|item| item.is_valid).count();
        let invalid_count = data.items.iter().filter(|item| !item.is_valid).count();

        assert_eq!(valid_count, 5);
        assert_eq!(invalid_count, 3);

        // Verify that valid signatures actually verify
        for item in &data.items {
            let verified = falcon512::verify(&item.message, &item.signature, &data.public_key);
            assert_eq!(verified, item.is_valid);
        }
    }

    #[test]
    fn test_generate_test_data_1024() {
        let mut rng = thread_rng();
        let data = generate_test_data_1024(&mut rng, 3, 2, false);

        assert_eq!(data.items.len(), 5);

        // First 3 should be valid, last 2 should be invalid (not shuffled)
        assert!(data.items[0].is_valid);
        assert!(data.items[1].is_valid);
        assert!(data.items[2].is_valid);
        assert!(!data.items[3].is_valid);
        assert!(!data.items[4].is_valid);

        // Verify correctness
        for item in &data.items {
            let verified = falcon1024::verify(&item.message, &item.signature, &data.public_key);
            assert_eq!(verified, item.is_valid);
        }
    }
}
