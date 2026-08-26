// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemNftService
import GemstoneServicesTestKit
import Foundation
import struct Gemstone.GemTransactionStateResult
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing
@testable import GemstoneServices

struct TransactionStateServiceTests {
    @Test
    func jobRefreshesTransactionAfterHashChange() async throws {
        let requests = RequestedHashes()
        let fixture = try makeFixture { store, walletId, transaction in
            await requests.record(transaction.id.hash)
            if transaction.id.hash == "hash" {
                let newId = TransactionId(chain: transaction.assetId.chain, hash: "new-hash")
                try store.renameTransaction(walletId: walletId, transactionId: transaction.id, newTransactionId: newId)
                return try GemTransactionStateResult(transactionId: newId.json(), state: TransactionState.inTransit.json())
            }
            _ = try store.updateTransaction(walletId: walletId, transactionId: transaction.id, state: .confirmed, fee: nil, blockNumber: nil, metadata: nil, confirmationEtaSeconds: nil)
            return try GemTransactionStateResult(transactionId: transaction.id.json(), state: TransactionState.confirmed.json())
        }
        let job = TransactionStateJob(
            wallet: TransactionWallet(transaction: fixture.transaction, wallet: fixture.wallet),
            service: fixture.service,
        )

        await expectRetry(job.run())
        await expectComplete(job.run())

        #expect(await requests.hashes == ["hash", "new-hash"])
    }

    @Test
    func jobRefreshesBalancesWhenEnteringInTransit() async throws {
        let sourceAsset = AssetId.mock(.bitcoin)
        try await confirmation("updates balances once") { updatedBalances in
            let fixture = try makeFixture(
                type: .transfer,
                balanceService: GemBalanceServiceMock { _, assetIds in
                    #expect(assetIds == [sourceAsset.identifier])
                    updatedBalances()
                },
            ) { store, walletId, transaction in
                _ = try store.updateTransaction(walletId: walletId, transactionId: transaction.id, state: .inTransit, fee: nil, blockNumber: nil, metadata: nil, confirmationEtaSeconds: nil)
                return try GemTransactionStateResult(transactionId: transaction.id.json(), state: TransactionState.inTransit.json())
            }
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
        let fixture = try makeFixture { _, _, transaction in
            try GemTransactionStateResult(transactionId: transaction.id.json(), state: TransactionState.confirmed.json())
        }
        let job = TransactionStateJob(
            wallet: TransactionWallet(transaction: fixture.transaction, wallet: fixture.wallet),
            service: fixture.service,
        )
        _ = try WalletStore.mock(db: fixture.db).deleteWallet(for: fixture.walletId)

        await expectCancelled(job.run())
    }

    @Test
    func updateStopsWhenTransactionRowIsGone() async throws {
        let fixture = try makeFixture { _, _, _ in nil }

        let result = await fixture.service.update(walletId: fixture.walletId, transaction: fixture.transaction)

        expectCancelled(result.status)
    }

    @Test
    func updateRetriesOnServiceFailure() async throws {
        let fixture = try makeFixture { _, _, _ in throw AnyError("offline") }

        let result = await fixture.service.update(walletId: fixture.walletId, transaction: fixture.transaction)

        expectRetry(result.status)
        #expect(result.transactionId == fixture.transaction.id)
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
                balanceService: GemBalanceServiceMock { walletId, assetIds in
                    #expect(walletId == wallet.id.id)
                    #expect(assetIds == [fromAsset.identifier, toAsset.identifier])
                    updatedBalances()
                },
                stakeService: GemStakeServiceMock(),
                nftService: GemNftService.mock(),
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
            balanceService: GemBalanceServiceMock(),
            stakeService: GemStakeServiceMock(),
            nftService: GemNftServiceMock(assets: [nftData], store: GemstoneNftStore(store: nftStore)),
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
    typealias Update = @Sendable (TransactionStore, WalletId, Transaction) async throws -> GemTransactionStateResult?

    struct Fixture {
        let db: DB
        let store: TransactionStore
        let walletId: WalletId
        let wallet: Wallet
        let transaction: Transaction
        let service: TransactionStateService
    }

    func makeFixture(
        type: TransactionType = .swap,
        balanceService: GemBalanceServiceMock = GemBalanceServiceMock(),
        update: @escaping Update,
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
            try makeSwapTransaction(fromAsset: fromAsset, toAsset: toAsset, state: .pending)
        } else {
            Transaction.mock(
                transactionId: TransactionId(chain: fromAsset.chain, hash: "hash"),
                type: type,
                state: .pending,
                assetId: fromAsset,
            )
        }
        try store.addTransactions(walletId: walletId, transactions: [transaction])

        let postProcessingService = TransactionPostProcessingService(
            transactionStore: store,
            balanceService: balanceService,
            stakeService: GemStakeServiceMock(),
            nftService: GemNftService.mock(),
        )
        let service = TransactionStateService(
            transactionStore: store,
            service: GemTransactionStateServiceMock { walletId, transaction in
                try await update(store, WalletId.from(id: walletId), Transaction(transaction))
            },
            postProcessingService: postProcessingService,
        )
        return Fixture(db: db, store: store, walletId: walletId, wallet: wallet, transaction: transaction, service: service)
    }

    func makeSwapTransaction(
        hash: String = "hash",
        fromAsset: AssetId,
        toAsset: AssetId,
        state: TransactionState,
    ) throws -> Transaction {
        let metadata = try #require(AnyCodableValue.encode(TransactionSwapMetadata(
            fromAsset: fromAsset,
            fromValue: "100000000",
            toAsset: toAsset,
            toValue: "10000000000000000000",
            provider: SwapProvider.thorchain.rawValue,
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

private actor RequestedHashes {
    private(set) var hashes: [String] = []

    func record(_ hash: String) {
        hashes.append(hash)
    }
}
