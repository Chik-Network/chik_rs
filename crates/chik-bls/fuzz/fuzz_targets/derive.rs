#![no_main]
use libfuzzer_sys::fuzz_target;

use chik_bls::derivable_key::DerivableKey;
use chik_bls::public_key::PublicKey;
use chik_bls::secret_key::SecretKey;
use chik_bls::signature::{sign, verify};

fuzz_target!(|data: &[u8]| {
    if data.len() < 32 {
        return;
    }

    let sk = SecretKey::from_seed(data);
    let pk = sk.public_key();

    // round-trip SecretKey
    let bytes = sk.to_bytes();
    assert_eq!(sk, SecretKey::from_bytes(&bytes).unwrap());

    // round-trip PublicKey
    let bytes = pk.to_bytes();
    assert_eq!(pk, PublicKey::from_bytes(&bytes).unwrap());

    // unhardened derivation
    let sk1 = sk.derive_unhardened(1337);
    let pk1 = pk.derive_unhardened(1337);

    let sig = sign(&sk1, b"foobar");
    assert!(verify(&sig, &pk1, b"foobar"));
});
