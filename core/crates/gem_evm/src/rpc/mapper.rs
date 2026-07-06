use chrono::DateTime;
use num_bigint::BigUint;
use std::sync::LazyLock;

use super::parsers::ProtocolParsers;
pub(crate) use super::transaction_payload::TRANSFER_TOPIC;
use super::transaction_payload::{Erc20ApprovalPayload, Erc20TransferPayload, NftTransferPayload, TransactionPayload};
use crate::{
    address::ethereum_address_checksum,
    registry::ContractRegistry,
    rpc::model::{Block, Transaction, TransactionReceipt, TransactionReplayTrace},
};
use primitives::{
    AssetId, NFTAssetId, Transaction as PrimitivesTransaction, TransactionType, chain::Chain, hex::decode_hex_utf8, transaction_metadata_types::TransactionNFTTransferMetadata,
};

pub static CONTRACT_REGISTRY: LazyLock<ContractRegistry> = LazyLock::new(ContractRegistry::default);
pub struct EthereumMapper;

impl EthereumMapper {
    pub fn map_transactions(chain: Chain, block: Block, transactions_receipts: Vec<TransactionReceipt>, traces: Option<Vec<TransactionReplayTrace>>) -> Vec<PrimitivesTransaction> {
        match traces {
            Some(traces) => block
                .transactions
                .into_iter()
                .zip(transactions_receipts.iter())
                .zip(traces.iter())
                .filter_map(|((transaction, receipt), trace)| {
                    EthereumMapper::map_transaction(chain, &transaction, receipt, Some(trace), &block.timestamp, Some(&CONTRACT_REGISTRY))
                })
                .collect(),
            None => block
                .transactions
                .into_iter()
                .zip(transactions_receipts.iter())
                .filter_map(|(transaction, receipt)| EthereumMapper::map_transaction(chain, &transaction, receipt, None, &block.timestamp, Some(&CONTRACT_REGISTRY)))
                .collect(),
        }
    }

