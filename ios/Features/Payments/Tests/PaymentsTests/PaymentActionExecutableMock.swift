// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Payments
import PaymentService
import Primitives

struct PaymentActionExecutableMock: PaymentActionExecutable {
    let results: [String]
    let transactionHash: String?

    init(results: [String] = [], transactionHash: String? = .none) {
        self.results = results
        self.transactionHash = transactionHash
    }

    @MainActor
    func execute(
        actions _: [PaymentAction],
        paymentId _: String,
        appMetadata _: TransactionAppMetadata,
        payment _: PaymentData,
        wallet _: Wallet,
        onSubmitted: @MainActor () -> Void,
    ) async throws -> PaymentActionResults {
        onSubmitted()
        return PaymentActionResults(results: results, transactionHash: transactionHash)
    }
}
