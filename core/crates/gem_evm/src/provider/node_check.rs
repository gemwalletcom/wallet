use chain_traits::{ChainBalances, ChainState, NodeCheckRecorder, NodeCheckReporter};
use gem_client::Client;
use gem_jsonrpc::types::{ERROR_CLIENT_ERROR, ERROR_METHOD_NOT_FOUND};
use num_traits::ToPrimitive;
use primitives::{Chain, NodeCheckProfile, NodeCheckReport};
use serde_json::{Value, json};

use crate::{
    jsonrpc::TransactionObject,
    rpc::{
        EthereumClient,
        model::{BlockTransactionsIds, Transaction, TransactionReceipt, TransactionReplayTrace},
    },
};

const MAX_RESULT_LENGTH: usize = 128;

#[derive(Clone, Copy)]
struct NodeFixture {
    address: &'static str,
    transaction_id: &'static str,
}

impl<C: Client + Clone> EthereumClient<C> {
    pub(super) async fn check_node_profile(&self, profile: NodeCheckProfile, reporter: &dyn NodeCheckReporter) -> NodeCheckReport {
        let mut recorder = NodeCheckRecorder::new(reporter);
        if !self.check_chain(&mut recorder).await {
            return recorder.finish();
        }

        let Some(fixture) = fixture(self.get_chain()) else {
            recorder.record::<(), _>("fixture", Err("not configured"), |_| String::new());
            return recorder.finish();
        };

        match profile {
            NodeCheckProfile::LoadBalancer => self.check_load_balancer(fixture, &mut recorder).await,
            NodeCheckProfile::Parser => {
                let _ = self.check_parser(fixture, &mut recorder).await;
            }
            NodeCheckProfile::ArchivalParser => self.check_archival_parser(fixture, &mut recorder).await,
        }
        recorder.finish()
    }

    async fn check_chain(&self, recorder: &mut NodeCheckRecorder<'_>) -> bool {
        let chain = self.get_chain();
        let expected = chain.network_id();
        let chain_id = ChainState::get_chain_id(self).await.map_err(|error| error.to_string()).and_then(|chain_id| {
            if chain_id == expected {
                Ok(chain_id)
            } else {
                Err(format!("expected {expected}, received {chain_id}"))
            }
        });
        if recorder.record("eth_chainId", chain_id, Clone::clone).is_none() {
            return false;
        }

        let block_number = ChainState::get_block_latest_number(self)
            .await
            .map_err(|error| error.to_string())
            .and_then(|block_number| if block_number > 0 { Ok(block_number) } else { Err("received zero".to_string()) });
        recorder.record("eth_blockNumber", block_number, ToString::to_string).is_some()
    }

    async fn check_load_balancer(&self, fixture: NodeFixture, recorder: &mut NodeCheckRecorder<'_>) {
        let balance = ChainBalances::get_balance_coin(self, fixture.address.to_string()).await;
        recorder.record("eth_getBalance", balance, |result| result.balance.available.to_string());

        let transaction_count = self.get_transaction_count(fixture.address).await;
        recorder.record("eth_getTransactionCount", transaction_count, Clone::clone);

        let _ = self.check_transaction_receipt(fixture.transaction_id, None, recorder).await;

        let fee_history = self.get_fee_history(1, vec![50]).await;
        recorder.record("eth_feeHistory", fee_history, |_| "available".to_string());

        let gas_price = self.call::<String>("eth_gasPrice".to_string(), json!([])).await;
        recorder.record("eth_gasPrice", gas_price, Clone::clone);

        let code = self.call::<String>("eth_getCode".to_string(), json!([fixture.address, "latest"])).await;
        recorder.record("eth_getCode", code, format_result);

        let syncing = self.call::<Value>("eth_syncing".to_string(), json!([])).await;
        recorder.record("eth_syncing", syncing, ToString::to_string);

        let call = self.eth_call::<String>(fixture.address, "0x").await;
        recorder.record("eth_call", call, format_result);

        let gas = self.estimate_gas(None, fixture.address, None, Some("0x")).await;
        recorder.record("eth_estimateGas", gas, Clone::clone);

        let transaction = TransactionObject::new_call_with_from(fixture.address, fixture.address, Vec::new());
        let trace = self.trace_call(&transaction).await;
        recorder.record("trace_call", trace, |_| "available".to_string());

        let params = json!([{ "to": fixture.address, "data": "0x" }, "latest"]);
        let calls = vec![("eth_call".to_string(), params.clone()), ("eth_call".to_string(), params)];
        let batch = self.client.batch_call::<Value>(calls).await.and_then(|results| results.take_all());
        recorder.record("json_rpc_batch", batch, |results| results.len().to_string());

        let broadcast = match self.send_raw_transaction("0x").await {
            Ok(_) => Err("invalid request was accepted".to_string()),
            Err(error) => match error.code {
                ERROR_METHOD_NOT_FOUND | ERROR_CLIENT_ERROR => Err(error.to_string()),
                _ => Ok(error.code),
            },
        };
        recorder.record("eth_sendRawTransaction", broadcast, ToString::to_string);
    }

