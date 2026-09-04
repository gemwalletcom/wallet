// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import protocol Gemstone.GemTransactionStateServiceProtocol
import protocol Gemstone.GemTransactionTracking

public final class GemstoneTransactionTracking: GemTransactionTracking, Sendable {
    private let service: any GemTransactionStateServiceProtocol

    public init(service: any GemTransactionStateServiceProtocol) {
        self.service = service
    }

    public func track(walletId: String, transactions: [String]) {
        Task {
            do {
                try await service.track(walletId: walletId, transactions: transactions)
            } catch {
                debugLog("TransactionTracking: tracking failed for \(walletId): \(error)")
            }
        }
    }
}
