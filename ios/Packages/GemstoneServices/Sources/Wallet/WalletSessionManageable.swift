// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol WalletSessionManageable: Sendable {
    var wallets: [Wallet] { get }
    var currentWallet: Wallet? { get }
    var currentWalletId: WalletId? { get }

    func getWallets() throws -> [Wallet]
    func getWallet(walletId: WalletId) throws -> Wallet
    func getCurrentWallet() throws -> Wallet
    func setCurrent(walletId: WalletId?) throws
}

public extension WalletSessionManageable {
    var wallets: [Wallet] {
        do {
            return try getWallets()
        } catch {
            debugLog("get wallets error: \(error)")
            return []
        }
    }

    func getCurrentWallet() throws -> Wallet {
        guard let currentWallet else {
            throw WalletSessionServiceError.noWallet
        }
        return currentWallet
    }

    func getWallet(walletId: WalletId) throws -> Wallet {
        guard let wallet = wallets.first(where: { $0.id == walletId }) else {
            throw WalletSessionServiceError.noWalletId
        }
        return wallet
    }

    func setCurrent(wallet: Wallet) async throws {
        try await MainActor.run {
            try setCurrent(walletId: wallet.id)
        }
    }

    func walletsCount() throws -> Int {
        try getWallets().count
    }

    func hasMulticoinWallet() -> Bool {
        do {
            return try getWallets().contains { $0.type == .multicoin }
        } catch {
            debugLog("wallets unavailable: \(error)")
            return false
        }
    }
}
