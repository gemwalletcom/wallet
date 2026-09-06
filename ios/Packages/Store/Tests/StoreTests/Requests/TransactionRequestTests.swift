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
struct TransactionRequestTests {
    @Test(arguments: [false, true])
    func observationSurvivesHashUpdate(existingTarget: Bool) async throws {
        let db = DB.mockAssets(assets: [.mock(asset: .mockEthereum())])
        let store = TransactionStore(db: db)
        let walletId = WalletId.mock()
        let pendingId = TransactionId(chain: .ethereum, hash: "pending")
        let confirmedId = TransactionId(chain: .ethereum, hash: "confirmed")
        let pending = Transaction.mock(transactionId: pendingId, state: .pending, assetId: Chain.ethereum.assetId)
        let confirmed = Transaction.mock(transactionId: confirmedId, state: .confirmed, assetId: Chain.ethereum.assetId, fee: "42")
        try store.addTransactions(walletId: walletId, transactions: [pending])
        if existingTarget {
            try store.addTransactions(walletId: walletId, transactions: [confirmed])
        }
        let stored = try store.getTransaction(walletId: walletId, transactionId: pendingId)
        let query = ObservableQuery(TransactionRequest(walletId: walletId, recordId: stored.recordId), initialValue: stored)
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

        let updated = Transaction.mock(transactionId: confirmedId, state: .confirmed, assetId: Chain.ethereum.assetId, fee: "84")
        try await expectUpdate(query) {
            try store.addTransactions(walletId: walletId, transactions: [updated])
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
        let db = DB.mockAssets(assets: [.mock(asset: .mockEthereum())])
        let store = TransactionStore(db: db)
        let oldId = TransactionId(chain: .ethereum, hash: "pending")
        let newId = TransactionId(chain: .ethereum, hash: "confirmed")
        try store.addTransactions(walletId: .mock(), transactions: [.mock(transactionId: oldId, assetId: Chain.ethereum.assetId)])
        let stored = try store.getTransaction(walletId: .mock(), transactionId: oldId)
        let query = ObservableQuery(TransactionRequest(walletId: .mock(), recordId: stored.recordId), initialValue: stored)

        try store.updateTransactionHash(walletId: .mock(), transactionId: oldId, hash: newId.hash)

        query.bind(dbQueue: db.dbQueue)
        #expect(query.value.recordId == stored.recordId)
        #expect(query.value.transaction.id == newId)
        #expect(throws: RecordError.self) {
            try store.getTransaction(walletId: .mock(), transactionId: oldId)
        }
    }

    @Test
    func missingTransactionThrows() throws {
        let db = DB.mockAssets()
        let request = TransactionRequest(walletId: .mock(), recordId: 1)
        try db.dbQueue.read { db in
            #expect(throws: RecordError.self) { try request.fetch(db) }
        }
    }

    private func expectUpdate(_ query: ObservableQuery<TransactionRequest>, update: () throws -> Void) async throws {
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
