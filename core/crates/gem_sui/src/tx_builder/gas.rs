use crate::{
    SuiError,
    address::SuiAddress,
    models::{Coin, Object, OwnedCoins},
};
use sui_types::{Digest, ObjectReference, TypeTag};

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
    let TypeTag::Struct(coin_type) = coin_type else {
        return Err(SuiError::invalid_input(format!("Invalid Sui gas coin type {}", coins.coin_type)));
    };
    let owner = SuiAddress::parse(sender)?.into();
    let balance = coins.address_balance;
    let (reservation_object_id, reservation_version, reservation_digest) =
        ObjectReference::coin_reservation(&coin_type, balance, context.epoch, context.chain_id, owner).into_parts();

    coins.coins.push(Coin {
        coin_type: coins.coin_type.clone(),
        balance,
        object: Object {
            object_id: reservation_object_id,
            version: reservation_version,
            digest: reservation_digest,
        },
    });
    coins.address_balance = 0;
    Ok(coins)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        SUI_COIN_TYPE,
        models::{Gas, TransferInput},
        tx_builder::encode_transfer,
    };
    use sui_types::Transaction;

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

        assert_eq!(
            transaction.gas_payment.objects,
            vec![ObjectReference::new(gas_object.object_id, gas_object.version, gas_object.digest)]
        );
    }
}
