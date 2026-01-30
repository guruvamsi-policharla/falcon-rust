use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use falcon_rust::{falcon1024, falcon512};
use rand::seq::SliceRandom;
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

                let (sk, pk) = falcon512::keygen(rng.gen());
                let (sk_invalid, _pk_invalid) = falcon512::keygen(rng.gen());

                let valid_msgs: Vec<[u8; 32]> = (0..num_valid).map(|_| rng.gen()).collect();
                let valid_sigs: Vec<_> = valid_msgs
                    .iter()
                    .map(|msg| falcon512::sign(msg, &sk))
                    .collect();

                let invalid_msgs: Vec<[u8; 32]> = (0..num_invalid).map(|_| rng.gen()).collect();
                let invalid_sigs: Vec<_> = invalid_msgs
                    .iter()
                    .map(|msg| falcon512::sign(msg, &sk_invalid))
                    .collect();

                let mut all_data: Vec<(bool, [u8; 32], falcon512::Signature)> = Vec::new();
                for (msg, sig) in valid_msgs.iter().zip(valid_sigs.iter()) {
                    all_data.push((true, *msg, sig.clone()));
                }
                for (msg, sig) in invalid_msgs.iter().zip(invalid_sigs.iter()) {
                    all_data.push((false, *msg, sig.clone()));
                }

                all_data.shuffle(&mut rng);

                let expanded_sigs: Vec<_> = all_data
                    .iter()
                    .map(|(_, msg, sig)| {
                        falcon512::ExpandedSignature::from_signature(msg, sig, &pk)
                    })
                    .collect();

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
                            for (i, (_, msg, _)) in all_data.iter().enumerate() {
                                falcon512::fverify_fullverify(
                                    msg,
                                    &expanded_sigs[i],
                                    &pk,
                                    &indices,
                                );
                            }
                        })
                    },
                );

                /*
                // Baseline full verify (only run once per fraction as it doesn't depend on indices)
                if fverify_indices_count == index_counts[0] {
                    group.bench_function(
                        format!("falcon512/baseline/invalid_{:.2}", invalid_fraction),
                        |b| {
                            b.iter(|| {
                                for (_, msg, sig) in &all_data {
                                    falcon512::verify(msg, sig, &pk);
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
                                for (i, (_, msg, _)) in all_data.iter().enumerate() {
                                    falcon512::verify_expanded(msg, &expanded_sigs[i], &pk);
                                }
                            })
                        },
                    );
                }
                */
            }

            /*
            // Falcon 1024
            {
                let num_invalid = (num_signatures as f64 * invalid_fraction) as usize;
                let num_valid = num_signatures - num_invalid;

                let (sk, pk) = falcon1024::keygen(rng.gen());
                let (sk_invalid, _) = falcon1024::keygen(rng.gen());

                let valid_msgs: Vec<[u8; 32]> = (0..num_valid).map(|_| rng.gen()).collect();
                let valid_sigs: Vec<_> = valid_msgs
                    .iter()
                    .map(|msg| falcon1024::sign(msg, &sk))
                    .collect();

                let invalid_msgs: Vec<[u8; 32]> = (0..num_invalid).map(|_| rng.gen()).collect();
                let invalid_sigs: Vec<_> = invalid_msgs
                    .iter()
                    .map(|msg| falcon1024::sign(msg, &sk_invalid))
                    .collect();

                let mut all_data: Vec<(bool, [u8; 32], falcon1024::Signature)> = Vec::new();
                for (msg, sig) in valid_msgs.iter().zip(valid_sigs.iter()) {
                    all_data.push((true, *msg, sig.clone()));
                }
                for (msg, sig) in invalid_msgs.iter().zip(invalid_sigs.iter()) {
                    all_data.push((false, *msg, sig.clone()));
                }

                all_data.shuffle(&mut rng);

                let expanded_sigs: Vec<_> = all_data
                    .iter()
                    .map(|(_, msg, sig)| {
                        falcon1024::ExpandedSignature::from_signature(msg, sig, &pk)
                    })
                    .collect();

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
                            for (i, (_, msg, _)) in all_data.iter().enumerate() {
                                falcon1024::fverify_fullverify(
                                    msg,
                                    &expanded_sigs[i],
                                    &pk,
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
                                for (_, msg, sig) in &all_data {
                                    falcon1024::verify(msg, sig, &pk);
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
                                for (i, (_, msg, _)) in all_data.iter().enumerate() {
                                    falcon1024::verify_expanded(msg, &expanded_sigs[i], &pk);
                                }
                            })
                        },
                    );
                }
            }
            */
        }
    }
    group.finish();
}

criterion_group!(benches, benchmark_stream);
criterion_main!(benches);
