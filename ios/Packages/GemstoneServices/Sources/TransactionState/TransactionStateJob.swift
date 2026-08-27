// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import Primitives

struct TransactionStateJob: Job {
    let id: String
    let configuration: JobConfiguration
    private let context: TransactionStateJobContext
    let service: TransactionStateService

    init(wallet: TransactionWallet, service: TransactionStateService) {
        id = wallet.transaction.id.identifier
        configuration = wallet.transaction.assetId.chain.transactionStateConfig
        context = TransactionStateJobContext(transactionWallet: wallet)
        self.service = service
    }

    func run() async -> JobStatus {
        let transactionWallet = await context.transactionWallet()
        let result = await service.update(walletId: transactionWallet.wallet.id, transaction: transactionWallet.transaction)
        let storedTransactionWallet: TransactionWallet?
        do {
            storedTransactionWallet = try await service.transactionWallet(
                walletId: transactionWallet.wallet.id,
                transactionId: result.transactionId,
            )
        } catch {
            debugLog("TransactionStateJob stopped: transaction \(result.transactionId.identifier) read failed: \(error)")
            return .cancelled
        }
        guard let currentTransactionWallet = storedTransactionWallet else {
            debugLog("TransactionStateJob stopped: transaction \(result.transactionId.identifier) no longer stored")
            return .cancelled
        }
        await context.update(currentTransactionWallet)
        return result.status
    }

    func nextInterval(after currentIntervalMs: UInt32) -> UInt32 {
        configuration.nextInterval(after: currentIntervalMs)
    }
}

private actor TransactionStateJobContext {
    private var currentTransactionWallet: TransactionWallet

    init(transactionWallet: TransactionWallet) {
        currentTransactionWallet = transactionWallet
    }

    func transactionWallet() -> TransactionWallet {
        currentTransactionWallet
    }

    func update(_ transactionWallet: TransactionWallet) {
        currentTransactionWallet = transactionWallet
    }
}
