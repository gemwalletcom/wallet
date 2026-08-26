// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemTransactionStateServiceProtocol
import GemstonePrimitives
import Primitives
import Store

struct TransactionStateUpdateResult {
    let transactionId: TransactionId
    let status: JobStatus
}

public struct TransactionStateService: Sendable {
    private let transactionStore: TransactionStore
    private let postProcessingService: TransactionPostProcessingService
    private let service: any GemTransactionStateServiceProtocol

    public init(
        transactionStore: TransactionStore,
        service: any GemTransactionStateServiceProtocol,
        postProcessingService: TransactionPostProcessingService,
    ) {
        self.transactionStore = transactionStore
        self.service = service
        self.postProcessingService = postProcessingService
    }

    func update(walletId: WalletId, transaction: Transaction) async -> TransactionStateUpdateResult {
        do {
            guard let result = try await service.update(walletId: walletId.id, transaction: transaction.json()) else {
                return TransactionStateUpdateResult(transactionId: transaction.id, status: .cancelled)
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

    func transactionWallet(walletId: WalletId, transactionId: TransactionId) throws -> TransactionWallet? {
        try transactionStore.getTransactionWallet(walletId: walletId, transactionId: transactionId)
    }

    func process(_ transactionWallet: TransactionWallet) async throws {
        try await postProcessingService.process(
            wallet: transactionWallet.wallet,
            transaction: transactionWallet.transaction,
        )
    }

    func updateBalances(_ transactionWallet: TransactionWallet) async {
        await postProcessingService.updateBalances(
            wallet: transactionWallet.wallet,
            transaction: transactionWallet.transaction,
        )
    }
}