    async fn check_parser(&self, fixture: NodeFixture, recorder: &mut NodeCheckRecorder<'_>) -> Option<(u64, usize)> {
        let transaction = self
            .call::<Option<Transaction>>("eth_getTransactionByHash".to_string(), json!([fixture.transaction_id]))
            .await
            .map_err(|error| error.to_string())
            .and_then(|transaction| transaction.ok_or_else(|| "returned null".to_string()))
            .and_then(|transaction| {
                if !transaction.hash.eq_ignore_ascii_case(fixture.transaction_id) {
                    return Err(format!("returned {}", transaction.hash));
                }
                let block_number = transaction
                    .block_number
                    .to_u64()
                    .ok_or_else(|| format!("transaction block number is too large: {}", transaction.block_number))?;
                Ok((block_number, transaction))
            });
        let (block_number, transaction) = recorder.record("eth_getTransactionByHash", transaction, |(block_number, _)| block_number.to_string())?;

        self.check_transaction_receipt(fixture.transaction_id, Some(&transaction), recorder).await?;

        let block = async {
            let block = self.get_block(block_number).await.map_err(|error| error.to_string())?;
            if !block.transactions.iter().any(|transaction| transaction.hash.eq_ignore_ascii_case(fixture.transaction_id)) {
                return Err(format!("transaction {} is missing", fixture.transaction_id));
            }
            let transaction_ids = self
                .call::<Option<BlockTransactionsIds>>("eth_getBlockByNumber".to_string(), json!([format!("0x{block_number:x}"), false]))
                .await
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "hash-only response returned null".to_string())?;
            let matches = block.timestamp == transaction_ids.timestamp
                && block.transactions.len() == transaction_ids.transactions.len()
                && block
                    .transactions
                    .iter()
                    .zip(&transaction_ids.transactions)
                    .all(|(transaction, transaction_id)| transaction.hash.eq_ignore_ascii_case(transaction_id));
            if matches {
                Ok(block)
            } else {
                Err("full and hash-only responses do not match".to_string())
            }
        }
        .await;
        let block = recorder.record("eth_getBlockByNumber", block, |_| block_number.to_string())?;

        let receipts = self.get_block_receipts(block_number).await.map_err(|error| error.to_string()).and_then(|receipts| {
            if receipts.len() == block.transactions.len() && receipts.iter().all(|receipt| receipt.block_number == transaction.block_number) {
                Ok(receipts)
            } else {
                Err("receipts do not match block transactions".to_string())
            }
        });
        recorder.record("eth_getBlockReceipts", receipts, |_| block_number.to_string())?;
        Some((block_number, block.transactions.len()))
    }

    async fn check_archival_parser(&self, fixture: NodeFixture, recorder: &mut NodeCheckRecorder<'_>) {
        let Some((block_number, transaction_count)) = self.check_parser(fixture, recorder).await else {
            return;
        };

        let traces = self
            .trace_replay_block_transactions(block_number)
            .await
            .map_err(|error| error.to_string())
            .and_then(|traces| {
                if traces.len() == transaction_count {
                    Ok(traces)
                } else {
                    Err("traces do not match block transactions".to_string())
                }
            });
        recorder.record("trace_replayBlockTransactions", traces, |_| block_number.to_string());

        let trace = self
            .call::<TransactionReplayTrace>("trace_replayTransaction".to_string(), json!([fixture.transaction_id, ["stateDiff"]]))
            .await;
        recorder.record("trace_replayTransaction", trace, |_| block_number.to_string());
    }

    async fn check_transaction_receipt(&self, transaction_id: &str, transaction: Option<&Transaction>, recorder: &mut NodeCheckRecorder<'_>) -> Option<TransactionReceipt> {
        let receipt = self
            .get_transaction_receipt(transaction_id)
            .await
            .map_err(|error| error.to_string())
            .and_then(|receipt| receipt.ok_or_else(|| "returned null".to_string()))
            .and_then(|receipt| {
                if !receipt.has_valid_block_reference() {
                    return Err("invalid block reference".to_string());
                }
                if transaction.is_some_and(|transaction| receipt.block_number != transaction.block_number) {
                    return Err("transaction and receipt block numbers do not match".to_string());
                }
                Ok(receipt)
            });
        recorder.record("eth_getTransactionReceipt", receipt, |receipt| receipt.block_number.to_string())
    }
}

