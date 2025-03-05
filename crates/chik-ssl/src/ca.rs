use rcgen::{Certificate, CertificateParams, KeyPair};
use std::sync::LazyLock;

pub const CHIK_CA_KEY: &str = include_str!("../chik_ca.key");
pub const CHIK_CA_CRT: &str = include_str!("../chik_ca.crt");

pub static CHIK_CA: LazyLock<Certificate> = LazyLock::new(load_ca_cert);
pub static CHIK_CA_KEY_PAIR: LazyLock<KeyPair> =
    LazyLock::new(|| KeyPair::from_pem(CHIK_CA_KEY).expect("could not load CA keypair"));

fn load_ca_cert() -> Certificate {
    let params =
        CertificateParams::from_ca_cert_pem(CHIK_CA_CRT).expect("could not create CA params");
    params
        .self_signed(&CHIK_CA_KEY_PAIR)
        .expect("could not create certificate")
}
