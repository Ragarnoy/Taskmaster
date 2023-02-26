use serde::Deserialize;

#[derive(Debug, Clone, Copy)]
pub struct Umask(pub u32);

impl<'de> Deserialize<'de> for Umask {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let umask = u32::from_str_radix(&s, 8).map_err(serde::de::Error::custom)?;
        Ok(Self(umask))
    }
}
