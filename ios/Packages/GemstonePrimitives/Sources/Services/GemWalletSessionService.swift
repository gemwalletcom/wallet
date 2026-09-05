// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemWalletSessionServiceProtocol
import Primitives

public extension GemWalletSessionServiceProtocol {
    var wallets: [Wallet] {
        get async {
            do {
                return try await getWallets()
            } catch {
                debugLog("get wallets error: \(error)")
                return []
            }
        }
    }

    var currentWallet: Wallet? {
        get async {
            do {
                return try await getCurrentWallet().map { $0.map() }
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
        try await getWallets().map { $0.map() }
    }

    func getWallet(walletId: WalletId) async throws -> Wallet {
        guard let wallet = try await getWallet(walletId: walletId.id) else {
            throw WalletSessionServiceError.noWalletId
        }
        return wallet.map()
    }

    func setCurrent(walletId: WalletId?) throws {
        try setCurrentWalletId(walletId: walletId?.id)
    }

    func setCurrent(wallet: Wallet) async throws {
        try await MainActor.run {
            try setCurrent(walletId: wallet.id)
        }
    }

    func showsRewards(wallets: [Wallet]) -> Bool {
        showsRewards(wallets: wallets.map { $0.map() })
    }
}
