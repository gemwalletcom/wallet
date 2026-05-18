// Copyright (c). Gem Wallet. All rights reserved.

import BalanceServiceTestKit
import EarnService
import Foundation
import NFTServiceTestKit
import Primitives
import PrimitivesTestKit
import StakeServiceTestKit
import Store
import StoreTestKit
import Testing
@testable import TransactionStateService
import TransactionStateServiceTestKit

struct TransactionStateServiceTests {
    @Test
    func inTransitSavesMetadataAndRetries() async throws {
        let finalMetadata = try #require(AnyCodableValue.encode(TransactionSwapMetadata(
            fromAsset: .mock(.bitcoin),
            fromValue: "100000000",
            toAsset: .mock(.ethereum),
            toValue: "9900000000000000000",
            provider: SwapProvider.thorchain.rawValue,
        )))
        let fixture = try makeFixture(stateChanges: TransactionChanges(
            state: .inTransit,
            changes: [.metadata(finalMetadata)],
        ))

        let status = await fixture.service.update(for: fixture.transaction).status

        expectRetry(status)
        let saved = try #require(fixture.store.getTransactions(state: .inTransit).first)
        let savedMetadata = try #require(saved.metadata?.decode(TransactionSwapMetadata.self))
        #expect(savedMetadata.toValue == "9900000000000000000")
    }

    @Test
    func terminalStatesSaveAndComplete() async throws {
        for state in [TransactionState.confirmed, .failed, .reverted] {
            let fixture = try makeFixture(stateChanges: TransactionChanges(state: state))

            let status = await fixture.service.update(for: fixture.transaction).status

            expectComplete(status)
            #expect(try fixture.store.getTransactions(state: state).count == 1)
        }
    }

    @Test
    func hashChangeWritesChangesToRenamedTransaction() async throws {
        let finalMetadata = try #require(AnyCodableValue.encode(TransactionSwapMetadata(
            fromAsset: .mock(.bitcoin),
            fromValue: "100000000",
            toAsset: .mock(.ethereum),
            toValue: "9900000000000000000",
            provider: SwapProvider.thorchain.rawValue,
        )))
        let fixture = try makeFixture(stateChanges: TransactionChanges(
            state: .inTransit,
            changes: [
                .metadata(finalMetadata),
                .hashChange(old: "hash", new: "new-hash"),
            ],
        ))

        let status = await fixture.service.update(for: fixture.transaction).status

        expectRetry(status)
        let saved = try #require(fixture.store.getTransactions(state: .inTransit).first)
        let savedMetadata = try #require(saved.metadata?.decode(TransactionSwapMetadata.self))
        #expect(saved.id.hash == "new-hash")
        #expect(savedMetadata.toValue == "9900000000000000000")
    }

    @Test
    func hashChangeUpdatesExistingTransaction() async throws {
        let finalMetadata = try #require(AnyCodableValue.encode(TransactionSwapMetadata(
            fromAsset: .mock(.bitcoin),
            fromValue: "100000000",
            toAsset: .mock(.ethereum),
            toValue: "9900000000000000000",
            provider: SwapProvider.thorchain.rawValue,
        )))
        let fixture = try makeFixture(stateChanges: TransactionChanges(
            state: .inTransit,
            changes: [
                .hashChange(old: "hash", new: "new-hash"),
                .metadata(finalMetadata),
            ],
        ))
        let existingTransaction = try makeSwapTransaction(
            hash: "new-hash",
            fromAsset: .mock(.bitcoin),
            toAsset: .mock(.ethereum),
            state: .pending,
        )
        try fixture.store.addTransactions(walletId: fixture.walletId, transactions: [existingTransaction])

        let status = await fixture.service.update(for: fixture.transaction).status

        expectRetry(status)
        #expect(try fixture.store.getTransactions(state: .pending).isEmpty)
        let saved = try #require(fixture.store.getTransactions(state: .inTransit).first)
        let savedMetadata = try #require(saved.metadata?.decode(TransactionSwapMetadata.self))
        #expect(saved.id.hash == "new-hash")
        #expect(savedMetadata.toValue == "9900000000000000000")
    }

    @Test
    func hashChangeDoesNotDowngradeCompletedTransaction() async throws {
        let fixture = try makeFixture(stateChanges: TransactionChanges(
            state: .inTransit,
            changes: [
                .hashChange(old: "hash", new: "new-hash"),
            ],
        ))
        let existingTransaction = try makeSwapTransaction(
            hash: "new-hash",
            fromAsset: .mock(.bitcoin),
            toAsset: .mock(.ethereum),
            state: .confirmed,
        )
        try fixture.store.addTransactions(walletId: fixture.walletId, transactions: [existingTransaction])

        let status = await fixture.service.update(for: fixture.transaction).status

        expectComplete(status)
        #expect(try fixture.store.getTransactions(state: .pending).isEmpty)
        let saved = try #require(fixture.store.getTransactions(state: .confirmed).first)
        #expect(saved.id.hash == "new-hash")
    }

    @Test
    func inTransitDoesNotDowngradeToPending() async throws {
        let fixture = try makeFixture(
            state: .inTransit,
            statusService: TransactionStatusServiceMock(stateChanges: TransactionChanges(state: .pending)),
        )

        let status = await fixture.service.update(for: fixture.transaction).status

        expectRetry(status)
        #expect(try fixture.store.getTransactions(state: .pending).isEmpty)
        #expect(try fixture.store.getTransactions(state: .inTransit).count == 1)
    }

    @Test
    func inTransitSwapUsesSwapStatusProvider() async throws {
        let statusService = TransactionStatusServiceMock(
            swapStatus: { _ in TransactionChanges(state: .confirmed) },
        )
        let fixture = try makeFixture(
            state: .inTransit,
            statusService: statusService,
        )

        let status = await fixture.service.update(for: fixture.transaction).status

        expectComplete(status)
        #expect(await statusService.regularRequestCount() == 0)
        let swapRequests = await statusService.swapRequests()
        #expect(swapRequests.count == 1)
        #expect(swapRequests.first?.state == .inTransit)
    }

    @Test
    func inTransitSwapWithoutProviderDoesNotUseChainStatusProvider() async throws {
        let statusService = TransactionStatusServiceMock(
            regularStatus: { _ in TransactionChanges(state: .confirmed) },
            swapStatus: { _ in TransactionChanges(state: .confirmed) },
        )
        let fixture = try makeFixture(
            state: .inTransit,
            provider: nil,
            statusService: statusService,
        )

        let status = await fixture.service.update(for: fixture.transaction).status

        expectRetry(status)
        #expect(await statusService.regularRequestCount() == 0)
        let swapRequests = await statusService.swapRequests()
        #expect(swapRequests.isEmpty)
        #expect(try fixture.store.getTransactions(state: .inTransit).count == 1)
    }

    @Test
    func jobRefreshesTransactionAfterHashChange() async throws {
        let statusService = TransactionStatusServiceMock(
            swapStatus: { request in
                if request.transaction.id == "hash" {
                    return TransactionChanges(
                        state: .inTransit,
                        changes: [.hashChange(old: "hash", new: "new-hash")],
                    )
                }
                return TransactionChanges(state: .confirmed)
            },
        )
        let fixture = try makeFixture(statusService: statusService)
        let job = TransactionStateJob(
            wallet: TransactionWallet(transaction: fixture.transaction, wallet: fixture.wallet),
            service: fixture.service,
        )

        await expectRetry(job.run())
        await expectComplete(job.run())

        #expect(await statusService.regularRequestCount() == 0)
        let swapRequests = await statusService.swapRequests()
        #expect(swapRequests.map(\.transaction.id) == ["hash", "new-hash"])
        #expect(swapRequests.map(\.state) == [TransactionState.pending, .inTransit])
    }
}

