// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives

struct TransactionStateJob: Job {
    let wallet: TransactionWallet
    let service: TransactionStateService

    var id: String {
        wallet.transaction.id.identifier
    }

    var configuration: JobConfiguration {
        wallet.transaction.assetId.chain.transactionStateConfig
    }

    func run() async -> JobStatus {
        await service.update(for: wallet.transaction)
    }

    func onComplete() async throws {
        try await service.process(wallet)
    }
}
