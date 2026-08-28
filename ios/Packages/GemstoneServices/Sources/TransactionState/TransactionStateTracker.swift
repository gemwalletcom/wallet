// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemTransactionStateServiceProtocol
import GemstonePrimitives
import Primitives

public struct TransactionStateTracker: Sendable {
    private let service: any GemTransactionStateServiceProtocol

    public init(service: any GemTransactionStateServiceProtocol) {
        self.service = service
    }

    public func trackPending() {
        Task {
            do {
                try await service.trackPending()
            } catch {
                debugLog("transaction state: pending tracking failed: \(error)")
            }
        }
    }

    public func track(wallet: Wallet, transactions: [Transaction]) {
        Task {
            do {
                try await service.track(walletId: wallet.id.id, transactions: transactions.map { try $0.json() })
            } catch {
                debugLog("transaction state: tracking failed: \(error)")
            }
        }
    }

    public func addNotificationTransaction(wallet: Wallet, assetId: AssetId, transaction: Transaction) async throws -> Asset? {
        guard let asset = try await service.addNotificationTransaction(
            wallet: wallet.json(),
            assetId: assetId.identifier,
            transaction: transaction.json(),
        ) else {
            return .none
        }
        track(wallet: wallet, transactions: [transaction])
        return try Asset(asset)
    }
}
