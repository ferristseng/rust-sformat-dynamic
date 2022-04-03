use crate::compile::{compile, CompiledFormat};
use serde::de::{self, Unexpected, Visitor};
use std::fmt;

struct CompiledFormatVisitor;

impl<'de> Visitor<'de> for CompiledFormatVisitor {
    type Value = CompiledFormat<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("expected a valid string format")
    }

    fn visit_borrowed_str<E>(self, val: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        compile(val).map_err(|_err| de::Error::invalid_value(Unexpected::Str(val), &self))
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<CompiledFormat<'de>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_str(CompiledFormatVisitor)
}