// MARK: - Private

private extension TransactionStateServiceTests {
    struct Fixture {
        let store: TransactionStore
        let walletId: WalletId
        let wallet: Wallet
        let transaction: Transaction
        let service: TransactionStateService
    }

    func makeFixture(stateChanges: TransactionChanges) throws -> Fixture {
        try makeFixture(statusService: TransactionStatusServiceMock(stateChanges: stateChanges))
    }

    func makeFixture(
        state: TransactionState = .pending,
        provider: SwapProvider? = .thorchain,
        statusService: any TransactionStatusServiceable,
    ) throws -> Fixture {
        let fromAsset = AssetId.mock(.bitcoin)
        let toAsset = AssetId.mock(.ethereum)
        let db = DB.mockAssets(assets: [
            .mock(asset: .mock(id: fromAsset)),
            .mock(asset: .mockEthereum()),
        ])
        let store = TransactionStore.mock(db: db)
        let wallet = Wallet.mock()
        let walletId = wallet.id
        let transaction = try makeSwapTransaction(
            fromAsset: fromAsset,
            toAsset: toAsset,
            state: state,
            provider: provider,
        )
        try store.addTransactions(walletId: walletId, transactions: [transaction])

        let postProcessingService = TransactionPostProcessingService(
            transactionStore: store,
            balanceUpdater: BalanceUpdaterMock(),
            stakeService: .mock(),
            earnService: .mock(),
            nftService: .mock(),
        )
        let service = TransactionStateService(
            transactionStore: store,
            postProcessingService: postProcessingService,
            statusService: statusService,
        )
        return Fixture(store: store, walletId: walletId, wallet: wallet, transaction: transaction, service: service)
    }

