use criterion::{criterion_group, criterion_main, Criterion};
use itertools::Itertools;
use pqcrypto_falcon::{falcon1024, falcon512};
use rand::distributions::{Distribution, Uniform};
use rand::{thread_rng, Rng};

const NUM_KEYS: usize = 2;
const SIGS_PER_KEY: usize = 10;

pub fn falcon_rust_operation(c: &mut Criterion) {
    let mut rng = thread_rng();
    let keys512 = (0..NUM_KEYS)
        .map(|_| falcon_rust::falcon512::keygen(rng.gen()))
        .collect_vec();
    let keys1024 = (0..NUM_KEYS)
        .map(|_| falcon_rust::falcon1024::keygen(rng.gen()))
        .collect_vec();
    let msgs512 = (0..NUM_KEYS * SIGS_PER_KEY)
        .map(|_| rng.gen::<[u8; 15]>())
        .collect_vec();
    let msgs1024 = (0..NUM_KEYS * SIGS_PER_KEY)
        .map(|_| rng.gen::<[u8; 15]>())
        .collect_vec();
    let sigs512 = (0..NUM_KEYS * SIGS_PER_KEY)
        .map(|i| falcon_rust::falcon512::sign(&msgs512[i], &keys512[i % NUM_KEYS].0))
        .collect_vec();
    let sigs1024 = (0..NUM_KEYS * SIGS_PER_KEY)
        .map(|i| falcon_rust::falcon1024::sign(&msgs1024[i], &keys1024[i % NUM_KEYS].0))
        .collect_vec();
    let mut expanded_sigs512 = Vec::new();
    let mut expanded_sigs1024 = Vec::new();
    for i in 0..NUM_KEYS * SIGS_PER_KEY {
        let sig512 = &sigs512[i];
        let sig1024 = &sigs1024[i];
        let expanded_sig512 = falcon_rust::falcon512::ExpandedSignature::from_signature(
            &msgs512[i],
            sig512,
            &keys512[i % NUM_KEYS].1,
        );
        let expanded_sig1024 = falcon_rust::falcon1024::ExpandedSignature::from_signature(
            &msgs1024[i],
            sig1024,
            &keys1024[i % NUM_KEYS].1,
        );
        expanded_sigs512.push(expanded_sig512);
        expanded_sigs1024.push(expanded_sig1024);
    }

    let mut group = c.benchmark_group("falcon-rust");
    group.sample_size(NUM_KEYS * SIGS_PER_KEY);
    let mut iterator_verify_512 = 0;
    group.bench_function("verify 512", |b| {
        b.iter(|| {
            assert!(falcon_rust::falcon512::verify(
                &msgs512[iterator_verify_512 % msgs512.len()],
                &sigs512[iterator_verify_512 % sigs512.len()],
                &keys512[iterator_verify_512 % NUM_KEYS].1,
            ));
            iterator_verify_512 += 1;
        })
    });
    let mut iterator_verify_1024 = 0;
    group.bench_function("verify 1024", |b| {
        b.iter(|| {
            assert!(falcon_rust::falcon1024::verify(
                &msgs1024[iterator_verify_1024 % msgs1024.len()],
                &sigs1024[iterator_verify_1024 % sigs1024.len()],
                &keys1024[iterator_verify_1024 % NUM_KEYS].1,
            ));
            iterator_verify_1024 += 1;
        })
    });

    let mut iterator_verify_expanded_512 = 0;
    group.bench_function("verify 512 expanded", |b| {
        b.iter(|| {
            assert!(falcon_rust::falcon512::verify_expanded(
                &msgs512[iterator_verify_expanded_512 % msgs512.len()],
                &expanded_sigs512[iterator_verify_expanded_512 % expanded_sigs512.len()],
                &keys512[iterator_verify_expanded_512 % NUM_KEYS].1,
            ));
            iterator_verify_expanded_512 += 1;
        })
    });

    let mut iterator_verify_expanded_1024 = 0;
    group.bench_function("verify 1024 expanded", |b| {
        b.iter(|| {
            assert!(falcon_rust::falcon1024::verify_expanded(
                &msgs1024[iterator_verify_expanded_1024 % msgs1024.len()],
                &expanded_sigs1024[iterator_verify_expanded_1024 % expanded_sigs1024.len()],
                &keys1024[iterator_verify_expanded_1024 % NUM_KEYS].1,
            ));
            iterator_verify_expanded_1024 += 1;
        })
    });
    group.finish();

    let mut group = c.benchmark_group("falcon-rust");
    group.sample_size(NUM_KEYS * SIGS_PER_KEY);

    let num_indices_list = [1, 4, 8, 16, 32, 48, 64];
    let mut rng = rand::thread_rng();

    // Benchmark fverify 512 with varying numbers of indices
    let step = Uniform::new(0, 512);
    for num_indices in num_indices_list {
        let mut iterator_verify_512 = 0;
        let indices: Vec<usize> = step.sample_iter(&mut rng).take(num_indices).collect();

        group.bench_function(format!("fast verify 512 - {} indices", num_indices), |b| {
            b.iter(|| {
                assert!(falcon_rust::falcon512::fverify(
                    &msgs512[iterator_verify_512 % msgs512.len()],
                    &expanded_sigs512[iterator_verify_512 % expanded_sigs512.len()],
                    &keys512[iterator_verify_512 % NUM_KEYS].1,
                    &indices
                ));
                iterator_verify_512 += 1;
            })
        });
    }

    // Benchmark fverify 1024 with varying numbers of indices
    let step = Uniform::new(0, 512);
    for num_indices in num_indices_list {
        let mut iterator_verify_1024 = 0;
        let indices: Vec<usize> = step.sample_iter(&mut rng).take(num_indices).collect();

        group.bench_function(format!("fast verify 1024 - {} indices", num_indices), |b| {
            b.iter(|| {
                assert!(falcon_rust::falcon1024::fverify(
                    &msgs1024[iterator_verify_1024 % msgs1024.len()],
                    &expanded_sigs1024[iterator_verify_1024 % expanded_sigs1024.len()],
                    &keys1024[iterator_verify_1024 % NUM_KEYS].1,
                    &indices
                ));
                iterator_verify_1024 += 1;
            })
        });
    }

    group.finish();
}

