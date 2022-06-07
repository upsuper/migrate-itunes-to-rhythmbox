use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};
use std::convert::TryFrom;
use std::fmt::{Error, Formatter};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TrackId(pub u64);

impl<'de> Deserialize<'de> for TrackId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(TrackIdVisitor)
    }
}

struct TrackIdVisitor;

impl<'de> Visitor<'de> for TrackIdVisitor {
    type Value = TrackId;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> Result<(), Error> {
        formatter.write_str("an unsigned integer for track id")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let id = TryFrom::try_from(v).map_err(E::custom)?;
        Ok(TrackId(id))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let id = v.parse().map_err(E::custom)?;
        Ok(TrackId(id))
    }
}
