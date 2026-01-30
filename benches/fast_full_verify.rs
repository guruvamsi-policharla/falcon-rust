use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use falcon_rust::{falcon1024, falcon512, test_utils};
use rand::{thread_rng, Rng};

fn benchmark_stream(c: &mut Criterion) {
    let num_signatures = 1000;
    let index_counts = [1, 8];
    let invalid_fractions = [1.0];

    let mut group = c.benchmark_group("stream_verification");
    group.sample_size(10);
    let mut rng = thread_rng();

    for &fverify_indices_count in &index_counts {
        for &invalid_fraction in &invalid_fractions {
            group.throughput(Throughput::Elements(num_signatures as u64));

            // Falcon 512
            {
                let num_invalid = (num_signatures as f64 * invalid_fraction) as usize;
                let num_valid = num_signatures - num_invalid;

                let test_data =
                    test_utils::generate_test_data_512(&mut rng, num_valid, num_invalid, true);
                let expanded_sigs = test_data.expanded_signatures();

                let indices: Vec<usize> = (0..fverify_indices_count)
                    .map(|_| rng.gen_range(0..512))
                    .collect();

                group.bench_function(
                    format!(
                        "falcon512/indices_{}/invalid_{:.2}",
                        fverify_indices_count, invalid_fraction
                    ),
                    |b| {
                        b.iter(|| {
                            for (i, item) in test_data.items.iter().enumerate() {
                                falcon512::fverify_fullverify(
                                    &item.message,
                                    &expanded_sigs[i],
                                    &test_data.public_key,
                                    &indices,
                                );
                            }
                        })
                    },
                );

                // Baseline full verify (only run once per fraction as it doesn't depend on indices)
                if fverify_indices_count == index_counts[0] {
                    group.bench_function(
                        format!("falcon512/baseline/invalid_{:.2}", invalid_fraction),
                        |b| {
                            b.iter(|| {
                                for item in &test_data.items {
                                    falcon512::verify(
                                        &item.message,
                                        &item.signature,
                                        &test_data.public_key,
                                    );
                                }
                            })
                        },
                    );

                    group.bench_function(
                        format!(
                            "falcon512/baseline_expanded/invalid_{:.2}",
                            invalid_fraction
                        ),
                        |b| {
                            b.iter(|| {
                                for (i, item) in test_data.items.iter().enumerate() {
                                    falcon512::verify_expanded(
                                        &item.message,
                                        &expanded_sigs[i],
                                        &test_data.public_key,
                                    );
                                }
                            })
                        },
                    );
                }
            }

            // Falcon 1024
            {
                let num_invalid = (num_signatures as f64 * invalid_fraction) as usize;
                let num_valid = num_signatures - num_invalid;

                let test_data =
                    test_utils::generate_test_data_1024(&mut rng, num_valid, num_invalid, true);
                let expanded_sigs = test_data.expanded_signatures();

                let indices: Vec<usize> = (0..fverify_indices_count)
                    .map(|_| rng.gen_range(0..1024))
                    .collect();

                group.bench_function(
                    format!(
                        "falcon1024/indices_{}/invalid_{:.2}",
                        fverify_indices_count, invalid_fraction
                    ),
                    |b| {
                        b.iter(|| {
                            for (i, item) in test_data.items.iter().enumerate() {
                                falcon1024::fverify_fullverify(
                                    &item.message,
                                    &expanded_sigs[i],
                                    &test_data.public_key,
                                    &indices,
                                );
                            }
                        })
                    },
                );

                // Baseline full verify (only run once per fraction as it doesn't depend on indices)
                if fverify_indices_count == index_counts[0] {
                    group.bench_function(
                        format!("falcon1024/baseline/invalid_{:.2}", invalid_fraction),
                        |b| {
                            b.iter(|| {
                                for item in &test_data.items {
                                    falcon1024::verify(
                                        &item.message,
                                        &item.signature,
                                        &test_data.public_key,
                                    );
                                }
                            })
                        },
                    );

                    group.bench_function(
                        format!(
                            "falcon1024/baseline_expanded/invalid_{:.2}",
                            invalid_fraction
                        ),
                        |b| {
                            b.iter(|| {
                                for (i, item) in test_data.items.iter().enumerate() {
                                    falcon1024::verify_expanded(
                                        &item.message,
                                        &expanded_sigs[i],
                                        &test_data.public_key,
                                    );
                                }
                            })
                        },
                    );
                }
            }
        }
    }
    group.finish();
}

criterion_group!(benches, benchmark_stream);
criterion_main!(benches);
