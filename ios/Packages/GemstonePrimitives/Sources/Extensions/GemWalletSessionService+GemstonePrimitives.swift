// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemWalletSessionServiceProtocol
import Primitives

public extension GemWalletSessionServiceProtocol {
    var wallets: [Wallet] {
        do {
            return try getWallets()
        } catch {
            debugLog("get wallets error: \(error)")
            return []
        }
    }

    var currentWallet: Wallet? {
        do {
            return try getCurrentWallet().map { try Wallet($0) }
        } catch {
            debugLog("current wallet unavailable: \(error)")
            return .none
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

    func getWallets() throws -> [Wallet] {
        try getWallets().map { try Wallet($0) }
    }

    func getCurrentWallet() throws -> Wallet {
        guard let currentWallet else {
            throw WalletSessionServiceError.noWallet
        }
        return currentWallet
    }

    func getWallet(walletId: WalletId) throws -> Wallet {
        guard let wallet = try getWallet(walletId: walletId.id) else {
            throw WalletSessionServiceError.noWalletId
        }
        return try Wallet(wallet)
    }

    func setCurrent(walletId: WalletId?) throws {
        try setCurrentWalletId(walletId: walletId?.id)
    }

    func setCurrent(wallet: Wallet) async throws {
        try await MainActor.run {
            try setCurrent(walletId: wallet.id)
        }
    }

    func showsRewards() -> Bool {
        do {
            return try showsRewards() as Bool
        } catch {
            debugLog("rewards availability unavailable: \(error)")
            return false
        }
    }

    func walletsCount() throws -> Int {
        try getWallets().count
    }
}
