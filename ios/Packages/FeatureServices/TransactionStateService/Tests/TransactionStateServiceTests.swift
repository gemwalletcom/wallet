// Copyright (c). Gem Wallet. All rights reserved.

import BalanceServiceTestKit
import EarnService
import EarnServiceTestKit
import Foundation
import GemAPITestKit
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
        let saved = try #require(fixture.store.getTransactions(states: [.inTransit]).first)
        let savedMetadata = try #require(saved.metadata?.decode(TransactionSwapMetadata.self))
        #expect(savedMetadata.toValue == "9900000000000000000")
    }

    @Test
    func terminalStatesSaveAndComplete() async throws {
        for state in [TransactionState.confirmed, .failed, .reverted] {
            let fixture = try makeFixture(stateChanges: TransactionChanges(state: state))

            let status = await fixture.service.update(for: fixture.transaction).status

            expectComplete(status)
            #expect(try fixture.store.getTransactions(states: [state]).count == 1)
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
        let saved = try #require(fixture.store.getTransactions(states: [.inTransit]).first)
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
        #expect(try fixture.store.getTransactions(states: [.pending]).isEmpty)
        let saved = try #require(fixture.store.getTransactions(states: [.inTransit]).first)
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
        #expect(try fixture.store.getTransactions(states: [.pending]).isEmpty)
        let saved = try #require(fixture.store.getTransactions(states: [.confirmed]).first)
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
        #expect(try fixture.store.getTransactions(states: [.pending]).isEmpty)
        #expect(try fixture.store.getTransactions(states: [.inTransit]).count == 1)
    }

    @Test
    func jobRefreshesTransactionAfterHashChange() async throws {
        let statusService = TransactionStatusServiceMock(
            update: { transaction in
                if transaction.id.hash == "hash" {
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

        // The job re-reads the transaction after the hash change, so the second
        // lookup goes out against the new hash.
        #expect(await statusService.requestedHashes() == ["hash", "new-hash"])
    }

    @Test
    func jobRefreshesBalancesWhenEnteringInTransit() async throws {
        let sourceAsset = AssetId.mock(.bitcoin)
        try await confirmation("updates balances once") { updatedBalances in
            let fixture = try makeFixture(
                type: .transfer,
                statusService: TransactionStatusServiceMock(stateChanges: TransactionChanges(state: .inTransit)),
                balanceUpdater: BalanceUpdaterMock { _, assetIds in
                    #expect(assetIds == [sourceAsset])
                    updatedBalances()
                },
            )
            let job = TransactionStateJob(
                wallet: TransactionWallet(transaction: fixture.transaction, wallet: fixture.wallet),
                service: fixture.service,
            )

            await expectRetry(job.run())
            await expectRetry(job.run())
        }
    }

    @Test
    func jobStopsWhenWalletRemoved() async throws {
        let fixture = try makeFixture(stateChanges: TransactionChanges(state: .confirmed))
        let job = TransactionStateJob(
            wallet: TransactionWallet(transaction: fixture.transaction, wallet: fixture.wallet),
            service: fixture.service,
        )
        _ = try WalletStore.mock(db: fixture.db).deleteWallet(for: fixture.walletId)

        await expectCancelled(job.run())

        #expect(try fixture.store.getTransactions(states: [.confirmed]).isEmpty)
    }

    @Test
    func postProcessingRefreshesSwapBalances() async throws {
        let fromAsset = AssetId.mock(.bitcoin)
        let toAsset = AssetId.mock(.ethereum)
        let wallet = Wallet.mock()
        let transaction = try makeSwapTransaction(
            fromAsset: fromAsset,
            toAsset: toAsset,
            state: .confirmed,
        )
        try await confirmation("updates balances") { updatedBalances in
            let postProcessingService = TransactionPostProcessingService(
                transactionStore: .mock(),
                balanceUpdater: BalanceUpdaterMock { updatedWallet, assetIds in
                    #expect(updatedWallet.id == wallet.id)
                    #expect(assetIds == [fromAsset, toAsset])
                    updatedBalances()
                },
                stakeService: .mock(),
                earnService: .mock(),
                nftService: .mock(),
            )

            try await postProcessingService.process(wallet: wallet, transaction: transaction)
        }
    }

    @Test
    func postProcessingRefreshesNftsAfterTransfer() async throws {
        let db = DB.mockWithChains([.ethereum])
        let walletStore = WalletStore.mock(db: db)
        let nftStore = NFTStore.mock(db: db)
        let wallet = Wallet.mock(id: .mock(), accounts: [.mock(chain: .ethereum)])
        try walletStore.addWallet(wallet)

        let collectionId = NFTCollectionId(chain: .ethereum, contractAddress: "0xcollection")
        let assetId = NFTAssetId(chain: .ethereum, contractAddress: "0xcollection", tokenId: "1")
        let nftData = NFTData(
            collection: .mock(id: collectionId, chain: .ethereum),
            assets: [.mock(id: assetId, collectionId: collectionId, chain: .ethereum)],
        )
        let postProcessingService = TransactionPostProcessingService(
            transactionStore: .mock(),
            balanceUpdater: BalanceUpdaterMock(),
            stakeService: .mock(),
            earnService: .mock(),
            nftService: .mock(
                apiService: GemAPINFTServiceMock(nftAssets: [nftData]),
                nftStore: nftStore,
            ),
        )

        try await postProcessingService.process(
            wallet: wallet,
            transaction: .mock(type: .transferNFT, state: .confirmed, assetId: .mock(.ethereum)),
        )

        let savedNFTs = try fetchNFTs(db: db, walletId: wallet.id)
        #expect(savedNFTs.map(\.collection.id) == [collectionId])
        #expect(savedNFTs.flatMap(\.assets).map(\.id) == [assetId])
    }
}

// MARK: - Private

private extension TransactionStateServiceTests {
    struct Fixture {
        let db: DB
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
        type: TransactionType = .swap,
        provider: SwapProvider? = .thorchain,
        statusService: any TransactionStatusServiceable,
        balanceUpdater: BalanceUpdaterMock = .init(),
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
        let transaction = if type == .swap {
            try makeSwapTransaction(
                fromAsset: fromAsset,
                toAsset: toAsset,
                state: state,
                provider: provider,
            )
        } else {
            Transaction.mock(
                transactionId: TransactionId(chain: fromAsset.chain, hash: "hash"),
                type: type,
                state: state,
                assetId: fromAsset,
            )
        }
        try store.addTransactions(walletId: walletId, transactions: [transaction])

        let postProcessingService = TransactionPostProcessingService(
            transactionStore: store,
            balanceUpdater: balanceUpdater,
            stakeService: .mock(),
            earnService: .mock(),
            nftService: .mock(),
        )
        let service = TransactionStateService(
            transactionStore: store,
            postProcessingService: postProcessingService,
            statusService: statusService,
        )
        return Fixture(db: db, store: store, walletId: walletId, wallet: wallet, transaction: transaction, service: service)
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
        return Transaction.mock(
            transactionId: TransactionId(chain: fromAsset.chain, hash: hash),
            type: .swap,
            state: state,
            assetId: fromAsset,
            metadata: metadata,
        )
    }

    func fetchNFTs(db: DB, walletId: WalletId) throws -> [NFTData] {
        try db.dbQueue.read { database in
            try NFTRequest(walletId: walletId, filter: .all).fetch(database)
        }
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

    func expectCancelled(_ status: JobStatus) {
        guard case .cancelled = status else {
            Issue.record("Expected cancelled")
            return
        }
    }
}

private actor TransactionStatusServiceMock: TransactionStatusServiceable {
    private let update: @Sendable (Transaction) -> TransactionChanges
    private var requests: [Transaction] = []

    init(stateChanges: TransactionChanges) {
        update = { _ in stateChanges }
    }

    init(update: @escaping @Sendable (Transaction) -> TransactionChanges) {
        self.update = update
    }

    func transactionUpdate(_ transaction: Transaction) async throws -> TransactionChanges {
        requests.append(transaction)
        return update(transaction)
    }

    func requestedHashes() -> [String] {
        requests.map(\.id.hash)
    }
}