fn format_result(result: &impl AsRef<str>) -> String {
    let result = result.as_ref();
    if result.len() <= MAX_RESULT_LENGTH {
        result.to_string()
    } else {
        "available".to_string()
    }
}

fn fixture(chain: Chain) -> Option<NodeFixture> {
    let fixture = match chain {
        Chain::Ethereum => NodeFixture {
            address: "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4",
            transaction_id: "0x98dd4d9a586620f84e8066f1b015d663f9c0c94c4e0e02377840c3e6d43e2ad3",
        },
        Chain::SmartChain => NodeFixture {
            address: "0x2A49C84B7173e21f9116B2798735f87531526b36",
            transaction_id: "0xa9f6e1d1a02ba5bb5aa9b3c83773ef9ac6d8fe9abb1fa4512d422f0194d5d833",
        },
        Chain::Polygon => NodeFixture {
            address: "0x2A49C84B7173e21f9116B2798735f87531526b36",
            transaction_id: "0x3d4eb72380e6095d0667c6ec3420719dbec7d1d8b1628464a03ee6850ee716ed",
        },
        Chain::Plasma => NodeFixture {
            address: "0x8192bf75cb263e543c4f2c06edb983139034aa0f",
            transaction_id: "0x6d83a79e228ddaa04107afb03cfd1b1b74b24429d322d8e79d756e559895d3a8",
        },
        Chain::Arbitrum => NodeFixture {
            address: "0x00000000000000000000000000000000000a4b05",
            transaction_id: "0x6a38409d346190d38a28be23db35dcda5dc88df0de99c23049c967c388359857",
        },
        Chain::Optimism => NodeFixture {
            address: "0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001",
            transaction_id: "0xc4edd56597745ae8fc8486b2cdf003ea52d9b37b0f72361eff3b5d73d62ae731",
        },
        Chain::Base => NodeFixture {
            address: "0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001",
            transaction_id: "0xb7f529ed53a7f716976cd53520677260b53edf011da7573374ccf8705b6b4a8e",
        },
        Chain::AvalancheC => NodeFixture {
            address: "0xa36c8b1737195e634019fe27ae13d52d2e96947f",
            transaction_id: "0x64317b42490640403cb5a1c0c9c8672a7aa6f0216f372be8113d1ea84ad7ce0d",
        },
        Chain::OpBNB => NodeFixture {
            address: "0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001",
            transaction_id: "0x8581e4d41399e899fcf0e828b3b986b45854375d617ce5abc565afbd54741955",
        },
        Chain::Fantom => NodeFixture {
            address: "0x56730257ec944da158fdb3af7bbfbacabeaf9dbe",
            transaction_id: "0x2c2c6b8a00eab2a8d948ee5ecf95730642ce03230870fe4e24657bfdff170254",
        },
        Chain::Gnosis => NodeFixture {
            address: "0x8c4c15870d27c1194b6893f6b94dd0ce9c2c8ba2",
            transaction_id: "0x3b6f77ef3007b5e54fe8de3b3bcda971528b35eda0669e4893a97b6a35a4c31c",
        },
        Chain::Manta => NodeFixture {
            address: "0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001",
            transaction_id: "0xc8aabd35fc1e43dde16709b2d489569202c47c273e3f59c7cbb5df8f9b0fe65a",
        },
        Chain::Blast => NodeFixture {
            address: "0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001",
            transaction_id: "0xf81fffef507b5a18f073f701f4cf0df050cdfab2e0d4869be8a186bb61e626a4",
        },
        Chain::ZkSync => NodeFixture {
            address: "0x0baa722aefa911a4f7e7657198bcdb9efc06bf38",
            transaction_id: "0x863aa2a481a309574009c53f2449bb21f9adb9d59bc56b4835d8f785c529fc02",
        },
        Chain::Linea => NodeFixture {
            address: "0x32c1e0876c6b2a907d06965d5625128daa4d893b",
            transaction_id: "0x4cd8dba40e71cdf21fc6da8020a6e75d98e549ec31c5bb5ce6e8929638cf9c7f",
        },
        Chain::Mantle => NodeFixture {
            address: "0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001",
            transaction_id: "0xf968326c238982141a97bca543f184f28e71d8db95882662558b4edc5476b30d",
        },
        Chain::Celo => NodeFixture {
            address: "0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001",
            transaction_id: "0xa6dcede6af9e3c0324971790bb03e07c820c13f84396e71864ed3dd5643e8e12",
        },
        Chain::World => NodeFixture {
            address: "0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001",
            transaction_id: "0x6bc975455d9552086286e75b5be6351d2b29f9b8be061f289cadc1ce5ca1de8f",
        },
        Chain::Sonic => NodeFixture {
            address: "0x7e62e6c99a80e28669a55fcef1316b78f97b4319",
            transaction_id: "0x46cffcb41f25a43ea91f05704eeb27bc45391f616e1bf7e2e30ace5ce263ceac",
        },
        Chain::SeiEvm => NodeFixture {
            address: "0x028a9fd11fc977de04d7b509e0c7b1e22545c7f3",
            transaction_id: "0x4fc879341cb99aeb24ef2388176bc0915a412273ff3fe93b905902adb64d949d",
        },
        Chain::Abstract => NodeFixture {
            address: "0x53244757268dada82a8064b6570651f0e30a647e",
            transaction_id: "0xe064ad2d215da437b8496a95fc6d6b1124930599ca1eabb9bad515921e666105",
        },
        Chain::Berachain => NodeFixture {
            address: "0xfffffffffffffffffffffffffffffffffffffffe",
            transaction_id: "0x6ce80fa54e067a9b36c7280eb93323b588636942805ef3643dd659c070b655bd",
        },
        Chain::Ink => NodeFixture {
            address: "0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001",
            transaction_id: "0x1e455c14cf075a83e2fb5bbd165ff53cc0eb1699709bdb665f709f8560503527",
        },
        Chain::Unichain => NodeFixture {
            address: "0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001",
            transaction_id: "0x2f931c88701faffc04dd65d5d05857dbaa76ec43a62116c6a69071c827d9c99e",
        },
        Chain::Hyperliquid => NodeFixture {
            address: "0xfe65cc490daf50ee9a0503669bd7ec465090c81c",
            transaction_id: "0x4785e5c28dbc8ec640b00a4985cf518926a5364a6843a48fe0e84edee3952093",
        },
        Chain::Monad => NodeFixture {
            address: "0x6f49a8f621353f12378d0046e7d7e4b9b249dc9e",
            transaction_id: "0xae2fe7ab7d6920d84b78126dc2ce82a1e227e4f70bd7f037c3747396d5a73c57",
        },
        Chain::XLayer => NodeFixture {
            address: "0xdeaddeaddeaddeaddeaddeaddeaddeaddead0001",
            transaction_id: "0xa6e649c54eaf86b5bb51e0230bf97499ff348e2e5e6527aaddc55183b7ec8211",
        },
        Chain::Robinhood => NodeFixture {
            address: "0x00000000000000000000000000000000000a4b05",
            transaction_id: "0xdd81e20bb08437587dc6f6e2a7f0d43bd96101ca51f051c42806a307636f10db",
        },
        Chain::Stable => NodeFixture {
            address: "0x8888888888888888888888888888888888888888",
            transaction_id: "0x312b2a62ab4927fc7805789184f7e87c8e2e1e87c6eaa01706e58a979a54d4df",
        },
        Chain::Bitcoin
        | Chain::BitcoinCash
        | Chain::Litecoin
        | Chain::Solana
        | Chain::Thorchain
        | Chain::Mayachain
        | Chain::Cosmos
        | Chain::Osmosis
        | Chain::Ton
        | Chain::Tron
        | Chain::Doge
        | Chain::Zcash
        | Chain::Aptos
        | Chain::Sui
        | Chain::Xrp
        | Chain::Celestia
        | Chain::Injective
        | Chain::Sei
        | Chain::Noble
        | Chain::Near
        | Chain::Stellar
        | Chain::Algorand
        | Chain::Polkadot
        | Chain::Cardano
        | Chain::HyperCore => return None,
    };
    Some(fixture)
}
