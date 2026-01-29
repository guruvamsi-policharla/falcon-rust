//! Benchmark comparing two approaches for processing a stream of signatures:
//! 1. Just run verify on all signatures
//! 2. Run fverify_fullverify on all (fverify first, then full verify only if passed)
//!
//! Usage: cargo run --release -- [num_signatures] [invalid_fraction] [fverify_indices]
//!   num_signatures: total number of signatures to process (default: 10000)
//!   invalid_fraction: fraction of signatures that are invalid, e.g. 0.1 = 10% (default: 0.1)
//!   fverify_indices: number of indices to check in fverify (default: 8)

use rand::{thread_rng, Rng};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let num_signatures: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(10000);
    let invalid_fraction: f64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.1);
    let fverify_indices: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(8);

    println!("=== Stream Signature Verification Benchmark ===");
    println!("Total signatures: {}", num_signatures);
    println!("Invalid fraction: {:.1}%", invalid_fraction * 100.0);
    println!("fverify indices: {}", fverify_indices);
    println!();

    // Run benchmarks for both Falcon variants
    benchmark_falcon512(num_signatures, invalid_fraction, fverify_indices);
    println!();
    benchmark_falcon1024(num_signatures, invalid_fraction, fverify_indices);
}

fn benchmark_falcon512(num_signatures: usize, invalid_fraction: f64, fverify_indices: usize) {
    println!("--- Falcon 512 ---");

    let mut rng = thread_rng();
    let num_invalid = (num_signatures as f64 * invalid_fraction) as usize;
    let num_valid = num_signatures - num_invalid;

    println!("Generating keys...");
    let (sk, pk) = falcon_rust::falcon512::keygen(rng.gen());
    // Generate a different key for invalid signatures
    let (sk_invalid, _) = falcon_rust::falcon512::keygen(rng.gen());

    println!("Generating {} valid signatures...", num_valid);
    let valid_msgs: Vec<[u8; 32]> = (0..num_valid).map(|_| rng.gen()).collect();
    let valid_sigs: Vec<_> = valid_msgs
        .iter()
        .map(|msg| falcon_rust::falcon512::sign(msg, &sk))
        .collect();

    println!(
        "Generating {} invalid signatures (signed with wrong key)...",
        num_invalid
    );
    let invalid_msgs: Vec<[u8; 32]> = (0..num_invalid).map(|_| rng.gen()).collect();
    // Sign with wrong key - these won't verify against pk
    let invalid_sigs: Vec<_> = invalid_msgs
        .iter()
        .map(|msg| falcon_rust::falcon512::sign(msg, &sk_invalid))
        .collect();

    // Combine and shuffle
    let mut all_data: Vec<(bool, [u8; 32], falcon_rust::falcon512::Signature)> = Vec::new();
    for (msg, sig) in valid_msgs.iter().zip(valid_sigs.iter()) {
        all_data.push((true, *msg, sig.clone()));
    }
    for (msg, sig) in invalid_msgs.iter().zip(invalid_sigs.iter()) {
        all_data.push((false, *msg, sig.clone()));
    }

    // Shuffle the data
    use rand::seq::SliceRandom;
    all_data.shuffle(&mut rng);

    println!("Precomputing expanded signatures for fverify...");
    let expanded_sigs: Vec<_> = all_data
        .iter()
        .map(|(_, msg, sig)| {
            falcon_rust::falcon512::ExpandedSignature::from_signature(msg, sig, &pk)
        })
        .collect();

    // Generate random indices for fverify
    let indices: Vec<usize> = (0..fverify_indices)
        .map(|_| rng.gen_range(0..512))
        .collect();

    println!("Running benchmarks...\n");

    // Approach 1: Just run verify on all
    let start = Instant::now();
    let mut verify_count = 0u64;
    for (_, msg, sig) in &all_data {
        if falcon_rust::falcon512::verify(msg, sig, &pk) {
            verify_count += 1;
        }
    }
    let verify_all_duration = start.elapsed();

    // Approach 2: fverify_fullverify (fverify first, then full verify if passed, avoiding redundant work)
    let start = Instant::now();
    let mut final_verify_count = 0u64;
    for (i, (_, msg, _)) in all_data.iter().enumerate() {
        if falcon_rust::falcon512::fverify_fullverify(msg, &expanded_sigs[i], &pk, &indices) {
            final_verify_count += 1;
        }
    }
    let fverify_fullverify_duration = start.elapsed();

    println!("APPROACH 1: Verify all");
    println!("  Total time: {:?}", verify_all_duration);
    println!("  Valid signatures found: {}", verify_count);
    println!(
        "  Throughput: {:.0} sigs/sec",
        num_signatures as f64 / verify_all_duration.as_secs_f64()
    );

    println!();

    println!("APPROACH 2: fverify_fullverify (fverify then full verify, no redundant work)");
    println!("  Total time: {:?}", fverify_fullverify_duration);
    println!("  Valid signatures found: {}", final_verify_count);
    println!(
        "  Throughput: {:.0} sigs/sec",
        num_signatures as f64 / fverify_fullverify_duration.as_secs_f64()
    );

    println!();

    let speedup = verify_all_duration.as_secs_f64() / fverify_fullverify_duration.as_secs_f64();
    if speedup > 1.0 {
        println!("Speedup (Approach 2 vs 1): {:.2}x faster", speedup);
    } else {
        println!("Slowdown (Approach 2 vs 1): {:.2}x slower", 1.0 / speedup);
    }
}

