use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType, KeyPair, SanType};
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

        params.alg = &rcgen::PKCS_RSA_SHA256;
        params.key_pair = Some(KeyPair::from_pem(&key_pem)?);

        let mut subject = DistinguishedName::new();
        subject.push(DnType::CommonName, "Chik");
        subject.push(DnType::OrganizationName, "Chik");
        subject.push(DnType::OrganizationalUnitName, "Organic Farming Division");
        params.distinguished_name = subject;

        params.subject_alt_names = vec![SanType::DnsName("chiknetwork.com".to_string())];

        params.not_before = OffsetDateTime::now_utc() - Duration::DAY;
        params.not_after = PrimitiveDateTime::new(
            Date::from_calendar_date(2100, Month::August, 2)?,
            Time::MIDNIGHT,
        )
        .assume_utc();

        let cert = Certificate::from_params(params)?;
        let cert_pem = cert.serialize_pem_with_signer(&CHIK_CA)?;

        Ok(ChikCertificate { cert_pem, key_pem })
    }
}
