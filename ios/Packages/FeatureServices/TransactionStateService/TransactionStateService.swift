// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemTransactionStateServiceProtocol
import GemstonePrimitives
import Primitives
import Store

public struct TransactionStateService: Sendable {
    private let service: GemTransactionStateServiceProtocol
    private let transactionStore: TransactionStore

    public init(
        service: GemTransactionStateServiceProtocol,
        transactionStore: TransactionStore,
    ) {
        self.service = service
        self.transactionStore = transactionStore
    }

    public func setup() {
        if let transactions = try? transactionStore.getTransactions(states: [.pending, .inTransit]) {
            monitor(transactions)
        }
    }

    public func addTransactions(wallet: Wallet, transactions: [Transaction]) throws {
        try transactionStore.addTransactions(walletId: wallet.walletId, transactions: transactions)
        monitor(transactions)
    }
}

// MARK: - Private

private extension TransactionStateService {
    func monitor(_ transactions: [Transaction]) {
        Task { await service.monitor(inputs: transactions.map { $0.map() }) }
    }
}
