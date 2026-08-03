// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import Foundation
import GemstonePrimitives
import Localization
import Primitives

public enum TransactionConfirmationError: Error, Equatable {
    case reverted
    case timedOut
}

extension TransactionConfirmationError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .reverted: Localized.Transaction.Status.failed
        case .timedOut: Localized.Errors.errorOccurred
        }
    }
}

public struct TransactionConfirmationWaiter: Sendable {
    private let chainService: any ChainServiceable
    private let timeout: Duration

    public init(
        chainService: any ChainServiceable,
        timeout: Duration = .seconds(120),
    ) {
        self.chainService = chainService
        self.timeout = timeout
    }

    public func wait(hash: String, chain: Chain, senderAddress: String) async throws {
        let configuration = chain.transactionStateConfig
        let request = TransactionStateRequest(id: hash, senderAddress: senderAddress, createdAt: Date(), blockNumber: 0)
        let deadline = ContinuousClock.now.advanced(by: timeout)
        var intervalMs = configuration.initialIntervalMs

        while ContinuousClock.now < deadline {
            switch try await chainService.transactionState(for: request).state {
            case .confirmed:
                return
            case .failed, .reverted:
                throw TransactionConfirmationError.reverted
            case .pending, .inTransit:
                try await Task.sleep(for: .milliseconds(Int(intervalMs)))
                intervalMs = configuration.nextInterval(after: intervalMs)
            }
        }
        throw TransactionConfirmationError.timedOut
    }
}
