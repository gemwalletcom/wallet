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
        try? service.getCurrentWallet().map { try Wallet($0) }
    }

    public var currentWalletId: WalletId? {
        guard let id = try? service.getCurrentWalletId() else { return nil }
        return try? WalletId.from(id: id)
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
