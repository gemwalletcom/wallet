// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemConfirmError
import struct Gemstone.GemConfirmData
import struct Gemstone.GemConfirmInput
import struct Gemstone.GemConfirmLoadOptions
import protocol Gemstone.GemConfirmServiceProtocol
import struct Gemstone.GemSendInput
import struct Gemstone.GemSendResult
import struct Gemstone.GemSignedTransaction
import enum Gemstone.GemTransactionInputType
import typealias Gemstone.Transaction
import Foundation
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct TransferExecutorTests {
    @Test
    func sendReportsEveryHashAndTracksReturnedTransactions() async throws {
        let tracked = Primitives.Transaction.mock()
        let confirmService = GemConfirmServiceMock(result: .success(GemSendResult(hashes: ["hash-1", "hash-2"], transactions: [try tracked.json()])))
        let executor = TransferExecutor(
            signer: TransactionSignerMock(signedTransactions: [GemSignedTransaction(data: "signed", type: .transfer)]),
            confirmService: confirmService,
            preferencesService: GemPreferencesServiceMock(),
            transactionStateScheduler: .mock(),
        )
        let reported = ReportedHashes()

        try await executor.execute(input: TransferConfirmationInput(
            data: .mock(),
            wallet: .mock(accounts: [Account.mock(chain: .ethereum)]),
            transactionData: .mock(),
            amount: .mock(),
            delegate: { result in reported.append(try? result.get()) },
        ))

        #expect(reported.values == ["hash-1", "hash-2"])
        #expect(confirmService.sentTransactions.map(\.data) == ["signed"])
    }

    @Test
    func partialBroadcastReportsBroadcastHashesAndRethrows() async throws {
        let confirmService = GemConfirmServiceMock(result: .failure(GemConfirmError.Broadcast(hashes: ["hash-1"], msg: "second leg failed")))
        let executor = TransferExecutor(
            signer: TransactionSignerMock(signedTransactions: [
                GemSignedTransaction(data: "first", type: .transfer),
                GemSignedTransaction(data: "second", type: .transfer),
            ]),
            confirmService: confirmService,
            preferencesService: GemPreferencesServiceMock(),
            transactionStateScheduler: .mock(),
        )
        let reported = ReportedHashes()

        await #expect(throws: (any Error).self) {
            try await executor.execute(input: TransferConfirmationInput(
                data: .mock(),
                wallet: .mock(accounts: [Account.mock(chain: .ethereum)]),
                transactionData: .mock(),
                amount: .mock(),
                delegate: { result in reported.append(try? result.get()) },
            ))
        }

        #expect(reported.values == ["hash-1"])
    }
}

private final class ReportedHashes: @unchecked Sendable {
    private let lock = NSLock()
    private var storage: [String] = []

    var values: [String] { lock.withLock { storage } }

    func append(_ hash: String?) {
        guard let hash else { return }
        lock.withLock { storage.append(hash) }
    }
}

private final class GemConfirmServiceMock: GemConfirmServiceProtocol, @unchecked Sendable {
    private let result: Result<GemSendResult, any Error>
    private(set) var sentTransactions: [GemSignedTransaction] = []

    init(result: Result<GemSendResult, any Error>) {
        self.result = result
    }

    func load(input _: GemConfirmInput, options _: GemConfirmLoadOptions) async throws -> GemConfirmData {
        fatalError("not used")
    }

    func broadcast(inputType _: GemTransactionInputType, transactions _: [GemSignedTransaction]) async throws -> [String] {
        fatalError("not used")
    }

    func send(input _: GemSendInput, transactions: [GemSignedTransaction]) async throws -> GemSendResult {
        sentTransactions = transactions
        return try result.get()
    }
}
