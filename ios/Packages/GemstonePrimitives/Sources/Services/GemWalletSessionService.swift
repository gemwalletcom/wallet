// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemWalletSessionServiceProtocol
import Primitives

public extension GemWalletSessionServiceProtocol {
    var currentWallet: Wallet? {
        get async {
            do {
                return try await getCurrentWallet().map { $0.toPrimitives() }
            } catch {
                debugLog("current wallet unavailable: \(error)")
                return .none
            }
        }
    }

    var currentWalletId: WalletId? {
        do {
            return try getCurrentWalletId().map { try WalletId.from(id: $0) }
        } catch {
            debugLog("current wallet id unavailable: \(error)")
            return .none
        }
    }

    func getWallets() async throws -> [Wallet] {
        try await getWallets().map { $0.toPrimitives() }
    }

    func requireWallet(walletId: WalletId) async throws -> Wallet {
        try await requireWallet(walletId: walletId.id).toPrimitives()
    }

    func setCurrent(walletId: WalletId?) throws {
        try setCurrentWalletId(walletId: walletId?.id)
    }
}
