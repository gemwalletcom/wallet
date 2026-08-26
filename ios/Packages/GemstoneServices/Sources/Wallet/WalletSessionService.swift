// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemWalletSessionServiceProtocol
import Primitives
import Store

public struct WalletSessionService: WalletSessionManageable {
    private let service: any GemWalletSessionServiceProtocol
    private let walletStore: WalletStore

    public init(
        service: any GemWalletSessionServiceProtocol,
        walletStore: WalletStore,
    ) {
        self.service = service
        self.walletStore = walletStore
    }

    public var currentWallet: Wallet? {
        guard let currentWalletId else { return nil }
        return try? walletStore.getWallet(id: currentWalletId)
    }

    public var currentWalletId: Primitives.WalletId? {
        guard let id = try? service.getCurrentWalletId() else { return nil }
        return try? WalletId.from(id: id)
    }

    public func setCurrent(walletId: WalletId?) {
        try? service.setCurrentWalletId(walletId: walletId?.id)
    }

    public func getWallets() throws -> [Wallet] {
        try walletStore.getWallets()
    }
}
