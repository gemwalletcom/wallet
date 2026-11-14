use crate::{GemstoneError, models::transaction::GemSignedTransaction};
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
use gem_spark::chain_signer::LightningChainSigner;
use gem_stellar::StellarChainSigner;
use gem_sui::signer::SuiChainSigner;
use gem_tempo::TempoSigner;
use gem_ton::signer::TonChainSigner;
use gem_tron::signer::TronChainSigner;
use gem_xrp::signer::XrpChainSigner;
use gem_zcash::ZcashChainSigner;
use primitives::swap::{ApprovalData, SwapData, SwapQuoteDataType};
use primitives::{Asset, BitcoinChain, Chain, ChainSigner, ChainType, SignerError, SignerInput, TransactionInputType, TransactionLoadInput, TransactionType, TransferDataOutputType};
use zeroize::Zeroizing;

pub struct ChainTransactionSigner {
    chain: Chain,
    signer: Box<dyn ChainSigner>,
}

impl ChainTransactionSigner {
    pub fn new(chain: Chain) -> Self {
        let signer: Box<dyn ChainSigner> = match chain.chain_type() {
            ChainType::Ethereum => match chain {
                Chain::Tempo => Box::new(TempoSigner),
                _ => Box::new(EvmChainSigner),
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
            ChainType::Lightning => Box::new(LightningChainSigner),
            ChainType::Bitcoin => match chain {
                Chain::Zcash => Box::new(ZcashChainSigner),
                _ => Box::new(BitcoinChainSigner::new(BitcoinChain::from_chain(chain).unwrap())),
            },
        };

        Self { chain, signer }
    }

    pub fn sign_input(&self, input: SignerInput, private_key: Zeroizing<Vec<u8>>) -> Result<Vec<GemSignedTransaction>, GemstoneError> {
        self.route(&input, private_key.as_slice())
    }

    fn route(&self, input: &SignerInput, private_key: &[u8]) -> Result<Vec<GemSignedTransaction>, GemstoneError> {
        let transaction_type = input.input_type.transaction_type();
        match &input.input.input_type {
            TransactionInputType::Withdrawal { .. } => self.one(input, private_key, transaction_type, "withdrawal", |signer, i, key| signer.sign_withdrawal(i, key)),
            TransactionInputType::Deposit { .. } => self.one(input, private_key, transaction_type, "deposit", |signer, i, key| signer.sign_deposit(i, key)),
            TransactionInputType::Transfer { asset } => {
                if asset.id.is_token() {
                    self.one(input, private_key, transaction_type, "token transfer", |signer, i, key| signer.sign_token_transfer(i, key))
                } else {
                    self.one(input, private_key, transaction_type, "transfer", |signer, i, key| signer.sign_transfer(i, key))
                }
            }
            TransactionInputType::TransferNft { .. } => self.one(input, private_key, transaction_type, "nft transfer", |signer, i, key| signer.sign_nft_transfer(i, key)),
            TransactionInputType::TokenApprove { .. } => self.one(input, private_key, transaction_type, "token approval", |signer, i, key| signer.sign_token_approval(i, key)),
            TransactionInputType::Generic { .. } => self.one(input, private_key, transaction_type, "data", |signer, i, key| signer.sign_data(i, key)),
            TransactionInputType::Payment { asset, extra, .. } => match extra.output_type {
                TransferDataOutputType::EncodedTransaction => self.one(input, private_key, transaction_type, "data", |signer, i, key| signer.sign_data(i, key)),
                TransferDataOutputType::Signature => {
                    let signature = self
                        .dispatch_message(&extra.data.clone().unwrap_or_default(), private_key, "typed data", |signer, message, key| signer.sign_message(message, key))
                        .map(|data| GemSignedTransaction { data, transaction_type, sequence: None })?;
                    match &extra.approval {
                        Some(approval) => Ok([self.sign_payment_approval(input, private_key, asset, approval)?, vec![signature]].concat()),
                        None => Ok(vec![signature]),
                    }
                }
            },
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

    fn sign_payment_approval(&self, input: &SignerInput, private_key: &[u8], asset: &Asset, approval: &ApprovalData) -> Result<Vec<GemSignedTransaction>, GemstoneError> {
        let approve = SignerInput {
            input: TransactionLoadInput {
                input_type: TransactionInputType::TokenApprove {
                    asset: asset.clone(),
                    approval_data: approval.clone(),
                },
                ..input.input.clone()
            },
            fee: input.fee.clone(),
        };
        self.one(&approve, private_key, TransactionType::TokenApproval, "token approval", |signer, i, key| signer.sign_token_approval(i, key))
    }

    fn sign_swap_transfer(&self, input: &SignerInput, private_key: &[u8], from_asset: &Asset, swap_data: &SwapData) -> Result<Vec<GemSignedTransaction>, GemstoneError> {
        let is_token = from_asset.id.is_token();
        let value = if input.input.is_max_value && !is_token { input.input.value.clone() } else { swap_data.quote.from_value.clone() };
        let mut rewritten = input.clone();
        rewritten.input.input_type = TransactionInputType::Transfer { asset: from_asset.clone() };
        rewritten.input.destination_address = swap_data.data.to.clone();
        rewritten.input.value = value;
        rewritten.input.memo = swap_data.data.memo.clone();
        if is_token {
            self.one(&rewritten, private_key, TransactionType::Swap, "token transfer", |signer, i, key| signer.sign_token_transfer(i, key))
        } else {
            self.one(&rewritten, private_key, TransactionType::Swap, "transfer", |signer, i, key| signer.sign_transfer(i, key))
        }
    }

    fn one<F>(&self, input: &SignerInput, private_key: &[u8], transaction_type: TransactionType, action: &'static str, method: F) -> Result<Vec<GemSignedTransaction>, GemstoneError>
    where
        F: Fn(&dyn ChainSigner, &SignerInput, &[u8]) -> Result<String, SignerError>,
    {
        let sequence = input.input.metadata.get_sequence().ok();
        method(self.signer.as_ref(), input, private_key)
            .map(|data| vec![GemSignedTransaction { data, transaction_type, sequence }])
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
        let sequence = input.input.metadata.get_sequence().ok();
        Ok(transactions
            .into_iter()
            .zip(transaction_types)
            .zip(0..)
            .map(|((data, transaction_type), position)| GemSignedTransaction {
                data,
                transaction_type,
                sequence: sequence.map(|sequence| sequence + position),
            })
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
    use gem_bitcoin::testkit::signer_mock::{TEST_PRIVATE_KEY as TEST_BITCOIN_PRIVATE_KEY, mock_transfer_input};
    use gem_evm::testkit::eip712_mock::mock_eip712_json;
    use primitives::contract_constants::HYPERLIQUID_ARBITRUM_DEPOSIT_ADDRESS;
    use primitives::hex;
    use primitives::known_assets::HYPERCORE_SPOT_USDC;
    use primitives::testkit::signer_mock::{TEST_EVM_RECIPIENT, TEST_PRIVATE_KEY, TEST_PRIVATE_KEY_ETHEREUM_ADDRESS};
    use primitives::{
        ApplicationMetadata, DelegationValidator, StakeType, SwapProvider, TransactionFee, TransactionLoadInput, TransactionLoadMetadata, TransferDataExtra, TransferDataOutputType, contract_call_data::ContractCallData, nft::NFTAsset,
    };

    fn signed(data: Vec<String>, transaction_type: TransactionType, sequence: Option<u64>) -> Vec<GemSignedTransaction> {
        data.into_iter()
            .map(|data| GemSignedTransaction {
                data,
                transaction_type: transaction_type.clone(),
                sequence,
            })
            .collect()
    }

    #[test]
    fn test_sign_input_ton_wallet_connect() {
        let request = include_str!("../../../crates/gem_ton/testdata/wallet_connect_dedust_send_message.json");
        let transaction = gem_wallet_connect::WalletConnectRequestHandler::decode_send_transaction(
            Chain::Ton,
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

        let signed = ChainTransactionSigner::new(Chain::Ton).sign_input(input, Zeroizing::new(private_key)).unwrap();

        assert_eq!(signed.len(), 1);
    }

    #[test]
    fn test_sign_input_payment() {
        let signer = ChainTransactionSigner::new(Chain::Ethereum);
        let key = TEST_PRIVATE_KEY.to_vec();
        let typed_data = mock_eip712_json(1).into_bytes();
        let payment = |approval, gas_limit| -> SignerInput { SignerInput::mock_evm(TransactionInputType::mock_payment(Asset::mock_erc20(), TransferDataExtra::mock_signature(typed_data.clone(), approval)), "0", gas_limit) };
        let approval = ApprovalData::mock();
        let approve = SignerInput::mock_evm(
            TransactionInputType::TokenApprove {
                asset: Asset::mock_erc20(),
                approval_data: approval.clone(),
            },
            "0",
            65000,
        );

        assert_eq!(
            signer.sign_input(payment(None, 0), Zeroizing::new(key.clone())).unwrap(),
            signed(vec![EvmChainSigner.sign_message(&typed_data, &key).unwrap()], TransactionType::Transfer, None),
            "a payment is signed as typed data, which takes no nonce"
        );
        assert_eq!(
            signer.sign_input(payment(Some(approval), 65000), Zeroizing::new(key.clone())).unwrap(),
            [
                signed(vec![EvmChainSigner.sign_token_approval(&approve, &key).unwrap()], TransactionType::TokenApproval, Some(0)),
                signed(vec![EvmChainSigner.sign_message(&typed_data, &key).unwrap()], TransactionType::Transfer, None),
            ]
            .concat(),
            "the approval is signed first"
        );
    }

    #[test]
    fn test_sign_input_routing() {
        let signer = ChainTransactionSigner::new(Chain::Ethereum);
        let key = TEST_PRIVATE_KEY.to_vec();
        let sign_one = |gem: SignerInput| signer.sign_input(gem, Zeroizing::new(key.clone())).unwrap();

        let native = SignerInput::mock_evm(TransactionInputType::Transfer { asset: Asset::mock() }, "1000000000000000000", 21000);
        assert_eq!(sign_one(native.clone()), signed(vec![EvmChainSigner.sign_transfer(&native, &key).unwrap()], TransactionType::Transfer, Some(0)));

        let token = SignerInput::mock_evm(TransactionInputType::Transfer { asset: Asset::mock_erc20() }, "1000000", 65000);
        assert_eq!(sign_one(token.clone()), signed(vec![EvmChainSigner.sign_token_transfer(&token, &key).unwrap()], TransactionType::Transfer, Some(0)));

        let deposit = SignerInput::mock_evm(TransactionInputType::Deposit { asset: Asset::mock_erc20() }, "5000000", 65000);
        assert_eq!(sign_one(deposit.clone()), signed(vec![EvmChainSigner.sign_token_transfer(&deposit, &key).unwrap()], TransactionType::Transfer, Some(0)));

        // TokenApprove must route to sign_token_approval, not sign_token_transfer (the resolved iOS divergence).
        let approve = SignerInput::mock_evm(
            TransactionInputType::TokenApprove {
                asset: Asset::mock(),
                approval_data: ApprovalData::mock(),
            },
            "0",
            65000,
        );
        assert_eq!(sign_one(approve.clone()), signed(vec![EvmChainSigner.sign_token_approval(&approve, &key).unwrap()], TransactionType::TokenApproval, Some(0)));

        let nft = SignerInput::mock_evm(
            TransactionInputType::TransferNft {
                asset: Asset::mock(),
                nft_asset: NFTAsset::mock(),
            },
            "0",
            100000,
        );
        assert_eq!(sign_one(nft.clone()), signed(vec![EvmChainSigner.sign_nft_transfer(&nft, &key).unwrap()], TransactionType::TransferNFT, Some(0)));

        let mut generic_extra = TransferDataExtra::mock_encoded_transaction(vec![0xab, 0xcd]);
        generic_extra.transaction_type = TransactionType::AssetActivation;
        let generic = SignerInput::mock_evm(
            TransactionInputType::Generic {
                asset: Asset::mock(),
                metadata: ApplicationMetadata::mock(),
                extra: generic_extra,
            },
            "0",
            100000,
        );
        assert_eq!(sign_one(generic.clone()), signed(vec![EvmChainSigner.sign_data(&generic, &key).unwrap()], TransactionType::AssetActivation, Some(0)));

        let stake = SignerInput::mock_evm_with_metadata(
            TransactionInputType::Stake {
                asset: Asset::mock(),
                stake_type: StakeType::Stake(DelegationValidator::mock()),
            },
            "1000000000000000000",
            200000,
            TransactionLoadMetadata::Evm {
                nonce: 5,
                chain_id: 1,
                contract_call: Some(ContractCallData::mock_with_call_data("3a29dbae0000000000000000000000000000000000000000000000000000000000000017")),
            },
        );
        assert_eq!(sign_one(stake.clone()), signed(EvmChainSigner.sign_stake(&stake, &key).unwrap(), TransactionType::StakeDelegate, Some(5)));

        let swap_contract = SignerInput::mock_evm(
            TransactionInputType::Swap {
                from_asset: Asset::mock(),
                to_asset: Asset::mock(),
                swap_data: SwapData::mock_contract(SwapProvider::UniswapV3, "1000000000000000000", "1000000", "1000000000000000000"),
            },
            "1000000000000000000",
            200000,
        );
        assert_eq!(sign_one(swap_contract.clone()), signed(EvmChainSigner.sign_swap(&swap_contract, &key).unwrap(), TransactionType::Swap, Some(0)));

        let swap_with_approval = SignerInput::mock_evm(
            TransactionInputType::Swap {
                from_asset: Asset::mock(),
                to_asset: Asset::mock(),
                swap_data: SwapData::mock_with_data_and_approval("abcd", Some("200000")),
            },
            "0",
            65000,
        );
        let swap_with_approval = sign_one(swap_with_approval);
        assert_eq!(
            swap_with_approval.iter().map(|transaction| transaction.transaction_type.clone()).collect::<Vec<_>>(),
            vec![TransactionType::TokenApproval, TransactionType::Swap]
        );
        assert_eq!(
            swap_with_approval.iter().map(|transaction| transaction.sequence).collect::<Vec<_>>(),
            vec![Some(0), Some(1)],
            "the swap after an approval is signed with the next nonce"
        );

        // Transfer swap -> rewritten as a native transfer to swap_data.data.to with quote.from_value
        // (NOT the input value), so it matches a hand-built transfer of that amount to that address.
        let transfer_swap = SignerInput::mock_evm(
            TransactionInputType::Swap {
                from_asset: Asset::mock(),
                to_asset: Asset::mock(),
                swap_data: SwapData::mock_transfer(SwapProvider::UniswapV3, "500", "400", TEST_EVM_RECIPIENT),
            },
            "999",
            21000,
        );
        let expected_transfer = SignerInput::mock_evm(TransactionInputType::Transfer { asset: Asset::mock() }, "500", 21000);
        assert_eq!(sign_one(transfer_swap), signed(vec![EvmChainSigner.sign_transfer(&expected_transfer, &key).unwrap()], TransactionType::Swap, Some(0)));

        // Token transfer swap uses quote.from_value and routes to sign_token_transfer.
        let token_swap = SignerInput::mock_evm(
            TransactionInputType::Swap {
                from_asset: Asset::mock_erc20(),
                to_asset: Asset::mock(),
                swap_data: SwapData::mock_transfer(SwapProvider::UniswapV3, "500", "400", TEST_EVM_RECIPIENT),
            },
            "999",
            65000,
        );
        let expected_token = SignerInput::mock_evm(TransactionInputType::Transfer { asset: Asset::mock_erc20() }, "500", 65000);
        assert_eq!(sign_one(token_swap), signed(vec![EvmChainSigner.sign_token_transfer(&expected_token, &key).unwrap()], TransactionType::Swap, Some(0)));

        // Max-amount native transfer swap keeps the (fee-adjusted) input value instead of quote.from_value.
        let mut max_swap = SignerInput::mock_evm(
            TransactionInputType::Swap {
                from_asset: Asset::mock(),
                to_asset: Asset::mock(),
                swap_data: SwapData::mock_transfer(SwapProvider::UniswapV3, "500", "400", TEST_EVM_RECIPIENT),
            },
            "777",
            21000,
        );
        max_swap.input.is_max_value = true;
        let expected_max = SignerInput::mock_evm(TransactionInputType::Transfer { asset: Asset::mock() }, "777", 21000);
        assert_eq!(sign_one(max_swap), signed(vec![EvmChainSigner.sign_transfer(&expected_max, &key).unwrap()], TransactionType::Swap, Some(0)));
    }

    #[test]
    fn test_sign_input_zcash_transfer() {
        let input = mock_transfer_input(BitcoinChain::Zcash);
        let key = TEST_BITCOIN_PRIVATE_KEY.to_vec();
        let expected = ZcashChainSigner.sign_transfer(&input, &key).unwrap();

        let signed_transfer = ChainTransactionSigner::new(Chain::Zcash).sign_input(input, Zeroizing::new(key)).unwrap();

        assert_eq!(signed_transfer, signed(vec![expected], TransactionType::Transfer, None));
    }

    #[test]
    fn test_sign_input_xrp_transfer_swap_destination_tag() {
        let key = hex::decode("a5576c0f63da10e584568c8d134569ff44017b0a249eb70657127ae04f38cc77").unwrap();
        let mut swap_data = SwapData::mock_transfer(SwapProvider::Thorchain, "10", "7", "rU893viamSnsfP3zjzM2KPxjqZjXSXK6VF");
        swap_data.data.memo = Some("180563378".to_string());
        let input = SignerInput::mock_with_input_type(
            TransactionInputType::Swap {
                from_asset: Asset::from_chain(Chain::Xrp),
                to_asset: Asset::from_chain(Chain::Stellar),
                swap_data,
            },
            "rfxdLwsZnoespnTDDb1Xhvbc8EFNdztaoq",
            "",
            "10",
            TransactionLoadMetadata::Xrp {
                sequence: 32_268_248,
                block_number: 32_268_269,
                is_destination_address_exist: true,
            },
        );

        let transactions = ChainTransactionSigner::new(Chain::Xrp).sign_input(input, Zeroizing::new(key)).unwrap();

        assert_eq!(
            transactions,
            signed(
                vec!["12000022000000002401ec5fd82e0ac32db2201b01ec600561400000000000000a684000000000000000732103d13e1152965a51a4a9fd9a8b4ea3dd82a4eba6b25fcad5f460a2342bb650333f74463044022022f3377266fb97ddccdad008a49ee0fb30c6a9ed1f10a3746e91bd493b32b41e022074db50461dfa87f74e1bf15f4567d58a78edea1f3cd2b441524b9c879183adce81144c55f5a78067206507580be7bb2686c8460adff983148132e4e20aecf29090ac428a9c43f230a829220d".to_string()],
                TransactionType::Swap,
                Some(32_268_248)
            )
        );
    }

    #[test]
    fn test_sign_input_hypercore_deposit() {
        let input = SignerInput::mock_with_input_type(
            TransactionInputType::Deposit { asset: HYPERCORE_SPOT_USDC.clone() },
            TEST_PRIVATE_KEY_ETHEREUM_ADDRESS,
            HYPERLIQUID_ARBITRUM_DEPOSIT_ADDRESS,
            "150000000",
            TransactionLoadMetadata::Hyperliquid { order: None },
        );

        let transactions = ChainTransactionSigner::new(Chain::HyperCore).sign_input(input, Zeroizing::new(TEST_PRIVATE_KEY.to_vec())).unwrap();
        let request: serde_json::Value = serde_json::from_str(&transactions[0].data).unwrap();

        assert_eq!(request["action"]["type"], "usdClassTransfer", "a HyperCore deposit never signs a spotSend to the bridge address");
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

        assert_eq!(ChainTransactionSigner::new(Chain::Ethereum).transaction_types(&input, 2).unwrap(), vec![TransactionType::TokenApproval, TransactionType::Swap]);
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
            map_signer_error(Chain::Solana, "token transfer", SignerError::SigningError("sign_token_transfer not implemented".to_string())).to_string(),
            "Signing error: token transfer not supported for chain solana"
        );
        assert_eq!(
            map_signer_error(Chain::Solana, "stake", SignerError::signing_error("sign: invalid private key")).to_string(),
            "Signing error: sign: invalid private key"
        );
    }
}
