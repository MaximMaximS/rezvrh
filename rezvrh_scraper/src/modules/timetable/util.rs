use serde::{de::IntoDeserializer, Deserialize};

pub fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let opt = Option::<String>::deserialize(de)?;
    let opt = opt.as_deref();
    match opt {
        None | Some("") => Ok(None),
        Some(s) => T::deserialize(s.into_deserializer()).map(Some),
    }
}

pub fn single_iter<T, I, E, F>(mut iter: I, err: F) -> Result<T, E>
where
    I: Iterator<Item = T>,
    F: FnOnce() -> E + Copy,
{
    let first = iter.next().ok_or_else(err)?;
    if iter.next().is_some() {
        return Err(err());
    }
    Ok(first)
}
