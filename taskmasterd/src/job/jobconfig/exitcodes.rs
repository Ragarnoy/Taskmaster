use serde::{Deserialize, Deserializer};
use serde_yaml::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExitCodes(Vec<i32>);

impl Default for ExitCodes {
    fn default() -> Self {
        Self(vec![0])
    }
}

/// Deserialize the ExitCodes with serde and handle both numbers and sequences of numbers
impl<'de> Deserialize<'de> for ExitCodes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Value::deserialize(deserializer)?;
        match v {
            Value::Number(x) => Ok(Self(vec![x
                .as_i64()
                .ok_or(serde::de::Error::custom("Invalid exit code"))?
                as i32])),
            Value::Sequence(x) => Ok(Self(
                x.iter()
                    .map(|x| {
                        x.as_i64()
                            .ok_or(serde::de::Error::custom("Invalid exit code"))
                    })
                    .map(|r: Result<i64, D::Error>| match r {
                        Ok(x) => Ok(x as i32),
                        Err(r) => Err(r),
                    })
                    .collect::<Result<Vec<i32>, _>>()?,
            )),
            _ => Err(serde::de::Error::custom("Invalid exit codes")),
        }
    }
}

impl ExitCodes {
    #[inline]
    pub fn is_valid(&self, code: i32) -> bool {
        self.0.contains(&code)
    }
}
