use criterion::{Criterion, criterion_group, criterion_main};
use kctf_pow::ChallengeParams;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("Difficulty 100", |b| {
        let chall = ChallengeParams::generate_challenge(100);
        b.iter(|| chall.clone().solve())
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
