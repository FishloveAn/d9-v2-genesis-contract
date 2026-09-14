use schemars::{gen::SchemaGenerator, schema::*, JsonSchema};
use serde::{Deserialize, Serialize};
use sp_core::crypto::{AccountId32, Ss58AddressFormat, Ss58Codec};

fn string_schema(pattern: &str) -> Schema {
    SchemaObject {
        instance_type: Some(InstanceType::String.into()),
        string: Some(Box::new(StringValidation {
            pattern: Some(pattern.into()),
            ..Default::default()
        })),
        ..Default::default()
    }
    .into()
}

macro_rules! decimal {
    ($name:ident, $inner:ty) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(pub $inner);
        impl TryFrom<String> for $name {
            type Error = String;
            fn try_from(value: String) -> Result<Self, String> {
                if value.is_empty()
                    || (value.len() > 1 && value.starts_with('0'))
                    || !value.bytes().all(|b| b.is_ascii_digit())
                {
                    return Err("expected canonical unsigned decimal string".into());
                }
                value
                    .parse::<$inner>()
                    .map(Self)
                    .map_err(|_| stringify!($name).to_owned() + " overflow")
            }
        }
        impl From<$name> for String {
            fn from(value: $name) -> String {
                value.0.to_string()
            }
        }
        impl JsonSchema for $name {
            fn schema_name() -> String {
                stringify!($name).into()
            }
            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                string_schema("^(0|[1-9][0-9]*)$")
            }
        }
    };
}
decimal!(Amount, u128);
decimal!(Millis, u64);
decimal!(Count, u64);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Address(String);

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Address {
    pub fn account_id(&self) -> AccountId32 {
        AccountId32::from_ss58check(&self.0).expect("Address constructor verified the checksum")
    }
    pub fn from_account_id(id: AccountId32) -> Self {
        Self(id.to_ss58check_with_version(Ss58AddressFormat::custom(9)))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<String> for Address {
    type Error = String;
    fn try_from(value: String) -> Result<Self, String> {
        let (id, prefix) = AccountId32::from_ss58check_with_version(&value)
            .map_err(|_| "invalid AccountId32 SS58 checksum/encoding")?;
        let canonical = id.to_ss58check_with_version(Ss58AddressFormat::custom(9));
        if prefix != Ss58AddressFormat::custom(9) || canonical != value {
            return Err("expected canonical D9 SS58 prefix 9".into());
        }
        Ok(Self(value))
    }
}
impl From<Address> for String {
    fn from(value: Address) -> String {
        value.0
    }
}
impl JsonSchema for Address {
    fn schema_name() -> String {
        "Address".into()
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        string_schema("^[1-9A-HJ-NP-Za-km-z]{47,48}$")
    }
}

macro_rules! hex_string {
    ($name:ident, $len:literal, $pattern:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(pub(crate) String);
        impl $name {
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl TryFrom<String> for $name {
            type Error = String;
            fn try_from(value: String) -> Result<Self, String> {
                if value.len() != $len
                    || !value
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                {
                    return Err(concat!("expected lowercase hex ", stringify!($name)).into());
                }
                Ok(Self(value))
            }
        }
        impl From<$name> for String {
            fn from(value: $name) -> String {
                value.0
            }
        }
        impl JsonSchema for $name {
            fn schema_name() -> String {
                stringify!($name).into()
            }
            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                string_schema($pattern)
            }
        }
    };
}
hex_string!(Digest, 64, "^[0-9a-f]{64}$");
hex_string!(Commit, 40, "^[0-9a-f]{40}$");
hex_string!(CeremonyNonce, 64, "^[0-9a-f]{64}$");
hex_string!(SignatureHex, 128, "^[0-9a-f]{128}$");
hex_string!(Pcr0Hex, 96, "^[0-9a-f]{96}$");
