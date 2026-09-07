// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import protocol Gemstone.GemTransactionStateServiceProtocol
import protocol Gemstone.GemTransactionStatusService
import struct Gemstone.Transaction

public final class GemstoneTransactionStatusService: GemTransactionStatusService, Sendable {
    private let service: any GemTransactionStateServiceProtocol

    public init(service: any GemTransactionStateServiceProtocol) {
        self.service = service
    }

    public func track(walletId: String, transactions: [Gemstone.Transaction]) {
        Task {
            do {
                try await service.track(walletId: walletId, transactions: transactions)
            } catch {
                debugLog("GemstoneTransactionStatusService: failed for \(walletId): \(error)")
            }
        }
    }
}