    pub fn map_transaction(
        chain: Chain,
        transaction: &Transaction,
        transaction_receipt: &TransactionReceipt,
        trace: Option<&TransactionReplayTrace>,
        timestamp: &BigUint,
        contract_registry: Option<&ContractRegistry>,
    ) -> Option<PrimitivesTransaction> {
        let state = transaction_receipt.get_state();
        let hash = transaction.hash.clone();
        let value = transaction.value.to_string();
        let fee = transaction_receipt.get_fee().to_string();
        let fee_asset_id = chain.as_asset_id();
        let from = ethereum_address_checksum(&transaction.from.clone()).ok()?;
        let to = ethereum_address_checksum(&transaction.to.clone().unwrap_or_default()).ok()?;
        let created_at = DateTime::from_timestamp(timestamp.clone().try_into().ok()?, 0)?;

        let payload = TransactionPayload::from_transaction(transaction, transaction_receipt, &from, &to);

        if transaction.to.is_some()
            && transaction.input.len() >= 8
            && let Some(tx) = ProtocolParsers::map_transaction(&chain, transaction, transaction_receipt, trace, contract_registry, created_at)
        {
            return Some(tx);
        }

        let build_nft_transfer = |transfer: NftTransferPayload| {
            let metadata = TransactionNFTTransferMetadata::from_asset_id(NFTAssetId::new(chain, &transfer.contract_address, &transfer.token_id.to_string()));

            PrimitivesTransaction::new(
                hash.clone(),
                AssetId::from_chain(chain),
                from.clone(),
                transfer.to,
                None,
                TransactionType::TransferNFT,
                state,
                fee.to_string(),
                fee_asset_id.clone(),
                "0".to_string(),
                None,
                serde_json::to_value(metadata).ok(),
                created_at,
            )
        };

        let build_erc20_transfer = |transfer: Erc20TransferPayload| {
            PrimitivesTransaction::new(
                hash.clone(),
                AssetId::from_token(chain, &transfer.contract_address),
                transfer.from,
                transfer.to,
                None,
                TransactionType::Transfer,
                state,
                fee.to_string(),
                fee_asset_id.clone(),
                transfer.value.to_string(),
                None,
                None,
                created_at,
            )
        };

        let build_erc20_approval = |approval: Erc20ApprovalPayload| {
            PrimitivesTransaction::new(
                hash.clone(),
                AssetId::from_token(chain, &approval.contract_address),
                from.clone(),
                approval.spender,
                None,
                TransactionType::TokenApproval,
                state,
                fee.to_string(),
                fee_asset_id.clone(),
                approval.value.to_string(),
                None,
                None,
                created_at,
            )
        };

        let build_native_asset_transaction = |transaction_type: TransactionType, memo: Option<String>, data: Option<String>| {
            PrimitivesTransaction::new(
                hash.clone(),
                chain.as_asset_id(),
                from.clone(),
                to.clone(),
                None,
                transaction_type,
                state,
                fee.clone(),
                fee_asset_id.clone(),
                value.clone(),
                memo,
                None,
                created_at,
            )
            .with_data(data)
        };

        match payload {
            TransactionPayload::NativeTransfer => Some(build_native_asset_transaction(TransactionType::Transfer, None, None)),
            TransactionPayload::NativeTransferWithCallData => {
                let memo = decode_hex_utf8(&transaction.input).filter(|value| !value.is_empty());
                Some(build_native_asset_transaction(TransactionType::Transfer, memo, Some(transaction.input.clone())))
            }
            TransactionPayload::Erc20Approve(approval) => Some(build_erc20_approval(approval)),
            TransactionPayload::Erc20Transfer(transfer) => Some(build_erc20_transfer(transfer)),
            TransactionPayload::Erc721Transfer(transfer) | TransactionPayload::Erc1155Transfer(transfer) => Some(build_nft_transfer(transfer)),
            TransactionPayload::SmartContractCall => Some(build_native_asset_transaction(TransactionType::SmartContractCall, None, None)),
            TransactionPayload::Unknown => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::testkit::TEST_TRANSACTION_ID;
    use crate::rpc::model::{Log, Transaction, TransactionReceipt};
    use crate::rpc::transaction_payload::INPUT_0X;
    use num_bigint::BigUint;
    use primitives::{
        Chain, JsonRpcResult,
        asset_constants::{ARBITRUM_USDC_ASSET_ID, ARBITRUM_USDT_ASSET_ID, ETHEREUM_DAI_ASSET_ID, ETHEREUM_USDC_ASSET_ID, ETHEREUM_USDT_ASSET_ID},
        contract_constants::{ETHEREUM_YO_PROTOCOL_CONTRACT, UNISWAP_PERMIT2_CONTRACT},
        testkit::json_rpc::load_json_rpc_result,
    };

    #[test]
    fn test_map_smart_contract_call() {
        let contract_call_tx = load_json_rpc_result::<Transaction>(include_str!("../../testdata/contract_call_tx.json"));
        let contract_call_receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../testdata/contract_call_tx_receipt.json"));

        let transaction = EthereumMapper::map_transaction(Chain::Ethereum, &contract_call_tx, &contract_call_receipt, None, &BigUint::from(1735671600u64), None).unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::SmartContractCall);
        assert_eq!(transaction.hash, "0x876707912c2d625723aa14bf268d83ede36c2657c70da500628e40e6b51577c9");
        assert_eq!(transaction.from, "0x39ab5f6f1269590225EdAF9ad4c5967B09243747");
        assert_eq!(transaction.to, "0xB907Dcc926b5991A149d04Cb7C0a4a25dC2D8f9a");
    }

    #[test]
    fn test_erc20_transfer() {
        let erc20_transfer_tx = serde_json::from_value::<JsonRpcResult<Transaction>>(serde_json::from_str(include_str!("../../testdata/transfer_erc20.json")).unwrap())
            .unwrap()
            .result;
        let erc20_transfer_receipt =
            serde_json::from_value::<JsonRpcResult<TransactionReceipt>>(serde_json::from_str(include_str!("../../testdata/transfer_erc20_receipt.json")).unwrap())
                .unwrap()
                .result;

        let transaction = EthereumMapper::map_transaction(Chain::Arbitrum, &erc20_transfer_tx, &erc20_transfer_receipt, None, &BigUint::from(1735671600u64), None).unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.asset_id, ARBITRUM_USDT_ASSET_ID.clone());
        assert_eq!(transaction.from, "0x8d7460E51bCf4eD26877cb77E56f3ce7E9f5EB8F");
        assert_eq!(transaction.to, "0x2Fc617E933a52713247CE25730f6695920B3befe");
        assert_eq!(transaction.value, "4801292");
    }

