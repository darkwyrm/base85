use base85::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::RngCore;

fn encode_benchmark(c: &mut Criterion) {
    let mut testdata = vec![0; 0x100000];
    rand::thread_rng().fill_bytes(&mut testdata);
    let encoded = encode(&testdata);

    c.bench_function("encoder", |b| {
        b.iter(|| {
            let _ = encode(black_box(&testdata));
        })
    });

    c.bench_function("decoder", |b| {
        b.iter(|| {
            let _ = decode(black_box(&encoded));
        })
    });
}

criterion_group!(benches, encode_benchmark);
criterion_main!(benches);
