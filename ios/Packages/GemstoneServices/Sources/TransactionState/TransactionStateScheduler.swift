// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct TransactionStateScheduler: Sendable {
    private let service: TransactionStateService
    private let runner: JobRunner = .init()

    public init(service: TransactionStateService) {
        self.service = service
    }

    public func setup() {
        Task {
            do {
                scheduleUpdate(for: try await service.pendingTransactions())
            } catch {
                debugLog("transaction state: pending transactions load failed: \(error)")
            }
        }
    }

    public func addTransactions(wallet: Wallet, transactions: [Transaction], currency: String) async throws {
        try await service.addTransactions(wallet: wallet, transactions: transactions, currency: currency)
        scheduleUpdate(for: transactions.map { TransactionWallet(transaction: $0, wallet: wallet) })
    }
}

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
