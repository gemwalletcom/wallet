use std::{cell::Cell, fmt};

use serde::{
    Deserialize, Deserializer, Serialize,
    de::{DeserializeSeed, EnumAccess, Error, SeqAccess, VariantAccess, Visitor},
};

use super::{
    Address, GasPayment, StructTag, Transaction, TransactionExpiration, TransactionKind, TypeTag,
    type_tag::{MAX_TYPE_TAG_DEPTH, MAX_TYPE_TAG_NODES},
};

#[derive(Serialize, Deserialize)]
pub(super) enum TransactionData {
    V1 {
        kind: TransactionKind,
        sender: Address,
        gas_payment: GasPayment,
        expiration: TransactionExpiration,
    },
}

impl From<TransactionData> for Transaction {
    fn from(data: TransactionData) -> Self {
        let TransactionData::V1 { kind, sender, gas_payment, expiration } = data;
        Self { kind, sender, gas_payment, expiration }
    }
}

impl From<Transaction> for TransactionData {
    fn from(transaction: Transaction) -> Self {
        let Transaction { kind, sender, gas_payment, expiration } = transaction;
        Self::V1 { kind, sender, gas_payment, expiration }
    }
}

pub(super) fn deserialize_proposers<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u32>, D::Error> {
    let proposers = Vec::<u32>::deserialize(deserializer)?;
    if proposers.is_empty() {
        return Err(D::Error::custom("Sui allowed proposers are empty"));
    }
    Ok(proposers)
}

impl<'de> Deserialize<'de> for TypeTag {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        TypeTagSeed { depth: 0, nodes: &Cell::new(0) }.deserialize(deserializer)
    }
}

#[derive(Clone, Copy)]
struct TypeTagSeed<'a> {
    depth: usize,
    nodes: &'a Cell<usize>,
}

impl<'de> DeserializeSeed<'de> for TypeTagSeed<'_> {
    type Value = TypeTag;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<TypeTag, D::Error> {
        self.nodes.set(self.nodes.get() + 1);
        if self.nodes.get() > MAX_TYPE_TAG_NODES || self.depth >= MAX_TYPE_TAG_DEPTH {
            return Err(D::Error::custom("Sui type is too large"));
        }
        deserializer.deserialize_enum("TypeTag", &[], self)
    }
}

impl<'de> Visitor<'de> for TypeTagSeed<'_> {
    type Value = TypeTag;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a Sui type tag")
    }

    fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<TypeTag, A::Error> {
        let (variant, access) = data.variant::<u32>()?;
        let nested = Self { depth: self.depth + 1, ..self };
        match variant {
            0 => access.unit_variant().map(|()| TypeTag::Bool),
            1 => access.unit_variant().map(|()| TypeTag::U8),
            2 => access.unit_variant().map(|()| TypeTag::U64),
            3 => access.unit_variant().map(|()| TypeTag::U128),
            4 => access.unit_variant().map(|()| TypeTag::Address),
            5 => access.unit_variant().map(|()| TypeTag::Signer),
            6 => access.newtype_variant_seed(nested).map(|element| TypeTag::Vector(Box::new(element))),
            7 => access.newtype_variant_seed(StructTagSeed(nested)).map(TypeTag::from),
            8 => access.unit_variant().map(|()| TypeTag::U16),
            9 => access.unit_variant().map(|()| TypeTag::U32),
            10 => access.unit_variant().map(|()| TypeTag::U256),
            _ => Err(A::Error::custom(format!("unknown Sui type tag variant {variant}"))),
        }
    }
}

struct StructTagSeed<'a>(TypeTagSeed<'a>);

impl<'de> DeserializeSeed<'de> for StructTagSeed<'_> {
    type Value = StructTag;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<StructTag, D::Error> {
        deserializer.deserialize_struct("StructTag", &["address", "module", "name", "type_params"], self)
    }
}

impl<'de> Visitor<'de> for StructTagSeed<'_> {
    type Value = StructTag;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a Sui struct tag")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut fields: A) -> Result<StructTag, A::Error> {
        let missing = || A::Error::custom("Sui struct tag is incomplete");
        Ok(StructTag {
            address: fields.next_element()?.ok_or_else(missing)?,
            module: fields.next_element()?.ok_or_else(missing)?,
            name: fields.next_element()?.ok_or_else(missing)?,
            type_params: fields.next_element_seed(TypeParamsSeed(self.0))?.ok_or_else(missing)?,
        })
    }
}

struct TypeParamsSeed<'a>(TypeTagSeed<'a>);

impl<'de> DeserializeSeed<'de> for TypeParamsSeed<'_> {
    type Value = Vec<TypeTag>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Vec<TypeTag>, D::Error> {
        deserializer.deserialize_seq(self)
    }
}

impl<'de> Visitor<'de> for TypeParamsSeed<'_> {
    type Value = Vec<TypeTag>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Sui type parameters")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut elements: A) -> Result<Vec<TypeTag>, A::Error> {
        let mut type_params = vec![];
        while let Some(type_param) = elements.next_element_seed(self.0)? {
            type_params.push(type_param);
        }
        Ok(type_params)
    }
}
