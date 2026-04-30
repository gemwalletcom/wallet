// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives

struct TransactionStateJob: Job {
    let transactionWallet: TransactionWallet
    let service: TransactionStateService

    var id: String {
        transactionWallet.transaction.id.identifier
    }

    var configuration: JobConfiguration {
        transactionWallet.transaction.assetId.chain.transactionStateConfig
    }

    func run() async -> JobStatus {
        await service.update(for: transactionWallet.transaction)
    }

    func onComplete() async throws {
        try await service.process(transactionWallet)
    }
}
