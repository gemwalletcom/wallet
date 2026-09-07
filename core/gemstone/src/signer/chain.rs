use crate::{
    GemstoneError,
    models::transaction::{GemSignedTransaction, GemSignerInput},
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
use gem_tempo::TempoSigner;
use gem_ton::signer::TonChainSigner;
use gem_tron::signer::TronChainSigner;
use gem_xrp::signer::XrpChainSigner;
use primitives::swap::{SwapData, SwapQuoteDataType};
use primitives::{Asset, BitcoinChain, Chain, ChainSigner, ChainType, SignerError, SignerInput, TransactionInputType, TransactionType};
use zeroize::Zeroizing;

pub struct ChainTransactionSigner {
    chain: Chain,
    signer: Box<dyn ChainSigner>,
}

impl ChainTransactionSigner {
    pub fn new(chain: Chain) -> Self {
        let signer: Box<dyn ChainSigner> = match chain.chain_type() {
            ChainType::Ethereum => match chain {
                Chain::Tempo => Box::new(EvmChainSigner::new(TempoSigner)),
                _ => Box::new(EvmChainSigner::default()),
            },
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

impl ChainTransactionSigner {
    pub fn sign_input(&self, input: GemSignerInput, private_key: Zeroizing<Vec<u8>>) -> Result<Vec<GemSignedTransaction>, GemstoneError> {
        let signer_input: SignerInput = input.into();
        self.route(&signer_input, private_key.as_slice())
    }

    fn route(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<GemSignedTransaction>, GemstoneError> {
        let transaction_type = input.input_type.transaction_type();
        match &input.input.input_type {
            TransactionInputType::Withdrawal { .. } => self.one(input, private_key, transaction_type, "withdrawal", |signer, i, key| signer.sign_withdrawal(i, key)),
            TransactionInputType::Transfer { asset } | TransactionInputType::Deposit { asset } => {
                if asset.id.is_token() {
                    self.one(input, private_key, transaction_type, "token transfer", |signer, i, key| signer.sign_token_transfer(i, key))
                } else {
                    self.one(input, private_key, transaction_type, "transfer", |signer, i, key| signer.sign_transfer(i, key))
                }
            }
            TransactionInputType::TransferNft { .. } => self.one(input, private_key, transaction_type, "nft transfer", |signer, i, key| signer.sign_nft_transfer(i, key)),
            TransactionInputType::TokenApprove { .. } => self.one(input, private_key, transaction_type, "token approval", |signer, i, key| signer.sign_token_approval(i, key)),
            TransactionInputType::Generic { .. } => self.one(input, private_key, transaction_type, "data", |signer, i, key| signer.sign_data(i, key)),
            TransactionInputType::Account { .. } => self.one(input, private_key, transaction_type, "account action", |signer, i, key| signer.sign_account_action(i, key)),
            TransactionInputType::Stake { .. } => self.many(input, private_key, "stake", |signer, i, key| signer.sign_stake(i, key)),
            TransactionInputType::Perpetual { .. } => self.many(input, private_key, "perpetual", |signer, i, key| signer.sign_perpetual(i, key)),
            TransactionInputType::Earn { .. } => self.many(input, private_key, "earn", |signer, i, key| signer.sign_earn(i, key)),
            TransactionInputType::Swap { from_asset, swap_data, .. } => match swap_data.data.data_type {
                SwapQuoteDataType::Contract => self.many(input, private_key, "swap", |signer, i, key| signer.sign_swap(i, key)),
                SwapQuoteDataType::Transfer => self.sign_swap_transfer(input, private_key, from_asset, swap_data),
            },
        }
    }

    fn sign_swap_transfer(&self, input: &SignerInput, private_key: &[u8], from_asset: &Asset, swap_data: &SwapData) -> Result<Vec<GemSignedTransaction>, GemstoneError> {
        let is_token = from_asset.id.is_token();
        let value = if input.input.is_max_value && !is_token {
            input.input.value.clone()
        } else {
            swap_data.quote.from_value.clone()
        };
        let mut rewritten = input.clone();
        rewritten.input.input_type = TransactionInputType::Transfer { asset: from_asset.clone() };
        rewritten.input.destination_address = swap_data.data.to.clone();
        rewritten.input.value = value;
        rewritten.input.memo = swap_data.data.memo.clone();
        if is_token {
            self.one(&rewritten, private_key, TransactionType::Swap, "token transfer", |signer, i, key| {
                signer.sign_token_transfer(i, key)
            })
        } else {
            self.one(&rewritten, private_key, TransactionType::Swap, "transfer", |signer, i, key| signer.sign_transfer(i, key))
        }
    }

    fn one<F>(
        &self,
        input: &SignerInput,
        private_key: &[u8],
        transaction_type: TransactionType,
        action: &'static str,
        method: F,
    ) -> Result<Vec<GemSignedTransaction>, GemstoneError>
    where
        F: Fn(&dyn ChainSigner, &SignerInput, &[u8]) -> Result<String, SignerError>,
    {
        method(self.signer.as_ref(), input, private_key)
            .map(|data| vec![GemSignedTransaction { data, transaction_type }])
            .map_err(|err| map_signer_error(self.chain, action, err))
    }

    fn many<F>(&self, input: &SignerInput, private_key: &[u8], action: &'static str, method: F) -> Result<Vec<GemSignedTransaction>, GemstoneError>
    where
        F: Fn(&dyn ChainSigner, &SignerInput, &[u8]) -> Result<Vec<String>, SignerError>,
    {
        let transactions = method(self.signer.as_ref(), input, private_key).map_err(|err| map_signer_error(self.chain, action, err))?;
        if transactions.is_empty() {
            return Err(map_signer_error(self.chain, action, SignerError::signing_error("signer returned no transactions")));
        }
        let transaction_types = self.transaction_types(input, transactions.len()).map_err(|err| map_signer_error(self.chain, action, err))?;
        Ok(transactions
            .into_iter()
            .zip(transaction_types)
            .map(|(data, transaction_type)| GemSignedTransaction { data, transaction_type })
            .collect())
    }

    fn transaction_types(&self, input: &SignerInput, count: usize) -> Result<Vec<TransactionType>, SignerError> {
        let transaction_type = input.input_type.transaction_type();
        let expected_transaction_types = match &input.input_type {
            TransactionInputType::Swap { swap_data: data, .. } if data.data.approval.is_some() && self.chain == Chain::Tempo => {
                vec![transaction_type]
            }
            TransactionInputType::Swap { swap_data: data, .. } if data.data.approval.is_some() => {
                vec![TransactionType::TokenApproval, transaction_type]
            }
            TransactionInputType::Earn { data, .. } if data.approval.is_some() => {
                vec![TransactionType::TokenApproval, transaction_type]
            }
            _ => return Ok(vec![transaction_type; count]),
        };

        if count != expected_transaction_types.len() {
            return Err(SignerError::signing_error("unexpected approval transaction count"));
        }

        Ok(expected_transaction_types)
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
        ApplicationMetadata, DelegationValidator, StakeType, SwapProvider, TransactionFee, TransactionLoadInput, TransactionLoadMetadata, TransferDataExtra,
        TransferDataOutputType, contract_call_data::ContractCallData, nft::NFTAsset,
    };

    fn signed(data: Vec<String>, transaction_type: TransactionType) -> Vec<GemSignedTransaction> {
        data.into_iter()
            .map(|data| GemSignedTransaction {
                data,
                transaction_type: transaction_type.clone(),
            })
            .collect()
    }

    #[test]
    fn test_sign_input_checksums_destination() {
        let mut gem: GemSignerInput = SignerInput::mock_evm(TransactionInputType::Transfer { asset: Asset::mock() }, "0", 21000).into();
        gem.input.destination_address = "0x5615e8ab93b9d695b6d4d6545f7792aa59e1069a".to_string();

        let signer_input: SignerInput = gem.into();

        assert_eq!(signer_input.input.destination_address, "0x5615E8AB93b9d695b6d4d6545f7792aA59e1069a");
    }

    #[test]
    fn test_sign_input_ton_wallet_connect() {
        let request = include_str!("../../../crates/gem_ton/testdata/wallet_connect_dedust_send_message.json");
        let transaction = gem_wallet_connect::WalletConnectRequestHandler::decode_send_transaction(
            gem_wallet_connect::WalletConnectTransactionType::Ton {
                output_type: TransferDataOutputType::EncodedTransaction,
            },
            request.to_string(),
        )
        .unwrap();
        let gem_wallet_connect::WalletConnectTransaction::Ton { data, .. } = transaction else {
            panic!("expected TON transaction");
        };
        let private_key = hex::decode("1e9d38b5274152a78dff1a86fa464ceadc1f4238ca2c17060c3c507349424a34").unwrap();
        let signer = gem_ton::signer::TonSigner::new(&private_key).unwrap();
        let sender = signer.address().encode_non_bounceable();
        let mut input = TransactionLoadInput::mock_sign_data(Chain::Ton, &data, TransferDataOutputType::EncodedTransaction);
        input.sender_address = sender;
        input.metadata = TransactionLoadMetadata::Ton {
            sender_token_address: None,
            recipient_token_address: None,
            sequence: 1,
        };
        let input = SignerInput::new(input, TransactionFee::mock());

        let signed = ChainTransactionSigner::new(Chain::Ton).sign_input(input.into(), Zeroizing::new(private_key)).unwrap();

        assert_eq!(signed.len(), 1);
    }

    #[test]
    fn test_sign_input_routing() {
        let signer = ChainTransactionSigner::new(Chain::Ethereum);
        let key = TEST_PRIVATE_KEY.to_vec();
        let sign_one = |gem: GemSignerInput| signer.sign_input(gem, Zeroizing::new(key.clone())).unwrap();

        let native: GemSignerInput = SignerInput::mock_evm(TransactionInputType::Transfer { asset: Asset::mock() }, "1000000000000000000", 21000).into();
        assert_eq!(
            sign_one(native.clone()),
            signed(vec![signer.sign_transfer(native, key.clone()).unwrap()], TransactionType::Transfer)
        );

        let token: GemSignerInput = SignerInput::mock_evm(TransactionInputType::Transfer { asset: Asset::mock_erc20() }, "1000000", 65000).into();
        assert_eq!(
            sign_one(token.clone()),
            signed(vec![signer.sign_token_transfer(token, key.clone()).unwrap()], TransactionType::Transfer)
        );

        // TokenApprove must route to sign_token_approval, not sign_token_transfer (the resolved iOS divergence).
        let approve: GemSignerInput = SignerInput::mock_evm(
            TransactionInputType::TokenApprove {
                asset: Asset::mock(),
                approval_data: primitives::swap::ApprovalData::mock(),
            },
            "0",
            65000,
        )
        .into();
        assert_eq!(
            sign_one(approve.clone()),
            signed(vec![signer.sign_token_approval(approve, key.clone()).unwrap()], TransactionType::TokenApproval)
        );

        let nft: GemSignerInput = SignerInput::mock_evm(
            TransactionInputType::TransferNft {
                asset: Asset::mock(),
                nft_asset: NFTAsset::mock(),
            },
            "0",
            100000,
        )
        .into();
        assert_eq!(
            sign_one(nft.clone()),
            signed(vec![signer.sign_nft_transfer(nft, key.clone()).unwrap()], TransactionType::TransferNFT)
        );

        let mut generic_extra = TransferDataExtra::mock_encoded_transaction(vec![0xab, 0xcd]);
        generic_extra.transaction_type = TransactionType::AssetActivation;
        let generic: GemSignerInput = SignerInput::mock_evm(
            TransactionInputType::Generic {
                asset: Asset::mock(),
                metadata: ApplicationMetadata::mock(),
                extra: generic_extra,
            },
            "0",
            100000,
        )
        .into();
        assert_eq!(
            sign_one(generic.clone()),
            signed(vec![signer.sign_data(generic, key.clone()).unwrap()], TransactionType::AssetActivation)
        );

        let stake: GemSignerInput = SignerInput::mock_evm_with_metadata(
            TransactionInputType::Stake {
                asset: Asset::mock(),
                stake_type: StakeType::Stake(DelegationValidator::mock()),
            },
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
        assert_eq!(
            sign_one(stake.clone()),
            signed(signer.sign_stake(stake, key.clone()).unwrap(), TransactionType::StakeDelegate)
        );

        let swap_contract: GemSignerInput = SignerInput::mock_evm(
            TransactionInputType::Swap {
                from_asset: Asset::mock(),
                to_asset: Asset::mock(),
                swap_data: SwapData::mock_contract(SwapProvider::UniswapV3, "1000000000000000000", "1000000", "1000000000000000000"),
            },
            "1000000000000000000",
            200000,
        )
        .into();
        assert_eq!(
            sign_one(swap_contract.clone()),
            signed(signer.sign_swap(swap_contract, key.clone()).unwrap(), TransactionType::Swap)
        );

        let swap_with_approval: GemSignerInput = SignerInput::mock_evm(
            TransactionInputType::Swap {
                from_asset: Asset::mock(),
                to_asset: Asset::mock(),
                swap_data: SwapData::mock_with_data_and_approval("abcd", Some("200000")),
            },
            "0",
            65000,
        )
        .into();
        assert_eq!(
            sign_one(swap_with_approval).into_iter().map(|transaction| transaction.transaction_type).collect::<Vec<_>>(),
            vec![TransactionType::TokenApproval, TransactionType::Swap]
        );

        // Transfer swap -> rewritten as a native transfer to swap_data.data.to with quote.from_value
        // (NOT the input value), so it matches a hand-built transfer of that amount to that address.
        let transfer_swap: GemSignerInput = SignerInput::mock_evm(
            TransactionInputType::Swap {
                from_asset: Asset::mock(),
                to_asset: Asset::mock(),
                swap_data: SwapData::mock_transfer(SwapProvider::UniswapV3, "500", "400", TEST_EVM_RECIPIENT),
            },
            "999",
            21000,
        )
        .into();
        let expected_transfer: GemSignerInput = SignerInput::mock_evm(TransactionInputType::Transfer { asset: Asset::mock() }, "500", 21000).into();
        assert_eq!(
            sign_one(transfer_swap),
            signed(vec![signer.sign_transfer(expected_transfer, key.clone()).unwrap()], TransactionType::Swap)
        );

        // Token transfer swap uses quote.from_value and routes to sign_token_transfer.
        let token_swap: GemSignerInput = SignerInput::mock_evm(
            TransactionInputType::Swap {
                from_asset: Asset::mock_erc20(),
                to_asset: Asset::mock(),
                swap_data: SwapData::mock_transfer(SwapProvider::UniswapV3, "500", "400", TEST_EVM_RECIPIENT),
            },
            "999",
            65000,
        )
        .into();
        let expected_token: GemSignerInput = SignerInput::mock_evm(TransactionInputType::Transfer { asset: Asset::mock_erc20() }, "500", 65000).into();
        assert_eq!(
            sign_one(token_swap),
            signed(vec![signer.sign_token_transfer(expected_token, key.clone()).unwrap()], TransactionType::Swap)
        );

        // Max-amount native transfer swap keeps the (fee-adjusted) input value instead of quote.from_value.
        let mut max_swap: GemSignerInput = SignerInput::mock_evm(
            TransactionInputType::Swap {
                from_asset: Asset::mock(),
                to_asset: Asset::mock(),
                swap_data: SwapData::mock_transfer(SwapProvider::UniswapV3, "500", "400", TEST_EVM_RECIPIENT),
            },
            "777",
            21000,
        )
        .into();
        max_swap.input.is_max_value = true;
        let expected_max: GemSignerInput = SignerInput::mock_evm(TransactionInputType::Transfer { asset: Asset::mock() }, "777", 21000).into();
        assert_eq!(
            sign_one(max_swap),
            signed(vec![signer.sign_transfer(expected_max, key.clone()).unwrap()], TransactionType::Swap)
        );

        let withdrawal: GemSignerInput = SignerInput::mock_evm(TransactionInputType::Withdrawal { asset: Asset::mock() }, "0", 21000).into();
        let crossed: SignerInput = withdrawal.into();
        assert!(
            matches!(crossed.input.input_type, TransactionInputType::Withdrawal { .. }),
            "a withdrawal keeps its variant across the FFI model"
        );
    }

    #[test]
    fn test_approval_transaction_types() {
        let input = SignerInput::mock_evm(
            TransactionInputType::Swap {
                from_asset: Asset::mock(),
                to_asset: Asset::mock(),
                swap_data: SwapData::mock_with_data_and_approval("abcd", Some("200000")),
            },
            "0",
            65000,
        );

        assert_eq!(
            ChainTransactionSigner::new(Chain::Ethereum).transaction_types(&input, 2).unwrap(),
            vec![TransactionType::TokenApproval, TransactionType::Swap]
        );
        assert!(ChainTransactionSigner::new(Chain::Ethereum).transaction_types(&input, 1).is_err());
        assert_eq!(ChainTransactionSigner::new(Chain::Tempo).transaction_types(&input, 1).unwrap(), vec![TransactionType::Swap]);
        assert!(ChainTransactionSigner::new(Chain::Tempo).transaction_types(&input, 2).is_err());
    }

    #[test]
    fn test_map_signer_error() {
        assert_eq!(
            map_signer_error(Chain::Bitcoin, "transfer", SignerError::DustThreshold),
            GemstoneError::SignerError {
                error: SignerError::DustThreshold,
                msg: "transaction amount is below the dust threshold".to_string(),
            }
        );
        assert_eq!(
            map_signer_error(Chain::Cardano, "transfer", SignerError::InsufficientFunds),
            GemstoneError::SignerError {
                error: SignerError::InsufficientFunds,
                msg: "insufficient balance".to_string(),
            }
        );
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
