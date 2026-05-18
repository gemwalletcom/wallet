// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives

public final class GemAPITransactionServiceMock: GemAPITransactionService, @unchecked Sendable {
    private var walletTransactionsResponse: TransactionsResponse
    private var assetTransactionsResponse: TransactionsResponse
    private var transactionResponse: Transaction?

    public init(
        walletTransactionsResponse: TransactionsResponse = TransactionsResponse(transactions: [], addressNames: []),
        assetTransactionsResponse: TransactionsResponse = TransactionsResponse(transactions: [], addressNames: []),
        transactionResponse: Transaction? = nil,
    ) {
        self.walletTransactionsResponse = walletTransactionsResponse
        self.assetTransactionsResponse = assetTransactionsResponse
        self.transactionResponse = transactionResponse
    }

    public func getDeviceTransactions(walletId _: WalletId, fromTimestamp _: Int) async throws -> TransactionsResponse {
        walletTransactionsResponse
    }

    public func getDeviceTransactionsForAsset(walletId _: WalletId, asset _: AssetId, fromTimestamp _: Int) async throws -> TransactionsResponse {
        assetTransactionsResponse
    }

    public func getDeviceTransaction(transactionId _: TransactionId) async throws -> Transaction {
        guard let transactionResponse else {
            throw NSError(domain: "GemAPITransactionServiceMock", code: 0)
        }
        return transactionResponse
    }

    public func setWalletTransactionsResponse(_ response: TransactionsResponse) {
        walletTransactionsResponse = response
    }

    public func setAssetTransactionsResponse(_ response: TransactionsResponse) {
        assetTransactionsResponse = response
    }

    public func setTransactionResponse(_ transaction: Transaction) {
        transactionResponse = transaction
    }
}
