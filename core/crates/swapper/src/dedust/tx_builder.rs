use super::{
    constants::{FEE_JETTON_FORWARD_AMOUNT, FEE_JETTON_TRANSFER_ATTACHMENT, JETTON_SWAP_ATTACHMENT, JETTON_SWAP_OPCODE, NATIVE_SWAP_OPCODE, SWAP_GAS},
    model::PoolRoute,
};
use crate::{
    Quote, SwapperError,
    fees::{ReferralFee, subtract_bps},
};
use gem_ton::{
    address::Address,
    models::{JettonTransfer, TonConnectMessage},
    tvm::{BagOfCells, Cell, CellBuilder},
};
use num_bigint::BigUint;

#[derive(Debug)]
pub(super) struct DedustSwap {
    pub amount: BigUint,
    pub fee: BigUint,
    pub pools: Vec<Address>,
    pub min_amount_out: BigUint,
    pub wallet: Address,
    pub recipient: Address,
    pub fee_receiver: Address,
}

impl DedustSwap {
    pub fn new(quote: &Quote, route: &PoolRoute, referral_fee: &ReferralFee) -> Result<Self, SwapperError> {
        let amount = subtract_bps(&quote.from_value, referral_fee.bps);
        let recipient = match quote.request.destination_address.is_empty() {
            true => &quote.request.wallet_address,
            false => &quote.request.destination_address,
        };
        Ok(Self {
            fee: &quote.from_value - &amount,
            amount,
            pools: route.pool_addresses()?,
            min_amount_out: subtract_bps(&quote.to_value, quote.data.slippage_bps),
            wallet: Address::parse(&quote.request.wallet_address)?,
            recipient: Address::parse(recipient)?,
            fee_receiver: Address::parse(&referral_fee.address)?,
        })
    }

    pub fn native_messages(&self, vault: &Address) -> Result<Vec<TonConnectMessage>, SwapperError> {
        let body = CellBuilder::new()
            .store_u32(32, NATIVE_SWAP_OPCODE)?
            .store_u64(64, 0)?
            .store_coins(&self.amount)?
            .store_cell(&self.swap_step(&self.pools)?)?
            .store_reference(&self.swap_params()?.into_arc())?
            .end_cell()?;
        let swap = TonConnectMessage::new(vault.encode_bounceable(), &(&self.amount + SWAP_GAS), Some(boc(body)?));
        let fee = TonConnectMessage::new(self.fee_receiver.encode_non_bounceable(), &self.fee, None);
        Ok(self.with_fee(swap, fee))
    }

    pub fn jetton_messages(&self, vault: &Address, jetton_wallet: &Address) -> Result<Vec<TonConnectMessage>, SwapperError> {
        let payload = CellBuilder::new()
            .store_u32(32, JETTON_SWAP_OPCODE)?
            .store_cell(&self.swap_step(&self.pools)?)?
            .store_reference(&self.swap_params()?.into_arc())?
            .end_cell()?;
        let swap_body = JettonTransfer::new(self.amount.clone(), *vault, Some(self.wallet), BigUint::from(SWAP_GAS), Some(payload.into_arc())).to_cell()?;
        let fee_body = JettonTransfer::new(self.fee.clone(), self.fee_receiver, Some(self.wallet), BigUint::from(FEE_JETTON_FORWARD_AMOUNT), None).to_cell()?;
        let swap = TonConnectMessage::new(jetton_wallet.encode_bounceable(), &BigUint::from(JETTON_SWAP_ATTACHMENT), Some(boc(swap_body)?));
        let fee = TonConnectMessage::new(jetton_wallet.encode_bounceable(), &BigUint::from(FEE_JETTON_TRANSFER_ATTACHMENT), Some(boc(fee_body)?));
        Ok(self.with_fee(swap, fee))
    }

    fn with_fee(&self, swap: TonConnectMessage, fee: TonConnectMessage) -> Vec<TonConnectMessage> {
        match self.fee == BigUint::ZERO {
            true => vec![swap],
            false => vec![swap, fee],
        }
    }

