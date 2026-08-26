// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemNotificationServiceProtocol
import Primitives

public struct InAppNotificationService: Sendable {
    private let service: any GemNotificationServiceProtocol

    public init(service: any GemNotificationServiceProtocol) {
        self.service = service
    }

    public func update(walletId: WalletId) async throws {
        try await service.sync(walletId: walletId.id)
    }

    public func markNotificationsRead() async throws {
        try await service.markRead()
    }
}
