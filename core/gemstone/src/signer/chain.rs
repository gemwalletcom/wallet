use crate::{
    GemstoneError,
    models::transaction::{GemSignerInput, GemTransactionInputType},
};
use gem_algorand::AlgorandChainSigner;
use gem_aptos::AptosChainSigner;
use gem_bitcoin::signer::BitcoinChainSigner;
use gem_cardano::signer::CardanoChainSigner;
use gem_cosmos::signer::CosmosChainSigner;
use gem_evm::signer::EvmChainSigner;
use gem_hypercore::signer::HyperCoreSigner;
use gem_near::NearChainSigner;
use gem_polkadot::signer::PolkadotChainSigner;
use gem_solana::signer::SolanaChainSigner;
use gem_stellar::StellarChainSigner;
use gem_sui::signer::SuiChainSigner;
use gem_ton::signer::TonChainSigner;
use gem_tron::signer::TronChainSigner;
use gem_xrp::signer::XrpChainSigner;
use primitives::swap::{SwapData, SwapQuoteDataType};
use primitives::{Asset, BitcoinChain, Chain, ChainSigner, ChainType, EVMChain, SignerError, SignerInput, TransactionInputType};
use zeroize::Zeroizing;

pub struct GemChainSigner {
    chain: Chain,
    signer: Box<dyn ChainSigner>,
}

impl GemChainSigner {
    pub fn new(chain: Chain) -> Self {
        let signer: Box<dyn ChainSigner> = match chain.chain_type() {
            ChainType::Ethereum => Box::new(EvmChainSigner::new(EVMChain::from_chain(chain).unwrap())),
            ChainType::Aptos => Box::new(AptosChainSigner),
            ChainType::HyperCore => Box::new(HyperCoreSigner),
            ChainType::Sui => Box::new(SuiChainSigner),
            ChainType::Solana => Box::new(SolanaChainSigner),
            ChainType::Ton => Box::new(TonChainSigner),
            ChainType::Tron => Box::new(TronChainSigner),
            ChainType::Cosmos => Box::new(CosmosChainSigner),
            ChainType::Near => Box::new(NearChainSigner),
            ChainType::Algorand => Box::new(AlgorandChainSigner),
            ChainType::Stellar => Box::new(StellarChainSigner),
            ChainType::Xrp => Box::new(XrpChainSigner),
            ChainType::Polkadot => Box::new(PolkadotChainSigner),
            ChainType::Cardano => Box::new(CardanoChainSigner),
            ChainType::Bitcoin => Box::new(BitcoinChainSigner::new(BitcoinChain::from_chain(chain).unwrap())),
        };

        Self { chain, signer }
    }

    pub fn sign_transfer(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "transfer", |signer, signer_input, key| signer.sign_transfer(signer_input, key))
    }

    pub fn sign_token_transfer(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "token transfer", |signer, signer_input, key| {
            signer.sign_token_transfer(signer_input, key)
        })
    }

    pub fn sign_nft_transfer(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "nft transfer", |signer, signer_input, key| signer.sign_nft_transfer(signer_input, key))
    }

    pub fn sign_swap(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        self.dispatch(input, private_key, "swap", |signer, signer_input, key| signer.sign_swap(signer_input, key))
    }

    pub fn sign_token_approval(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "token approval", |signer, signer_input, key| {
            signer.sign_token_approval(signer_input, key)
        })
    }

    pub fn sign_stake(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        self.dispatch(input, private_key, "stake", |signer, signer_input, key| signer.sign_stake(signer_input, key))
    }

    pub fn sign_account_action(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "account action", |signer, signer_input, key| {
            signer.sign_account_action(signer_input, key)
        })
    }

    pub fn sign_perpetual(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        self.dispatch(input, private_key, "perpetual", |signer, signer_input, key| signer.sign_perpetual(signer_input, key))
    }

    pub fn sign_withdrawal(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "withdrawal", |signer, signer_input, key| signer.sign_withdrawal(signer_input, key))
    }

    pub fn sign_data(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        self.dispatch(input, private_key, "data", |signer, signer_input, key| signer.sign_data(signer_input, key))
    }

    pub fn sign_earn(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        self.dispatch(input, private_key, "earn", |signer, signer_input, key| signer.sign_earn(signer_input, key))
    }

    pub fn sign_message(&self, message: Vec<u8>, private_key: Vec<u8>) -> Result<String, GemstoneError> {
        let private_key = Zeroizing::new(private_key);
        self.dispatch_message(&message, private_key.as_slice(), "message", |signer, msg, key| signer.sign_message(msg, key))
    }
}

