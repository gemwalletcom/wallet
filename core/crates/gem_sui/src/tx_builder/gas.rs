use std::array::from_fn;

use crate::{
    SuiError,
    address::SuiAddress,
    models::{Coin, Object, OwnedCoins},
    types::{Address, Digest, TypeTag},
};
use gem_hash::blake2::blake2b_256;
use primitives::hex::decode_hex_literal;

const ACCUMULATOR_ROOT: Address = Address::new(decode_hex_literal("0000000000000000000000000000000000000000000000000000000000000acc"));
const CHILD_OBJECT_ID_SCOPE: u8 = 0xf0;
const RESERVATION_DIGEST_MARKER: [u8; 20] = [0xac; 20];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GasReservationContext {
    pub epoch: u64,
    pub chain_id: Digest,
}

pub(crate) fn requires_address_balance_reservation(coins: &OwnedCoins<Coin>) -> bool {
    coins.coins.is_empty() && coins.address_balance > 0
}

pub(crate) fn reserve_address_balance(mut coins: OwnedCoins<Coin>, sender: &str, context: GasReservationContext) -> Result<OwnedCoins<Coin>, SuiError> {
    if !requires_address_balance_reservation(&coins) {
        return Ok(coins);
    }

    let coin_type = coins.coin_type.parse::<TypeTag>()?;
    let TypeTag::Struct(_) = coin_type else {
        return Err(SuiError::invalid_input(format!("Invalid Sui gas coin type {}", coins.coin_type)));
    };
    let balance = coins.address_balance;

    coins.coins.push(Coin {
        coin_type: coins.coin_type.clone(),
        balance,
        object: reservation_object(coin_type, balance, context, SuiAddress::parse(sender)?.into())?,
    });
    coins.address_balance = 0;
    Ok(coins)
}

fn reservation_object(coin_type: TypeTag, balance: u64, context: GasReservationContext, owner: Address) -> Result<Object, SuiError> {
    let key_type: TypeTag = format!("0x2::accumulator::Key<0x2::balance::Balance<{coin_type}>>").parse()?;
    let key_type_bcs = bcs::to_bytes(&key_type).map_err(SuiError::from_display)?;
    let key_length = (owner.as_bytes().len() as u64).to_le_bytes();
    let accumulator_id = blake2b_256(&[&[CHILD_OBJECT_ID_SCOPE], ACCUMULATOR_ROOT.as_bytes(), &key_length, owner.as_bytes(), &key_type_bcs].concat());
    let chain_id = context.chain_id.into_inner();
    let digest = [balance.to_le_bytes().as_slice(), &(context.epoch as u32).to_le_bytes(), &RESERVATION_DIGEST_MARKER].concat();

    Ok(Object {
        object_id: Address::new(from_fn(|index| accumulator_id[index] ^ chain_id[index])),
        version: 0,
        digest: Digest::try_from(digest)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        SUI_COIN_TYPE,
        models::{Gas, TransferInput},
        tx_builder::encode_transfer,
        types::{ObjectInput, Transaction},
    };
    use primitives::hex;

    #[test]
    fn test_reserve_address_balance() {
        let balance = 319_050_000_000;
        let coins = OwnedCoins::new(SUI_COIN_TYPE.to_string(), vec![], balance);
        let sender = "0x1b4cd8b734f2465614678ca0450ce9c4f2ff4835c6a7545522892a1a8fb67991";
        let context = GasReservationContext {
            epoch: 1_187,
            chain_id: Digest::new([7; 32]),
        };

        let reserved = reserve_address_balance(coins, sender, context).unwrap();

        assert_eq!(reserved.total(), balance);
        assert_eq!(reserved.address_balance, 0);
        assert_eq!(reserved.coins.len(), 1);
        assert_eq!(reserved.coins[0].balance, balance);
        assert_eq!(&reserved.coins[0].object.digest.as_bytes()[12..], &[0xac; 20]);
        assert_eq!(reserved.coins[0].object.object_id.to_string(), "0xc71aeeb7c5a54992d6bfe5227baa96975d42f80f9801e9df59aaabd89829bc87");
        assert_eq!(reserved.coins[0].object.digest.to_string(), "9fCkwTEkNC9ebFKu8NdXvhgt1wBe8RTgKfH9f2D2ESnX");

        let with_coins = OwnedCoins::new(SUI_COIN_TYPE.to_string(), vec![Coin::mock_sui()], 500);
        assert_eq!(reserve_address_balance(with_coins.clone(), sender, context).unwrap(), with_coins);

        let no_balance = OwnedCoins::new(SUI_COIN_TYPE.to_string(), vec![], 0);
        assert_eq!(reserve_address_balance(no_balance.clone(), sender, context).unwrap(), no_balance);

        let gas_object = reserved.coins[0].object;
        let output = encode_transfer(&TransferInput {
            sender: sender.to_string(),
            recipient: "0xcf3abaeecfaf42990b8481c03000000000000000000000000000000000000000".to_string(),
            amount: 5_000_000_000,
            coins: reserved,
            send_max: false,
            gas: Gas { budget: 50_000_000, price: 100 },
        })
        .unwrap();
        let transaction: Transaction = bcs::from_bytes(&output.tx_data).unwrap();

        assert_eq!(transaction.gas_payment.objects.into_iter().map(ObjectInput::ImmutableOrOwned).collect::<Vec<_>>(), vec![gas_object.to_input()]);
        assert_eq!(hex::encode(&output.hash), "351fd5ce9557934e8b720731f517bc4a4eefe43aa02c6cb6a21552dfe81e97a3");
    }
}
