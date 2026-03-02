use base85::*;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::RngCore;
use std::hint::black_box;

fn encode_noalloc_remapped_benchmark(c: &mut Criterion) {
    let mut testdata = vec![0; 0x100000];
    rand::thread_rng().fill_bytes(&mut testdata);
    let encoded = encode2(&testdata);

    c.bench_function("encoder", |b| {
        b.iter(|| {
            let _ = encode2(black_box(&testdata));
        })
    });

    c.bench_function("encoder_prime", |b| {
        b.iter(|| {
            let _ = encode2(black_box(&testdata[..100003]));
        })
    });

    c.bench_function("encoder_short", |b| {
        b.iter(|| {
            let _ = encode2(black_box(&testdata[..11]));
        })
    });

    c.bench_function("decoder", |b| {
        b.iter(|| {
            let _ = decode2(black_box(encoded.as_bytes()));
        })
    });
}

criterion_group!(benches, encode_noalloc_remapped_benchmark);
criterion_main!(benches);