impl GemChainSigner {
    pub fn sign_input(&self, input: GemSignerInput, private_key: Vec<u8>) -> Result<Vec<String>, GemstoneError> {
        // Withdrawal is gemstone-only and lowers to a plain Transfer, so capture it before conversion.
        let is_withdrawal = matches!(input.input.input_type, GemTransactionInputType::Withdrawal { .. });
        let signer_input: SignerInput = input.into();
        let private_key = Zeroizing::new(private_key);
        self.route(&signer_input, private_key.as_slice(), is_withdrawal)
    }

    fn route(&self, input: &SignerInput, private_key: &[u8], is_withdrawal: bool) -> Result<Vec<String>, GemstoneError> {
        if is_withdrawal {
            return self.one(input, private_key, "withdrawal", |signer, i, key| signer.sign_withdrawal(i, key));
        }
        match &input.input.input_type {
            TransactionInputType::Transfer(asset) | TransactionInputType::Deposit(asset) => {
                if asset.id.is_token() {
                    self.one(input, private_key, "token transfer", |signer, i, key| signer.sign_token_transfer(i, key))
                } else {
                    self.one(input, private_key, "transfer", |signer, i, key| signer.sign_transfer(i, key))
                }
            }
            TransactionInputType::TransferNft(_, _) => self.one(input, private_key, "nft transfer", |signer, i, key| signer.sign_nft_transfer(i, key)),
            TransactionInputType::TokenApprove(_, _) => self.one(input, private_key, "token approval", |signer, i, key| signer.sign_token_approval(i, key)),
            TransactionInputType::Generic(_, _, _) => self.one(input, private_key, "data", |signer, i, key| signer.sign_data(i, key)),
            TransactionInputType::Account(_, _) => self.one(input, private_key, "account action", |signer, i, key| signer.sign_account_action(i, key)),
            TransactionInputType::Stake(_, _) => self.many(input, private_key, "stake", |signer, i, key| signer.sign_stake(i, key)),
            TransactionInputType::Perpetual(_, _) => self.many(input, private_key, "perpetual", |signer, i, key| signer.sign_perpetual(i, key)),
            TransactionInputType::Earn(_, _, _) => self.many(input, private_key, "earn", |signer, i, key| signer.sign_earn(i, key)),
            TransactionInputType::Swap(from_asset, _, swap_data) => match swap_data.data.data_type {
                SwapQuoteDataType::Contract => self.many(input, private_key, "swap", |signer, i, key| signer.sign_swap(i, key)),
                SwapQuoteDataType::Transfer => self.sign_swap_transfer(input, private_key, from_asset, swap_data),
            },
        }
    }

    fn sign_swap_transfer(&self, input: &SignerInput, private_key: &[u8], from_asset: &Asset, swap_data: &SwapData) -> Result<Vec<String>, GemstoneError> {
        let is_token = from_asset.id.is_token();
        let value = if input.input.is_max_value && !is_token {
            input.input.value.clone()
        } else {
            swap_data.quote.from_value.clone()
        };
        let mut rewritten = input.clone();
        rewritten.input.input_type = TransactionInputType::Transfer(from_asset.clone());
        rewritten.input.destination_address = swap_data.data.to.clone();
        rewritten.input.value = value;
        rewritten.input.memo = swap_data.data.memo.clone();
        if is_token {
            self.one(&rewritten, private_key, "token transfer", |signer, i, key| signer.sign_token_transfer(i, key))
        } else {
            self.one(&rewritten, private_key, "transfer", |signer, i, key| signer.sign_transfer(i, key))
        }
    }

