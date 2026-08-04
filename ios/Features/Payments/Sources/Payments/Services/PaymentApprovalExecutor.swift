// Copyright (c). Gem Wallet. All rights reserved.

import ChainService
import Foundation
import PaymentService
import Primitives

public struct PaymentApprovalExecutor: PaymentApprovalExecutable {
    private let chainServiceFactory: any ChainServiceFactorable

    public init(chainServiceFactory: any ChainServiceFactorable) {
        self.chainServiceFactory = chainServiceFactory
    }

    public func waitForApproval(hash: String, assetId: AssetId, wallet: Wallet) async throws {
        let chain = assetId.chain
        try await TransactionConfirmationWaiter(chainService: chainServiceFactory.service(for: chain))
            .wait(hash: hash, chain: chain, senderAddress: try wallet.account(for: chain).address)
    }
}
