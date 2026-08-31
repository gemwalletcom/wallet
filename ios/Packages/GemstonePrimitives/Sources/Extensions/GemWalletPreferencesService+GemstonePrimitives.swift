// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemDiscoveryStep
import protocol Gemstone.GemWalletPreferencesServiceProtocol
import Primitives

public extension GemWalletPreferencesServiceProtocol {
    func getPerpetualAccountMode(walletId: WalletId) throws -> Primitives.PerpetualAccountMode {
        try Primitives.PerpetualAccountMode(getPerpetualAccountMode(walletId: walletId.id))
    }

    func isInitialLoadCompleted(walletId: WalletId, step: GemDiscoveryStep) throws -> Bool {
        try isInitialLoadCompleted(walletId: walletId.id, step: step)
    }

    func getAssetsTimestamp(walletId: WalletId) -> UInt64 {
        getAssetsTimestamp(walletId: walletId.id)
    }

    func resetTransactionsTimestamp(walletId: WalletId) throws {
        try resetTransactionsTimestamp(walletId: walletId.id)
    }

    func deletePreferences(walletId: WalletId) throws {
        try deletePreferences(walletId: walletId.id)
    }
}