    func makeSwapTransaction(
        hash: String = "hash",
        fromAsset: AssetId,
        toAsset: AssetId,
        state: TransactionState,
        provider: SwapProvider? = .thorchain,
    ) throws -> Transaction {
        let metadata = try #require(AnyCodableValue.encode(TransactionSwapMetadata(
            fromAsset: fromAsset,
            fromValue: "100000000",
            toAsset: toAsset,
            toValue: "10000000000000000000",
            provider: provider?.rawValue,
        )))
        return Transaction(
            id: TransactionId(chain: fromAsset.chain, hash: hash),
            assetId: fromAsset,
            from: "sender",
            to: "recipient",
            contract: nil,
            type: .swap,
            state: state,
            blockNumber: "7",
            sequence: "1",
            fee: "1",
            feeAssetId: fromAsset,
            value: "100000000",
            memo: nil,
            direction: .outgoing,
            utxoInputs: [],
            utxoOutputs: [],
            metadata: metadata,
            createdAt: Date(timeIntervalSince1970: 1234),
        )
    }

    func expectRetry(_ status: JobStatus) {
        guard case .retry = status else {
            Issue.record("Expected retry")
            return
        }
    }

    func expectComplete(_ status: JobStatus) {
        guard case .complete = status else {
            Issue.record("Expected complete")
            return
        }
    }
}

private actor TransactionStatusServiceMock: TransactionStatusServiceable {
    private let regularStatus: @Sendable (TransactionStateRequest) -> TransactionChanges
    private let swapStatus: @Sendable (TransactionSwapStateRequest) -> TransactionChanges
    private var regularRequests: [TransactionStateRequest] = []
    private var recordedSwapRequests: [TransactionSwapStateRequest] = []

    init(stateChanges: TransactionChanges) {
        regularStatus = { _ in stateChanges }
        swapStatus = { _ in stateChanges }
    }

    init(
        regularStatus: @escaping @Sendable (TransactionStateRequest) -> TransactionChanges = { _ in TransactionChanges(state: .pending) },
        swapStatus: @escaping @Sendable (TransactionSwapStateRequest) -> TransactionChanges,
    ) {
        self.regularStatus = regularStatus
        self.swapStatus = swapStatus
    }

    func transactionStatus(chain _: Primitives.Chain, request: TransactionStateRequest) async throws -> TransactionChanges {
        regularRequests.append(request)
        return regularStatus(request)
    }

    func transactionSwapStatus(chain _: Primitives.Chain, request: TransactionSwapStateRequest) async throws -> TransactionChanges {
        recordedSwapRequests.append(request)
        return swapStatus(request)
    }

    func regularRequestCount() -> Int {
        regularRequests.count
    }

    func swapRequests() -> [TransactionSwapStateRequest] {
        recordedSwapRequests
    }
}