    #[test]
    fn test_map_transaction_by_hash() {
        let transaction = serde_json::from_value::<JsonRpcResult<Transaction>>(serde_json::from_str(include_str!("../../testdata/transfer_nft_eip721.json")).unwrap())
            .unwrap()
            .result;
        let transaction_receipt =
            serde_json::from_value::<JsonRpcResult<TransactionReceipt>>(serde_json::from_str(include_str!("../../testdata/transfer_nft_eip721_receipt.json")).unwrap())
                .unwrap()
                .result;

        let transaction = EthereumMapper::map_transaction(Chain::Ethereum, &transaction, &transaction_receipt, None, &BigUint::from(1735671600u64), None).unwrap();
        assert_eq!(transaction.hash, TEST_TRANSACTION_ID);
        assert_eq!(transaction.transaction_type, TransactionType::TransferNFT);

        assert_eq!(transaction.asset_id, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(transaction.from, "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4");
        assert_eq!(transaction.to, "0xf1158986419F6058231b0Dbd7A78Ff0674ebBc50");
        assert_eq!(transaction.value, "0");
        assert_eq!(
            transaction.metadata,
            Some(serde_json::json!({
                "assetId": "ethereum_0x47A00fC8590C11bE4c419D9Ae50DEc267B6E24ee::9143"
            }))
        );
    }

    #[test]
    fn test_nft_eip721_safe_transfer() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../testdata/transfer_nft_eip721_safe_transfer.json"));
        let transaction_receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../testdata/transfer_nft_eip721_safe_transfer_receipt.json"));

        let transaction = EthereumMapper::map_transaction(Chain::Ethereum, &transaction, &transaction_receipt, None, &BigUint::from(1782990479u64), None).unwrap();
        assert_eq!(transaction.hash, "0x13bac6f98a228d71e9e31641629c0c0d8b2c46f93cfe4eafee5371548ab374ca");
        assert_eq!(transaction.transaction_type, TransactionType::TransferNFT);
        assert_eq!(transaction.asset_id, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(transaction.from, "0x3835e41EA342975eEEF8AaCf0c3809A38F6c04f1");
        assert_eq!(transaction.to, "0x951454CaD517FcB54a5A60f20C934Df90966b2a7");
        assert_eq!(transaction.value, "0");
        assert_eq!(
            transaction.metadata,
            Some(serde_json::json!({
                "assetId": "ethereum_0x47A00fC8590C11bE4c419D9Ae50DEc267B6E24ee::3757"
            }))
        );
    }

    #[test]
    fn test_nft_eip1155_transfer() {
        let transaction = serde_json::from_value::<JsonRpcResult<Transaction>>(serde_json::from_str(include_str!("../../testdata/transfer_nft_eip1155.json")).unwrap())
            .unwrap()
            .result;
        let transaction_receipt =
            serde_json::from_value::<JsonRpcResult<TransactionReceipt>>(serde_json::from_str(include_str!("../../testdata/transfer_nft_eip1155_receipt.json")).unwrap())
                .unwrap()
                .result;

        let transaction = EthereumMapper::map_transaction(Chain::Ethereum, &transaction, &transaction_receipt, None, &BigUint::from(1735671600u64), None).unwrap();
        assert_eq!(transaction.transaction_type, TransactionType::TransferNFT);

        assert_eq!(transaction.asset_id, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(transaction.from, "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4");
        assert_eq!(transaction.to, "0xEE67a32a55318a211CE4BB5051Ed98c679851143");
        assert_eq!(transaction.value, "0");
        assert_eq!(
            transaction.metadata,
            Some(serde_json::json!({
                "assetId": "ethereum_0xD4416b13d2b3a9aBae7AcD5D6C2BbDBE25686401::78312089388574796712357673212383836573632856632295981350303734331484536429721"
            }))
        );
    }

    #[test]
    fn test_smart_contract_erc20_transfer() {
        let sc_erc20_tx = serde_json::from_value::<JsonRpcResult<Transaction>>(serde_json::from_str(include_str!("../../testdata/contract_erc20_tx.json")).unwrap())
            .unwrap()
            .result;
        let sc_erc20_receipt =
            serde_json::from_value::<JsonRpcResult<TransactionReceipt>>(serde_json::from_str(include_str!("../../testdata/contract_erc20_receipt.json")).unwrap())
                .unwrap()
                .result;

        let transaction = EthereumMapper::map_transaction(Chain::Arbitrum, &sc_erc20_tx, &sc_erc20_receipt, None, &BigUint::from(1735671600u64), None).unwrap();

        assert_eq!(transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(transaction.asset_id, ARBITRUM_USDC_ASSET_ID.clone());
        assert_eq!(transaction.from, "0x2Df1c51E09aECF9cacB7bc98cB1742757f163dF7");
        assert_eq!(transaction.to, "0x0D9DAB1A248f63B0a48965bA8435e4de7497a3dC");
        assert_eq!(transaction.value, "930678651");
    }

    #[test]
    fn test_native_transfer_high_gas_limit() {
        let transaction = serde_json::from_value::<JsonRpcResult<Transaction>>(serde_json::from_str(include_str!("../../testdata/transfer_high_gas_limit.json")).unwrap())
            .unwrap()
            .result;
        let transaction_receipt =
            serde_json::from_value::<JsonRpcResult<TransactionReceipt>>(serde_json::from_str(include_str!("../../testdata/transfer_high_gas_limit_receipt.json")).unwrap())
                .unwrap()
                .result;

        let result = EthereumMapper::map_transaction(Chain::Ethereum, &transaction, &transaction_receipt, None, &BigUint::from(1735671600u64), None);

        assert!(result.is_some());
        let tx = result.unwrap();
        assert_eq!(tx.transaction_type, TransactionType::Transfer);
        assert_eq!(tx.asset_id, AssetId::from_chain(Chain::Ethereum));
        assert_eq!(tx.from, "0x8D25Fb438C6efCD08679ffA82766869B50E24608");
        assert_eq!(tx.to, "0x0700572b54ccA24Dad0eD4Cdad2c3d3ab6dB652a");
        assert_eq!(tx.value, "2739900000000000000");
        assert_eq!(tx.id.to_string(), "ethereum_0x0c0626172dbba6984a2e95b3abf1caba39cf11d3c9bc99d7de9ac814671c0cb1");
    }

    #[test]
    fn test_arbitrum_native_self_transfer_with_l1_gas() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../testdata/arbitrum_native_self_transfer_transaction.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../testdata/arbitrum_native_self_transfer_receipt.json"));

        let mapped_transaction = EthereumMapper::map_transaction(Chain::Arbitrum, &transaction, &receipt, None, &BigUint::from(1746499794u64), None).unwrap();

        assert_eq!(mapped_transaction.transaction_type, TransactionType::Transfer);
        assert_eq!(mapped_transaction.asset_id, AssetId::from_chain(Chain::Arbitrum));
        assert_eq!(mapped_transaction.from, "0x951454CaD517FcB54a5A60f20C934Df90966b2a7");
        assert_eq!(mapped_transaction.to, "0x951454CaD517FcB54a5A60f20C934Df90966b2a7");
        assert_eq!(mapped_transaction.value, "0");
    }

    #[test]
    fn test_zero_value_empty_transaction_to_different_address_is_not_transfer() {
        let transaction = Transaction {
            hash: "0xabc123".to_string(),
            from: "0xf1a3687303606a6fd48179ce503164cdcbabeab6".to_string(),
            gas: 50000,
            input: INPUT_0X.to_string(),
            to: Some("0x0700572b54cca24dad0ed4cdad2c3d3ab6db652a".to_string()),
            block_number: BigUint::from(1000u32),
            value: BigUint::from(0u8),
        };
        let receipt = TransactionReceipt {
            gas_used: BigUint::from(21442u64),
            effective_gas_price: BigUint::from(5_000_000_000u64),
            l1_fee: None,
            logs: vec![],
            status: "0x1".to_string(),
            block_hash: "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            block_number: BigUint::from(1000u32),
        };

        assert_eq!(
            EthereumMapper::map_transaction(Chain::Arbitrum, &transaction, &receipt, None, &BigUint::from(1735671600u64), None),
            None
        );
    }

    #[test]
    fn test_erc20_approve() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../testdata/approve.json"));
        let mut receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../testdata/approve_receipt.json"));

        let result = EthereumMapper::map_transaction(Chain::Ethereum, &transaction, &receipt, None, &BigUint::from(1735671600u64), None).unwrap();
        assert_eq!(result.transaction_type, TransactionType::TokenApproval);
        assert_eq!(result.asset_id, ETHEREUM_DAI_ASSET_ID.clone());
        assert_eq!(result.from, "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4");
        assert_eq!(result.to, UNISWAP_PERMIT2_CONTRACT);
        assert_eq!(result.value, "115792089237316195423570985008687907853269984665640564039457584007913129639935");

        receipt.logs.push(Log {
            address: "0x0000000000000000000000000000000000001010".to_string(),
            topics: vec!["0x4dfe1bbbcf077ddc3e01291eea2d5c70c2b422b415d95645b9adcfd678cb1d63".to_string()],
            data: "0x".to_string(),
            transaction_hash: None,
        });

        let result = EthereumMapper::map_transaction(Chain::Ethereum, &transaction, &receipt, None, &BigUint::from(1735671600u64), None).unwrap();
        assert_eq!(result.transaction_type, TransactionType::TokenApproval);
        assert_eq!(result.asset_id, ETHEREUM_DAI_ASSET_ID.clone());
        assert_eq!(result.from, "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4");
        assert_eq!(result.to, UNISWAP_PERMIT2_CONTRACT);
        assert_eq!(result.value, "115792089237316195423570985008687907853269984665640564039457584007913129639935");
    }

    #[test]
    fn test_map_smartchain_staking_transaction() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../testdata/smartchain/transaction_staking_delegate.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../testdata/smartchain/transaction_staking_delegate_receipt.json"));
        let tx = EthereumMapper::map_transaction(Chain::SmartChain, &transaction, &receipt, None, &BigUint::from(1735671600u64), None).unwrap();

        assert_eq!(tx.transaction_type, TransactionType::StakeDelegate);
        assert_eq!(tx.from, "0x51eD60604637989d19D29e43c5D94B098A0d1Af7");
        assert_eq!(tx.to, "0xd34403249B2d82AAdDB14e778422c966265e5Fb5");
        assert_eq!(tx.contract.as_deref(), Some("0x0000000000000000000000000000000000002002"));
        assert_eq!(tx.value, "1000000000000000000");
        assert_eq!(tx.metadata, None);
    }

