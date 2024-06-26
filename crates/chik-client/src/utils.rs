use chik_traits::{chik_error::Result, Streamable};

pub fn stream<T: Streamable>(value: &T) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    value.stream(&mut bytes)?;
    Ok(bytes)
}