fn falcon_c_ffi_operation(c: &mut Criterion) {
    let mut rng = thread_rng();
    let keys512 = (0..NUM_KEYS).map(|_| falcon512::keypair()).collect_vec();
    let keys1024 = (0..NUM_KEYS).map(|_| falcon1024::keypair()).collect_vec();

    let msgs512 = (0..NUM_KEYS * SIGS_PER_KEY)
        .map(|_| rng.gen::<[u8; 15]>())
        .collect_vec();
    let sigs512 = msgs512
        .iter()
        .enumerate()
        .map(|(i, msg)| falcon512::detached_sign(msg, &keys512[i % NUM_KEYS].1))
        .collect_vec();
    let msgs1024 = (0..NUM_KEYS * SIGS_PER_KEY)
        .map(|_| rng.gen::<[u8; 15]>())
        .collect_vec();
    let sigs1024 = msgs1024
        .iter()
        .enumerate()
        .map(|(i, msg)| falcon1024::detached_sign(msg, &keys1024[i % NUM_KEYS].1))
        .collect_vec();

    let mut group = c.benchmark_group("c ffi");
    group.sample_size(NUM_KEYS * SIGS_PER_KEY);
    let mut iterator_verify_512 = 0;
    group.bench_function("verify 512", |b| {
        b.iter(|| {
            assert!(falcon512::verify_detached_signature(
                &sigs512[iterator_verify_512 % sigs512.len()],
                &msgs512[iterator_verify_512 % msgs512.len()],
                &keys512[iterator_verify_512 % NUM_KEYS].0,
            )
            .is_ok());
            iterator_verify_512 += 1;
        })
    });
    let mut iterator_verify_1024 = 0;
    group.bench_function("verify 1024", |b| {
        b.iter(|| {
            assert!(falcon1024::verify_detached_signature(
                &sigs1024[iterator_verify_1024 % sigs1024.len()],
                &msgs1024[iterator_verify_1024 % msgs1024.len()],
                &keys1024[iterator_verify_1024 % NUM_KEYS].0,
            )
            .is_ok());
            iterator_verify_1024 += 1;
        })
    });
    group.finish();
}

criterion_group!(benches, falcon_rust_operation, falcon_c_ffi_operation);
criterion_main!(benches);
