// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import BlockchainTestKit
import ChainServiceTestKit
import struct Gemstone.GemConfirmData
import struct Gemstone.GemConfirmInput
import struct Gemstone.GemConfirmLoadOptions
import protocol Gemstone.GemConfirmServiceProtocol
import struct Gemstone.GemSignedTransaction
import enum Gemstone.GemTransactionInputType
import Primitives
import PrimitivesTestKit
import Signer
import SignerTestKit
import Store
import StoreTestKit
import Testing
import TransactionStateService
import TransactionStateServiceTestKit
@testable import Transfer

struct TransferExecutorTests {
    @Test
    func paymentPendingTransactionUsesTransferData() throws {
        let memo = "ck:262:operator:m:1787598390"
        let asset = Asset.mockSolanaUSDC()
        let transferData = TransferData.mockPayment(
            asset: asset,
            transaction: "encoded-transaction",
            recipient: RecipientData(
                recipient: Recipient(name: nil, address: "recipient", memo: memo),
                amount: nil,
            ),
            amount: .exact(19_000_000),
        )

        let transaction = try TransactionFactory.makePendingTransaction(
            wallet: .mock(accounts: [.mock(chain: .solana)]),
            transferData: transferData,
            transactionData: .mock(),
            amount: .mock(value: 19_000_000),
            hash: "hash",
            transactionType: .transfer,
        )

        #expect(transaction.assetId == asset.id)
        #expect(transaction.value == "19000000")
        #expect(transaction.memo == memo)
    }

    @Test
    func genericPendingTransactionUsesSimulationHeader() throws {
        let headerAsset = Asset.mockSolanaUSDC()
        let transferData = TransferData.mockPayment(
            asset: .mockSolana(),
            transaction: "encoded-transaction",
            recipient: RecipientData(
                recipient: Recipient(name: nil, address: "recipient", memo: nil),
                amount: nil,
            ),
            amount: .exact(.zero),
        )

        let transaction = try TransactionFactory.makePendingTransaction(
            wallet: .mock(accounts: [.mock(chain: .solana)]),
            transferData: transferData,
            transactionData: .mock(),
            amount: .mock(value: .zero),
            hash: "hash",
            transactionType: .transfer,
            simulation: SimulationResult(
                warnings: [],
                balanceChanges: [],
                payload: [],
                header: SimulationHeader(assetId: headerAsset.id, value: "19000000", isUnlimited: false),
            ),
        )

        #expect(transaction.assetId == headerAsset.id)
        #expect(transaction.value == "19000000")
    }

    @Test
    func hyperCorePerpetualStoresPrimaryOrder() async throws {
        let db = DB.mockAssets(assets: [.mock(asset: Asset.mockHypercoreUSDC())])
        let transactionStore = TransactionStore(db: db)
        let executor = TransferExecutor(
            signer: TransactionSignerMock(signedTransactions: [
                GemSignedTransaction(data: "update_leverage", transactionType: .perpetualOpenPosition),
                GemSignedTransaction(data: "primary_order", transactionType: .perpetualOpenPosition),
                GemSignedTransaction(data: "position_tpsl", transactionType: .perpetualOpenPosition),
            ]),
            confirmService: GemConfirmServiceMock(hashes: ["action:1", "order:413978262893", "action:2"]),
            assetsEnabler: .mock(),
            transactionStateScheduler: .mock(transactionStore: transactionStore),
        )

        let input = TransferConfirmationInput(
            data: .mock(type: .perpetual(Asset.mockHypercoreUSDC(), .open(.mock()))),
            wallet: .mock(accounts: [Account.mock(chain: .hyperCore)]),
            transactionData: .mock(feeAsset: Asset.mockHypercoreUSDC()),
            amount: .mock(),
            delegate: nil,
        )
        try await executor.execute(input: input)

        let transactions = try transactionStore.getTransactions(states: [.pending])
        #expect(transactions.count == 1)
        #expect(transactions.first?.id.hash == "order:413978262893")
    }

