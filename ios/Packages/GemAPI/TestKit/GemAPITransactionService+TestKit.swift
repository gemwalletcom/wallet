// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives

public final class GemAPITransactionServiceMock: GemAPITransactionService, @unchecked Sendable {
    private var walletTransactionsResponse: TransactionsResponse
    private var assetTransactionsResponse: TransactionsResponse

    public init(
        walletTransactionsResponse: TransactionsResponse = TransactionsResponse(transactions: [], addressNames: []),
        assetTransactionsResponse: TransactionsResponse = TransactionsResponse(transactions: [], addressNames: []),
    ) {
        self.walletTransactionsResponse = walletTransactionsResponse
        self.assetTransactionsResponse = assetTransactionsResponse
    }

    public func getDeviceTransactions(walletId _: String, fromTimestamp _: Int) async throws -> TransactionsResponse {
        walletTransactionsResponse
    }

    public func getDeviceTransactionsForAsset(walletId _: String, asset _: AssetId, fromTimestamp _: Int) async throws -> TransactionsResponse {
        assetTransactionsResponse
    }

    public func setWalletTransactionsResponse(_ response: TransactionsResponse) {
        walletTransactionsResponse = response
    }

    public func setAssetTransactionsResponse(_ response: TransactionsResponse) {
        assetTransactionsResponse = response
    }
}