fn benchmark_falcon1024(num_signatures: usize, invalid_fraction: f64, fverify_indices: usize) {
    println!("--- Falcon 1024 ---");

    let mut rng = thread_rng();
    let num_invalid = (num_signatures as f64 * invalid_fraction) as usize;
    let num_valid = num_signatures - num_invalid;

    println!("Generating keys...");
    let (sk, pk) = falcon_rust::falcon1024::keygen(rng.gen());
    // Generate a different key for invalid signatures
    let (sk_invalid, _) = falcon_rust::falcon1024::keygen(rng.gen());

    println!("Generating {} valid signatures...", num_valid);
    let valid_msgs: Vec<[u8; 32]> = (0..num_valid).map(|_| rng.gen()).collect();
    let valid_sigs: Vec<_> = valid_msgs
        .iter()
        .map(|msg| falcon_rust::falcon1024::sign(msg, &sk))
        .collect();

    println!(
        "Generating {} invalid signatures (signed with wrong key)...",
        num_invalid
    );
    let invalid_msgs: Vec<[u8; 32]> = (0..num_invalid).map(|_| rng.gen()).collect();
    // Sign with wrong key - these won't verify against pk
    let invalid_sigs: Vec<_> = invalid_msgs
        .iter()
        .map(|msg| falcon_rust::falcon1024::sign(msg, &sk_invalid))
        .collect();

    // Combine and shuffle
    let mut all_data: Vec<(bool, [u8; 32], falcon_rust::falcon1024::Signature)> = Vec::new();
    for (msg, sig) in valid_msgs.iter().zip(valid_sigs.iter()) {
        all_data.push((true, *msg, sig.clone()));
    }
    for (msg, sig) in invalid_msgs.iter().zip(invalid_sigs.iter()) {
        all_data.push((false, *msg, sig.clone()));
    }

    // Shuffle the data
    use rand::seq::SliceRandom;
    all_data.shuffle(&mut rng);

    println!("Precomputing expanded signatures for fverify...");
    let expanded_sigs: Vec<_> = all_data
        .iter()
        .map(|(_, msg, sig)| {
            falcon_rust::falcon1024::ExpandedSignature::from_signature(msg, sig, &pk)
        })
        .collect();

    // Generate random indices for fverify
    let indices: Vec<usize> = (0..fverify_indices)
        .map(|_| rng.gen_range(0..1024))
        .collect();

    println!("Running benchmarks...\n");

    // Approach 1: Just run verify on all
    let start = Instant::now();
    let mut verify_count = 0u64;
    for (_, msg, sig) in &all_data {
        if falcon_rust::falcon1024::verify(msg, sig, &pk) {
            verify_count += 1;
        }
    }
    let verify_all_duration = start.elapsed();

    // Approach 2: fverify_fullverify (fverify first, then full verify if passed, avoiding redundant work)
    let start = Instant::now();
    let mut final_verify_count = 0u64;
    for (i, (_, msg, _)) in all_data.iter().enumerate() {
        if falcon_rust::falcon1024::fverify_fullverify(msg, &expanded_sigs[i], &pk, &indices) {
            final_verify_count += 1;
        }
    }
    let fverify_fullverify_duration = start.elapsed();

    println!("APPROACH 1: Verify all");
    println!("  Total time: {:?}", verify_all_duration);
    println!("  Valid signatures found: {}", verify_count);
    println!(
        "  Throughput: {:.0} sigs/sec",
        num_signatures as f64 / verify_all_duration.as_secs_f64()
    );

    println!();

    println!("APPROACH 2: fverify_fullverify (fverify then full verify, no redundant work)");
    println!("  Total time: {:?}", fverify_fullverify_duration);
    println!("  Valid signatures found: {}", final_verify_count);
    println!(
        "  Throughput: {:.0} sigs/sec",
        num_signatures as f64 / fverify_fullverify_duration.as_secs_f64()
    );

    println!();

    let speedup = verify_all_duration.as_secs_f64() / fverify_fullverify_duration.as_secs_f64();
    if speedup > 1.0 {
        println!("Speedup (Approach 2 vs 1): {:.2}x faster", speedup);
    } else {
        println!("Slowdown (Approach 2 vs 1): {:.2}x slower", 1.0 / speedup);
    }
}
