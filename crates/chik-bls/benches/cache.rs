use chik_bls::aggregate_verify;
use chik_bls::{sign, BlsCache, SecretKey, Signature};
use criterion::{criterion_group, criterion_main, Criterion};
use rand::rngs::StdRng;
use rand::{seq::SliceRandom, Rng, SeedableRng};

fn cache_benchmark(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(1337);
    let mut data = [0u8; 32];
    rng.fill(data.as_mut_slice());

    let sk = SecretKey::from_seed(&data);
    let msg = b"The quick brown fox jumps over the lazy dog";

    let mut pks = Vec::new();

    let mut agg_sig = Signature::default();
    for i in 0..1000 {
        let derived = sk.derive_hardened(i);
        let pk = derived.public_key();
        let sig = sign(&derived, msg);
        agg_sig.aggregate(&sig);
        pks.push(pk);
    }

    let bls_cache = BlsCache::default();

    c.bench_function("bls_cache.aggregate_verify, 0% cache hits", |b| {
        let cache = bls_cache.clone();
        b.iter(|| {
            assert!(cache.aggregate_verify(pks.iter().zip([&msg].iter().cycle()), &agg_sig));
        });
    });

    // populate 10% of keys
    bls_cache.aggregate_verify(pks[0..100].iter().zip([&msg].iter().cycle()), &agg_sig);
    c.bench_function("bls_cache.aggregate_verify, 10% cache hits", |b| {
        let cache = bls_cache.clone();
        b.iter(|| {
            assert!(cache.aggregate_verify(pks.iter().zip([&msg].iter().cycle()), &agg_sig));
        });
    });

    // populate another 10% of keys
    bls_cache.aggregate_verify(pks[100..200].iter().zip([&msg].iter().cycle()), &agg_sig);
    c.bench_function("bls_cache.aggregate_verify, 20% cache hits", |b| {
        let cache = bls_cache.clone();
        b.iter(|| {
            assert!(cache.aggregate_verify(pks.iter().zip([&msg].iter().cycle()), &agg_sig));
        });
    });

    // populate another 30% of keys
    bls_cache.aggregate_verify(pks[200..500].iter().zip([&msg].iter().cycle()), &agg_sig);
    c.bench_function("bls_cache.aggregate_verify, 50% cache hits", |b| {
        let cache = bls_cache.clone();
        b.iter(|| {
            assert!(cache.aggregate_verify(pks.iter().zip([&msg].iter().cycle()), &agg_sig));
        });
    });

    // populate all other keys
    bls_cache.aggregate_verify(pks[500..1000].iter().zip([&msg].iter().cycle()), &agg_sig);
    c.bench_function("bls_cache.aggregate_verify, 100% cache hits", |b| {
        let cache = bls_cache.clone();
        b.iter(|| {
            assert!(cache.aggregate_verify(pks.iter().zip([&msg].iter().cycle()), &agg_sig));
        });
    });

    c.bench_function("bls_cache.aggregate_verify, no cache", |b| {
        b.iter(|| {
            assert!(aggregate_verify(
                &agg_sig,
                pks.iter().map(|pk| (pk, &msg[..]))
            ));
        });
    });

    // Add more pairs to the cache so we can evict a relatively larger number
    for i in 1_000..20_000 {
        let derived = sk.derive_hardened(i);
        let pk = derived.public_key();
        let sig = sign(&derived, msg);
        agg_sig.aggregate(&sig);
        pks.push(pk);
    }
    bls_cache.aggregate_verify(
        pks[1_000..20_000].iter().zip([&msg].iter().cycle()),
        &agg_sig,
    );

    c.bench_function("bls_cache.evict 5% of the items", |b| {
        let cache = bls_cache.clone();
        let mut pks_shuffled = pks.clone();
        pks_shuffled.shuffle(&mut rng);
        b.iter(|| {
            if cache.is_empty() {
                return;
            }
            cache.evict(pks_shuffled.iter().take(1_000).zip([&msg].iter().cycle()));
        });
    });

    c.bench_function("bls_cache.evict 100% of the items", |b| {
        let cache = bls_cache.clone();
        let mut pks_shuffled = pks.clone();
        pks_shuffled.shuffle(&mut rng);
        b.iter(|| {
            if cache.is_empty() {
                return;
            }
            cache.evict(pks_shuffled.iter().zip([&msg].iter().cycle()));
        });
    });
}

criterion_group!(cache, cache_benchmark);
criterion_main!(cache);