    @Test
    func swapTransactions() async throws {
        let spender = "0x000000000022D473030F116dDEE9F6B43aC78BA3"
        let approvalValue = "115792089237316195423570985008687907853269984665640564039457584007913129639935"
        let fromAsset = Asset.mockEthereumUSDT()
        let swapData = SwapData.mock(data: SwapQuoteData(
            to: "0x111111125421cA6dc452d289314280a0f8842A65",
            dataType: .contract,
            value: "0",
            data: "swap-data",
            memo: nil,
            approval: ApprovalData(token: fromAsset.id.tokenId ?? "", spender: spender, value: approvalValue, isUnlimited: true),
            gasLimit: nil,
        ))
        let db = DB.mockAssets()
        let transactionStore = TransactionStore(db: db)
        let executor = TransferExecutor(
            signer: TransactionSignerMock(signedTransactions: [
                GemSignedTransaction(data: "approval_tx", transactionType: .tokenApproval),
                GemSignedTransaction(data: "swap_tx", transactionType: .swap),
            ]),
            confirmService: GemConfirmServiceMock(hashes: ["hash0", "hash1"]),
            assetsEnabler: .mock(),
            transactionStateScheduler: .mock(transactionStore: transactionStore),
        )

        let input = TransferConfirmationInput(
            data: .mock(type: .swap(fromAsset, .mock(), swapData)),
            wallet: .mock(accounts: [.mock(chain: .ethereum), .mock(chain: .bitcoin)]),
            transactionData: .mock(),
            amount: .mock(),
            delegate: nil,
        )
        try await executor.execute(input: input)

        let transactions = try transactionStore.getTransactions(states: [.pending])
        #expect(transactions.count == 2)
        #expect(transactions.map(\.id.hash).sorted() == ["hash0", "hash1"])
        let approvalTransaction = try #require(transactions.first { $0.id.hash == "hash0" })
        #expect(approvalTransaction.assetId == fromAsset.id)
        #expect(approvalTransaction.to == spender)
        #expect(approvalTransaction.value == approvalValue)
        #expect(approvalTransaction.type == .tokenApproval)
        let swapTransaction = try #require(transactions.first { $0.id.hash == "hash1" })
        #expect(swapTransaction.to == swapData.data.to)
        #expect(swapTransaction.type == .swap)
    }

    @Test
    func genericApprovalTransaction() async throws {
        let token = Asset.mockEthereumUSDT()
        let spender = "0x000000000022D473030F116dDEE9F6B43aC78BA3"
        let approval = try ApprovalData(
            token: #require(token.id.tokenId),
            spender: spender,
            value: "100",
            isUnlimited: false,
        )
        let db = DB.mockAssets()
        let transactionStore = TransactionStore(db: db)
        let executor = TransferExecutor(
            signer: TransactionSignerMock(signedTransactions: [
                GemSignedTransaction(data: "approval_tx", transactionType: .tokenApproval),
            ]),
            confirmService: GemConfirmServiceMock(hashes: ["approval-hash"]),
            assetsEnabler: .mock(),
            transactionStateScheduler: .mock(transactionStore: transactionStore),
        )
        let transferData = TransferData.mock(
            type: .generic(
                asset: .mockEthereum(),
                metadata: .mock(),
                extra: .mock(
                    to: approval.token,
                    transactionType: .tokenApproval,
                    approval: approval,
                ),
            ),
        )

        try await executor.execute(input: TransferConfirmationInput(
            data: transferData,
            wallet: .mock(accounts: [.mock(chain: .ethereum)]),
            transactionData: .mock(),
            amount: .mock(),
            delegate: nil,
        ))

        let transaction = try #require(transactionStore.getTransactions(states: [.pending]).first)
        #expect(transaction.assetId == token.id)
        #expect(transaction.to == spender)
        #expect(transaction.value == approval.value)
        #expect(transaction.type == .tokenApproval)
    }

    @Test
    func hyperCoreSpotSwapStoresOnlyFinalOrder() async throws {
        let hype = Asset.mockHypercore()
        let usdc = Asset.mockHypercoreSpotUSDC()
        let db = DB.mockAssets(assets: [.mock(asset: hype), .mock(asset: usdc)])
        let transactionStore = TransactionStore(db: db)
        let executor = TransferExecutor(
            signer: TransactionSignerMock(signedTransactions: [
                GemSignedTransaction(data: "approve_referral", transactionType: .swap),
                GemSignedTransaction(data: "approve_agent", transactionType: .swap),
                GemSignedTransaction(data: "place_order", transactionType: .swap),
            ]),
            confirmService: GemConfirmServiceMock(hashes: [
                "action:1",
                "action:2",
                "order:413978262893",
            ]),
            assetsEnabler: .mock(),
            transactionStateScheduler: .mock(transactionStore: transactionStore),
        )
        let swapData = SwapData.mock(
            quote: .mock(
                providerData: SwapProviderData(
                    provider: .hyperliquid,
                    name: "Hyperliquid",
                    protocolName: "Hyperliquid",
                ),
            ),
        )

        let input = TransferConfirmationInput(
            data: .mock(type: .swap(hype, usdc, swapData)),
            wallet: .mock(accounts: [Account.mock(chain: .hyperCore)]),
            transactionData: .mock(feeAsset: Asset.mockHypercoreSpotUSDC()),
            amount: .mock(),
            delegate: nil,
        )
        try await executor.execute(input: input)

        let transactions = try transactionStore.getTransactions(states: [.pending])
        #expect(transactions.count == 1)
        #expect(transactions.first?.id.hash == "order:413978262893")
        #expect(transactions.first?.type == .swap)
    }

