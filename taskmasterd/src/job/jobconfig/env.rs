use serde::Deserialize;
use serde::Deserializer;
use serde_yaml::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Env(HashMap<String, String>);

impl<'de> Deserialize<'de> for Env {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Value::deserialize(deserializer)?;
        let v = v
            .as_mapping()
            .ok_or_else(|| serde::de::Error::custom("Expected a mapping in env"))?;
        let mut env = HashMap::new();
        for (k, v) in v {
            let k = k
                .as_str()
                .ok_or_else(|| serde::de::Error::custom("Expected a string as key in env"))?;
            let v = match v {
                Value::String(s) => s.to_string(),
                Value::Number(n) => n.to_string(),
                _ => {
                    return Err(serde::de::Error::custom(
                        "Expected a string or a number as value in env",
                    ))
                }
            };
            env.insert(k.to_string(), v);
        }
        Ok(Env(env))
    }
}
