use chik_traits::{Streamable, chik_error::Result};

pub fn stream<T: Streamable>(value: &T) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    value.stream(&mut bytes)?;
    Ok(bytes)
}