    fn swap_step(&self, pools: &[Address]) -> Result<Cell, SwapperError> {
        let (pool, next_pools) = pools.split_first().ok_or(SwapperError::InvalidRoute)?;
        let next = match next_pools.is_empty() {
            true => None,
            false => Some(self.swap_step(next_pools)?.into_arc()),
        };
        let limit = match next {
            Some(_) => BigUint::ZERO,
            None => self.min_amount_out.clone(),
        };
        Ok(CellBuilder::new().store_address(pool)?.store_bit(false)?.store_coins(&limit)?.store_maybe_reference(next.as_ref())?.end_cell()?)
    }

    fn swap_params(&self) -> Result<Cell, SwapperError> {
        Ok(CellBuilder::new()
            .store_u32(32, 0)?
            .store_address(&self.recipient)?
            .store_null_address()?
            .store_maybe_reference(None)?
            .store_maybe_reference(None)?
            .end_cell()?)
    }
}

fn boc(cell: Cell) -> Result<String, SwapperError> {
    Ok(BagOfCells::from_root(cell).to_base64(true)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ProviderData,
        dedust::testkit::{NATIVE_VAULT, TEST_USDT_WALLET, TON_NOT_POOL, TON_USDT_POOL, USDT_VAULT, mock_request, not, ton, usdt},
        fees::default_referral_fees,
    };
    use primitives::AssetId;

    fn swap(from: AssetId, to: AssetId, from_value: u64, to_value: u64, pools: &[&str], referral_fee: &ReferralFee) -> DedustSwap {
        let quote = Quote::mock_with_request(&mock_request(from, to, from_value));
        let quote = Quote {
            to_value: BigUint::from(to_value),
            data: ProviderData { slippage_bps: 100, ..quote.data.clone() },
            ..quote
        };
        let route = PoolRoute {
            pools: pools.iter().map(ToString::to_string).collect(),
        };
        DedustSwap::new(&quote, &route, referral_fee).unwrap()
    }

    fn messages(json: &str) -> Vec<TonConnectMessage> {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_native_messages() {
        let swap = swap(ton(), usdt(), 1_000_000_000, 1_361_182, &[TON_USDT_POOL], &default_referral_fees().ton);

        assert_eq!(swap.native_messages(&Address::parse(NATIVE_VAULT).unwrap()).unwrap(), messages(include_str!("testdata/ton_to_usdt_messages.json")));
    }

    #[test]
    fn test_jetton_messages() {
        let swap = swap(usdt(), ton(), 5_000_000, 3_629_366_982, &[TON_USDT_POOL], &default_referral_fees().ton);
        let jetton_messages = swap.jetton_messages(&Address::parse(USDT_VAULT).unwrap(), &Address::parse(TEST_USDT_WALLET).unwrap()).unwrap();

        assert_eq!(jetton_messages, messages(include_str!("testdata/usdt_to_ton_messages.json")));
    }

    #[test]
    fn test_multi_hop_jetton_messages() {
        let swap = swap(usdt(), not(), 5_000_000, 12_507_066_545_494, &[TON_USDT_POOL, TON_NOT_POOL], &default_referral_fees().ton);
        let jetton_messages = swap.jetton_messages(&Address::parse(USDT_VAULT).unwrap(), &Address::parse(TEST_USDT_WALLET).unwrap()).unwrap();

        assert_eq!(jetton_messages, messages(include_str!("testdata/usdt_to_not_messages.json")));
    }

    #[test]
    fn test_messages_without_fee() {
        let referral_fee = ReferralFee { bps: 0, ..default_referral_fees().ton };
        let swap = swap(ton(), usdt(), 1_000_000_000, 1_361_182, &[TON_USDT_POOL], &referral_fee);

        assert_eq!(swap.fee, BigUint::ZERO);
        assert_eq!(swap.native_messages(&Address::parse(NATIVE_VAULT).unwrap()).unwrap().len(), 1);
    }
}
