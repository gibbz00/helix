use super::AutoPairConfig;
use crate::auto_pairs::AutoPairs;
use regex::Regex;
use serde::Deserialize;

pub fn deserialize_regex<'de, D>(deserializer: D) -> Result<Option<Regex>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)?
        .map(|buf| Regex::new(&buf).map_err(serde::de::Error::custom))
        .transpose()
}

pub fn deserialize_toml_to_json_value<'de, D>(
    deserializer: D,
) -> Result<Option<serde_json::Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<toml::Value>::deserialize(deserializer)?
        .map(|toml| toml.try_into().map_err(serde::de::Error::custom))
        .transpose()
}

pub fn deserialize_tab_width<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: serde::Deserializer<'de>,
{
    usize::deserialize(deserializer).and_then(|n| {
        if n > 0 && n <= 16 {
            Ok(n)
        } else {
            Err(serde::de::Error::custom(
                "tab width must be a value from 1 to 16 inclusive",
            ))
        }
    })
}

pub fn deserialize_auto_pairs<'de, D>(deserializer: D) -> Result<Option<AutoPairs>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<AutoPairConfig>::deserialize(deserializer)?.and_then(AutoPairConfig::into))
}

pub fn default_timeout() -> u64 {
    20
}
