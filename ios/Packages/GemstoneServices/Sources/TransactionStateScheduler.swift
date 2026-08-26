// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Store

public struct TransactionStateScheduler: Sendable {
    private let transactionStore: TransactionStore
    private let service: TransactionStateService
    private let runner: JobRunner = .init()

    public init(
        transactionStore: TransactionStore,
        service: TransactionStateService,
    ) {
        self.transactionStore = transactionStore
        self.service = service
    }

    public func setup() {
        if let transactionWallets = try? transactionStore.getTransactionWallets(states: [.pending, .inTransit]) {
            scheduleUpdate(for: transactionWallets)
        }
    }

    public func addTransactions(wallet: Wallet, transactions: [Transaction]) throws {
        try transactionStore.addTransactions(
            walletId: wallet.id,
            transactions: transactions,
        )
        scheduleUpdate(for: transactions.map { TransactionWallet(transaction: $0, wallet: wallet) })
    }
}

// MARK: - Private

extension TransactionStateScheduler {
    private func scheduleUpdate(for transactionWallets: [TransactionWallet]) {
        let jobs = transactionWallets.map {
            TransactionStateJob(wallet: $0, service: service)
        }
        Task {
            for job in jobs {
                await runner.addJob(job: job)
            }
        }
    }
}
