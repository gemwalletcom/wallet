// Copyright (c). Gem Wallet. All rights reserved.

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

    func makeFixture(type: TransactionType = .swap, update: @escaping Update) throws -> Fixture {
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

        let service = TransactionStateService(
            service: GemTransactionStateServiceMock(store: GemstoneTransactionStateStore(store: store)) { walletId, transaction in
                try await update(store, WalletId.from(id: walletId), Transaction(transaction))
            },
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