    @Test
    func hyperCoreUnstakeStoresFinalAction() async throws {
        let db = DB.mockAssets(assets: [
            .mock(asset: .mockHypercore()),
            .mock(asset: Asset.mockHypercoreSpotUSDC()),
        ])
        let transactionStore = TransactionStore(db: db)
        let executor = TransferExecutor(
            signer: TransactionSignerMock(signedTransactions: [
                GemSignedTransaction(data: "undelegate", transactionType: .stakeUndelegate),
                GemSignedTransaction(data: "withdraw", transactionType: .stakeUndelegate),
            ]),
            confirmService: GemConfirmServiceMock(hashes: [
                "action:tokenDelegate:3001423:unstake:1780078264488",
                "action:cWithdraw:3001423:1780078264489",
            ]),
            assetsEnabler: .mock(),
            transactionStateScheduler: .mock(transactionStore: transactionStore),
        )

        let input = TransferConfirmationInput(
            data: .mock(type: .stake(.mockHypercore(), .unstake(.mock()))),
            wallet: .mock(accounts: [Account.mock(chain: .hyperCore)]),
            transactionData: .mock(feeAsset: Asset.mockHypercoreSpotUSDC()),
            amount: .mock(),
            delegate: nil,
        )
        try await executor.execute(input: input)

        let transactions = try transactionStore.getTransactions(states: [.pending])
        #expect(transactions.count == 1)
        #expect(transactions.first?.id.hash == "action:cWithdraw:3001423:1780078264489")
    }

    @Test
    func hyperCoreTransferKeepsTransaction() async throws {
        let db = DB.mockAssets()
        let transactionStore = TransactionStore(db: db)
        let executor = TransferExecutor(
            signer: TransactionSignerMock(signedTransactions: [GemSignedTransaction(data: "tx", transactionType: .transfer)]),
            confirmService: GemConfirmServiceMock(hashes: ["hash"]),
            assetsEnabler: .mock(),
            transactionStateScheduler: .mock(transactionStore: transactionStore),
        )

        let input = TransferConfirmationInput(
            data: .mock(type: .transfer(.mockEthereum())),
            wallet: .mock(accounts: [.mock(chain: .ethereum)]),
            transactionData: .mock(),
            amount: .mock(),
            delegate: nil,
        )

        try await executor.execute(input: input)

        let transactions = try transactionStore.getTransactions(states: [.pending])
        #expect(transactions.count == 1)
        #expect(transactions.first?.id.hash == "hash")
    }

    @Test
    func perpetualModifyDoesNotStoreTransaction() async throws {
        let db = DB.mockAssets(assets: [.mock(asset: Asset.mockHypercoreUSDC())])
        let transactionStore = TransactionStore(db: db)
        let executor = TransferExecutor(
            signer: TransactionSignerMock(signedTransactions: [GemSignedTransaction(data: "modify_tx", transactionType: .perpetualModifyPosition)]),
            confirmService: GemConfirmServiceMock(hashes: ["hash"]),
            assetsEnabler: .mock(),
            transactionStateScheduler: .mock(transactionStore: transactionStore),
        )

        let input = TransferConfirmationInput(
            data: .mock(type: .perpetual(Asset.mockHypercoreUSDC(), .mockModify())),
            wallet: .mock(accounts: [Account.mock(chain: .hyperCore)]),
            transactionData: .mock(),
            amount: .mock(),
            delegate: nil,
        )

        try await executor.execute(input: input)

        let transactions = try transactionStore.getTransactions(states: [.pending])
        #expect(transactions.isEmpty)
    }
}

private final class GemConfirmServiceMock: GemConfirmServiceProtocol, @unchecked Sendable {
    private let hashes: [String]

    init(hashes: [String]) {
        self.hashes = hashes
    }

    func broadcast(inputType _: GemTransactionInputType, transactions: [GemSignedTransaction]) async throws -> [String] {
        Array(hashes.prefix(transactions.count))
    }

    func load(input _: GemConfirmInput, options _: GemConfirmLoadOptions) async throws -> GemConfirmData {
        throw AnyError("not supported")
    }
}
