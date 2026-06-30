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
        let result = await service.update(for: transactionWallet.transaction)
        if let currentTransactionWallet = try? service.transactionWallet(
            walletId: transactionWallet.wallet.id,
            transactionId: result.transactionId,
        ) {
            await context.update(currentTransactionWallet)
        }
        return result.status
    }

    func nextInterval(after currentIntervalMs: UInt32) -> UInt32 {
        configuration.nextInterval(after: currentIntervalMs)
    }

    func onComplete() async throws {
        try await service.process(context.transactionWallet())
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
