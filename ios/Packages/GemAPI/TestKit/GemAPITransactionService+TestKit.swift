// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives

public final class GemAPITransactionServiceMock: GemAPITransactionService, @unchecked Sendable {
    public init() {}

    public func getDeviceTransactions(walletId _: String, fromTimestamp _: Int) async throws -> TransactionsResponse {
        TransactionsResponse(transactions: [], addressNames: [])
    }

    public func getDeviceTransactionsForAsset(walletId _: String, asset _: AssetId, fromTimestamp _: Int) async throws -> TransactionsResponse {
        TransactionsResponse(transactions: [], addressNames: [])
    }
}