    fn one<F>(&self, input: &SignerInput, private_key: &[u8], action: &'static str, method: F) -> Result<Vec<String>, GemstoneError>
    where
        F: Fn(&dyn ChainSigner, &SignerInput, &[u8]) -> Result<String, SignerError>,
    {
        method(self.signer.as_ref(), input, private_key)
            .map(|signature| vec![signature])
            .map_err(|err| map_signer_error(self.chain, action, err))
    }

    fn many<F>(&self, input: &SignerInput, private_key: &[u8], action: &'static str, method: F) -> Result<Vec<String>, GemstoneError>
    where
        F: Fn(&dyn ChainSigner, &SignerInput, &[u8]) -> Result<Vec<String>, SignerError>,
    {
        method(self.signer.as_ref(), input, private_key).map_err(|err| map_signer_error(self.chain, action, err))
    }

    fn dispatch<T, F>(&self, input: GemSignerInput, private_key: Vec<u8>, action: &'static str, method: F) -> Result<T, GemstoneError>
    where
        F: Fn(&dyn ChainSigner, &SignerInput, &[u8]) -> Result<T, SignerError>,
    {
        let signer_input: SignerInput = input.into();
        let private_key = Zeroizing::new(private_key);

        method(self.signer.as_ref(), &signer_input, private_key.as_slice()).map_err(|err| map_signer_error(self.chain, action, err))
    }

    fn dispatch_message<T, F>(&self, message: &[u8], private_key: &[u8], action: &'static str, method: F) -> Result<T, GemstoneError>
    where
        F: Fn(&dyn ChainSigner, &[u8], &[u8]) -> Result<T, SignerError>,
    {
        method(self.signer.as_ref(), message, private_key).map_err(|err| map_signer_error(self.chain, action, err))
    }
}

fn map_signer_error(chain: Chain, action: &str, error: SignerError) -> GemstoneError {
    match error {
        SignerError::SigningError(message) if message == format!("sign_{} not implemented", action.replace(' ', "_")) => unsupported_error(chain, action),
        error => GemstoneError::from(error),
    }
}

fn unsupported_error(chain: Chain, action: &str) -> GemstoneError {
    SignerError::SigningError(format!("{action} not supported for chain {:?}", chain)).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::testkit::signer_mock::{TEST_EVM_RECIPIENT, TEST_PRIVATE_KEY};
    use primitives::{
        DelegationValidator, StakeType, SwapProvider, TransactionLoadMetadata, TransferDataExtra, WalletConnectionSessionAppMetadata, contract_call_data::ContractCallData,
        nft::NFTAsset,
    };

    #[test]
    fn test_sign_input_routing() {
        let signer = GemChainSigner::new(Chain::Ethereum);
        let key = TEST_PRIVATE_KEY.to_vec();
        let sign_one = |gem: GemSignerInput| signer.sign_input(gem, key.clone()).unwrap();

        let native: GemSignerInput = SignerInput::mock_evm(TransactionInputType::Transfer(Asset::mock()), "1000000000000000000", 21000).into();
        assert_eq!(sign_one(native.clone()), vec![signer.sign_transfer(native, key.clone()).unwrap()]);

        let token: GemSignerInput = SignerInput::mock_evm(TransactionInputType::Transfer(Asset::mock_erc20()), "1000000", 65000).into();
        assert_eq!(sign_one(token.clone()), vec![signer.sign_token_transfer(token, key.clone()).unwrap()]);

        // TokenApprove must route to sign_token_approval, not sign_token_transfer (the resolved iOS divergence).
        let approve: GemSignerInput = SignerInput::mock_evm(TransactionInputType::TokenApprove(Asset::mock(), primitives::swap::ApprovalData::mock()), "0", 65000).into();
        assert_eq!(sign_one(approve.clone()), vec![signer.sign_token_approval(approve, key.clone()).unwrap()]);

        let nft: GemSignerInput = SignerInput::mock_evm(TransactionInputType::TransferNft(Asset::mock(), NFTAsset::mock()), "0", 100000).into();
        assert_eq!(sign_one(nft.clone()), vec![signer.sign_nft_transfer(nft, key.clone()).unwrap()]);

        let generic: GemSignerInput = SignerInput::mock_evm(
            TransactionInputType::Generic(
                Asset::mock(),
                WalletConnectionSessionAppMetadata::mock(),
                TransferDataExtra::mock_encoded_transaction(vec![0xab, 0xcd]),
            ),
            "0",
            100000,
        )
        .into();
        assert_eq!(sign_one(generic.clone()), vec![signer.sign_data(generic, key.clone()).unwrap()]);

        let stake: GemSignerInput = SignerInput::mock_evm_with_metadata(
            TransactionInputType::Stake(Asset::mock(), StakeType::Stake(DelegationValidator::mock())),
            "1000000000000000000",
            200000,
            TransactionLoadMetadata::Evm {
                nonce: 5,
                chain_id: 1,
                contract_call: Some(ContractCallData::mock_with_call_data(
                    "3a29dbae0000000000000000000000000000000000000000000000000000000000000017",
                )),
            },
        )
        .into();
        assert_eq!(sign_one(stake.clone()), signer.sign_stake(stake, key.clone()).unwrap());

        let swap_contract: GemSignerInput = SignerInput::mock_evm(
            TransactionInputType::Swap(
                Asset::mock(),
                Asset::mock(),
                SwapData::mock_contract(SwapProvider::UniswapV3, "1000000000000000000", "1000000", "1000000000000000000"),
            ),
            "1000000000000000000",
            200000,
        )
        .into();
        assert_eq!(sign_one(swap_contract.clone()), signer.sign_swap(swap_contract, key.clone()).unwrap());

        // Transfer swap -> rewritten as a native transfer to swap_data.data.to with quote.from_value
        // (NOT the input value), so it matches a hand-built transfer of that amount to that address.
        let transfer_swap: GemSignerInput = SignerInput::mock_evm(
            TransactionInputType::Swap(
                Asset::mock(),
                Asset::mock(),
                SwapData::mock_transfer(SwapProvider::UniswapV3, "500", "400", TEST_EVM_RECIPIENT),
            ),
            "999",
            21000,
        )
        .into();
        let expected_transfer: GemSignerInput = SignerInput::mock_evm(TransactionInputType::Transfer(Asset::mock()), "500", 21000).into();
        assert_eq!(sign_one(transfer_swap), vec![signer.sign_transfer(expected_transfer, key.clone()).unwrap()]);

        // Token transfer swap uses quote.from_value and routes to sign_token_transfer.
        let token_swap: GemSignerInput = SignerInput::mock_evm(
            TransactionInputType::Swap(
                Asset::mock_erc20(),
                Asset::mock(),
                SwapData::mock_transfer(SwapProvider::UniswapV3, "500", "400", TEST_EVM_RECIPIENT),
            ),
            "999",
            65000,
        )
        .into();
        let expected_token: GemSignerInput = SignerInput::mock_evm(TransactionInputType::Transfer(Asset::mock_erc20()), "500", 65000).into();
        assert_eq!(sign_one(token_swap), vec![signer.sign_token_transfer(expected_token, key.clone()).unwrap()]);

        // Max-amount native transfer swap keeps the (fee-adjusted) input value instead of quote.from_value.
        let mut max_swap: GemSignerInput = SignerInput::mock_evm(
            TransactionInputType::Swap(
                Asset::mock(),
                Asset::mock(),
                SwapData::mock_transfer(SwapProvider::UniswapV3, "500", "400", TEST_EVM_RECIPIENT),
            ),
            "777",
            21000,
        )
        .into();
        max_swap.input.is_max_value = true;
        let expected_max: GemSignerInput = SignerInput::mock_evm(TransactionInputType::Transfer(Asset::mock()), "777", 21000).into();
        assert_eq!(sign_one(max_swap), vec![signer.sign_transfer(expected_max, key.clone()).unwrap()]);

        // Gemstone-only Withdrawal lowers to a plain Transfer when converted to the canonical input.
        let mut withdrawal: GemSignerInput = SignerInput::mock_evm(TransactionInputType::Transfer(Asset::mock()), "0", 21000).into();
        withdrawal.input.input_type = GemTransactionInputType::Withdrawal { asset: Asset::mock() };
        let lowered: SignerInput = withdrawal.into();
        assert!(matches!(lowered.input.input_type, TransactionInputType::Transfer(_)));
    }

    #[test]
    fn test_map_signer_error() {
        assert_eq!(
            map_signer_error(Chain::Solana, "stake", SignerError::SigningError("sign_stake not implemented".to_string())).to_string(),
            "Signing error: stake not supported for chain solana"
        );
        assert_eq!(
            map_signer_error(
                Chain::Solana,
                "token transfer",
                SignerError::SigningError("sign_token_transfer not implemented".to_string())
            )
            .to_string(),
            "Signing error: token transfer not supported for chain solana"
        );
        assert_eq!(
            map_signer_error(Chain::Solana, "stake", SignerError::signing_error("sign: invalid private key")).to_string(),
            "Signing error: sign: invalid private key"
        );
    }
}
