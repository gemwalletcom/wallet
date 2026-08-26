// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemTransactionsServiceProtocol
import GemstonePrimitives
import Primitives
import Store

public final class TransactionsService: Sendable {
    private let service: any GemTransactionsServiceProtocol
    public let transactionStore: TransactionStore

    public init(
        service: any GemTransactionsServiceProtocol,
        transactionStore: TransactionStore,
    ) {
        self.service = service
        self.transactionStore = transactionStore
    }

    public func updateAll(walletId: WalletId) async throws {
        try await service.sync(walletId: walletId.id, assetId: nil)
    }

    public func updateForAsset(walletId: WalletId, assetId: AssetId) async throws {
        try await service.sync(walletId: walletId.id, assetId: assetId.identifier)
    }

    public func addTransaction(walletId: WalletId, transaction: Transaction) throws {
        try transactionStore.addTransactions(walletId: walletId, transactions: [transaction])
    }

    public func getTransaction(walletId: WalletId, transactionId: TransactionId) throws -> TransactionExtended {
        try transactionStore.getTransaction(walletId: walletId, transactionId: transactionId)
    }
}
