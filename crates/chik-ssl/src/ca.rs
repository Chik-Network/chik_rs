use lazy_static::lazy_static;
use rcgen::{Certificate, CertificateParams, KeyPair};

pub const CHIK_CA_KEY: &str = include_str!("../chik_ca.key");
pub const CHIK_CA_CRT: &str = include_str!("../chik_ca.crt");

lazy_static! {
    pub static ref CHIK_CA: Certificate = load_ca_cert();
}

fn load_ca_cert() -> Certificate {
    let key_pair = KeyPair::from_pem(CHIK_CA_KEY).expect("could not load CA keypair");
    let params = CertificateParams::from_ca_cert_pem(CHIK_CA_CRT, key_pair)
        .expect("could not create CA params");
    Certificate::from_params(params).expect("could not create certificate")
}
