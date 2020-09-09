pub mod name_mapper;

use serde::de::{self, Deserializer, Deserialize, Visitor};
use serde::{ser, Serialize, Serializer};

use std::fmt;
use name_mapper::Name;

impl<'de> Deserialize<'de> for name_mapper::Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de> {

            struct NameVisitor;

            impl<'de> Visitor<'de> for NameVisitor {
                type Value = Name;

                fn visit_str<E>(self, value: &str) -> Result<Name,E>
                where
                    E: de::Error
                {
                    let namer = name_mapper::namer();
                    Ok(namer.get_name(String::from(value)))
                }

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("`name`")
                }
    }
    deserializer.deserialize_identifier(NameVisitor)
    }
}

impl Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let namer = name_mapper::namer();
        let string = namer.get_string(&self);
        serializer.serialize_str(&string)
    }
}
