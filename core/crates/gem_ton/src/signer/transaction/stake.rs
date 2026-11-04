use num_bigint::BigUint;
use primitives::{DelegationBase, SignerError, SignerInput, StakeType};

use super::{
    request::{TransferPayload, TransferRequest},
    sign::optional_ton_attachment,
};
use crate::{
    address::Address,
    constants::TONSTAKERS_POOL_ADDRESS,
    signer::signer::TonSigner,
    tvm::{Cell, CellBuilder},
};

const TONSTAKERS_STAKE_OPCODE: u32 = 0x47d54391;
const TONSTAKERS_STAKE_QUERY_ID: u64 = 1;
const TONSTAKERS_PARTNER_CODE: u64 = 0x0000_0010_6765_cd3b;
const JETTON_BURN_OPCODE: u32 = 0x595f07bc;
const JETTON_BURN_QUERY_ID: u64 = 0;

impl TonSigner {
    pub fn sign_stake(&self, input: &SignerInput, expire_at: Option<u32>) -> Result<String, SignerError> {
        let request = match input.input_type.get_stake_type()? {
            StakeType::Stake(_) => build_stake_request(input)?,
            StakeType::Unstake(delegation) => build_unstake_request(input, &delegation.base)?,
            StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Withdraw(_) | StakeType::Freeze(_) | StakeType::Unfreeze(_) => {
                return SignerError::invalid_input_err("unsupported TON stake type");
            }
        };
        self.sign_requests(vec![request], input.metadata.get_sequence()?, expire_at)
    }
}

fn build_stake_request(input: &SignerInput) -> Result<TransferRequest, SignerError> {
    let payload = CellBuilder::new()
        .store_u32(32, TONSTAKERS_STAKE_OPCODE)?
        .store_u64(64, TONSTAKERS_STAKE_QUERY_ID)?
        .store_u64(64, TONSTAKERS_PARTNER_CODE)?
        .end_cell()?
        .into_arc();
    TransferRequest::new_contract_transfer(TONSTAKERS_POOL_ADDRESS, &input.value + stake_attachment(input)?, TransferPayload::Custom(payload))
}

fn build_unstake_request(input: &SignerInput, delegation: &DelegationBase) -> Result<TransferRequest, SignerError> {
    let jetton_wallet = input.metadata.get_sender_token_address()?.ok_or_else(|| SignerError::invalid_input("missing Tonstakers jetton wallet"))?;
    let shares = unstake_shares(&input.value, delegation);
    if shares == BigUint::ZERO {
        return SignerError::invalid_input_err("Tonstakers unstake burns no shares");
    }
    let payload = build_burn_payload(&shares, &Address::parse(&input.sender_address)?)?.into_arc();
    TransferRequest::new_contract_transfer(&jetton_wallet, stake_attachment(input)?, TransferPayload::Custom(payload))
}

fn build_burn_payload(shares: &BigUint, owner: &Address) -> Result<Cell, SignerError> {
    let wait_till_round_end = false;
    let fill_or_kill = false;
    let flags = CellBuilder::new().store_bit(wait_till_round_end)?.store_bit(fill_or_kill)?.end_cell()?.into_arc();
    Ok(CellBuilder::new()
        .store_u32(32, JETTON_BURN_OPCODE)?
        .store_u64(64, JETTON_BURN_QUERY_ID)?
        .store_coins(shares)?
        .store_address(owner)?
        .store_maybe_reference(Some(&flags))?
        .end_cell()?)
}

fn unstake_shares(value: &BigUint, delegation: &DelegationBase) -> BigUint {
    if value >= &delegation.balance {
        return delegation.shares.clone();
    }
    value * &delegation.shares / &delegation.balance
}

fn stake_attachment(input: &SignerInput) -> Result<BigUint, SignerError> {
    let attachment = optional_ton_attachment(input)?;
    if attachment == BigUint::ZERO {
        return SignerError::invalid_input_err("missing Tonstakers message value");
    }
    Ok(attachment)
}

#[cfg(test)]
mod tests {
    use num_bigint::BigInt;
    use primitives::{Asset, AssetId, Chain, Delegation, DelegationValidator, FeeOption, TransactionFee, TransactionInputType, TransactionLoadMetadata};

    use super::*;
    use crate::{
        constants::{TONSTAKERS_STAKE_ATTACHMENT, TONSTAKERS_UNSTAKE_ATTACHMENT},
        signer::testkit::{TEST_ADDRESS, mock_signer},
    };

    const TEST_JETTON_WALLET: &str = "EQAlgB03OjJKdXrlwZiGJD5snSzPKF2VL5bErJn_cqJANGH9";

