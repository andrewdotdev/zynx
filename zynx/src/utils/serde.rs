pub mod flags_numeric {
    use bitflags::Flags;

    pub fn serialize<B, S>(flags: &B, serializer: S) -> Result<S::Ok, S::Error>
    where
        B: Flags,
        B::Bits: ::serde::Serialize,
        S: ::serde::Serializer,
    {
        ::serde::Serialize::serialize(&flags.bits(), serializer)
    }

    pub fn deserialize<'de, B, D>(deserializer: D) -> Result<B, D::Error>
    where
        B: Flags,
        B::Bits: ::serde::Deserialize<'de> + Copy,
        D: ::serde::Deserializer<'de>,
    {
        let bits = <B::Bits as ::serde::Deserialize<'de>>::deserialize(deserializer)?;
        Ok(B::from_bits_retain(bits))
    }
}

pub mod flags_numeric_opt {
    use bitflags::Flags;
    use serde::Deserialize;

    pub fn serialize<B, S>(v: &Option<B>, serializer: S) -> Result<S::Ok, S::Error>
    where
        B: Flags,
        B::Bits: ::serde::Serialize,
        S: ::serde::Serializer,
    {
        match v {
            Some(flags) => ::serde::Serialize::serialize(&flags.bits(), serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, B, D>(deserializer: D) -> Result<Option<B>, D::Error>
    where
        B: Flags,
        B::Bits: ::serde::Deserialize<'de> + Copy,
        D: ::serde::Deserializer<'de>,
    {
        let bits_opt = Option::<B::Bits>::deserialize(deserializer)?;
        Ok(bits_opt.map(B::from_bits_retain))
    }
}

// Decode wire unions by their numeric discriminator, never by overlapping shape.
// Keep the discriminator in the value so the selected model retains it.
macro_rules! numeric_union {
    ($name:ident, $pointer:literal, { $($pattern:pat => $variant:ident),+ $(,)? }) => {
        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = <serde_json::Value as serde::Deserialize>::deserialize(deserializer)?;
                let tag = value.pointer($pointer).and_then(serde_json::Value::as_u64)
                    .ok_or_else(|| <D::Error as serde::de::Error>::custom(
                        concat!(stringify!($name), " requires numeric ", $pointer)))?;
                match tag {
                    $($pattern => serde_json::from_value(value).map(Self::$variant)
                        .map_err(<D::Error as serde::de::Error>::custom),)+
                    _ => Err(<D::Error as serde::de::Error>::custom(format!(
                        "unsupported {} {}: {}", stringify!($name), $pointer, tag))),
                }
            }
        }
    };
}
pub(crate) use numeric_union;

/// RESUMED has no event fields but uses an empty object on the wire.
pub mod empty_event {
    pub fn serialize<S: ::serde::Serializer>(_: &(), serializer: S) -> Result<S::Ok, S::Error> {
        use ::serde::ser::SerializeMap;
        serializer.serialize_map(Some(0))?.end()
    }

    pub fn deserialize<'de, D: ::serde::Deserializer<'de>>(d: D) -> Result<(), D::Error> {
        let value = <serde_json::Value as ::serde::Deserialize>::deserialize(d)?;
        if value.is_null() || value.as_object().is_some_and(|map| map.is_empty()) {
            Ok(())
        } else {
            Err(<D::Error as ::serde::de::Error>::custom(
                "expected empty event object or null",
            ))
        }
    }
}

/// Preserve an explicit JSON null as Some(None), distinct from an absent field.
pub fn present_nullable<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: ::serde::Deserialize<'de>,
    D: ::serde::Deserializer<'de>,
{
    <Option<T> as ::serde::Deserialize>::deserialize(deserializer).map(Some)
}

pub mod flags_numeric_nullable {
    use bitflags::Flags;

    pub fn serialize<B, S>(value: &Option<Option<B>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        B: Flags,
        B::Bits: ::serde::Serialize,
        S: ::serde::Serializer,
    {
        ::serde::Serialize::serialize(
            &value.as_ref().and_then(|v| v.as_ref()).map(B::bits),
            serializer,
        )
    }

    pub fn deserialize<'de, B, D>(deserializer: D) -> Result<Option<Option<B>>, D::Error>
    where
        B: Flags,
        B::Bits: ::serde::Deserialize<'de> + Copy,
        D: ::serde::Deserializer<'de>,
    {
        super::flags_numeric_opt::deserialize(deserializer).map(Some)
    }
}
