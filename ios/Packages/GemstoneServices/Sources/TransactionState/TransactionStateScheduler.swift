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
        trackPendingTransactions()
    }

    public func trackPendingTransactions() {
        Task {
            do {
                scheduleUpdate(for: try await service.pendingTransactions())
            } catch {
                debugLog("transaction state: pending transactions load failed: \(error)")
            }
        }
    }

    public func addTransactions(wallet: Wallet, transactions: [Transaction], currency: String) async throws {
        try await service.addTransactions(wallet: wallet, transactions: transactions)
        track(wallet: wallet, transactions: transactions, currency: currency)
    }

    public func addNotificationTransaction(wallet: Wallet, assetId: AssetId, transaction: Transaction, currency: String) async throws -> Asset? {
        guard let asset = try await service.addNotificationTransaction(wallet: wallet, assetId: assetId, transaction: transaction) else {
            return nil
        }
        track(wallet: wallet, transactions: [transaction], currency: currency)
        return asset
    }

    public func track(wallet: Wallet, transactions: [Transaction], currency: String) {
        scheduleUpdate(for: transactions.map { TransactionWallet(transaction: $0, wallet: wallet) })
        Task {
            do {
                try await service.enableAssets(wallet: wallet, transactions: transactions, currency: currency)
            } catch {
                debugLog("transaction state: asset enabling failed: \(error)")
            }
        }
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
