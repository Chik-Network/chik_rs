use std::str::FromStr;

use rcgen::{CertificateParams, DistinguishedName, DnType, Ia5String, KeyPair, SanType};
use rsa::{
    pkcs8::{EncodePrivateKey, LineEnding},
    RsaPrivateKey,
};
use time::{Date, Duration, Month, OffsetDateTime, PrimitiveDateTime, Time};

mod ca;
mod error;

pub use ca::*;
pub use error::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChikCertificate {
    pub cert_pem: String,
    pub key_pem: String,
}

impl ChikCertificate {
    pub fn generate() -> Result<ChikCertificate> {
        let mut rng = rand::thread_rng();

        let key = RsaPrivateKey::new(&mut rng, 2048)?;
        let key_pem = key.to_pkcs8_pem(LineEnding::default())?.to_string();

        let mut params = CertificateParams::default();

        let mut subject = DistinguishedName::new();
        subject.push(DnType::CommonName, "Chik");
        subject.push(DnType::OrganizationName, "Chik");
        subject.push(DnType::OrganizationalUnitName, "Organic Farming Division");
        params.distinguished_name = subject;

        params.subject_alt_names = vec![SanType::DnsName(Ia5String::from_str("chiknetwork.com")?)];

        params.not_before = OffsetDateTime::now_utc() - Duration::DAY;
        params.not_after = PrimitiveDateTime::new(
            Date::from_calendar_date(2100, Month::August, 2)?,
            Time::MIDNIGHT,
        )
        .assume_utc();

        let key_pair = KeyPair::from_pem_and_sign_algo(&key_pem, &rcgen::PKCS_RSA_SHA256)?;
        let cert = params.signed_by(&key_pair, &CHIK_CA, &CHIK_CA_KEY_PAIR)?;
        let cert_pem = cert.pem();

        Ok(ChikCertificate { cert_pem, key_pem })
    }
}
