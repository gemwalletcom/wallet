// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemTransactionStateServiceProtocol
import GemstonePrimitives
import Primitives

struct TransactionStateUpdateResult {
    let transactionId: TransactionId
    let status: JobStatus
}

public struct TransactionStateService: Sendable {
    private let service: any GemTransactionStateServiceProtocol

    public init(service: any GemTransactionStateServiceProtocol) {
        self.service = service
    }

    func update(walletId: WalletId, transaction: Transaction) async -> TransactionStateUpdateResult {
        do {
            guard let result = try await service.update(walletId: walletId.id, transaction: transaction.json()) else {
                return TransactionStateUpdateResult(transactionId: transaction.id, status: .cancelled)
            }
            for failure in result.failures {
                debugLog("TransactionStateService post-processing \(failure.step) failed: \(failure.message)")
            }
            let state = try Primitives.TransactionState(result.state)
            return TransactionStateUpdateResult(
                transactionId: try Primitives.TransactionId(result.transactionId),
                status: state.isCompleted ? .complete : .retry(),
            )
        } catch {
            return TransactionStateUpdateResult(
                transactionId: transaction.id,
                status: .retry(error: String(describing: error)),
            )
        }
    }

    func transactionWallet(walletId: WalletId, transactionId: TransactionId) async throws -> TransactionWallet? {
        try await service.getTransaction(walletId: walletId.id, transactionId: transactionId.json())
            .map { try TransactionWallet(transaction: Transaction($0.transaction), wallet: Wallet($0.wallet)) }
    }

    func pendingTransactions() async throws -> [TransactionWallet] {
        try await service.pendingTransactions().map { try TransactionWallet(transaction: Transaction($0.transaction), wallet: Wallet($0.wallet)) }
    }

    func addNotificationTransaction(wallet: Wallet, assetId: AssetId, transaction: Transaction) async throws -> Asset? {
        try await service.addNotificationTransaction(wallet: wallet.json(), assetId: assetId.identifier, transaction: transaction.json())
            .map { try Asset($0) }
    }

    func addTransactions(wallet: Wallet, transactions: [Transaction]) async throws {
        try await service.addTransactions(walletId: wallet.id.id, transactions: transactions.map { try $0.json() })
    }

    func setAssetsEnabled(wallet: Wallet, transactions: [Transaction]) async throws {
        try await service.enableTransactionAssets(walletId: wallet.id.id, transactions: transactions.map { try $0.json() })
    }
}