    #[test]
    fn test_sign_stake() {
        let mut input = SignerInput::mock_with_input_type(
            TransactionInputType::Stake {
                asset: Asset::from_chain(Chain::Ton),
                stake_type: StakeType::Stake(DelegationValidator::mock()),
            },
            TEST_ADDRESS,
            TONSTAKERS_POOL_ADDRESS,
            "5000000000",
            TransactionLoadMetadata::mock_ton(1),
        );
        input.fee = TransactionFee::new_from_fee_with_option(BigInt::from(0), FeeOption::TokenAccountCreation, BigInt::from(TONSTAKERS_STAKE_ATTACHMENT), AssetId::from_chain(Chain::Ton));

        assert_eq!(
            mock_signer().sign_stake(&input, Some(1_000_000_000)).unwrap(),
            "te6cckEBBAEAxQABRYgB/CemYzWmEwzyIuxaWl0M/plaY17LUYh8wbNOtxiaDc4MAQGcpdszAirkbgToIhFt0C1RkrUslWp6ni4VA1l3nJgT76ytSYFTLXrk03pf5m+ZUEi+RWtxtuvBRBZbrqruKXuNCCmpoxc7msoAAAAAAQADAgFqYgBSLYv5QgSRTbwbBxlIIQeJ8n8QfIa/FfxiVjOBks8RfSgLLQXgAAAAAAAAAAAAAAAAAAEDAChH1UORAAAAAAAAAAEAAAAQZ2XNO+0HCpo="
        );

        input.fee = TransactionFee::mock();
        assert_eq!(mock_signer().sign_stake(&input, Some(1_000_000_000)).unwrap_err().to_string(), "Invalid input: missing Tonstakers message value");
    }

    #[test]
    fn test_sign_unstake() {
        let delegation = Delegation::mock_base(DelegationBase {
            balance: BigUint::from(10_000_000_000u64),
            shares: BigUint::from(8_000_000_000u64),
            ..DelegationBase::mock()
        });
        let mut input = SignerInput::mock_with_input_type(
            TransactionInputType::Stake {
                asset: Asset::from_chain(Chain::Ton),
                stake_type: StakeType::Unstake(delegation),
            },
            TEST_ADDRESS,
            TONSTAKERS_POOL_ADDRESS,
            "5000000000",
            TransactionLoadMetadata::mock_ton_jetton(1, TEST_JETTON_WALLET),
        );
        input.fee = TransactionFee::new_from_fee_with_option(BigInt::from(0), FeeOption::TokenAccountCreation, BigInt::from(TONSTAKERS_UNSTAKE_ATTACHMENT), AssetId::from_chain(Chain::Ton));

        assert_eq!(
            mock_signer().sign_stake(&input, Some(1_000_000_000)).unwrap(),
            "te6cckEBBQEA5gABRYgB/CemYzWmEwzyIuxaWl0M/plaY17LUYh8wbNOtxiaDc4MAQGcCl3dL+snI2itXsOnXKj24LX4pjsHy2dFCiggPsS0OqKiKJUOayj9poHW5dTManu4a6iX0FBZK+EGC0ufdH4zBimpoxc7msoAAAAAAQADAgFoYgASwA6bnRklOr1y4MxDEh82TpZnlC7Kl8tiVkz/uVEgGiH0rdQAAAAAAAAAAAAAAAAAAQMBZFlfB7wAAAAAAAAAAE7msoAIALGrip93CRFe/VWhm46zlR7f7z+SqqINgmc2Bv2BSAaPBAABIPGka4w="
        );

        input.input.value = BigUint::from(1u32);
        assert_eq!(mock_signer().sign_stake(&input, Some(1_000_000_000)).unwrap_err().to_string(), "Invalid input: Tonstakers unstake burns no shares");

        input.input.metadata = TransactionLoadMetadata::mock_ton(1);
        assert_eq!(mock_signer().sign_stake(&input, Some(1_000_000_000)).unwrap_err().to_string(), "Invalid input: missing Tonstakers jetton wallet");

        input.input.input_type = TransactionInputType::Stake {
            asset: Asset::from_chain(Chain::Ton),
            stake_type: StakeType::Withdraw(Delegation::mock()),
        };
        assert_eq!(mock_signer().sign_stake(&input, Some(1_000_000_000)).unwrap_err().to_string(), "Invalid input: unsupported TON stake type");
    }

    #[test]
    fn test_unstake_shares() {
        let delegation = DelegationBase {
            balance: BigUint::from(1_100u32),
            shares: BigUint::from(1_000u32),
            ..DelegationBase::mock()
        };

        assert_eq!(unstake_shares(&BigUint::from(550u32), &delegation), BigUint::from(500u32));
        assert_eq!(unstake_shares(&BigUint::from(1_200u32), &delegation), BigUint::from(1_000u32));
        assert_eq!(unstake_shares(&BigUint::from(1u32), &delegation), BigUint::ZERO);
    }
}