    #[test]
    fn test_mayan_native_swap() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../testdata/mayan_native_swap_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../testdata/mayan_native_swap_tx_receipt.json"));

        let tx = EthereumMapper::map_transaction(Chain::Polygon, &transaction, &receipt, None, &BigUint::from(1735671600u64), None).unwrap();

        assert_eq!(tx.transaction_type, TransactionType::SmartContractCall);
        assert_eq!(tx.asset_id, AssetId::from_chain(Chain::Polygon));
        assert_eq!(tx.from, "0x551Ac3629eC87F3957b1074FaF48d22A5a26ecec");
        assert_eq!(tx.to, "0x337685fdaB40D39bd02028545a4FfA7D287cC3E2");
        assert_eq!(tx.value, "124798001816181500204");
    }

    #[test]
    fn test_mayan_token_swap() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../testdata/mayan_token_swap_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../testdata/mayan_token_swap_tx_receipt.json"));

        let tx = EthereumMapper::map_transaction(Chain::Polygon, &transaction, &receipt, None, &BigUint::from(1735671600u64), None).unwrap();

        assert_eq!(tx.transaction_type, TransactionType::SmartContractCall);
        assert_eq!(tx.asset_id, AssetId::from_chain(Chain::Polygon));
        assert_eq!(tx.from, "0x0DC153E9225a0d74460d806C08c961a3EC0ef17D");
        assert_eq!(tx.to, "0x337685fdaB40D39bd02028545a4FfA7D287cC3E2");
        assert_eq!(tx.value, "0");
    }

    #[test]
    fn test_native_transfer_with_memo() {
        let memo = "=:LTC.LTC:ltc1qexample";
        let input = format!("0x{}", hex::encode(memo));

        let transaction = Transaction {
            hash: "0xabc123".to_string(),
            from: "0xf1a3687303606a6fd48179ce503164cdcbabeab6".to_string(),
            to: Some("0x0700572b54cca24dad0ed4cdad2c3d3ab6db652a".to_string()),
            value: BigUint::from(1_000_000_000_000_000_000u64),
            gas: 50000,
            input: input.clone(),
            block_number: BigUint::from(1000u32),
        };

        let receipt = TransactionReceipt {
            gas_used: BigUint::from(22496u32),
            effective_gas_price: BigUint::from(5_000_000_000u64),
            l1_fee: None,
            logs: vec![],
            status: "0x1".to_string(),
            block_hash: "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            block_number: BigUint::from(1000u32),
        };

        let tx = EthereumMapper::map_transaction(Chain::SmartChain, &transaction, &receipt, None, &BigUint::from(1735671600u64), None).unwrap();

        assert_eq!(tx.transaction_type, TransactionType::Transfer);
        assert_eq!(tx.asset_id, AssetId::from_chain(Chain::SmartChain));
        assert_eq!(tx.memo, Some(memo.to_string()));
        assert_eq!(tx.data, Some(input));
    }

    #[test]
    fn test_claim_rewards_erc20_transfer() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../testdata/claim_rewards_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../testdata/claim_rewards_receipt.json"));

        let tx = EthereumMapper::map_transaction(Chain::Ethereum, &transaction, &receipt, None, &BigUint::from(1735671600u64), None).unwrap();

        assert_eq!(tx.transaction_type, TransactionType::Transfer);
        assert_eq!(tx.asset_id, ETHEREUM_USDC_ASSET_ID.clone());
        assert_eq!(tx.from, "0x34DeFF97889f3A6A483E3b9255cAFCB9a6e03588");
        assert_eq!(tx.to, "0x0533d3A18D3f812eCFcC838B59B34fEc4d18E4AC");
        assert_eq!(tx.value, "3900075892");
    }

    #[test]
    fn test_yo_deposit() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../testdata/yo_deposit_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../testdata/yo_deposit_receipt.json"));

        let tx = EthereumMapper::map_transaction(Chain::Ethereum, &transaction, &receipt, None, &BigUint::from(1735671600u64), None).unwrap();

        assert_eq!(tx.transaction_type, TransactionType::EarnDeposit);
        assert_eq!(tx.asset_id, ETHEREUM_USDT_ASSET_ID.clone());
        assert_eq!(tx.from, "0x8d7460E51bCf4eD26877cb77E56f3ce7E9f5EB8F");
        assert_eq!(tx.to, ETHEREUM_YO_PROTOCOL_CONTRACT);
        assert_eq!(tx.value, "1466009");
    }

    #[test]
    fn test_yo_withdraw() {
        let transaction = load_json_rpc_result::<Transaction>(include_str!("../../testdata/yo_withdraw_tx.json"));
        let receipt = load_json_rpc_result::<TransactionReceipt>(include_str!("../../testdata/yo_withdraw_receipt.json"));

        let tx = EthereumMapper::map_transaction(Chain::Ethereum, &transaction, &receipt, None, &BigUint::from(1735671600u64), None).unwrap();

        assert_eq!(tx.transaction_type, TransactionType::EarnWithdraw);
        assert_eq!(tx.asset_id, ETHEREUM_USDT_ASSET_ID.clone());
        assert_eq!(tx.from, "0x8d7460E51bCf4eD26877cb77E56f3ce7E9f5EB8F");
        assert_eq!(tx.to, ETHEREUM_YO_PROTOCOL_CONTRACT);
        assert_eq!(tx.value, "1466126");
    }
}
