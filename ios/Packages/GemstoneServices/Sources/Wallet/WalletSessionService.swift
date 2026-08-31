// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemWalletSessionServiceProtocol
import GemstonePrimitives
import Primitives

public struct WalletSessionService: WalletSessionManageable {
    private let service: any GemWalletSessionServiceProtocol

    public init(service: any GemWalletSessionServiceProtocol) {
        self.service = service
    }

    public var currentWallet: Wallet? {
        do {
            return try service.getCurrentWallet().map { try Wallet($0) }
        } catch {
            debugLog("current wallet unavailable: \(error)")
            return .none
        }
    }

    public var currentWalletId: WalletId? {
        do {
            return try service.getCurrentWalletId().map { try WalletId.from(id: $0) }
        } catch {
            debugLog("current wallet id unavailable: \(error)")
            return .none
        }
    }

    public func setCurrent(walletId: WalletId?) throws {
        try service.setCurrentWalletId(walletId: walletId?.id)
    }

    public func getWallets() throws -> [Wallet] {
        try service.getWallets().map { try Wallet($0) }
    }

    public func getWallet(walletId: WalletId) throws -> Wallet {
        guard let wallet = try service.getWallet(walletId: walletId.id) else {
            throw WalletSessionServiceError.noWalletId
        }
        return try Wallet(wallet)
    }
}
