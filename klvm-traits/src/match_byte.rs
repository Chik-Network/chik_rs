use num_bigint::BigInt;

use crate::{FromKlvm, FromKlvmError, KlvmDecoder, KlvmEncoder, ToKlvm, ToKlvmError};

#[derive(Debug, Copy, Clone)]
pub struct MatchByte<const BYTE: u8>;

impl<N, const BYTE: u8> ToKlvm<N> for MatchByte<BYTE> {
    fn to_klvm(&self, encoder: &mut impl KlvmEncoder<Node = N>) -> Result<N, ToKlvmError> {
        if BYTE == 0 {
            return encoder.encode_atom(&[]);
        }
        let number = BigInt::from(BYTE);
        let bytes = number.to_signed_bytes_be();
        encoder.encode_atom(&bytes)
    }
}

impl<N, const BYTE: u8> FromKlvm<N> for MatchByte<BYTE> {
    fn from_klvm(decoder: &impl KlvmDecoder<Node = N>, node: N) -> Result<Self, FromKlvmError> {
        match decoder.decode_atom(&node)? {
            [] if BYTE == 0 => Ok(Self),
            [byte] if *byte == BYTE && BYTE > 0 => Ok(Self),
            _ => Err(FromKlvmError::Custom(format!(
                "expected an atom with a single byte value of {}",
                BYTE
            ))),
        }
    }
}
