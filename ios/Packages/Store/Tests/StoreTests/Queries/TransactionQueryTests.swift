// Copyright (c). Gem Wallet. All rights reserved.

import GRDB
import Observation
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing
import XCTest

@MainActor
struct TransactionQueryTests {
    @Test(arguments: [false, true])
    func observationSurvivesHashUpdate(existingTarget: Bool) async throws {
        let db = DB.mock(wallets: [.mock(accounts: [.mock(chain: .ethereum)])], assets: [.mock(asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18))])
        let store = TransactionStore(db: db)
        let walletId = WalletId.mock()
        let pendingId = TransactionId(chain: .ethereum, hash: "pending")
        let confirmedId = TransactionId(chain: .ethereum, hash: "confirmed")
        let pending = Transaction.mock(id: pendingId, assetId: Chain.ethereum.assetId, state: .pending, fee: "1", feeAssetId: Chain.ethereum.assetId)
        let confirmed = Transaction.mock(id: confirmedId, assetId: Chain.ethereum.assetId, state: .confirmed, fee: "42", feeAssetId: Chain.ethereum.assetId)
        try store.addTransactions(walletId: walletId, transactions: [.mock(pending)])
        if existingTarget {
            try store.addTransactions(walletId: walletId, transactions: [.mock(confirmed)])
        }
        let stored = try store.getTransaction(walletId: walletId, transactionId: pendingId)
        let query = ObservableQuery(TransactionQuery(walletId: walletId, recordId: stored.recordId), initialValue: stored)
        query.bind(dbQueue: db.dbQueue)
        #expect(query.value.transaction.id == pendingId)

        try await expectUpdate(query) {
            try store.updateTransactionHash(walletId: walletId, transactionId: pendingId, hash: confirmedId.hash)
        }
        #expect(query.value.recordId == stored.recordId)
        #expect(query.value.transaction.id == confirmedId)
        #expect(query.value.transaction.state == (existingTarget ? .confirmed : .pending))
        #expect(query.value.transaction.fee == (existingTarget ? "42" : "1"))
        #expect(throws: RecordError.self) {
            try store.getTransaction(walletId: walletId, transactionId: pendingId)
        }

        let updated = Transaction.mock(id: confirmedId, assetId: Chain.ethereum.assetId, state: .confirmed, fee: "84", feeAssetId: Chain.ethereum.assetId)
        try await expectUpdate(query) {
            try store.addTransactions(walletId: walletId, transactions: [.mock(updated)])
        }
        #expect(query.value.recordId == stored.recordId)
        #expect(query.value.transaction.state == .confirmed)
        #expect(query.value.transaction.fee == "84")
        let count = try await db.dbQueue.read { try TransactionRecord.fetchCount($0) }
        #expect(count == 1)

        try store.deleteTransaction(walletId: walletId, transactionId: confirmedId)
        #expect(throws: RecordError.self) {
            try store.getTransaction(walletId: walletId, transactionId: confirmedId)
        }
    }

    @Test
    func recordIdentitySurvivesHashUpdateBeforeObservationStarts() throws {
        let db = DB.mock(wallets: [.mock(accounts: [.mock(chain: .ethereum)])], assets: [.mock(asset: .mock(id: .mock(chain: .ethereum), name: "Ethereum", symbol: "ETH", decimals: 18))])
        let store = TransactionStore(db: db)
        let oldId = TransactionId(chain: .ethereum, hash: "pending")
        let newId = TransactionId(chain: .ethereum, hash: "confirmed")
        try store.addTransactions(walletId: .mock(), transactions: [.mock(.mock(id: oldId, assetId: Chain.ethereum.assetId, feeAssetId: Chain.ethereum.assetId))])
        let stored = try store.getTransaction(walletId: .mock(), transactionId: oldId)
        let query = ObservableQuery(TransactionQuery(walletId: .mock(), recordId: stored.recordId), initialValue: stored)

        try store.updateTransactionHash(walletId: .mock(), transactionId: oldId, hash: newId.hash)

        query.bind(dbQueue: db.dbQueue)
        #expect(query.value.recordId == stored.recordId)
        #expect(query.value.transaction.id == newId)
        #expect(throws: RecordError.self) {
            try store.getTransaction(walletId: .mock(), transactionId: oldId)
        }
    }

    private func expectUpdate(_ query: ObservableQuery<TransactionQuery>, update: () throws -> Void) async throws {
        let expectation = XCTestExpectation(description: "Transaction observation updated")
        withObservationTracking {
            _ = query.value
        } onChange: {
            expectation.fulfill()
        }
        try update()
        let result = await XCTWaiter.fulfillment(of: [expectation], timeout: 3)
        #expect(result == .completed)
    }
}
