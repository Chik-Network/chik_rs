use klvmr::{
    allocator::{NodePtr, SExp},
    Allocator,
};

use crate::{Error, FromKlvm, Result, ToKlvm};

#[derive(Debug, Copy, Clone)]
pub struct MatchByte<const BYTE: u8>;

impl<const BYTE: u8> ToKlvm for MatchByte<BYTE> {
    fn to_klvm(&self, a: &mut Allocator) -> Result<NodePtr> {
        match BYTE {
            0 => Ok(a.null()),
            1 => Ok(a.one()),
            _ => Ok(a.new_number(BYTE.into())?),
        }
    }
}

impl<const BYTE: u8> FromKlvm for MatchByte<BYTE> {
    fn from_klvm(a: &Allocator, node: NodePtr) -> Result<Self> {
        if let SExp::Atom = a.sexp(node) {
            match a.atom(node) {
                [] if BYTE == 0 => Ok(Self),
                [byte] if *byte == BYTE && BYTE > 0 => Ok(Self),
                _ => Err(Error::Custom(format!(
                    "expected an atom with a value of {}",
                    BYTE
                ))),
            }
        } else {
            Err(Error::ExpectedAtom(node))
        }
    }
}
